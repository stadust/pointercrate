use crate::{
    error::PointercrateError,
    model::{
        demonlist::{creator::Creators, player::EmbeddedPlayer, record::EmbeddedRecordP},
        Model,
    },
    operation::Get,
    schema::{demons, players},
    Result,
};
use derive_more::Display;
use diesel::{
    dsl::max, pg::Pg, BoolExpressionMethods, Expression, ExpressionMethods, PgConnection, QueryDsl,
    QueryResult, Queryable, RunQueryDsl,
};
use joinery::Joinable;
use log::{debug, warn};
use serde_derive::Serialize;
use std::hash::{Hash, Hasher};

mod get;
mod paginate;
mod patch;
mod post;

pub use self::{paginate::DemonPagination, patch::PatchDemon, post::PostDemon};
use crate::{
    citext::{CiStr, CiString},
    context::RequestContext,
    model::By,
};

/// Struct modelling a demon in the database
#[derive(Debug, Identifiable, Serialize, Hash, Display)]
#[table_name = "demons"]
#[primary_key("name")]
#[display(fmt = "{} (at {})", name, position)]
pub struct Demon {
    /// The [`Demon`]'s Geometry Dash level name
    pub name: CiString,

    /// The [`Demon`]'s position on the demonlist
    ///
    /// Positions for consecutive demons are always consecutive positive integers
    pub position: i16,

    /// The minimal progress a [`Player`] must achieve on this [`Demon`] to have their record
    /// accepted
    pub requirement: i16,

    pub video: Option<String>,

    /// The player-ID of this [`Demon`]'s verifer
    pub verifier: EmbeddedPlayer,

    /// The player-ID of this [`Demon`]'s publisher
    pub publisher: EmbeddedPlayer,
}

/// Struct modelling a minimal representation of a [`Demon`] in the database
///
/// These representations are used whenever a different object references a demon, or when a list of
/// demons is requested
#[derive(Debug, Hash, Eq, PartialEq, Serialize, Display)]
#[display(fmt = "{} (at {})", name, position)]
pub struct PartialDemon {
    pub name: CiString,
    pub position: i16,
    // TODO: when implemented return host here instead of publisher
    pub publisher: CiString,
    pub video: Option<String>,
}

impl Queryable<<<PartialDemon as Model>::Selection as Expression>::SqlType, Pg> for PartialDemon {
    type Row = (CiString, i16, CiString, Option<String>);

    fn build(row: Self::Row) -> Self {
        PartialDemon {
            name: row.0,
            position: row.1,
            publisher: row.2,
            video: row.3,
        }
    }
}

impl Model for PartialDemon {
    type From = diesel::query_source::joins::JoinOn<
        diesel::query_source::joins::Join<
            demons::table,
            players::table,
            diesel::query_source::joins::Inner,
        >,
        diesel::expression::operators::Eq<demons::columns::publisher, players::columns::id>,
    >;
    type Selection = (demons::name, demons::position, players::name, demons::video);

    fn from() -> Self::From {
        diesel::query_source::joins::Join::new(
            demons::table,
            players::table,
            diesel::query_source::joins::Inner,
        )
        .on(demons::publisher.eq(players::id))
    }

    fn selection() -> Self::Selection {
        Self::Selection::default()
    }
}

impl Queryable<<<Demon as Model>::Selection as Expression>::SqlType, Pg> for Demon {
    #[allow(clippy::type_complexity)]
    type Row = (
        CiString,
        i16,
        i16,
        Option<String>,
        CiString,
        i32,
        bool,
        CiString,
        i32,
        bool,
    );

    fn build(row: Self::Row) -> Self {
        Demon {
            name: row.0,
            position: row.1,
            requirement: row.2,
            video: row.3,
            verifier: EmbeddedPlayer {
                name: row.4,
                id: row.5,
                banned: row.6,
            },
            publisher: EmbeddedPlayer {
                name: row.7,
                id: row.8,
                banned: row.9,
            },
        }
    }
}

table! {
    use crate::citext::CiText;
    use diesel::sql_types::*;

    demon_verifier_publisher_join (vid, pid) {
        vname -> CiText,
        vid -> Int4,
        vbanned -> Bool,
        pname -> CiText,
        pid -> Int4,
        pbanned -> Bool,
    }
}

allow_tables_to_appear_in_same_query!(demons, demon_verifier_publisher_join);

