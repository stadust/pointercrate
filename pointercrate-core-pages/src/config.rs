pub fn google_analytics_tag() -> String {
    std::env::var("ANALYTICS_TAG")
        .expect("No google analytics tag configured. Please remove all google analytics code from your custom copy of pointercrate")
}
