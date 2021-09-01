use chrono::{DateTime, Duration, NaiveDateTime, Utc};
use dash_rs::{
    model::{
        creator::Creator,
        level::{Featured, Level, LevelData, LevelLength, ListedLevel, Objects, Password},
        song::{MainSong, NewgroundsSong},
        GameVersion,
    },
    request::level::{LevelRequest, LevelRequestType, LevelsRequest, SearchFilters},
    response::ResponseError,
    Base64Decoded, PercentDecoded, ProcessError, ThunkContent,
};
use futures::{FutureExt, StreamExt};
use log::{error, info, trace};
use reqwest::{header::CONTENT_TYPE, Client};
use sqlx::{Error, Pool, Postgres};
use std::{
    borrow::Cow,
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
};

pub use dash_rs::{
    model::level::{DemonRating, LevelRating},
    Thunk,
};

#[derive(Debug)]
pub enum GDIntegrationResult {
    Success(Level<'static, ()>, LevelData<'static>, Option<NewgroundsSong<'static>>),
    DemonNotFoundByName,
    DemonNotYetCached,
    LevelDataNotFound,
    LevelDataNotCached,
}

impl PgCache {
    pub async fn data_for_demon(
        &self, http_client: Client, level_id: Option<u64>, name: String, demon_id: i32,
    ) -> Result<GDIntegrationResult, ()> {
        trace!("Retrieving data for demon {:?}", name);

        match level_id {
            None => {
                info!("Data for demon {} not cached, trying to download!", demon_id);

                let request = LevelsRequest::default()
                    .request_type(LevelRequestType::MostLiked)
                    .search(&name)
                    .with_rating(LevelRating::Demon(DemonRating::Hard))
                    .search_filters(SearchFilters::default().rated());

                let entry = match self.lookup_levels_request(&request).await {
                    Err(CacheError::Db(_err)) => return Err(()),
                    Err(_) => return Err(()), // shouldn't be reachable
                    Ok(entry) => entry,
                };

                match entry {
                    CacheEntry::Absent => Ok(GDIntegrationResult::DemonNotFoundByName),
                    // Okay we _could_ do something more elaborate here if we dont have a "Missing" variant, but honestly I dont care
                    _ => {
                        tokio::spawn(self.clone().find_demon(http_client, name.clone(), demon_id).map(|_| ()));

                        Ok(GDIntegrationResult::DemonNotYetCached)
                    },
                }
            },
            Some(level_id) => {
                let entry = match self.lookup_level(level_id).await {
                    Err(CacheError::Db(_err)) => return Err(()),
                    Err(_) => return Err(()), // shouldn't be reachable
                    Ok(entry) => entry,
                };

                trace!(
                    "Lookup of level with id {} (associated with demon '{}') yielded: {:?}",
                    level_id,
                    name,
                    entry
                );

                let level = match entry {
                    // Something went very wrong
                    CacheEntry::Missing => return Ok(GDIntegrationResult::LevelDataNotCached),
                    CacheEntry::Absent => return Ok(GDIntegrationResult::LevelDataNotFound),
                    CacheEntry::Expired(level, _) => {
                        tokio::spawn(self.clone().find_demon(http_client.clone(), name.clone(), demon_id).map(|_| ()));

                        level
                    },
                    CacheEntry::Live(level, _) => level,
                };

                let level_data = match self.lookup_level_data(level_id).await {
                    Err(CacheError::Db(_err)) => return Err(()),
                    Err(_) => return Err(()), // shouldn't be reachable
                    Ok(CacheEntry::Missing) => return Ok(GDIntegrationResult::LevelDataNotCached),
                    Ok(CacheEntry::Absent) => return Ok(GDIntegrationResult::LevelDataNotFound),
                    Ok(CacheEntry::Expired(level_data, _)) => {
                        tokio::spawn(
                            self.clone()
                                .download_demon(http_client, level.level_id.into(), demon_id)
                                .map(|_| ()),
                        );

                        level_data
                    },
                    Ok(CacheEntry::Live(level_data, _)) => level_data,
                };

                let song = match level.custom_song {
                    Some(id) =>
                        self.lookup_newgrounds_song(id)
                            .await
                            .ok()
                            .map(|entry| {
                                match entry {
                                    CacheEntry::Expired(song, _) | CacheEntry::Live(song, _) => Some(song),
                                    _ => None,
                                }
                            })
                            .flatten(),
                    None => None,
                };

                Ok(GDIntegrationResult::Success(level, level_data, song))
            },
        }
    }

