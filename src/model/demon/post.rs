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
    fn create_from(mut data: PostDemon, connection: &PgConnection) -> Result<Demon> {
        Demon::validate_requirement(&mut data.requirement)?;

        let video = match data.video {
            Some(ref video) => Some(video::validate(video)?),
            None => None,
        };

        connection.transaction(|| {
            Demon::validate_name(&mut data.name, connection)?;
            Demon::validate_position(&mut data.position, connection)?;

            let publisher = Player::get(&data.publisher, connection)?;
            let verifier = Player::get(&data.verifier, connection)?;

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

            for creator in &data.creators {
                Creator::create_from((&inserted_demon.name[..], &creator[..]), connection)?;
            }

            Ok(inserted_demon)
        })
    }
}