impl Model for Demon {
    #[allow(clippy::type_complexity)]
    type From = diesel::query_source::joins::JoinOn<
        diesel::query_source::joins::Join<
            demons::table,
            demon_verifier_publisher_join::table,
            diesel::query_source::joins::Inner,
        >,
        diesel::dsl::And<
            diesel::expression::operators::Eq<
                demons::publisher,
                demon_verifier_publisher_join::pid,
            >,
            diesel::expression::operators::Eq<demons::verifier, demon_verifier_publisher_join::vid>,
        >,
    >;
    type Selection = (
        demons::name,
        demons::position,
        demons::requirement,
        demons::video,
        demon_verifier_publisher_join::vname,
        demon_verifier_publisher_join::vid,
        demon_verifier_publisher_join::vbanned,
        demon_verifier_publisher_join::pname,
        demon_verifier_publisher_join::pid,
        demon_verifier_publisher_join::pbanned,
    );

    fn from() -> Self::From {
        diesel::query_source::joins::Join::new(
            demons::table,
            demon_verifier_publisher_join::table,
            diesel::query_source::joins::Inner,
        )
        .on(demons::publisher
            .eq(demon_verifier_publisher_join::pid)
            .and(demons::verifier.eq(demon_verifier_publisher_join::vid)))
    }

    fn selection() -> Self::Selection {
        Self::Selection::default()
    }
}

/// Absolutely minimal representation of a demon to be sent when a demon is part of another object
#[derive(Debug, Hash, Serialize, Queryable, Display)]
#[display(fmt = "{} (at {})", name, position)]
pub struct EmbeddedDemon {
    pub position: i16,
    pub name: CiString,
}

impl Model for EmbeddedDemon {
    type From = demons::table;
    type Selection = (demons::position, demons::name);

    fn from() -> Self::From {
        demons::table
    }

    fn selection() -> Self::Selection {
        Self::Selection::default()
    }
}

#[derive(Debug, Serialize, Display)]
#[display(fmt = "{}", demon)]
pub struct DemonWithCreatorsAndRecords {
    #[serde(flatten)]
    pub demon: Demon,
    pub creators: Creators,
    pub records: Vec<EmbeddedRecordP>,
}

impl Hash for DemonWithCreatorsAndRecords {
    fn hash<H: Hasher>(&self, state: &mut H) {
        // We only hash the demon here, because the creators don't matter for the ETag value - they
        // are modified through a different endpoint than the demon objects themselves, and
        // conflicting access to them is impossible anyway
        self.demon.hash(state)
    }
}

impl DemonWithCreatorsAndRecords {
    pub fn headline(&self) -> String {
        let publisher = &self.demon.publisher.name;
        let verifier = &self.demon.verifier.name;

        let creator = match &self.creators.0[..] {
            [] => "Unknown".to_string(),
            [creator] => creator.name.to_string(),
            many => {
                let mut iter = many.iter();
                let fst = iter.next().unwrap();

                format!(
                    "{} and {}",
                    iter.map(|player| &player.name).join_with(", ").to_string(),
                    fst.name
                )
            },
        };

        // no comparison between &String and String, so just make it a reference
        let creator = &CiString(creator);

        if creator == verifier && creator == publisher {
            format!("by {}", creator)
        } else if creator != verifier && verifier == publisher {
            format!("by {}, verified and published by {}", creator, verifier)
        } else if creator != verifier && creator != publisher && publisher != verifier {
            format!(
                "by {}, verified by {}, published by {}",
                creator, verifier, publisher
            )
        } else if creator == verifier && creator != publisher {
            format!("by {}, published by {}", creator, publisher)
        } else if creator == publisher && creator != verifier {
            format!("by {}, verified by {}", creator, verifier)
        } else {
            "If you're seeing this, file a bug report".to_string()
        }
    }

    pub fn short_headline(&self) -> String {
        let demon = &self.demon;

        if demon.publisher == demon.verifier {
            format!("verified and published by {}", demon.verifier.name)
        } else {
            format!(
                "published by {}, verified by {}",
                demon.publisher.name, demon.verifier.name
            )
        }
    }
}
impl By<demons::position, i16> for Demon {}
impl By<demons::name, &CiStr> for Demon {}

impl Demon {
    /// Increments the position of all demons with positions equal to or greater than the given one,
    /// by one.
    pub fn shift_down(starting_at: i16, connection: &PgConnection) -> QueryResult<()> {
        diesel::update(demons::table)
            .filter(demons::position.ge(starting_at))
            .set(demons::position.eq(demons::position + 1))
            .execute(connection)
            .map(|_| ())
    }

