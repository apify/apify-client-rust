//! Log redirection: forwarding the live log of a run to a [`LogSink`].
//!
//! [`StreamedLog`] is the equivalent of the reference client's `StreamedLog`. It follows the log
//! of a run in a background task, splits it into messages at the timestamps the Actor's log lines
//! start with, and hands every message, prefixed, to a [`LogSink`]. [`ActorClient::call`] uses it
//! to redirect the log of the run it waits for.
//!
//! [`ActorClient::call`]: crate::clients::actor::ActorClient::call

use std::fmt;
use std::sync::Arc;
use std::time::Duration;

use chrono::{DateTime, Utc};
use futures_util::StreamExt;
use tokio::sync::watch;
use tokio::task::JoinHandle;

use crate::clients::log::{LogClient, LogOptions};
use crate::error::{ApifyClientError, ApifyClientResult};

/// How long to wait before reopening the log stream of a run that has not logged anything yet.
/// The API serves such a log as an empty stream that ends at once. The pause doubles up to
/// [`EMPTY_LOG_STREAM_MAX_RETRY`].
const EMPTY_LOG_STREAM_RETRY: Duration = Duration::from_millis(500);
/// The longest pause between two reopenings of an empty log stream.
const EMPTY_LOG_STREAM_MAX_RETRY: Duration = Duration::from_secs(5);
/// How long [`StreamedLog::stop`] lets the lines already in flight arrive before it cuts the
/// log stream.
const STOP_GRACE: Duration = Duration::from_secs(1);
/// The length of the ISO 8601 timestamp (`2024-01-31T12:00:00.000Z`) every log message starts
/// with.
const TIMESTAMP_LEN: usize = 24;
/// The shape of that timestamp: `d` is a digit, any other byte stands for itself.
const TIMESTAMP_SHAPE: &[u8; TIMESTAMP_LEN] = b"dddd-dd-ddTdd:dd:dd.dddZ";

/// A destination of redirected log messages, such as the terminal or a logging framework.
///
/// Every message arrives complete (a multi-line message, such as a stack trace, in one piece)
/// and with the prefix of the redirection already added. A closure taking `&str` is a sink too:
///
/// ```
/// use apify_client::clients::streamed_log::{LogRedirect, LogSink};
///
/// let to_stderr = LogRedirect::to_sink(|line: &str| eprintln!("{line}"));
/// # let _ = to_stderr;
/// ```
pub trait LogSink: Send + Sync + 'static {
    /// Receives one message of the redirected log.
    fn line(&self, message: &str);
}

impl<F> LogSink for F
where
    F: Fn(&str) + Send + Sync + 'static,
{
    fn line(&self, message: &str) {
        self(message)
    }
}

/// The default [`LogSink`]: prints every message to the standard output.
#[derive(Debug, Clone, Copy, Default)]
pub struct StdoutLogSink;

impl LogSink for StdoutLogSink {
    fn line(&self, message: &str) {
        println!("{message}");
    }
}

/// Where the log of a run goes: the `log` option of [`ActorClient::call`] and the `to_log` option
/// of [`RunClient::get_streamed_log`].
///
/// [`ActorClient::call`]: crate::clients::actor::ActorClient::call
/// [`RunClient::get_streamed_log`]: crate::clients::run::RunClient::get_streamed_log
#[derive(Clone, Default)]
pub enum LogRedirect {
    /// Print the log to the standard output, with every message prefixed with
    /// `"{actor name} runId:{run id} -> "`. Costs two extra requests, for the run and its Actor.
    #[default]
    Default,
    /// Do not redirect the log.
    Disabled,
    /// Send the log to this sink, without a prefix.
    Sink(Arc<dyn LogSink>),
}

impl LogRedirect {
    /// Redirects the log to `sink`.
    pub fn to_sink(sink: impl LogSink) -> Self {
        Self::Sink(Arc::new(sink))
    }
}

impl fmt::Debug for LogRedirect {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Default => f.write_str("Default"),
            Self::Disabled => f.write_str("Disabled"),
            Self::Sink(_) => f.write_str("Sink(..)"),
        }
    }
}

