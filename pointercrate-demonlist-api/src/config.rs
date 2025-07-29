pub fn submission_webhook() -> Option<String> {
    std::env::var("DISCORD_WEBHOOK").ok()
}

pub fn gd_connector_endpoint() -> Option<String> {
    std::env::var("GD_CONNECTOR_ENDPOINT").ok()
}
