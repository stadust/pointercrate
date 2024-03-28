#[macro_export]
macro_rules! ratelimits {
    ($struct_name: ident {$($tokens:tt)*}) => {
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
                $name: governor::RateLimiter<governor::state::direct::NotKeyed, governor::state::InMemoryState, governor::clock::DefaultClock> | governor::RateLimiter::direct(governor::Quota::new(nonzero_ext::nonzero!($capacity), std::time::Duration::from_secs($seconds)).unwrap())  // new field
            ] $($remaining)*);  // remaining, unprocessed fields as token stream
    };

    (@struct@ $struct_name: ident [$($field: ident: $type: ty | $init: expr),*] $name: ident[$capacity: tt per $seconds: tt per $key_type: ty] => $message: expr, $($remaining: tt)*) => {
        ratelimits!(@struct@
            $struct_name [
                $($field: $type | $init,)*
                $name: governor::RateLimiter<$key_type, governor::state::keyed::DefaultKeyedStateStore<$key_type>, governor::clock::DefaultClock> | governor::RateLimiter::keyed(governor::Quota::new(nonzero_ext::nonzero!($capacity), std::time::Duration::from_secs($seconds)).unwrap())
            ] $($remaining)*);
    };

    (@method@ $name: ident[$capacity: tt per $seconds: tt] => $message: expr, $($remaining: tt)*) => {
        pub(crate) fn $name(&self) -> Result<(), pointercrate_core::error::CoreError> {
            use governor::clock::{Clock, Reference};

            let now = governor::clock::DefaultClock::default().now();

            self.$name.check().map_err(|too_early| {
                pointercrate_core::error::CoreError::Ratelimited {
                    message: $message.to_string(),
                    remaining: too_early.earliest_possible().duration_since(now).into(),
                }
            })
        }
        ratelimits!(@method@  $($remaining)*);
    };

    (@method@ $name: ident[$capacity: tt per $seconds: tt per $key_type: ty] => $message: expr, $($remaining: tt)*) => {
        pub(crate) fn $name(&self, ip: $key_type) -> Result<(), pointercrate_core::error::CoreError> {
            use governor::clock::{Clock, Reference};

            let now = governor::clock::DefaultClock::default().now();

            self.$name.check_key(&ip).map_err(|too_early| {
                pointercrate_core::error::CoreError::Ratelimited {
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
