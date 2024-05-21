use crate::demon::MinimalDemon;
use derive_more::Constructor;
pub use paginate::{NationalityRankingPagination, RankedNation};
use pointercrate_core::etag::Taggable;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use sqlx::PgConnection;

mod get;
mod paginate;

#[derive(Debug, PartialEq, Eq, Serialize, Hash, Constructor, Deserialize, Clone)]
pub struct Nationality {
    #[serde(rename = "country_code")]
    pub iso_country_code: String,
    pub nation: String,
    pub subdivision: Option<Subdivision>,
}

#[derive(Debug, Serialize, Hash)]
pub struct BestRecord {
    id: i32,
    demon: String,
    position: i16,
    progress: i16,
    players: Vec<String>,
}

#[derive(Debug, Serialize, Hash)]
pub struct MiniDemon {
    id: i32,
    demon: String,
    position: i16,
    player: String,
}

#[derive(Debug, Serialize, Hash)]
pub struct MiniDemonWithPlayers {
    id: i32,
    demon: String,
    position: i16,
    players: Vec<String>,
}

#[derive(Debug, Hash, Serialize)]
pub struct NationalityRecord {
    pub nation: Nationality,

    #[serde(rename = "records")]
    pub best_records: Vec<BestRecord>,
    pub created: Vec<MiniDemonWithPlayers>,
    pub verified: Vec<MiniDemon>,
    pub published: Vec<MiniDemon>,
    pub unbeaten: Vec<MinimalDemon>,
}

impl Taggable for NationalityRecord {}

#[derive(Debug, Eq, PartialEq, Clone, Serialize, Hash, Constructor, Deserialize)]
pub struct Subdivision {
    pub iso_code: String,
    pub name: String,
}

#[derive(Debug, Eq, PartialEq, Hash, Copy, Clone)]
pub enum Continent {
    Asia,
    Europe,
    AustraliaAndOceania,
    Africa,
    NorthAmerica,
    SouthAmerica,
    MiddleAmerica,
}

impl Continent {
    pub fn to_sql(&self) -> String {
        match self {
            Continent::Asia => "Asia",
            Continent::Europe => "Europe",
            Continent::AustraliaAndOceania => "Australia and Oceania",
            Continent::Africa => "Africa",
            Continent::NorthAmerica => "North America",
            Continent::SouthAmerica => "South America",
            Continent::MiddleAmerica => "Central America",
        }
        .to_owned()
    }
}

impl<'de> Deserialize<'de> for Continent {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let string = String::deserialize(deserializer)?.to_lowercase();

        match &string[..] {
            "asia" => Ok(Continent::Asia),
            "europe" => Ok(Continent::Europe),
            "australia" => Ok(Continent::AustraliaAndOceania),
            "africa" => Ok(Continent::Africa),
            "north america" => Ok(Continent::NorthAmerica),
            "south america" => Ok(Continent::SouthAmerica),
            "central america" => Ok(Continent::MiddleAmerica),
            _ => Err(serde::de::Error::invalid_value(
                serde::de::Unexpected::Str(&string),
                &"'Asia', 'Europe', 'Australia', 'Africa', 'North America', 'South America' or 'Central America'",
            )),
        }
    }
}

impl Serialize for Continent {
    fn serialize<S>(&self, serializer: S) -> Result<<S as Serializer>::Ok, <S as Serializer>::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(match self {
            Continent::Asia => "asia",
            Continent::Europe => "europe",
            Continent::AustraliaAndOceania => "australia",
            Continent::Africa => "africa",
            Continent::NorthAmerica => "north america",
            Continent::SouthAmerica => "south america",
            Continent::MiddleAmerica => "central america",
        })
    }
}

impl Nationality {
    /// Checks whether [`self`] and `other` refer to the same country (but potentially different subdivisions)
    pub fn same_country_as(&self, other: &Nationality) -> bool {
        self.iso_country_code == other.iso_country_code
    }

    /// Updates the score for this [`Nationality`] and contained [`Subdivision`] (if set).
    pub async fn update_nation_score(&self, connection: &mut PgConnection) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "UPDATE nationalities SET score = coalesce(score_of_nation($1), 0) WHERE iso_country_code = $1",
            self.iso_country_code
        )
        .execute(&mut *connection)
        .await?;
        if let Some(ref subdivision) = self.subdivision {
            sqlx::query!(
                "UPDATE subdivisions SET score = coalesce(score_of_subdivision($1, $2), 0) WHERE nation = $1 AND iso_code = $2",
                self.iso_country_code,
                subdivision.iso_code
            )
            .execute(&mut *connection)
            .await?;
        }

        Ok(())
    }
}
