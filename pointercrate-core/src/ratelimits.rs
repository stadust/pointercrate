#[macro_export]
macro_rules! ratelimits {
    ($struct_name: ident {$($tokens:tt)*}) => {
        use nonzero_ext::nonzero;
        use pointercrate_core::error::CoreError;
        use std::{
            net::IpAddr,
            time::{Duration, Instant},
        };
        use governor::{
            clock::{Clock, DefaultClock, Reference},
            state::{direct::NotKeyed, keyed::DefaultKeyedStateStore, InMemoryState},
            Quota, RateLimiter,
        };

        ratelimits!(@struct@ $struct_name [] $($tokens)*);

        impl $struct_name {
            ratelimits!(@method@  $($tokens)*);
        }
    };

    // Accumulates the fields of the struct as a comma separated list of the form "name: type | initializer_expression"
    (@struct@ $struct_name: ident [$($field: ident: $type: ty | $init: expr),*] $name: ident[$capacity: tt per $seconds: tt] => $message: expr, $($remaining: tt)*) => {
        ratelimits!(@struct@
            $struct_name [
                $($field: $type | $init,)*  // already processed fields
                $name: RateLimiter<NotKeyed, InMemoryState, DefaultClock> | RateLimiter::direct(Quota::new(nonzero!($capacity), Duration::from_secs($seconds)).unwrap())  // new field
            ] $($remaining)*);  // remaining, unprocessed fields as token stream
    };

    (@struct@ $struct_name: ident [$($field: ident: $type: ty | $init: expr),*] $name: ident[$capacity: tt per $seconds: tt per ip] => $message: expr, $($remaining: tt)*) => {
        ratelimits!(@struct@
            $struct_name [
                $($field: $type | $init,)*
                $name: RateLimiter<IpAddr, DefaultKeyedStateStore<IpAddr>, DefaultClock> | RateLimiter::keyed(Quota::new(nonzero!($capacity), Duration::from_secs($seconds)).unwrap())
            ] $($remaining)*);
    };

    (@method@ $name: ident[$capacity: tt per $seconds: tt] => $message: expr, $($remaining: tt)*) => {
        pub(crate) fn $name(&self) -> Result<(), CoreError> {
            let now = DefaultClock::default().now();

            self.$name.check().map_err(|too_early| {
                CoreError::Ratelimited {
                    message: $message.to_string(),
                    remaining: too_early.earliest_possible().duration_since(now).into(),
                }
            })
        }
        ratelimits!(@method@  $($remaining)*);
    };

    (@method@ $name: ident[$capacity: tt per $seconds: tt per ip] => $message: expr, $($remaining: tt)*) => {
        pub(crate) fn $name(&self, ip: IpAddr) -> Result<(), CoreError> {
            let now = DefaultClock::default().now();

            self.$name.check_key(&ip).map_err(|too_early| {
                CoreError::Ratelimited {
                    message: $message.to_string(),
                    remaining: too_early.earliest_possible().duration_since(now).into(),
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
                #[allow(deprecated)] // the governor API mentions that using Quota::new() is fine since our ratelimits are given as "burst per duration"
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
