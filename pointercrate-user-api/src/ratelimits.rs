use std::net::IpAddr;

use pointercrate_core::ratelimits;

ratelimits! {
    UserRatelimits {
        registrations[1u32 per 86400 per IpAddr] => "Too many registrations!",
        soft_registrations[5u32 per 21600 per IpAddr] => "Too many failed registration attempts!",
        login_attempts[3u32 per 1800 per IpAddr] => "Too many login attempts!",
    }
}