/// Options of [`RunClient::get_streamed_log`](crate::clients::run::RunClient::get_streamed_log).
#[derive(Debug, Clone)]
pub struct StreamedLogOptions {
    /// Where to redirect the log. [`LogRedirect::Disabled`] makes `get_streamed_log` return
    /// `None`.
    pub to_log: LogRedirect,
    /// Redirect the log from its beginning. When `false`, messages stamped before the call are
    /// dropped. Defaults to `true`.
    pub from_start: bool,
}

impl Default for StreamedLogOptions {
    fn default() -> Self {
        Self {
            to_log: LogRedirect::Default,
            from_start: true,
        }
    }
}

/// What the redirection task is asked to do.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Control {
    /// Keep redirecting.
    Run,
    /// Finish the lines in flight, then read the rest of the log and end.
    Stop,
    /// The grace period of a stop is over: cut the log stream.
    Abort,
}

struct Running {
    control: watch::Sender<Control>,
    task: JoinHandle<Option<ApifyClientError>>,
}

/// The redirection of one run's log to a [`LogSink`], created by
/// [`RunClient::get_streamed_log`](crate::clients::run::RunClient::get_streamed_log).
///
/// [`start`](Self::start) spawns a task that follows the log, and [`stop`](Self::stop) ends it.
/// Dropping a started `StreamedLog` cancels the task without reading the rest of the log.
///
/// Compared with the reference client, a line that continues a message in a later chunk is
/// emitted as a message of its own, instead of being dropped.
pub struct StreamedLog {
    log: LogClient,
    sink: Arc<dyn LogSink>,
    prefix: String,
    relevancy_limit: Option<DateTime<Utc>>,
    running: Option<Running>,
}

impl fmt::Debug for StreamedLog {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("StreamedLog")
            .field("prefix", &self.prefix)
            .field("started", &self.running.is_some())
            .finish()
    }
}

impl StreamedLog {
    pub(crate) fn new(
        log: LogClient,
        sink: Arc<dyn LogSink>,
        prefix: String,
        from_start: bool,
    ) -> Self {
        Self {
            log,
            sink,
            prefix,
            relevancy_limit: (!from_start).then(Utc::now),
            running: None,
        }
    }

    /// Starts redirecting the log, in a task spawned on the current Tokio runtime. Fails with
    /// [`ApifyClientError::InvalidArgument`] when the redirection is already started.
    pub fn start(&mut self) -> ApifyClientResult<()> {
        if self.running.is_some() {
            return Err(ApifyClientError::InvalidArgument(
                "the log redirection is already started".to_string(),
            ));
        }
        let (control, receiver) = watch::channel(Control::Run);
        let task = tokio::spawn(redirect(
            self.log.clone(),
            self.sink.clone(),
            self.prefix.clone(),
            self.relevancy_limit,
            receiver,
        ));
        self.running = Some(Running { control, task });
        Ok(())
    }

    /// Stops redirecting the log. Waits up to one second for the lines in flight, then cuts the
    /// log stream. If the stream has delivered nothing yet, reads the whole log in one request
    /// instead. Fails with [`ApifyClientError::InvalidArgument`] when the redirection is not
    /// started, and with the error that ended the redirection early, if there was one.
    pub async fn stop(&mut self) -> ApifyClientResult<()> {
        let Some(Running { control, mut task }) = self.running.take() else {
            return Err(ApifyClientError::InvalidArgument(
                "the log redirection is not started".to_string(),
            ));
        };
        control.send_replace(Control::Stop);
        let outcome = match tokio::time::timeout(STOP_GRACE, &mut task).await {
            Ok(outcome) => outcome,
            Err(_) => {
                control.send_replace(Control::Abort);
                task.await
            }
        };
        match outcome {
            Ok(None) | Err(_) => Ok(()),
            Ok(Some(error)) => Err(error),
        }
    }
}

impl Drop for StreamedLog {
    fn drop(&mut self) {
        if let Some(running) = &self.running {
            running.task.abort();
        }
    }
}

fn is_stopping(control: &watch::Receiver<Control>) -> bool {
    *control.borrow() != Control::Run
}