    async fn find_demon(self, http_client: Client, demon_name: String, demon: i32) -> Result<(), ()> {
        let request = LevelsRequest::default()
            .request_type(LevelRequestType::MostLiked)
            .search(&demon_name)
            .with_rating(LevelRating::Demon(DemonRating::Hard))
            .search_filters(SearchFilters::default().rated());

        trace!("Trying to find demon {} via request {:?}", demon_name, request);

        let request_result = http_client
            .post(&request.to_url())
            .body(request.to_string())
            .header(CONTENT_TYPE, "application/x-www-form-urlencoded")
            .send()
            .await;

        trace!("Request result is {:?}", request_result);

        match request_result {
            Ok(response) =>
                match response.text().await {
                    Ok(text) =>
                        match dash_rs::response::parse_get_gj_levels_response(&text[..]) {
                            Err(ResponseError::NotFound) =>
                                self.mark_levels_request_result_as_absent(&request)
                                    .await
                                    .map_err(|err| error!("Error marking result to {:?} as absent:  {:?}", request, err))
                                    .map(|_| ()),
                            Ok(demons) =>
                                if demons.len() == 0 {
                                    self.mark_levels_request_result_as_absent(&request)
                                        .await
                                        .map_err(|err| error!("Error marking result to {:?} as absent:  {:?}", request, err))
                                        .map(|_| ())
                                } else {
                                    trace!("Request to find demon {} yielded result {:?}", demon_name, demons);

                                    self.store_levels_request(&request, &demons)
                                        .await
                                        .map_err(|err| error!("Error storing levels request result: {:?}", err))?;

                                    let hardest = demons
                                        .into_iter()
                                        .filter(|demon| demon.name.to_lowercase() == demon_name.to_lowercase())
                                        .max_by(|x, y| x.difficulty.cmp(&y.difficulty));

                                    match hardest {
                                        Some(hardest) => {
                                            trace!("The hardest demon I could find with name '{}' is {:?}", demon_name, hardest);

                                            self.download_demon(http_client, hardest.level_id.into(), demon).await
                                        },
                                        None => {
                                            error!("Could not find a level whose name matches '{}'", demon_name);

                                            Err(())
                                        },
                                    }
                                },
                            Err(err) => Err(error!("Error processing response to request {:?}: {:?}", request, err)),
                        },
                    Err(error) => Err(error!("Error reading server response: {:?}", error)),
                },
            Err(error) => Err(error!("Error making request: {:?}", error)),
        }
    }

    async fn download_demon(self, http_client: Client, request: LevelRequest<'static>, demon_id: i32) -> Result<(), ()> {
        trace!("Downloading demon with id {}", request.level_id);

        let request_result = http_client
            .post(&request.to_url())
            .body(request.to_string())
            .header(CONTENT_TYPE, "application/x-www-form-urlencoded")
            .send()
            .await;

        match request_result {
            Ok(response) => {
                let content = response.text().await;

                match content {
                    Ok(text) =>
                        match dash_rs::response::parse_download_gj_level_response(&text[..]) {
                            Ok(demon) => {
                                self.store_level_data(demon.level_id, &demon.level_data)
                                    .await
                                    .map_err(|err| error!("Error storing demon '{}': {:?}", demon.name, err))?;

                                sqlx::query!("UPDATE demons SET level_id = $1 WHERE id = $2", request.level_id as i64, demon_id)
                                    .execute(&self.pool)
                                    .await
                                    .map_err(|err| error!("Error updating level_id: {:?}", err))?;

                                sqlx::query!("DELETE FROM download_lock WHERE level_id = $1", request.level_id as i64)
                                    .execute(&self.pool)
                                    .await
                                    .map_err(|err| error!("Error freeing download lock: {:?}", err))?;

                                Ok(info!("Successfully retrieved demon data!"))
                            },
                            Err(ResponseError::NotFound) =>
                                self.mark_level_data_as_absent(request.level_id)
                                    .await
                                    .map_err(|err| error!("Error marking level as absent: {:?}", err))
                                    .map(|_| ()),
                            Err(err) => Err(error!("Error processing response: {:?}", err)),
                        },
                    Err(err) => Err(error!("Error making http request: {:?}", err)),
                }
            },
            Err(err) => Err(error!("Error making http request: {:?}", err)),
        }
    }
}

// FIXME: Right now this implementation always stores processed data. In case of processing failure,
// it refuses to store the object. In the future, we probably want to store the unprocessed data
// then with a special flag. However, this is not yet supported by dash-rs, as dash-rs doesnt
// support owned, unprocessed data.

#[derive(Debug)]
pub struct CacheEntryMeta {
    made: NaiveDateTime,
    key: i64,
    absent: bool,
}

#[derive(Debug)]
pub enum CacheError {
    Db(Error),
    Malformed(ProcessError),
    MalformedLevelData,
}

impl From<Error> for CacheError {
    fn from(err: Error) -> Self {
        CacheError::Db(err)
    }
}

impl From<ProcessError> for CacheError {
    fn from(err: ProcessError) -> Self {
        CacheError::Malformed(err)
    }
}

#[derive(Debug)]
pub enum CacheEntry<T> {
    Missing,
    Absent,
    Expired(T, CacheEntryMeta),
    Live(T, CacheEntryMeta),
}

#[derive(Clone)]
pub struct PgCache {
    pool: Pool<Postgres>,
    expire_after: Duration,
}

impl PgCache {
    pub fn new(pool: Pool<Postgres>, expire_after: Duration) -> Self {
        PgCache { pool, expire_after }
    }

