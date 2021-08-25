#[macro_export]
macro_rules! ratelimits {
    ($struct_name: ident {$($tokens:tt)*}) => {
        use nonzero_ext::nonzero;
        use pointercrate_core::error::CoreError;
        use ratelimit_meter::{KeyedRateLimiter, NonConformance, DirectRateLimiter};
        use std::{
            net::IpAddr,
            time::{Duration, Instant},
        };

        ratelimits!(@struct@ $struct_name [] $($tokens)*);

        impl $struct_name {
            ratelimits!(@method@  $($tokens)*);
        }

        // probably fine, lol
        unsafe impl Sync for $struct_name {}
    };

    (@struct@ $struct_name: ident [$($field: ident: $type: ty | $init: expr),*] $name: ident[$capacity: tt per $seconds: tt] => $message: expr, $($remaining: tt)*) => {
        ratelimits!(@struct@ $struct_name [$($field: $type | $init,)* $name: DirectRateLimiter | DirectRateLimiter::new(nonzero!($capacity), Duration::from_secs($seconds))] $($remaining)*);
    };

    (@struct@ $struct_name: ident [$($field: ident: $type: ty | $init: expr),*] $name: ident[$capacity: tt per $seconds: tt per ip] => $message: expr, $($remaining: tt)*) => {
        ratelimits!(@struct@ $struct_name [$($field: $type | $init,)* $name: KeyedRateLimiter<IpAddr> | KeyedRateLimiter::new(nonzero!($capacity), Duration::from_secs($seconds))] $($remaining)*);
    };

    (@method@ $name: ident[$capacity: tt per $seconds: tt] => $message: expr, $($remaining: tt)*) => {
        pub(crate) fn $name(&self) -> Result<(), CoreError> {
            let now = Instant::now();

            self.$name.clone().check_at(now).map_err(|too_early| {
                CoreError::Ratelimited {
                    message: $message.to_string(),
                    remaining: too_early.earliest_possible() - now,
                }
            })
        }
        ratelimits!(@method@  $($remaining)*);
    };

    (@method@ $name: ident[$capacity: tt per $seconds: tt per ip] => $message: expr, $($remaining: tt)*) => {
        pub(crate) fn $name(&self, ip: IpAddr) -> Result<(), CoreError> {
            let now = Instant::now();

            self.$name.clone().check_at(ip, now).map_err(|too_early| {
                CoreError::Ratelimited {
                    message: $message.to_string(),
                    remaining: too_early.earliest_possible() - now,
                }
            })
        }
        ratelimits!(@method@  $($remaining)*);
    };

    (@struct@ $struct_name: ident [$($field: ident: $type: ty | $init: expr),*]) => {
        pub struct $struct_name {
            $(
                $field: $type,
            )*
        }

        impl $struct_name {
            pub(crate) fn new() -> Self {
                $struct_name {
                    $(
                        $field: $init,
                    )*
                }
            }
        }
    };
    (@method@) => {};
}
