use crate::{model::Model, schema::nationality};
use serde_derive::Serialize;

#[derive(Queryable, Debug, PartialEq, Eq, Serialize, Hash)]
pub struct Nationality {
    pub name: String,
    pub country_code: String,
}

/// The different between 'A', as unicode codepoint (65), and '🇦', as unicode codepoint (127462)
const MAGIC: u32 = 127397;

impl Nationality {
    pub fn to_emoji(&self) -> String {
        self.country_code
            .chars()
            .map(|c| unsafe { std::char::from_u32_unchecked((c as u32) + MAGIC) })
            .collect()
    }
}
