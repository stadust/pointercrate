use dash_rs::{
    model::{
        creator::Creator,
        level::{Featured, Level, LevelData, LevelLength, Password},
        song::{MainSong, NewgroundsSong},
        GameVersion,
    },
    request::level::{LevelRequest, LevelRequestType, LevelsRequest, SearchFilters},
    response::{parse_download_gj_level_response, parse_get_gj_levels_response},
};
use log::{error, trace};
use pointercrate_core::ratelimits;
use pointercrate_demonlist::demon::Demon;
use reqwest::{header::CONTENT_TYPE, Client};
use sqlx::{Pool, Postgres};
use std::{borrow::Cow, sync::Arc};

pub use dash_rs::{
    model::level::{DemonRating, LevelRating},
    Thunk,
};
use reqwest::header::HeaderMap;

ratelimits! {
    IntegrationRatelimits {
        demon_refresh[1u32 per 86400 per i32] => "Only one refresh per day per demon",
        throttle[1u32 per 60] => "Wait at least 1 minute between level requests",
        throttle_throttle[1u32 per 600 per i32] => "Only hit the global throttle rate limit once per 10 minutes per demon",
    }
}

pub type IntegrationLevel = Level<'static, LevelData<'static>, Option<NewgroundsSong<'static>>>;

impl GeometryDashConnector {
    /// Attempts to pull the Geometry Dash level data for the given [`Demon`] from the database
    ///
    /// If the last time the data for this demon was sought on the Geomeetry Dash servers was over 24h ago,
    /// re-query them for updated data.
    pub async fn load_level_for_demon(&self, demon: &Demon) -> Option<IntegrationLevel> {
        if self.ratelimits.throttle_throttle(demon.base.id).is_ok()
            && self.ratelimits.throttle().is_ok()
            && self.ratelimits.demon_refresh(demon.base.id).is_ok()
        {
            tokio::spawn(self.clone().refresh_demon_data(demon.base.name.clone(), demon.base.id));
        }

        if let Some(level_id) = demon.level_id {
            let level = self
                .lookup_level(level_id)
                .await?
                .with_data(self.lookup_level_data(level_id).await?);

            let song = match level.custom_song {
                Some(id) => self.lookup_newgrounds_song(id).await,
                None => None,
            };

            return Some(level.with_custom_song(song));
        }

        None
    }

    pub async fn refresh_demon_data(self, name: String, demon_id: i32) {
        // Lookup demon by name
        let request = LevelsRequest::default()
            // Heuristic: list level have a lot of likes
            .request_type(LevelRequestType::MostLiked)
            .search(&name)
            // passing any `LevelRating::Demon` variant here will result in filtering by arbitrary demon difficulty
            .with_rating(LevelRating::Demon(DemonRating::Hard))
            .search_filters(SearchFilters::default().rated());

        let Ok(response) = self.make_request(request.to_url(), request.to_string()).await else {
            return;
        };
        let Ok(demons) = parse_get_gj_levels_response(&response) else {
            return;
        };
        let Some(mut hardest) = demons
            .into_iter()
            // Geometry Dash servers only do a substring match, so we have to ensure the name is equal to what we're looking for
            .filter(|demon| demon.name.trim().eq_ignore_ascii_case(name.trim()))
            .max_by(|x, y| x.difficulty.cmp(&y.difficulty))
        else {
            return;
        };

        let request = LevelRequest::new(hardest.level_id);
        let Ok(response) = self.make_request(request.to_url(), request.to_string()).await else {
            return;
        };
        let Ok(mut level) = parse_download_gj_level_response(&response) else {
            return;
        };

        if let Some(newgrounds_song) = &mut hardest.custom_song {
            self.store_newgrounds_song(newgrounds_song).await;
        }

        if let Some(creator) = &hardest.creator {
            self.store_creator(creator).await;
        }

        self.store_level(&hardest, level.creator, level.custom_song).await;
        self.store_level_data(level.level_id, &mut level.level_data).await;

        let _ = sqlx::query!("UPDATE demons SET level_id = $1 WHERE id = $2", level.level_id as i64, demon_id)
            .execute(&self.pool)
            .await;
    }

    async fn make_request(&self, url: String, body: String) -> Result<String, reqwest::Error> {
        let response = self.http_client
            .post(url)
              // boomlings.com rejects any request with a User-Agent header set, so make sure reqwest doesn't "helpfully" add one
            .headers(HeaderMap::new())
            .body(body)
            .header(CONTENT_TYPE, "application/x-www-form-urlencoded")
            .send()
            .await?;

        response.text().await
    }
}

