pub fn database_url() -> String {
    std::env::var("DATABASE_URL").expect("DATABASE_URL is not set")
}
