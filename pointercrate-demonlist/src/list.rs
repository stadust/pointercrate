use std::str::FromStr;

use pointercrate_core::error::CoreError;
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum List {
    Demonlist, // only consists of rated demons (Demonlist)
    RatedPlus, // consists of ALL demons (Rated+ List)
}

impl List {
    pub fn as_str(&self) -> &'static str {
        match self {
            List::Demonlist => "demonlist",
            List::RatedPlus => "ratedplus",
        }
    }

    pub fn to_key(&self) -> &'static str {
        match self {
            List::Demonlist => "list-demonlist",
            List::RatedPlus => "list-ratedplus",
        }
    }
}

impl Default for List {
    fn default() -> Self {
        List::Demonlist
    }
}

impl FromStr for List {
    type Err = CoreError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "demonlist" => Ok(List::Demonlist),
            "ratedplus" => Ok(List::RatedPlus),
            _ => Err(CoreError::UnprocessableEntity),
        }
    }
}

impl ToString for List {
    fn to_string(&self) -> String {
        self.as_str().to_string()
    }
}

impl Serialize for List {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for List {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let string = String::deserialize(deserializer)?;

        List::from_str(&string[..])
            .map_err(|_| serde::de::Error::invalid_value(serde::de::Unexpected::Str(&string), &"'demonlist', 'ratedplus'"))
    }
}