// FIXME: Right now this implementation always stores processed data. In case of processing failure,
// it refuses to store the object. In the future, we probably want to store the unprocessed data
// then with a special flag.

#[derive(Clone)]
pub struct GeometryDashConnector {
    pool: Pool<Postgres>,
    http_client: Client,
    ratelimits: Arc<IntegrationRatelimits>,
}

impl GeometryDashConnector {
    pub fn new(pool: Pool<Postgres>) -> Self {
        GeometryDashConnector {
            pool,
            http_client: Client::new(),
            ratelimits: Arc::new(IntegrationRatelimits::new()),
        }
    }

    pub async fn lookup_creator(&self, user_id: u64) -> Option<Creator<'static>> {
        let mut connection = self.pool.acquire().await.ok()?;

        let creator_row = sqlx::query!("SELECT * FROM gj_creator WHERE user_id = $1", user_id as i64)
            .fetch_one(&mut *connection)
            .await
            .ok()?;

        Some(Creator {
            user_id: creator_row.user_id as u64,
            name: Cow::Owned(creator_row.name),
            account_id: creator_row.account_id.map(|id| id as u64),
        })
    }

    pub async fn store_creator(&self, creator: &Creator<'_>) {
        let Ok(mut connection) = self.pool.begin().await else { return };

        let _ = sqlx::query!(
            "INSERT INTO gj_creator (user_id, name, account_id) VALUES ($1, $2, $3) ON CONFLICT (user_id) DO UPDATE SET name = \
             EXCLUDED.name, account_id = EXCLUDED.account_id",
            creator.user_id as i64,
            creator.name.to_string(), // FIXME: figure out why it doesnt accept a reference
            creator.account_id.map(|id| id as i64)
        )
        .execute(&mut *connection)
        .await;

        let _ = connection.commit().await;
    }

    pub async fn lookup_newgrounds_song(&self, song_id: u64) -> Option<NewgroundsSong<'static>> {
        let mut connection = self.pool.acquire().await.ok()?;

        let song_row = sqlx::query!("SELECT * from gj_newgrounds_song WHERE song_id = $1", song_id as i64)
            .fetch_one(&mut *connection)
            .await
            .ok()?;

        Some(NewgroundsSong {
            song_id,
            name: Cow::Owned(song_row.song_name),
            index_3: song_row.index_3 as u64,
            artist: Cow::Owned(song_row.song_artist),
            filesize: song_row.filesize,
            index_6: song_row.index_6.map(Cow::Owned),
            index_7: song_row.index_7.map(Cow::Owned),
            index_8: Cow::Owned(song_row.index_8),
            link: Thunk::Processed(Cow::Owned(song_row.song_link)),
        })
    }

    pub async fn store_newgrounds_song(&self, song: &NewgroundsSong<'_>) {
        let Ok(mut connection) = self.pool.begin().await else { return };

        // FIXME: this
        let Ok(song_link) = song.link.as_processed() else { return };

        let _ = sqlx::query!(
            "INSERT INTO gj_newgrounds_song (song_id, song_name, index_3, song_artist, filesize, index_6, index_7, index_8, song_link) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9) ON CONFLICT (song_id) DO UPDATE SET song_id = $1, song_name = $2, index_3 = $3, \
             song_artist = $4, filesize = $5, index_6 = $6, index_7 = $7, index_8 = $8, song_link = $9",
            song.song_id as i64,
            song.name.as_ref(),
            song.index_3 as i64,
            song.artist.as_ref(),
            song.filesize,
            song.index_6.as_deref(),
            song.index_7.as_deref(),
            &song.index_8.as_ref(),
            song_link.as_ref()
        )
        .execute(&mut *connection)
        .await;

        let _ = connection.commit().await;
    }

    pub async fn lookup_level_data(&self, level_id: u64) -> Option<LevelData<'static>> {
        let mut connection = self.pool.acquire().await.ok()?;

        let row = sqlx::query!("SELECT * FROM gj_level_data WHERE level_id = $1", level_id as i64)
            .fetch_one(&mut *connection)
            .await
            .ok()?;

        Some(LevelData {
            level_data: Thunk::Processed(bincode::deserialize(&row.level_data[..]).unwrap()),
            password: Thunk::Processed(match row.level_password {
                None => Password::NoCopy,
                Some(-1) => Password::FreeCopy,
                Some(number) => Password::PasswordCopy(number as u32),
            }),
            time_since_upload: Cow::Owned(row.time_since_upload),
            time_since_update: Cow::Owned(row.time_since_update),
            index_36: Cow::Owned(row.index_36.unwrap_or(String::new())),
            index_40: Cow::Owned(String::new()), // TODO: maybe one day we'll care about these
            index_52: Cow::Owned(String::new()),
            index_53: Cow::Owned(String::new()),
            index_57: Cow::Owned(String::new()),
        })
    }

    pub async fn store_level_data(&self, level_id: u64, data: &mut LevelData<'_>) {
        let Ok(mut connection) = self.pool.begin().await else { return };

        // FIXME: this
        trace!("Starting to parse level data");
        let Ok(objects) = data.level_data.process() else {
            return error!("Error processing level data for {}", level_id);
        };
        let Ok(serialized_objects) = bincode::serialize(&objects) else {
            return;
        };
        trace!("Finished parsing level data");

        let Ok(password) = data.password.process() else { return };

        let _ = sqlx::query!(
            "INSERT INTO gj_level_data(level_id,level_data,level_password,time_since_upload,time_since_update,index_36) VALUES \
             ($1,$2,$3,$4,$5,$6) ON CONFLICT(level_id) DO UPDATE SET \
             level_id=EXCLUDED.level_id,level_data=EXCLUDED.level_data,level_password=EXCLUDED.level_password,time_since_upload=EXCLUDED.\
             time_since_upload,time_since_update=EXCLUDED.time_since_update,index_36=EXCLUDED.index_36",
            level_id as i64,
            serialized_objects,
            match password {
                Password::NoCopy => None,
                Password::FreeCopy => Some(-1),
                Password::PasswordCopy(pw) => Some(*pw as i32),
            },
            data.time_since_upload.as_ref(),
            data.time_since_update.as_ref(),
            data.index_36.as_ref()
        )
        .execute(&mut *connection)
        .await;

        let _ = connection.commit().await;
    }

    pub async fn lookup_level(&self, level_id: u64) -> Option<Level<'static, ()>> {
        let mut connection = self.pool.acquire().await.ok()?;

        let row = sqlx::query!("SELECT * FROM gj_level WHERE level_id = $1", level_id as i64)
            .fetch_one(&mut *connection)
            .await
            .ok()?;

        Some(Level {
            level_id,
            name: Cow::Owned(row.level_name),
            description: row.description.map(|description| Thunk::Processed(Cow::Owned(description))),
            version: row.level_version as u32,
            creator: row.creator_id as u64,
            difficulty: i16_to_level_rating(row.difficulty, row.is_demon),
            downloads: row.downloads as u32,
            main_song: row.main_song.map(|id| MainSong::from(id as u8)),
            gd_version: GameVersion::from(row.gd_version as u8),
            likes: row.likes,
            length: i16_to_level_length(row.level_length),
            stars: row.stars as u8,
            featured: Featured::from(row.featured),
            copy_of: row.copy_of.map(|id| id as u64),
            two_player: row.two_player,
            custom_song: row.custom_song_id.map(|id| id as u64),
            coin_amount: row.coin_amount as u8,
            coins_verified: row.coins_verified,
            stars_requested: row.stars_requested.map(|req| req as u8),
            is_epic: row.is_epic,
            object_amount: row.object_amount.map(|count| count as u32),
            index_46: row.index_46.map(Cow::Owned),
            index_47: row.index_47.map(Cow::Owned),
            level_data: (),
        })
    }

    // This must be the most horrifying piece of code I have ever written.
    async fn store_level<T, U, V>(&self, level: &Level<'_, T, U, V>, creator_id: u64, custom_song_id: Option<u64>) {
        let Ok(mut connection) = self.pool.begin().await else { return };

        let Ok(description) = level.description.as_ref().map(|thunk| thunk.as_processed()).transpose() else {
            return;
        };

        let _ = sqlx::query!(
            "INSERT INTO \
             gj_level(level_id,level_name,description,level_version,creator_id,difficulty,is_demon,downloads,main_song,gd_version,likes,\
             level_length,stars,featured,copy_of,two_player,custom_song_id,coin_amount,coins_verified,stars_requested,is_epic,\
             object_amount,index_46,index_47) VALUES \
             ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,$16,$17,$18,$19,$20,$21,$22,$23,$24) ON CONFLICT(level_id) DO UPDATE SET \
             level_id=EXCLUDED.level_id,level_name=EXCLUDED.level_name,description=EXCLUDED.description,level_version=EXCLUDED.\
             level_version,creator_id=EXCLUDED.creator_id,difficulty=EXCLUDED.difficulty,is_demon=EXCLUDED.is_demon,downloads=EXCLUDED.\
             downloads,main_song=EXCLUDED.main_song,gd_version=EXCLUDED.gd_version,likes=EXCLUDED.likes,level_length=EXCLUDED.\
             level_length,stars=EXCLUDED.stars,featured=EXCLUDED.featured,copy_of=EXCLUDED.copy_of,two_player=EXCLUDED.two_player,\
             custom_song_id=EXCLUDED.custom_song_id,coin_amount=EXCLUDED.coin_amount,coins_verified=EXCLUDED.coins_verified,\
             stars_requested=EXCLUDED.stars_requested,is_epic=EXCLUDED.is_epic,object_amount=EXCLUDED.object_amount,index_46=EXCLUDED.\
             index_46,index_47=EXCLUDED.index_47",
            level.level_id as i64,
            level.name.as_ref(),
            description.map(|cow| cow.to_string()),
            level.version as i32,
            creator_id as i64,
            level_rating_to_i16(level.difficulty),
            level.difficulty.is_demon(),
            level.downloads as i32,
            level.main_song.map(|song| song.main_song_id as i16),
            u8::from(level.gd_version) as i16,
            level.likes,
            level_length_to_i16(level.length),
            level.stars as i16,
            i32::from(level.featured),
            level.copy_of.map(|id| id as i64),
            level.two_player,
            custom_song_id.map(|id| id as i64),
            level.coin_amount as i16,
            level.coins_verified,
            level.stars_requested.map(|req| req as i16),
            level.is_epic,
            level.object_amount.map(|objects| objects as i32),
            level.index_46.as_deref(),
            level.index_47.as_deref()
        )
        .execute(&mut *connection)
        .await;

        let _ = connection.commit().await;
    }
}

