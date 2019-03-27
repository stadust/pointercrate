use derive_more::Display;
use ipnetwork::IpNetwork;
use nonzero_ext::nonzero;
use ratelimit_meter::{DirectRateLimiter, KeyedRateLimiter, NonConformance};
use std::time::{Duration, Instant};

#[derive(Debug, Display, Clone, Copy)]
pub enum RatelimitScope {
    #[display(fmt = "You're submitting too many records too fast!")]
    RecordSubmission,

    #[display(fmt = "Too many records are being submitted right now!")]
    RecordSubmissionGlobal,

    #[display(fmt = "Well that's pretty unfortunate!")]
    NewSubmitter,

    #[display(fmt = "Too many registrations!")]
    Registration,

    #[display(fmt = "Too many login attempts!")]
    Login,
}

#[derive(Debug, Clone)]
pub struct Ratelimits {
    record_submission: KeyedRateLimiter<IpNetwork>,
    record_submission_global: DirectRateLimiter,
    new_submitters: DirectRateLimiter,
    registrations: KeyedRateLimiter<IpNetwork>,
    login_attempts: KeyedRateLimiter<IpNetwork>,
}

impl Ratelimits {
    pub fn initialize() -> Self {
        Ratelimits {
            // 3 per 20  minutes
            record_submission: KeyedRateLimiter::new(nonzero!(3u32), Duration::from_secs(20 * 60)),
            // 20 per hour
            record_submission_global: DirectRateLimiter::new(
                nonzero!(20u32),
                Duration::from_secs(3600),
            ),
            // 5 per hour
            new_submitters: DirectRateLimiter::new(nonzero!(5u32), Duration::from_secs(3600)),
            // 1 per day
            registrations: KeyedRateLimiter::new(nonzero!(1u32), Duration::from_secs(3600 * 24)),
            // 3 per 30 minutes
            login_attempts: KeyedRateLimiter::new(nonzero!(3u32), Duration::from_secs(1800)),
        }
    }

    pub fn check(&self, scope: RatelimitScope, ip: IpNetwork) -> Result<(), Duration> {
        let now = Instant::now();

        match scope {
            RatelimitScope::RecordSubmission => self.record_submission.clone().check_at(ip, now),
            RatelimitScope::RecordSubmissionGlobal =>
                self.record_submission_global.clone().check_at(now),
            RatelimitScope::NewSubmitter => self.new_submitters.clone().check_at(now),
            RatelimitScope::Registration => self.registrations.clone().check_at(ip, now),
            RatelimitScope::Login => self.login_attempts.clone().check_at(ip, now),
        }
        .map_err(|too_early| too_early.earliest_possible() - now) // TODO: add jitter
    }
}
