macro_rules! header {
    ($req:expr, $header:expr) => {
        match $req.headers().get($header) {
            Some(value) =>
                Some(
                    value
                        .to_str()
                        .map_err(|_| PointercrateError::InvalidHeaderValue { header: $header })?,
                ),
            None => None,
        }
    };
}

pub mod cond;
pub mod ip;
