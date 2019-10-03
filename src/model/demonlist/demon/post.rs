use super::{Demon, FullDemon};
use crate::{
    citext::{CiStr, CiString},
    context::RequestContext,
    model::demonlist::{creator::Creator, player::DatabasePlayer},
    operation::{Get, Post},
    schema::demons,
    video, Result,
};
use diesel::{insert_into, Connection, RunQueryDsl};
use log::info;
use serde_derive::Deserialize;
use std::collections::HashSet;

#[derive(Deserialize, Debug)]
pub struct PostDemon {
    name: CiString,
    position: i16,
    requirement: i16,
    verifier: CiString,
    publisher: CiString,
    creators: Vec<CiString>,
    video: Option<String>,
}

#[derive(Insertable, Debug)]
#[table_name = "demons"]
pub struct NewDemon<'a> {
    name: &'a CiStr,
    position: i16,
    requirement: i16,
    verifier: i32,
    publisher: i32,
    video: Option<&'a String>,
}

impl Post<PostDemon> for Demon {
    fn create_from(mut data: PostDemon, ctx: RequestContext) -> Result<Demon> {
        ctx.check_permissions(perms!(ListModerator or ListAdministrator))?;

        let connection = ctx.connection();

        info!("Creating new demon from {:?}", data);

        Demon::validate_requirement(&mut data.requirement)?;

        let video = match data.video {
            Some(ref video) => Some(video::validate(video)?),
            None => None,
        };

        connection.transaction(|| {
            Demon::validate_name(&mut data.name, connection)?;
            Demon::validate_position(&mut data.position, connection)?;

            let publisher = DatabasePlayer::get(data.publisher.as_ref(), ctx)?;
            let verifier = DatabasePlayer::get(data.verifier.as_ref(), ctx)?;

            let new = NewDemon {
                name: data.name.as_ref(),
                position: data.position,
                requirement: data.requirement,
                verifier: verifier.id,
                publisher: publisher.id,
                video: video.as_ref(),
            };

            Demon::shift_down(new.position, connection)?;

            insert_into(demons::table)
                .values(&new)
                .execute(connection)?;

            let creators_hash: HashSet<CiString> = data.creators.into_iter().collect();

            for creator in creators_hash {
                Creator::create_from(
                    (data.name.as_ref(), creator.as_ref()),
                    RequestContext::Internal(connection),
                )?;
            }

            Ok(Demon {
                name: data.name,
                position: data.position,
                requirement: data.requirement,
                video: data.video,
                publisher,
                verifier,
            })
        })
    }
}

impl Post<PostDemon> for FullDemon {
    fn create_from(data: PostDemon, ctx: RequestContext) -> Result<Self> {
        FullDemon::get(Demon::create_from(data, ctx)?, ctx)
    }
}