fn level_rating_to_i16(level_rating: LevelRating) -> i16 {
    match level_rating {
        LevelRating::Unknown(unknown) => (unknown as i16) * 100,
        LevelRating::Auto => 1,
        LevelRating::Easy => 2,
        LevelRating::Normal => 3,
        LevelRating::Hard => 4,
        LevelRating::Harder => 5,
        LevelRating::Insane => 6,
        LevelRating::NotAvailable => 7,
        LevelRating::Demon(demon_rating) => match demon_rating {
            DemonRating::Unknown(unknown) => (unknown as i16) * 100,
            DemonRating::Easy => 1,
            DemonRating::Medium => 2,
            DemonRating::Hard => 3,
            DemonRating::Insane => 4,
            DemonRating::Extreme => 5,
        },
    }
}

fn i16_to_level_rating(value: i16, is_demon: bool) -> LevelRating {
    if value.abs() >= 100 || value == 0 {
        if is_demon {
            return LevelRating::Demon(DemonRating::Unknown((value / 100) as i32));
        } else {
            return LevelRating::Unknown((value / 100) as i32);
        }
    }

    if !is_demon {
        match value {
            1 => LevelRating::Auto,
            2 => LevelRating::Easy,
            3 => LevelRating::NotAvailable,
            4 => LevelRating::Hard,
            5 => LevelRating::Harder,
            6 => LevelRating::Insane,
            7 => LevelRating::NotAvailable,
            _ => unreachable!(),
        }
    } else {
        LevelRating::Demon(match value {
            1 => DemonRating::Easy,
            2 => DemonRating::Medium,
            3 => DemonRating::Hard,
            4 => DemonRating::Insane,
            5 => DemonRating::Extreme,
            _ => unreachable!(),
        })
    }
}

fn level_length_to_i16(length: LevelLength) -> i16 {
    match length {
        LevelLength::Unknown(value) => (value as i16) * 100,
        LevelLength::Tiny => 1,
        LevelLength::Short => 2,
        LevelLength::Medium => 3,
        LevelLength::Long => 4,
        LevelLength::ExtraLong => 5,
        LevelLength::Platformer => 6,
    }
}

fn i16_to_level_length(value: i16) -> LevelLength {
    if value.abs() >= 100 || value == 0 {
        return LevelLength::Unknown((value / 100) as i32);
    }

    match value {
        1 => LevelLength::Tiny,
        2 => LevelLength::Short,
        3 => LevelLength::Medium,
        4 => LevelLength::Long,
        5 => LevelLength::ExtraLong,
        6 => LevelLength::Platformer,
        _ => unreachable!(),
    }
}
