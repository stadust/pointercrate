//! this crate is a burning pile of trash

pub mod gd;

pub fn set_gd_connector_endpoint(endpoint: String) {
    dash_rs::request::GD_SERVER_ENDPOINT_BASE_URL
        .set(endpoint)
        .expect("GD_SERVER_ENDPOINT_BASE_URL to be uninitialized")
}
