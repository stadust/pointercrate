pub fn submission_webhook() -> Option<String> {
    std::env::var("DISCORD_WEBHOOK").ok()
}

pub fn abstract_api_key() -> Option<String> {
    std::env::var("ABSTRACT_API_KEY").ok()
}
