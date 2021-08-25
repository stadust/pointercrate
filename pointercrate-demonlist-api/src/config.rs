pub fn submission_webhook() -> Option<String> {
    std::env::var("DISCORD_WEBHOOK").ok()
}
