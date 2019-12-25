pub use self::{
    paginate::{DemonIdPagination, DemonPagination},
    patch::PatchDemon,
    post::PostDemon,
};
use crate::{
    citext::{CiStr, CiString},
    context::RequestContext,
    error::PointercrateError,
    model::{
        demonlist::{creator::Creators, player::DatabasePlayer, record::MinimalRecordP},
        Model,
    },
    operation::Get,
    schema::demons,
    Result,
};
use derive_more::Display;
use diesel::{
    dsl::{max, Select},
    pg::Pg,
    Expression, ExpressionMethods, PgConnection, QueryDsl, QueryResult, Queryable, RunQueryDsl,
    Table,
};
use joinery::Joinable;
use log::{debug, info, warn};
use serde_derive::Serialize;
use std::hash::{Hash, Hasher};

mod get;
mod paginate;
mod patch;
mod post;

table! {
    use crate::citext::CiText;
    use diesel::sql_types::*;

    demons_pv (name) {
        id -> Int4,
        name -> CiText,
        position -> Int2,
        requirement -> Int2,
        video -> Nullable<Varchar>,
        publisher_id -> Int4,
        publisher_name -> CiText,
        publisher_banned -> Bool,
        verifier_id -> Int4,
        verifier_name -> CiText,
        verifier_banned -> Bool,
    }
}

/// Struct modelling a demon. These objects are returned from the paginating `/demons/` endpoint
#[derive(Debug, Serialize, Hash, Display, Eq, PartialEq, Identifiable)]
#[display(fmt = "{} (at {})", name, position)]
#[table_name = "demons_pv"]
pub struct Demon {
    /// The [`Demon`]'s internal pointercrate ID
    pub id: i32,

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

    /// The player-ID of this [`Demon`]'s publisher
    pub publisher: DatabasePlayer,

    /// The player-ID of this [`Demon`]'s verifier
    pub verifier: DatabasePlayer,
}

table! {
    use crate::citext::CiText;
    use diesel::sql_types::*;

    demons_p (name) {
        id -> Int4,
        name -> CiText,
        position -> Int2,
        video -> Nullable<Varchar>,
        publisher_id -> Int4,
        publisher_name -> CiText,
        publisher_banned -> Bool,
    }
}

/// Temporary solution. In the future this will become `ListedDemon` and contain
/// id, name, position, video and publisher name of all demons that have a non-null position
#[derive(Debug, Hash, Eq, PartialEq, Serialize, Display, Identifiable)]
#[display(fmt = "{} (at {})", name, position)]
#[table_name = "demons_p"]
pub struct MinimalDemonP {
    pub id: i32,
    pub name: CiString,
    pub position: i16,
    pub video: Option<String>,
    pub publisher: DatabasePlayer,
}

// doesn't need its own view

/// Absolutely minimal representation of a demon to be sent when a demon is part of another object
#[derive(Debug, Hash, Serialize, Queryable, Display)]
#[display(fmt = "{} (at {})", name, position)]
pub struct MinimalDemon {
    pub id: i32,
    pub position: i16,
    pub name: CiString,
}

// Since we dont have a custom view for this, we can't derive Identifiable
impl MinimalDemon {
    pub fn all() -> Select<demons::table, (demons::id, demons::position, demons::name)> {
        demons::table.select((demons::id, demons::position, demons::name))
    }
}

impl Model for Demon {
    type Selection = <demons_pv::table as Table>::AllColumns;

    fn selection() -> Self::Selection {
        demons_pv::all_columns
    }
}

impl Queryable<<<demons_pv::table as Table>::AllColumns as Expression>::SqlType, Pg> for Demon {
    #[allow(clippy::type_complexity)]
    type Row = (
        i32,
        CiString,
        i16,
        i16,
        Option<String>,
        i32,
        CiString,
        bool,
        i32,
        CiString,
        bool,
    );

    fn build(row: Self::Row) -> Self {
        Demon {
            id: row.0,
            name: row.1,
            position: row.2,
            requirement: row.3,
            video: row.4,
            publisher: DatabasePlayer {
                id: row.5,
                name: row.6,
                banned: row.7,
            },
            verifier: DatabasePlayer {
                id: row.8,
                name: row.9,
                banned: row.10,
            },
        }
    }
}

impl Model for MinimalDemonP {
    type Selection = <demons_p::table as Table>::AllColumns;

    fn selection() -> Self::Selection {
        demons_p::all_columns
    }
}

impl Queryable<<<demons_p::table as Table>::AllColumns as Expression>::SqlType, Pg>
    for MinimalDemonP
{
    type Row = (i32, CiString, i16, Option<String>, i32, CiString, bool);

    fn build(row: Self::Row) -> Self {
        MinimalDemonP {
            id: row.0,
            name: row.1,
            position: row.2,
            video: row.3,
            publisher: DatabasePlayer {
                id: row.4,
                name: row.5,
                banned: row.6,
            },
        }
    }
}

/// Struct modelling the "full" version of a demon.
///
/// In addition to containing publisher/verifier information it also contains a list of the demon's
/// creators and a list of accepted records
#[derive(Debug, Serialize, Display)]
#[display(fmt = "{}", demon)]
pub struct FullDemon {
    #[serde(flatten)]
    pub demon: Demon,
    pub creators: Creators,
    pub records: Vec<MinimalRecordP>,
}

impl Hash for FullDemon {
    fn hash<H: Hasher>(&self, state: &mut H) {
        // We only hash the demon here, because the creators don't matter for the ETag value - they
        // are modified through a different endpoint than the demon objects themselves, and
        // conflicting access to them is impossible anyway
        self.demon.hash(state)
    }
}

impl FullDemon {
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

impl Demon {
    by!(by_position, demons_pv::position, i16);

    by!(by_id, demons_pv::id, i32);

    // TODO: remove this
    by!(by_name, demons_pv::name, &CiStr);

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
            .filter(demons::id.eq(self.id))
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
            .filter(demons::id.eq(self.id))
            .set(demons::position.eq(to))
            .execute(connection)?;

        info!(
            "Moved demon {} from {} to {} successfully!",
            self.name, self.position, to
        );

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
