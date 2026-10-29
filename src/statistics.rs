//! Statistics of the API calls made through a client.

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;

/// Statistics of the API calls made through an [`ApifyClient`](crate::ApifyClient), read with
/// [`ApifyClient::stats`](crate::ApifyClient::stats).
///
/// The counters are shared by every clone of the client and by every resource client obtained
/// from it.
#[derive(Debug, Default)]
pub struct Statistics {
    calls: AtomicU64,
    requests: AtomicU64,
    rate_limit_errors: Mutex<Vec<u64>>,
}

impl Statistics {
    /// Number of client function calls: each call counts once, however many attempts it took.
    pub fn calls(&self) -> u64 {
        self.calls.load(Ordering::Relaxed)
    }

    /// Number of API requests: every attempt of every call counts.
    pub fn requests(&self) -> u64 {
        self.requests.load(Ordering::Relaxed)
    }

    /// Number of times the API returned a `429` rate-limit error, by attempt: an error on the first
    /// attempt is counted at index 0, an error on the first retry at index 1, and so on.
    pub fn rate_limit_errors(&self) -> Vec<u64> {
        self.lock_rate_limit_errors().clone()
    }

    pub(crate) fn add_call(&self) {
        self.calls.fetch_add(1, Ordering::Relaxed);
    }

    pub(crate) fn add_request(&self) {
        self.requests.fetch_add(1, Ordering::Relaxed);
    }

    /// Counts a `429` returned on the 1-based `attempt`.
    pub(crate) fn add_rate_limit_error(&self, attempt: u32) {
        let index = attempt.saturating_sub(1) as usize;
        let mut errors = self.lock_rate_limit_errors();
        if errors.len() <= index {
            errors.resize(index + 1, 0);
        }
        errors[index] += 1;
    }

    /// The counters hold plain numbers, so a panic while one was held cannot leave them in an
    /// inconsistent state and the poison is ignored.
    fn lock_rate_limit_errors(&self) -> std::sync::MutexGuard<'_, Vec<u64>> {
        self.rate_limit_errors
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rate_limit_errors_are_counted_by_attempt() {
        let stats = Statistics::default();
        stats.add_rate_limit_error(1);
        stats.add_rate_limit_error(3);
        stats.add_rate_limit_error(3);
        assert_eq!(stats.rate_limit_errors(), vec![1, 0, 2]);
    }

    #[test]
    fn calls_and_requests_are_counted_separately() {
        let stats = Statistics::default();
        stats.add_call();
        stats.add_request();
        stats.add_request();
        assert_eq!((stats.calls(), stats.requests()), (1, 2));
    }
}
