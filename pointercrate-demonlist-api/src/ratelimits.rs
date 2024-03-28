use std::net::IpAddr;

use pointercrate_core::ratelimits;

ratelimits! {
    DemonlistRatelimits {
        record_submission[3u32 per 1200 per IpAddr] => "You're submitting too many records too fast!",

        record_submission_global[20u32 per 3600] => "Too many records are being submitted right now!",

        new_submitters[7u32 per 3600] => "DDoS protection ratelimit",

        geolocate[1u32 per 2_678_400 per IpAddr] => "You can only geolocate once per month!",

        add_demon[1u32 per 60] => "Please don't spam the button, rSteel",
    }
}

#[cfg(test)]
mod test {
    use crate::ratelimits::DemonlistRatelimits;
    use pointercrate_core::error::CoreError;

    #[test]
    fn test_non_burst_ratelimit() {
        let ratelimits = DemonlistRatelimits::new();
        let pass = ratelimits.add_demon();

        assert!(pass.is_ok());

        let fail = ratelimits.add_demon();

        assert!(fail.is_err());

        match fail.unwrap_err() {
            CoreError::Ratelimited { remaining, .. } => {
                assert!(remaining.as_secs() <= 60);
                assert!(remaining.as_secs() >= 50); // execution shouldnt take longer than 10
                                                    // seconds
            },
            err => panic!("Got unexpected error {:?}, expected CoreError::RateLimited", err),
        }
    }

    #[test]
    fn test_burst_ratelimits() {
        let ratelimits = DemonlistRatelimits::new();

        for _ in 1..=7 {
            assert!(ratelimits.new_submitters().is_ok());
        }

        let fail = ratelimits.new_submitters();

        assert!(fail.is_err());

        match fail.unwrap_err() {
            CoreError::Ratelimited { remaining, .. } => {
                assert!(remaining.as_secs() <= 3600);
                assert!(remaining.as_secs() >= 500); // 7 tokens per hour -> refresh one every 3600
                                                     // / 7 ~ 514 seconds. Execution of test
                                                     // shouldn't take longer than 14s
            },
            err => panic!("Got unexpected error {:?}, expected CoreError::RateLimited", err),
        }
    }
}