/// Resolves once the grace period of a stop is over (or the [`StreamedLog`] is gone).
async fn aborted(mut control: watch::Receiver<Control>) {
    let _ = control.wait_for(|c| *c == Control::Abort).await;
}

/// Resolves once a stop is requested (or the [`StreamedLog`] is gone).
async fn stopped(mut control: watch::Receiver<Control>) {
    let _ = control.wait_for(|c| *c != Control::Run).await;
}

/// The redirection task. Returns the error that ended it early.
async fn redirect(
    log: LogClient,
    sink: Arc<dyn LogSink>,
    prefix: String,
    relevancy_limit: Option<DateTime<Utc>>,
    control: watch::Receiver<Control>,
) -> Option<ApifyClientError> {
    let mut messages = MessageSplitter::new(relevancy_limit);
    let emit = |message: &str| sink.line(&format!("{prefix}{message}"));
    let raw = LogOptions {
        raw: Some(true),
        download: None,
    };
    let mut retry = EMPTY_LOG_STREAM_RETRY;
    // The API serves the log of a run that has not logged anything yet as an empty stream that
    // ends at once, so reopen it until the first bytes arrive. Once stopped, read the whole log
    // in one request instead.
    loop {
        if is_stopping(&control) {
            match log.get_bytes_with_options(raw.clone()).await {
                Ok(content) => messages.feed(&content.unwrap_or_default(), &emit),
                Err(error) => return Some(error),
            }
            break;
        }
        let opened = tokio::select! {
            opened = log.stream_with_options(raw.clone()) => opened,
            () = aborted(control.clone()) => continue,
        };
        let stream = match opened {
            Ok(Some(stream)) => stream,
            Ok(None) => return None,
            Err(_) if is_stopping(&control) => continue,
            Err(error) => return Some(error),
        };
        if is_stopping(&control) {
            continue;
        }
        futures_util::pin_mut!(stream);
        let mut received_any = false;
        loop {
            let item = tokio::select! {
                item = stream.next() => item,
                () = aborted(control.clone()) => break,
            };
            match item {
                Some(Ok(chunk)) => {
                    received_any |= !chunk.is_empty();
                    messages.feed(&chunk, &emit);
                    if is_stopping(&control) {
                        break;
                    }
                }
                Some(Err(_)) if is_stopping(&control) => break,
                Some(Err(error)) => {
                    messages.finish(&emit);
                    return Some(error);
                }
                None => break,
            }
        }
        if received_any {
            break;
        }
        tokio::select! {
            () = tokio::time::sleep(retry) => {}
            () = stopped(control.clone()) => {}
        }
        retry = (retry * 2).min(EMPTY_LOG_STREAM_MAX_RETRY);
    }
    // Whatever is left may be incomplete, or the last line of the log without a line break.
    messages.finish(&emit);
    None
}

/// Splits the bytes of a raw log into messages. A message starts with a line that begins with an
/// ISO 8601 timestamp and runs up to the next such line, so a stack trace stays in one piece.
struct MessageSplitter {
    /// The incomplete last line of the bytes fed so far, led by the line break before it.
    remainder: Vec<u8>,
    /// Messages stamped before this are dropped.
    relevancy_limit: Option<DateTime<Utc>>,
    /// Whether the message in progress at the end of the previous batch was relevant, so the
    /// lines that continue it are too.
    relevant: bool,
}

impl MessageSplitter {
    fn new(relevancy_limit: Option<DateTime<Utc>>) -> Self {
        Self {
            remainder: Vec::new(),
            relevancy_limit,
            relevant: relevancy_limit.is_none(),
        }
    }

    /// Emits every message in the complete lines of `chunk` (and the lines held back before it),
    /// and holds back the incomplete last line.
    fn feed(&mut self, chunk: &[u8], emit: &dyn Fn(&str)) {
        self.remainder.extend_from_slice(chunk);
        let Some(last_line_break) = self.remainder.iter().rposition(|&b| b == b'\n') else {
            return;
        };
        let complete: Vec<u8> = self.remainder.drain(..last_line_break).collect();
        self.emit_messages(&String::from_utf8_lossy(&complete), emit);
    }

