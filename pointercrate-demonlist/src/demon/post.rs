use crate::{
    creator::Creator,
    demon::{Demon, FullDemon, MinimalDemon},
    error::Result,
    player::{recompute_scores, DatabasePlayer},
};
use log::info;
use serde::Deserialize;
use sqlx::PgConnection;

#[derive(Deserialize, Debug)]
pub struct PostDemon {
    name: String,
    position: i16,
    requirement: i16,
    verifier: String,
    publisher: String,
    creators: Vec<String>,
    video: Option<String>,
    level_id: Option<i64>,
}

impl FullDemon {
    /// Must be run within a transaction!
    pub async fn create_from(data: PostDemon, connection: &mut PgConnection) -> Result<FullDemon> {
        info!("Creating new demon from {:?}", data);

        Demon::validate_requirement(data.requirement)?;
        let level_id = data.level_id.map(Demon::validate_level_id).transpose()?;

        let video = match data.video {
            Some(ref video) => Some(crate::video::validate(video)?),
            None => None,
        };

        Demon::validate_position(data.position, connection).await?;

        let publisher = DatabasePlayer::by_name_or_create(data.publisher.as_ref(), connection).await?;
        let verifier = DatabasePlayer::by_name_or_create(data.verifier.as_ref(), connection).await?;

        Demon::shift_down(data.position, connection).await?;

        let created = sqlx::query!(
            "INSERT INTO demons (name, position, requirement, video, verifier, publisher, level_id) VALUES ($1::text,$2,$3,$4::text,$5,$6, $7) \
             RETURNING id, thumbnail",
            data.name.to_string(),
            data.position,
            data.requirement,
            video.as_ref(),
            verifier.id,
            publisher.id,
            data.level_id
        )
        .fetch_one(&mut *connection)
        .await?;

        let demon = Demon {
            base: MinimalDemon {
                id: created.id,
                position: data.position,
                name: data.name,
            },
            requirement: data.requirement,
            video,
            thumbnail: created.thumbnail,
            publisher,
            verifier,
            level_id,
        };

        let mut creators = Vec::new();

        for creator in data.creators {
            let player = DatabasePlayer::by_name_or_create(creator.as_ref(), &mut *connection).await?;
            Creator::insert(&demon.base, &player, connection).await?;

            creators.push(player);
        }

        recompute_scores(connection).await?;

        Ok(FullDemon {
            demon,
            creators,
            records: Vec::new(),
        })
    }
}

#[cfg(test)]
mod tests {
    use sqlx::{pool::PoolConnection, Postgres};

    use crate::{
        demon::{FullDemon, PostDemon},
        error::DemonlistError,
    };

    const DEFAULT_THUMBNAIL: &str = "https://i.ytimg.com/vi/zebrafishes/mqdefault.jpg";

    #[sqlx::test(migrations = "../migrations")]
    async fn test_default_thumbnail_no_video(mut conn: PoolConnection<Postgres>) {
        let demon = FullDemon::create_from(
            PostDemon {
                name: "Bloodbath".to_owned(),
                position: 1,
                requirement: 90,
                verifier: "Riot".to_owned(),
                publisher: "Riot".to_owned(),
                creators: Vec::new(),
                video: None,
                level_id: None,
            },
            &mut conn,
        )
        .await
        .unwrap();

        assert_eq!(demon.demon.thumbnail, DEFAULT_THUMBNAIL);
    }

    #[sqlx::test(migrations = "../migrations")]
    async fn test_default_thumbnail_linked_banned(mut conn: PoolConnection<Postgres>) {
        sqlx::query!("INSERT INTO players (name, link_banned) VALUES ('Riot', TRUE)")
            .execute(&mut *conn)
            .await
            .unwrap();

        let demon = FullDemon::create_from(
            PostDemon {
                name: "Bloodbath".to_owned(),
                position: 1,
                requirement: 90,
                verifier: "Riot".to_owned(),
                publisher: "Riot".to_owned(),
                creators: Vec::new(),
                video: Some("https://www.youtube.com/watch?v=dQw4w9WgXcQ".to_owned()),
                level_id: None,
            },
            &mut conn,
        )
        .await
        .unwrap();

        assert_eq!(demon.demon.thumbnail, DEFAULT_THUMBNAIL);
    }

    #[sqlx::test(migrations = "../migrations")]
    async fn test_default_thumbnail_with_video(mut conn: PoolConnection<Postgres>) {
        let demon = FullDemon::create_from(
            PostDemon {
                name: "Bloodbath".to_owned(),
                position: 1,
                requirement: 90,
                verifier: "Riot".to_owned(),
                publisher: "Riot".to_owned(),
                creators: Vec::new(),
                video: Some("https://www.youtube.com/watch?v=dQw4w9WgXcQ".to_owned()),
                level_id: None,
            },
            &mut conn,
        )
        .await
        .unwrap();

        assert_eq!(demon.demon.thumbnail, "https://i.ytimg.com/vi/dQw4w9WgXcQ/mqdefault.jpg");
    }

    #[sqlx::test(migrations = "../migrations")]
    async fn test_invalid_level_id(mut conn: PoolConnection<Postgres>) {
        let error = FullDemon::create_from(
            PostDemon {
                name: "Bloodbath".to_owned(),
                position: 1,
                requirement: 90,
                verifier: "Riot".to_owned(),
                publisher: "Riot".to_owned(),
                creators: Vec::new(),
                video: None,
                level_id: Some(-1),
            },
            &mut conn,
        )
        .await
        .unwrap_err();

        assert_eq!(error, DemonlistError::InvalidLevelId);
    }
}