    fn make_cache_entry<T>(&self, meta: CacheEntryMeta, t: T) -> CacheEntry<T> {
        if DateTime::<Utc>::from_utc(meta.made, Utc) - Utc::now() < self.expire_after {
            CacheEntry::Live(t, meta)
        } else {
            CacheEntry::Expired(t, meta)
        }
    }

    pub async fn lookup_creator(&self, user_id: u64) -> Result<CacheEntry<Creator<'static>>, CacheError> {
        let mut connection = self.pool.acquire().await?;

        let meta = sqlx::query_as!(
            CacheEntryMeta,
            "SELECT user_id AS key, cached_at AS made, absent FROM gj_creator_meta WHERE user_id = $1",
            user_id as i64
        )
        .fetch_one(&mut *connection)
        .await;

        let meta = match meta {
            Err(sqlx::Error::RowNotFound) => return Ok(CacheEntry::Missing),
            Err(err) => return Err(err.into()),
            Ok(meta) if meta.absent => return Ok(CacheEntry::Absent),
            Ok(meta) => meta,
        };

        let creator_row = sqlx::query!("SELECT * FROM gj_creator WHERE user_id = $1", user_id as i64)
            .fetch_one(&mut *connection)
            .await?;

        let creator = Creator {
            user_id: creator_row.user_id as u64,
            name: Cow::Owned(creator_row.name),
            account_id: creator_row.account_id.map(|id| id as u64),
        };

        Ok(self.make_cache_entry(meta, creator))
    }

