use crate::{error::PointercrateError, Result};
use derive_more::Display;
use nonzero_ext::nonzero;
use ratelimit_meter::{DirectRateLimiter, KeyedRateLimiter, NonConformance};
use std::{
    net::IpAddr,
    time::{Duration, Instant},
};

#[derive(Debug, Display, Clone, Copy, PartialEq, Eq)]
pub enum RatelimitScope {
    #[display(fmt = "You're submitting too many records too fast!")]
    RecordSubmission,

    #[display(fmt = "Too many records are being submitted right now!")]
    RecordSubmissionGlobal,

    #[display(fmt = "Well that's pretty unfortunate!")]
    NewSubmitter,

    #[display(fmt = "Too many registrations!")]
    Registration,

    #[display(fmt = "Too many failed registration attempts")]
    SoftRegistration,

    #[display(fmt = "Too many login attempts!")]
    Login,
}

#[derive(Debug, Clone)]
pub struct Ratelimits {
    record_submission: KeyedRateLimiter<IpAddr>,
    record_submission_global: DirectRateLimiter,
    new_submitters: DirectRateLimiter,
    registrations: KeyedRateLimiter<IpAddr>,
    soft_registrations: KeyedRateLimiter<IpAddr>,
    login_attempts: KeyedRateLimiter<IpAddr>,
}

#[derive(Copy, Clone)]
pub struct PreparedRatelimits<'a> {
    ratelimits: &'a Ratelimits,
    ip: IpAddr,
}

impl PreparedRatelimits<'_> {
    pub fn check(&self, scope: RatelimitScope) -> Result<()> {
        self.ratelimits.check(scope, self.ip)
    }
}

impl Ratelimits {
    pub fn initialize() -> Self {
        Ratelimits {
            // 3 per 20  minutes
            record_submission: KeyedRateLimiter::new(nonzero!(3u32), Duration::from_secs(20 * 60)),
            // 20 per hour
            record_submission_global: DirectRateLimiter::new(nonzero!(20u32), Duration::from_secs(3600)),
            // 5 per hour
            new_submitters: DirectRateLimiter::new(nonzero!(5u32), Duration::from_secs(3600)),
            // 1 per day
            registrations: KeyedRateLimiter::new(nonzero!(1u32), Duration::from_secs(3600 * 24)),
            // 5 per 6 hours
            soft_registrations: KeyedRateLimiter::new(nonzero!(5u32), Duration::from_secs(3600 * 6)),
            // 3 per 30 minutes
            login_attempts: KeyedRateLimiter::new(nonzero!(3u32), Duration::from_secs(1800)),
        }
    }

    pub fn prepare(&self, ip: IpAddr) -> PreparedRatelimits {
        PreparedRatelimits { ratelimits: self, ip }
    }

    pub fn check(&self, scope: RatelimitScope, ip: IpAddr) -> Result<()> {
        let now = Instant::now();

        match scope {
            RatelimitScope::RecordSubmission => self.record_submission.clone().check_at(ip, now),
            RatelimitScope::RecordSubmissionGlobal => self.record_submission_global.clone().check_at(now),
            RatelimitScope::NewSubmitter => self.new_submitters.clone().check_at(now),
            RatelimitScope::Registration => self.registrations.clone().check_at(ip, now),
            RatelimitScope::SoftRegistration => self.soft_registrations.clone().check_at(ip, now),
            RatelimitScope::Login => self.login_attempts.clone().check_at(ip, now),
        }
        .map_err(|too_early| {
            PointercrateError::Ratelimited {
                scope,
                remaining: too_early.earliest_possible() - now,
            }
        }) // TODO: add jitter
    }
}
