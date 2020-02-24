use crate::{
    cistring::CiString,
    error::PointercrateError,
    model::demonlist::{
        demon::{Demon, MinimalDemon},
        player::DatabasePlayer,
    },
    Result,
};
use futures::stream::StreamExt;
use serde::{Deserialize, Serialize};
use sqlx::{row::Row, PgConnection};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct DemonIdPagination {
    #[serde(rename = "before")]
    before_id: Option<i32>,

    #[serde(rename = "after")]
    after_id: Option<i32>,

    limit: Option<u8>,

    name: Option<CiString>,

    requirement: Option<i16>,

    verifier_id: Option<i32>,
    publisher_id: Option<i32>,

    verifier_name: Option<CiString>,
    publisher_name: Option<CiString>,

    #[serde(rename = "requirement__gt")]
    requirement_gt: Option<i16>,

    #[serde(rename = "requirement__lt")]
    requirement_lt: Option<i16>,
}

impl DemonIdPagination {
    pub async fn page(&self, connection: &mut PgConnection) -> Result<Vec<Demon>> {
        if let Some(limit) = self.limit {
            if limit < 1 || limit > 100 {
                return Err(PointercrateError::InvalidPaginationLimit)
            }
        }

        if let (Some(after), Some(before)) = (self.before_id, self.after_id) {
            if after < before {
                return Err(PointercrateError::AfterSmallerBefore)
            }
        }

        let mut sql_query = "SELECT demons.id AS demon_id, demons.name::text AS demon_name, demons.position, demons.requirement, \
                             demons.video::text,
               verifiers.id AS verifier_id, verifiers.name::text AS verifier_name, verifiers.banned AS verifier_banned,
               publishers.id AS publisher_id, publishers.name::text AS publisher_name, publishers.banned AS publisher_banned
        FROM demons
        INNER JOIN players AS verifiers ON verifiers.id=demons.verifier
        INNER JOIN players AS publishers ON publishers.id=demons.publisher
        WHERE "
            .to_string();

        let query = filter!(sql_query[
            demons.name = self.name,
            demons.requirement = self.requirement,
            demons.requirement < self.requirement_lt,
            demons.requirement > self.requirement_gt,
            verifiers.id = self.verifier_id,
            publishers.id = self.publisher_id,
            verifiers.name = self.verifier_name,
            publishers.name = self.publisher_name,
            demons.id > self.after_id,
            demons.id < self.before_id
        ] limit self.limit);

        let mut stream = query.fetch(connection);

        let mut demons = Vec::new();

        while let Some(row) = stream.next().await {
            let row = row?;

            let video: Option<String> = row.get("video");

            demons.push(Demon {
                base: MinimalDemon {
                    id: row.get("demon_id"),
                    name: CiString(row.get("demon_name")),
                    position: row.get("position"),
                },
                requirement: row.get("requirement"),
                video,
                publisher: DatabasePlayer {
                    id: row.get("publisher_id"),
                    name: CiString(row.get("publisher_name")),
                    banned: row.get("publisher_banned"),
                },
                verifier: DatabasePlayer {
                    id: row.get("verifier_id"),
                    name: CiString(row.get("verifier_name")),
                    banned: row.get("verifier_banned"),
                },
            })
        }

        Ok(demons)
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct DemonPositionPagination {
    #[serde(rename = "before")]
    before_position: Option<i16>,

    #[serde(rename = "after")]
    after_position: Option<i16>,

    limit: Option<u8>,

    name: Option<CiString>,

    requirement: Option<i16>,

    verifier_id: Option<i32>,
    publisher_id: Option<i32>,

    verifier_name: Option<CiString>,
    publisher_name: Option<CiString>,

    #[serde(rename = "requirement__gt")]
    requirement_gt: Option<i16>,

    #[serde(rename = "requirement__lt")]
    requirement_lt: Option<i16>,
}

impl DemonPositionPagination {
    pub async fn page(&self, connection: &mut PgConnection) -> Result<Vec<Demon>> {
        if let Some(limit) = self.limit {
            if limit < 1 || limit > 100 {
                return Err(PointercrateError::InvalidPaginationLimit)
            }
        }

        if let (Some(after), Some(before)) = (self.before_position, self.after_position) {
            if after < before {
                return Err(PointercrateError::AfterSmallerBefore)
            }
        }

        let mut sql_query = "SELECT demons.id AS demon_id, demons.name::text AS demon_name, demons.position, demons.requirement, \
                             demons.video::text, verifiers.id AS verifier_id, verifiers.name::text AS verifier_name, verifiers.banned AS \
                             verifier_banned,publishers.id AS publisher_id, publishers.name::text AS publisher_name, publishers.banned AS \
                             publisher_banned
        FROM demons
        INNER JOIN players AS verifiers ON verifiers.id=demons.verifier
        INNER JOIN players AS publishers ON publishers.id=demons.publisher
        WHERE "
            .to_string();

        let query = filter!(sql_query[
            demons.name = self.name,
            demons.requirement = self.requirement,
            demons.requirement < self.requirement_lt,
            demons.requirement > self.requirement_gt,
            verifiers.id = self.verifier_id,
            publishers.id = self.publisher_id,
            verifiers.name = self.verifier_name,
            publishers.name = self.publisher_name,
            demons.position > self.after_position,
            demons.position < self.before_position
        ] limit self.limit);

        let mut stream = query.fetch(connection);

        let mut demons = Vec::new();

        while let Some(row) = stream.next().await {
            let row = row?;

            let video: Option<String> = row.get("video");

            demons.push(Demon {
                base: MinimalDemon {
                    id: row.get("demon_id"),
                    name: CiString(row.get("demon_name")),
                    position: row.get("position"),
                },
                requirement: row.get("requirement"),
                video,
                publisher: DatabasePlayer {
                    id: row.get("publisher_id"),
                    name: CiString(row.get("publisher_name")),
                    banned: row.get("publisher_banned"),
                },
                verifier: DatabasePlayer {
                    id: row.get("verifier_id"),
                    name: CiString(row.get("verifier_name")),
                    banned: row.get("verifier_banned"),
                },
            })
        }

        Ok(demons)
    }
}
