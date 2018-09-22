pub enum Authorization {
    Unauthorized,
    Basic(String, String),
    Token(String),
}
