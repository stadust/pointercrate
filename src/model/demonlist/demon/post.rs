use crate::{
    cistring::CiString,
    model::demonlist::{
        creator::Creator,
        demon::{Demon, FullDemon, MinimalDemon},
        player::DatabasePlayer,
    },
    Result,
};
use log::info;
use serde::Deserialize;
use sqlx::PgConnection;

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

impl FullDemon {
    /// Must be run within a transaction!
    pub async fn create_from(data: PostDemon, connection: &mut PgConnection) -> Result<FullDemon> {
        info!("Creating new demon from {:?}", data);

        Demon::validate_requirement(data.requirement)?;

        let video = match data.video {
            Some(ref video) => Some(crate::video::validate(video)?),
            None => None,
        };

        Demon::validate_position(data.position, connection).await?;

        let publisher = DatabasePlayer::by_name_or_create(data.publisher.as_ref(), connection).await?;
        let verifier = DatabasePlayer::by_name_or_create(data.verifier.as_ref(), connection).await?;

        Demon::shift_down(data.position, connection).await?;

        let id_of_inserted = sqlx::query!(
            "INSERT INTO demons (name, position, requirement, video, verifier, publisher) VALUES ($1::text,$2,$3,$4::text,$5,$6) \
             RETURNING id",
            data.name.to_string(),
            data.position,
            data.requirement,
            video.as_ref().map(AsRef::as_ref).unwrap_or("").to_string(), // FIXME(sqlx)
            verifier.id,
            publisher.id
        )
        .fetch_one(connection)
        .await?
        .id;

        let demon = Demon {
            base: MinimalDemon {
                id: id_of_inserted,
                position: data.position,
                name: data.name,
            },
            requirement: data.requirement,
            video,
            publisher,
            verifier,
        };

        let mut creators = Vec::new();

        for creator in data.creators {
            let player = DatabasePlayer::by_name_or_create(creator.as_ref(), connection).await?;
            Creator::insert(&demon.base, &player, connection).await?;

            creators.push(player);
        }

        Ok(FullDemon {
            demon,
            creators,
            records: Vec::new(),
        })
    }
}
