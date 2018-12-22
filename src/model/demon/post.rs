use super::Demon;
use crate::{
    error::PointercrateError,
    model::{creator::Creator, Player},
    operation::{Get, Post},
    schema::demons,
    video, Result,
};
use diesel::{
    dsl::max, insert_into, result::Error, select, Connection, Expression, ExpressionMethods,
    OptionalExtension, PgConnection, QueryDsl, RunQueryDsl,
};
use log::info;
use serde_derive::Deserialize;

#[derive(Deserialize, Debug)]
pub struct PostDemon {
    name: String,
    position: i16,
    requirement: i16,
    verifier: String,
    publisher: String,
    creators: Vec<String>,
    video: Option<String>,
}

#[derive(Insertable, Debug)]
#[table_name = "demons"]
pub struct NewDemon<'a> {
    name: &'a str,
    position: i16,
    requirement: i16,
    verifier: i32,
    publisher: i32,
    video: Option<&'a String>,
}

impl Post<PostDemon> for Demon {
    fn create_from(data: PostDemon, connection: &PgConnection) -> Result<Demon> {
        Demon::validate_requirement(data.requirement)?;

        let video = match data.video {
            Some(ref video) => Some(video::validate(video)?),
            None => None,
        };

        connection.transaction(|| {
            let existing = demons::table
                .select(demons::position)
                .filter(demons::name.eq(&data.name))
                .get_result(connection)
                .optional()?;

            if let Some(position) = existing {
                return Err(PointercrateError::DemonExists { position })
            }

            let maximal = Demon::max_position(connection)? + 1;

            if data.position < 1 || data.position > maximal {
                return Err(PointercrateError::InvalidPosition { maximal })
            }

            let publisher = Player::get(data.publisher, connection)?;
            let verifier = Player::get(data.verifier, connection)?;

            let new = NewDemon {
                name: &data.name,
                position: data.position,
                requirement: data.requirement,
                verifier: verifier.id,
                publisher: publisher.id,
                video: video.as_ref(),
            };

            Demon::shift_down(connection, new.position)?;

            let inserted_demon = insert_into(demons::table)
                .values(&new)
                .get_result::<Demon>(connection)?;

            for creator in data.creators {
                Creator::create_from((inserted_demon.name.clone(), creator), connection)?;
            }

            Ok(inserted_demon)
        })
    }
}
