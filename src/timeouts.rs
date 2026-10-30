//! Request timeout tiers.
//!
//! Every client method picks the tier that fits the expected duration of its request: a simple
//! metadata read or write is [`Timeout::Short`], listing, batch and trigger operations are
//! [`Timeout::Medium`], and downloads, uploads and streaming are [`Timeout::Long`]. Polling that
//! waits for a job to finish uses [`Timeout::NoTimeout`]. The duration of each tier is set on
//! [`ApifyClientBuilder`](crate::ApifyClientBuilder), and a single resource client can override the
//! tier of all its requests with its `with_timeout` method.

use std::time::Duration;

/// Default duration of the [`Timeout::Short`] tier.
pub const DEFAULT_TIMEOUT_SHORT: Duration = Duration::from_secs(5);
/// Default duration of the [`Timeout::Medium`] tier.
pub const DEFAULT_TIMEOUT_MEDIUM: Duration = Duration::from_secs(30);
/// Default duration of the [`Timeout::Long`] tier.
pub const DEFAULT_TIMEOUT_LONG: Duration = Duration::from_secs(360);
/// Default cap on the timeout of a single request attempt.
pub const DEFAULT_TIMEOUT_MAX: Duration = Duration::from_secs(360);

/// The value [`HttpRequest::timeout`](crate::http_client::HttpRequest::timeout) carries for a
/// request that must run for as long as it takes ([`Timeout::NoTimeout`]). An
/// [`HttpBackend`](crate::http_client::HttpBackend) must not abort such a request on its own.
pub const NO_TIMEOUT: Duration = Duration::MAX;

/// How long the API holds a `waitForFinish` response, however long the parameter asks for.
pub(crate) const MAX_WAIT_FOR_FINISH_HOLD: Duration = Duration::from_secs(60);

/// Timeout of a single API request.
///
/// A tier name picks the duration configured for that tier on
/// [`ApifyClientBuilder`](crate::ApifyClientBuilder), [`Timeout::Custom`] is an exact duration,
/// and [`Timeout::NoTimeout`] lets the request run for as long as it takes. A tier or an explicit
/// duration above the client's `timeout_max` is capped at it. The timeout applies to each attempt
/// separately and doubles with every retry, up to the same cap.
///
/// With [`Timeout::NoTimeout`] the client never aborts the request, so a connection that stalls
/// is not retried either.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum Timeout {
    /// Simple metadata reads and writes (default 5 seconds).
    Short,
    /// Listing, batch and trigger operations (default 30 seconds).
    Medium,
    /// Downloads, uploads and streaming (default 360 seconds).
    Long,
    /// No client-side timeout.
    NoTimeout,
    /// An exact duration, capped at the client's `timeout_max`.
    Custom(Duration),
}

/// The configured duration of every timeout tier plus the cap on a single attempt.
#[derive(Debug, Clone, Copy)]
pub(crate) struct TimeoutTiers {
    pub(crate) short: Duration,
    pub(crate) medium: Duration,
    pub(crate) long: Duration,
    pub(crate) max: Duration,
}

impl Default for TimeoutTiers {
    fn default() -> Self {
        Self {
            short: DEFAULT_TIMEOUT_SHORT,
            medium: DEFAULT_TIMEOUT_MEDIUM,
            long: DEFAULT_TIMEOUT_LONG,
            max: DEFAULT_TIMEOUT_MAX,
        }
    }
}

impl TimeoutTiers {
    /// Resolves `timeout` to the duration of the first attempt of a request, capped at `max`.
    /// [`Timeout::NoTimeout`] resolves to [`NO_TIMEOUT`].
    pub(crate) fn resolve(&self, timeout: Timeout) -> Duration {
        let requested = match timeout {
            Timeout::Short => self.short,
            Timeout::Medium => self.medium,
            Timeout::Long => self.long,
            Timeout::NoTimeout => return NO_TIMEOUT,
            Timeout::Custom(duration) => duration,
        };
        requested.min(self.max)
    }

    /// Resolves a request that asks the API to hold its response with `waitForFinish`: the hold
    /// the caller asked for (at most [`MAX_WAIT_FOR_FINISH_HOLD`]) plus the round trip the
    /// `tier` allows, so the client does not abort a request the API is still holding. An
    /// aborted `start` or `build` would be retried, creating a second resource.
    pub(crate) fn resolve_with_wait_hold(
        &self,
        tier: Timeout,
        wait_for_finish_secs: Option<i64>,
    ) -> Timeout {
        let hold = wait_for_finish_secs
            .filter(|&secs| secs > 0)
            .map(|secs| Duration::from_secs(secs as u64).min(MAX_WAIT_FOR_FINISH_HOLD));
        if tier == Timeout::NoTimeout {
            return tier;
        }
        match hold {
            Some(hold) => Timeout::Custom(self.resolve(tier).saturating_add(hold)),
            None => tier,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tiers_resolve_to_their_configured_duration_capped_at_max() {
        let tiers = TimeoutTiers {
            short: Duration::from_secs(1),
            medium: Duration::from_secs(2),
            long: Duration::from_secs(100),
            max: Duration::from_secs(10),
        };
        assert_eq!(tiers.resolve(Timeout::Short), Duration::from_secs(1));
        assert_eq!(tiers.resolve(Timeout::Medium), Duration::from_secs(2));
        assert_eq!(tiers.resolve(Timeout::Long), Duration::from_secs(10));
        assert_eq!(
            tiers.resolve(Timeout::Custom(Duration::from_secs(50))),
            Duration::from_secs(10)
        );
        assert_eq!(tiers.resolve(Timeout::NoTimeout), NO_TIMEOUT);
    }

    #[test]
    fn wait_hold_is_added_to_the_tier_and_capped_at_one_minute() {
        let tiers = TimeoutTiers::default();
        assert_eq!(
            tiers.resolve_with_wait_hold(Timeout::Short, None),
            Timeout::Short
        );
        assert_eq!(
            tiers.resolve_with_wait_hold(Timeout::Short, Some(0)),
            Timeout::Short
        );
        assert_eq!(
            tiers.resolve_with_wait_hold(Timeout::Short, Some(10)),
            Timeout::Custom(Duration::from_secs(15))
        );
        assert_eq!(
            tiers.resolve_with_wait_hold(Timeout::Medium, Some(999)),
            Timeout::Custom(Duration::from_secs(90))
        );
    }
}