    /// Emits the held-back last line as well, when the log is over.
    fn finish(&mut self, emit: &dyn Fn(&str)) {
        self.remainder.push(b'\n');
        self.feed(&[], emit);
    }

    fn emit_messages(&mut self, text: &str, emit: &dyn Fn(&str)) {
        let mut message = String::new();
        for line in text.split('\n') {
            if starts_with_timestamp(line) {
                self.emit_message(&message, emit);
                message.clear();
                self.relevant = self.is_relevant(&line[..TIMESTAMP_LEN]);
            }
            if !message.is_empty() {
                message.push('\n');
            }
            message.push_str(line);
        }
        self.emit_message(&message, emit);
    }

    fn emit_message(&self, message: &str, emit: &dyn Fn(&str)) {
        let message = message.trim();
        if self.relevant && !message.is_empty() {
            emit(message);
        }
    }

    fn is_relevant(&self, timestamp: &str) -> bool {
        match (
            self.relevancy_limit,
            DateTime::parse_from_rfc3339(timestamp),
        ) {
            (Some(limit), Ok(time)) => time >= limit,
            _ => true,
        }
    }
}

/// Whether `line` begins with an ISO 8601 timestamp, such as `2024-01-31T12:00:00.000Z`.
fn starts_with_timestamp(line: &str) -> bool {
    let bytes = line.as_bytes();
    bytes.len() >= TIMESTAMP_LEN
        && bytes
            .iter()
            .zip(TIMESTAMP_SHAPE)
            .all(|(&byte, &shape)| match shape {
                b'd' => byte.is_ascii_digit(),
                _ => byte == shape,
            })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;

    fn split(limit: Option<DateTime<Utc>>, chunks: &[&str], finish: bool) -> Vec<String> {
        let out = RefCell::new(Vec::new());
        let emit = |m: &str| out.borrow_mut().push(m.to_string());
        let mut splitter = MessageSplitter::new(limit);
        for chunk in chunks {
            splitter.feed(chunk.as_bytes(), &emit);
        }
        if finish {
            splitter.finish(&emit);
        }
        out.into_inner()
    }

    #[test]
    fn multi_line_messages_stay_together_and_the_last_line_waits_for_its_line_break() {
        let log = "2024-01-01T00:00:00.000Z first\n  at trace\n2024-01-01T00:00:01.000Z second\n2024-01-01T00:00:02.000Z third";
        assert_eq!(
            split(None, &[log], false),
            vec![
                "2024-01-01T00:00:00.000Z first\n  at trace".to_string(),
                "2024-01-01T00:00:01.000Z second".to_string(),
            ]
        );
        assert_eq!(
            split(None, &[log], true),
            vec![
                "2024-01-01T00:00:00.000Z first\n  at trace".to_string(),
                "2024-01-01T00:00:01.000Z second".to_string(),
                "2024-01-01T00:00:02.000Z third".to_string(),
            ]
        );
    }

    #[test]
    fn a_line_split_across_chunks_is_emitted_whole() {
        let out = split(
            None,
            &[
                "2024-01-01T00:00:00.000Z hel",
                "lo\n2024-01-01T00:00:01.000Z x\n",
            ],
            true,
        );
        assert_eq!(
            out,
            vec![
                "2024-01-01T00:00:00.000Z hello".to_string(),
                "2024-01-01T00:00:01.000Z x".to_string()
            ]
        );
    }

    #[test]
    fn messages_stamped_before_the_limit_are_dropped() {
        let limit = DateTime::parse_from_rfc3339("2024-01-01T00:00:01.000Z")
            .unwrap()
            .with_timezone(&Utc);
        let log = "2024-01-01T00:00:00.000Z old\n  old trace\n2024-01-01T00:00:01.000Z new\n";
        assert_eq!(
            split(Some(limit), &[log], true),
            vec!["2024-01-01T00:00:01.000Z new".to_string()]
        );
    }

    #[test]
    fn lines_without_a_timestamp_continue_the_previous_message() {
        let out = split(None, &["2024-01-01T00:00:00.000Z a\n", "continued\n"], true);
        assert_eq!(
            out,
            vec![
                "2024-01-01T00:00:00.000Z a".to_string(),
                "continued".to_string()
            ]
        );
    }
}
