use pointercrate_core::ratelimits;

ratelimits! {
    DemonlistRatelimits {
        record_submission[3u32 per 1200 per ip] => "You're submitting too many records too fast!",

        record_submission_global[20u32 per 3600] => "Too many records are being submitted right now!",

        new_submitters[7u32 per 3600] => "DDoS protection ratelimit",

        geolocate[1u32 per 2_678_4000 per ip] => "You can only geolocate once per month!",
    }
}