    /// Decrements the position of all demons with positions equal to or smaller than the given one,
    /// by one.
    pub fn shift_up(until: i16, connection: &PgConnection) -> QueryResult<()> {
        diesel::update(demons::table)
            .filter(demons::position.le(until))
            .set(demons::position.eq(demons::position - 1))
            .execute(connection)
            .map(|_| ())
    }

    pub fn mv(&mut self, to: i16, connection: &PgConnection) -> QueryResult<()> {
        if to == self.position {
            warn!("No-op move of demon {}", self.name);

            return Ok(())
        }

        // FIXME: Temporarily move the demon somewhere else because otherwise the unique constraints
        // complains. I actually dont know why, its DEFERRABLE INITIALLY IMMEDIATE (whatever the
        // fuck that means, it made it work in the python version)
        diesel::update(demons::table)
            .filter(demons::name.eq(&self.name))
            .set(demons::position.eq(-1))
            .execute(connection)?;

        if to > self.position {
            debug!(
                "Target position {} is greater than current position {}, shifting demons towards lower position",
                to, self.position
            );

            diesel::update(demons::table)
                .filter(demons::position.gt(self.position))
                .filter(demons::position.le(to))
                .set(demons::position.eq(demons::position - 1))
                .execute(connection)?;
        } else if to < self.position {
            debug!(
                "Target position {} is lesser than current position {}, shifting demons towards higher position",
                to, self.position
            );

            diesel::update(demons::table)
                .filter(demons::position.ge(to))
                .filter(demons::position.lt(self.position))
                .set(demons::position.eq(demons::position + 1))
                .execute(connection)?;
        }

        debug!("Performing actual move to position {}", to);

        diesel::update(demons::table)
            .filter(demons::name.eq(&self.name))
            .set(demons::position.eq(to))
            .execute(connection)?;

        self.position = to;

        Ok(())
    }

    pub fn max_position(connection: &PgConnection) -> Result<i16> {
        let option = demons::table
            .select(max(demons::position))
            .get_result::<Option<i16>>(connection)?;

        Ok(option.unwrap_or(0))
    }

    pub fn validate_requirement(requirement: &mut i16) -> Result<()> {
        if *requirement < 0 || *requirement > 100 {
            return Err(PointercrateError::InvalidRequirement)
        }

        Ok(())
    }

    pub fn validate_name(name: &mut CiString, connection: &PgConnection) -> Result<()> {
        *name = CiString(name.trim().to_string());

        match Demon::get(name.as_ref(), RequestContext::Internal(connection)) {
            Ok(demon) =>
                Err(PointercrateError::DemonExists {
                    position: demon.position,
                }),
            Err(PointercrateError::ModelNotFound { .. }) => Ok(()),
            Err(err) => Err(err),
        }
    }

    pub fn validate_position(position: &mut i16, connection: &PgConnection) -> Result<()> {
        let maximal = Demon::max_position(connection)? + 1;

        if *position < 1 || *position > maximal {
            return Err(PointercrateError::InvalidPosition { maximal })
        }

        Ok(())
    }

    pub fn validate_video(video: &mut String) -> Result<()> {
        *video = crate::video::validate(video)?;

        Ok(())
    }

    pub fn score(&self, progress: i16) -> f64 {
        let mut score =
            150f64 * f64::exp((1f64 - f64::from(self.position)) * (1f64 / 30f64).ln() / (-149f64));

        if progress != 100 {
            score *= 0.25f64
                + (f64::from(progress) - f64::from(self.requirement))
                    / (100f64 - f64::from(self.requirement))
                    * 0.25f64
        }

        score
    }
}

impl Into<PartialDemon> for Demon {
    fn into(self) -> PartialDemon {
        PartialDemon {
            name: self.name,
            position: self.position,
            publisher: self.publisher.name,
            video: self.video,
        }
    }
}

impl Into<EmbeddedDemon> for Demon {
    fn into(self) -> EmbeddedDemon {
        EmbeddedDemon {
            position: self.position,
            name: self.name,
        }
    }
}

impl Into<EmbeddedDemon> for PartialDemon {
    fn into(self) -> EmbeddedDemon {
        EmbeddedDemon {
            position: self.position,
            name: self.name,
        }
    }
}