    pub async fn store_creator<'a>(&self, creator: &Creator<'a>) -> Result<CacheEntryMeta, CacheError> {
        let mut connection = self.pool.begin().await?;

        let meta = sqlx::query_as!(
            CacheEntryMeta,
            "INSERT INTO gj_creator_meta (user_id, cached_at, absent) VALUES($1, $2, FALSE) ON CONFLICT (user_id) DO UPDATE SET cached_at \
             = EXCLUDED.cached_at, absent = FALSE RETURNING user_id AS key, cached_at AS made, absent",
            creator.user_id as i64,
            Utc::now().naive_utc()
        )
        .fetch_one(&mut connection)
        .await?;

        sqlx::query!(
            "INSERT INTO gj_creator (user_id, name, account_id) VALUES ($1, $2, $3) ON CONFLICT (user_id) DO UPDATE SET name = \
             EXCLUDED.name, account_id = EXCLUDED.account_id",
            creator.user_id as i64,
            creator.name.to_string(), // FIXME: figure out why it doesnt accept a reference
            creator.account_id.map(|id| id as i64)
        )
        .execute(&mut connection)
        .await?;

        connection.commit().await?;

        Ok(meta)
    }

    pub async fn lookup_newgrounds_song(&self, song_id: u64) -> Result<CacheEntry<NewgroundsSong<'static>>, CacheError> {
        let mut connection = self.pool.acquire().await?;

        let meta = sqlx::query_as!(
            CacheEntryMeta,
            "SELECT song_id AS key, cached_at AS made, absent FROM gj_newgrounds_song_meta WHERE song_id = $1",
            song_id as i64
        )
        .fetch_one(&mut *connection)
        .await;

        let meta = match meta {
            Err(sqlx::Error::RowNotFound) => return Ok(CacheEntry::Missing),
            Err(err) => return Err(err.into()),
            Ok(meta) if meta.absent => return Ok(CacheEntry::Absent),
            Ok(meta) => meta,
        };

        let song_row = sqlx::query!("SELECT * from gj_newgrounds_song WHERE song_id = $1", song_id as i64)
            .fetch_one(&mut *connection)
            .await?;

        let song = NewgroundsSong {
            song_id,
            name: Cow::Owned(song_row.song_name),
            index_3: song_row.index_3 as u64,
            artist: Cow::Owned(song_row.song_artist),
            filesize: song_row.filesize,
            index_6: song_row.index_6.map(Cow::Owned),
            index_7: song_row.index_7.map(Cow::Owned),
            index_8: Cow::Owned(song_row.index_8),
            link: Thunk::Processed(PercentDecoded(Cow::Owned(song_row.song_link))),
        };

        Ok(self.make_cache_entry(meta, song))
    }

    pub async fn store_newgrounds_song<'a>(&self, song: &NewgroundsSong<'a>) -> Result<CacheEntryMeta, CacheError> {
        let mut connection = self.pool.begin().await?;

        let meta = sqlx::query_as!(
            CacheEntryMeta,
            "INSERT INTO gj_newgrounds_song_meta (song_id, cached_at, absent) VALUES ($1, $2, FALSE) ON CONFLICT (song_id) DO UPDATE SET \
             cached_at = EXCLUDED.cached_at, absent = FALSE RETURNING song_id AS key, cached_at AS made, absent",
            song.song_id as i64,
            Utc::now().naive_utc()
        )
        .fetch_one(&mut connection)
        .await?;

        // FIXME: this
        let song_link = match song.link {
            Thunk::Unprocessed(unprocessed) => PercentDecoded::from_unprocessed(unprocessed)?.0.to_string(),
            Thunk::Processed(ref link) => link.0.to_string(),
        };

        sqlx::query!(
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
            song_link
        )
        .execute(&mut connection)
        .await?;

        connection.commit().await?;

        Ok(meta)
    }

    pub async fn mark_level_data_as_absent<'a>(&self, level_id: u64) -> Result<CacheEntryMeta, CacheError> {
        let mut connection = self.pool.begin().await?;

        let meta = sqlx::query_as!(
            CacheEntryMeta,
            "INSERT INTO gj_level_data_meta (level_id, cached_at, absent) VALUES ($1, $2, TRUE) ON CONFLICT (level_id) DO UPDATE SET \
             cached_at = EXCLUDED.cached_at, absent = TRUE RETURNING level_id AS key, cached_at AS made, absent",
            level_id as i64,
            Utc::now().naive_utc()
        )
        .fetch_one(&mut connection)
        .await?;

        connection.commit().await?;

        Ok(meta)
    }

    pub async fn lookup_level_data<'a>(&self, level_id: u64) -> Result<CacheEntry<LevelData<'static>>, CacheError> {
        let mut connection = self.pool.acquire().await?;

        let meta = sqlx::query_as!(
            CacheEntryMeta,
            "SELECT level_id AS key, cached_at AS made, absent FROM gj_level_data_meta WHERE level_id = $1",
            level_id as i64
        )
        .fetch_one(&mut *connection)
        .await;

        let meta = match meta {
            Err(sqlx::Error::RowNotFound) => return Ok(CacheEntry::Missing),
            Err(err) => return Err(err.into()),
            Ok(meta) if meta.absent => return Ok(CacheEntry::Absent),
            Ok(meta) => meta,
        };

        let row = sqlx::query!("SELECT * FROM gj_level_data WHERE level_id = $1", level_id as i64)
            .fetch_one(&mut *connection)
            .await?;

        let level = LevelData {
            level_data: Thunk::Processed(bincode::deserialize(&row.level_data[..]).unwrap()),
            password: match row.level_password {
                None => Password::NoCopy,
                Some(-1) => Password::FreeCopy,
                Some(number) => Password::PasswordCopy(number as u32),
            },
            time_since_upload: Cow::Owned(row.time_since_upload),
            time_since_update: Cow::Owned(row.time_since_update),
            index_36: row.index_36.map(Cow::Owned),
        };

        Ok(self.make_cache_entry(meta, level))
    }

    pub async fn store_level_data<'a>(&self, level_id: u64, data: &LevelData<'a>) -> Result<CacheEntryMeta, CacheError> {
        let mut connection = self.pool.begin().await?;

        let meta = sqlx::query_as!(
            CacheEntryMeta,
            "INSERT INTO gj_level_data_meta (level_id, cached_at, absent) VALUES ($1, $2, FALSE) ON CONFLICT (level_id) DO UPDATE SET \
             cached_at = EXCLUDED.cached_at, absent = FALSE RETURNING level_id AS key, cached_at AS made, absent",
            level_id as i64,
            Utc::now().naive_utc()
        )
        .fetch_one(&mut connection)
        .await?;

        // FIXME: this
        trace!("Starting to parse level data");
        let objects = match data.level_data {
            Thunk::Unprocessed(unprocessed) => {
                let processed = Objects::from_unprocessed(unprocessed).map_err(|err| {
                    error!("Error processing level data: {:?}", err);

                    CacheError::MalformedLevelData
                })?;

                bincode::serialize(&processed)
            },
            Thunk::Processed(ref proc) => bincode::serialize(proc),
        }
        .map_err(|err| {
            error!("Error binary serializing level data: {:?}", err);

            CacheError::MalformedLevelData
        })?;
        trace!("Finished parsing level data");

        sqlx::query!(
            "INSERT INTO gj_level_data(level_id,level_data,level_password,time_since_upload,time_since_update,index_36) VALUES \
             ($1,$2,$3,$4,$5,$6) ON CONFLICT(level_id) DO UPDATE SET \
             level_id=EXCLUDED.level_id,level_data=EXCLUDED.level_data,level_password=EXCLUDED.level_password,time_since_upload=EXCLUDED.\
             time_since_upload,time_since_update=EXCLUDED.time_since_update,index_36=EXCLUDED.index_36",
            level_id as i64,
            objects,
            match data.password {
                Password::NoCopy => None,
                Password::FreeCopy => Some(-1),
                Password::PasswordCopy(pw) => Some(pw as i32),
            },
            data.time_since_upload.as_ref(),
            data.time_since_update.as_ref(),
            data.index_36.as_deref()
        )
        .execute(&mut *connection)
        .await?;

        connection.commit().await?;

        Ok(meta)
    }

    pub async fn mark_levels_request_result_as_absent<'a>(&self, request: &LevelsRequest<'a>) -> Result<CacheEntryMeta, CacheError> {
        trace!("Marking result of request {:?} as absent!", request);

        let hash = {
            let mut hasher = DefaultHasher::new();
            request.hash(&mut hasher);
            hasher.finish() as i64
        };

        let mut connection = self.pool.begin().await?;

        let meta = sqlx::query_as!(
            CacheEntryMeta,
            "INSERT INTO gj_level_request_meta (request_hash, cached_at, absent) VALUES ($1, $2, TRUE) ON CONFLICT (request_hash) DO \
             UPDATE SET cached_at = EXCLUDED.cached_at, absent = TRUE RETURNING request_hash AS key, cached_at AS made, absent",
            hash,
            Utc::now().naive_utc()
        )
        .fetch_one(&mut connection)
        .await?;

        connection.commit().await?;

        Ok(meta)
    }

    pub async fn lookup_levels_request<'a>(
        &self, request: &LevelsRequest<'a>,
    ) -> Result<CacheEntry<Vec<CacheEntry<Level<'static, ()>>>>, CacheError> {
        let hash = {
            let mut hasher = DefaultHasher::new();
            request.hash(&mut hasher);
            hasher.finish() as i64
        };

        let mut connection = self.pool.acquire().await?;

        let meta = sqlx::query_as!(
            CacheEntryMeta,
            "SELECT request_hash AS key, cached_at AS made, absent FROM gj_level_request_meta WHERE request_hash = $1",
            hash
        )
        .fetch_one(&mut *connection)
        .await;

        let meta = match meta {
            Err(sqlx::Error::RowNotFound) => return Ok(CacheEntry::Missing),
            Err(err) => return Err(err.into()),
            Ok(meta) if meta.absent => return Ok(CacheEntry::Absent),
            Ok(meta) => meta,
        };

        let mut stream =
            sqlx::query!("SELECT level_id from gj_level_request_results WHERE request_hash = $1", hash).fetch(&mut *connection);
        let mut levels = Vec::new();

        while let Some(row) = stream.next().await {
            let level_id = row?.level_id as u64;

            levels.push(self.lookup_level(level_id).await?);
        }

        Ok(self.make_cache_entry(meta, levels))
    }

    pub async fn store_levels_request<'a, 'b>(
        &self, request: &LevelsRequest<'a>, levels: &Vec<ListedLevel<'b>>,
    ) -> Result<CacheEntryMeta, CacheError> {
        let hash = {
            let mut hasher = DefaultHasher::new();
            request.hash(&mut hasher);
            hasher.finish() as i64
        };

        let mut connection = self.pool.begin().await?;

        let meta = sqlx::query_as!(
            CacheEntryMeta,
            "INSERT INTO gj_level_request_meta (request_hash, cached_at, absent) VALUES ($1, $2, FALSE) ON CONFLICT (request_hash) DO \
             UPDATE SET cached_at = EXCLUDED.cached_at, absent = FALSE RETURNING request_hash AS key, cached_at AS made, absent",
            hash,
            Utc::now().naive_utc()
        )
        .fetch_one(&mut connection)
        .await?;

        for level in levels {
            self.store_level(
                level,
                level.creator.as_ref().map(|c| c.user_id).unwrap_or(0),
                level.custom_song.as_ref().map(|n| n.song_id),
            )
            .await?;

            // FIXME: We do not correctly handle the case where newgrounds song/creator is absent. For
            // pointercrate, this should not be a problem however.

            if let Some(ref creator) = level.creator {
                self.store_creator(creator).await?;
            }

            if let Some(ref song) = level.custom_song {
                self.store_newgrounds_song(song).await?;
            }

            sqlx::query!(
                "INSERT INTO gj_level_request_results(level_id, request_hash) VALUES ($1, $2)",
                level.level_id as i64,
                hash
            )
            .execute(&mut *connection)
            .await?;
        }

        connection.commit().await?;

        Ok(meta)
    }

    pub async fn lookup_level(&self, level_id: u64) -> Result<CacheEntry<Level<'static, ()>>, CacheError> {
        let mut connection = self.pool.acquire().await?;

        let meta = sqlx::query_as!(
            CacheEntryMeta,
            "SELECT level_id AS key, cached_at AS made, absent FROM gj_level_meta WHERE level_id = $1",
            level_id as i64
        )
        .fetch_one(&mut *connection)
        .await;

        let meta = match meta {
            Err(sqlx::Error::RowNotFound) => return Ok(CacheEntry::Missing),
            Err(err) => return Err(err.into()),
            Ok(meta) if meta.absent => return Ok(CacheEntry::Absent),
            Ok(meta) => meta,
        };

        let row = sqlx::query!("SELECT * FROM gj_level WHERE level_id = $1", level_id as i64)
            .fetch_one(&mut connection)
            .await?;

        let level = Level {
            level_id,
            name: Cow::Owned(row.level_name),
            description: row
                .description
                .map(|description| Thunk::Processed(Base64Decoded(Cow::Owned(description)))),
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
        };

        Ok(self.make_cache_entry(meta, level))
    }

    // This must be the most horrifying piece of code I have ever written.
    async fn store_level<'a, T, U, V>(
        &self, level: &Level<'a, T, U, V>, creator_id: u64, custom_song_id: Option<u64>,
    ) -> Result<CacheEntryMeta, CacheError> {
        let mut connection = self.pool.begin().await?;

        let meta = sqlx::query_as!(
            CacheEntryMeta,
            "INSERT INTO gj_level_meta (level_id, cached_at, absent) VALUES ($1, $2, FALSE) ON CONFLICT (level_id) DO UPDATE SET \
             cached_at = EXCLUDED.cached_at, absent = FALSE RETURNING level_id AS key, cached_at AS made, absent",
            level.level_id as i64,
            Utc::now().naive_utc()
        )
        .fetch_one(&mut connection)
        .await?;

        sqlx::query!(
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
            match &level.description {
                Some(thunk) => {
                    Some(match thunk {
                        Thunk::Processed(processed) => processed.0.to_string(),
                        Thunk::Unprocessed(unproc) => Base64Decoded::from_unprocessed(unproc)?.0.to_string(),
                    })
                },
                None => None,
            },
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
        .execute(&mut connection)
        .await?;

        connection.commit().await?;

        Ok(meta)
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
        LevelRating::Demon(demon_rating) =>
            match demon_rating {
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
            return LevelRating::Demon(DemonRating::Unknown((value / 100) as i32))
        } else {
            return LevelRating::Unknown((value / 100) as i32)
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
    }
}

fn i16_to_level_length(value: i16) -> LevelLength {
    if value.abs() >= 100 || value == 0 {
        return LevelLength::Unknown((value / 100) as i32)
    }

    match value {
        1 => LevelLength::Tiny,
        2 => LevelLength::Short,
        3 => LevelLength::Medium,
        4 => LevelLength::Long,
        5 => LevelLength::ExtraLong,
        _ => unreachable!(),
    }
}
