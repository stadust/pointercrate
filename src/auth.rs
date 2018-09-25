use serde_derive::Deserialize;

#[derive(Debug)]
pub enum Authorization {
    Unauthorized,
    Basic(String, String),
    Token(String),
}

#[derive(Debug, Deserialize)]
pub struct Claims {
    pub id: i32,
}
