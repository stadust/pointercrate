use pointercrate_core::localization::tr;
use pointercrate_core::ratelimits;
use std::net::IpAddr;

ratelimits! {
    UserRatelimits {
        registrations[1u32 per 86400 per IpAddr] => tr("error-user-ratelimit-registration"),
        soft_registrations[5u32 per 21600 per IpAddr] => tr("error-user-ratelimit-soft-registration"),
        login_attempts[3u32 per 1800 per IpAddr] => tr("error-user-ratelimit-login"),
    }
}
