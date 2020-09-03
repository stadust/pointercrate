use crate::{model::demonlist::demon::Demon, state::PointercrateState, Result};
use chrono::{DateTime, Duration, NaiveDateTime, Utc};
use dash_rs::{
    model::{
        creator::Creator,
        level::{
            object::{speed::Speed, ObjectData},
            DemonRating, Level, LevelRating, ListedLevel, Password,
        },
    },
    request::level::{LevelRequest, LevelRequestType, LevelsRequest, SearchFilters},
    HasRobtopFormat,
};
use log::error;
use reqwest::Client;
use sqlx::{pool::PoolConnection, PgConnection, Pool, Row};
use std::borrow::Cow;

pub struct CacheEntryMeta {
    made: NaiveDateTime,
    key: i64,
    absent: bool,
}

pub enum CacheEntry<T> {
    Missing,
    Absent,
    Expired(T, CacheEntryMeta),
    Live(T, CacheEntryMeta),
}

pub struct PgCache {
    pool: Pool<PgConnection>,
    expire_after: Duration,
}

impl PgCache {
    fn make_cache_entry<T>(&self, meta: CacheEntryMeta, t: T) -> CacheEntry<T> {
        if DateTime::<Utc>::from_utc(meta.made, Utc) - Utc::now() < self.expire_after {
            CacheEntry::Live(t, meta)
        } else {
            CacheEntry::Expired(t, meta)
        }
    }

    pub async fn lookup_creator(&self, user_id: i64) -> Result<CacheEntry<Creator<'static>>> {
        let mut connection = self.pool.acquire().await?;
        let mut connection = &mut *connection;
        let meta = sqlx::query_as!(
            CacheEntryMeta,
            "SELECT user_id AS key, cached_at AS made, absent FROM gj_creator_meta WHERE user_id = $1",
            user_id
        )
        .fetch_one(connection)
        .await;

        let meta = match meta {
            Err(sqlx::Error::NotFound) => return Ok(CacheEntry::Missing),
            Err(err) => return Err(err.into()),
            Ok(meta) => meta,
        };

        struct _Creator {
            user_id: i64,
            name: String,
            account_id: Option<i64>,
        }

        let creator_row = sqlx::query_as!(_Creator, "SELECT * FROM gj_creator WHERE user_id = $1", user_id)
            .fetch_one(connection)
            .await?;

        let creator = Creator {
            user_id: creator_row.user_id as u64,
            name: Cow::Owned(creator_row.name),
            account_id: creator_row.account_id.map(|id| id as u64),
        };

        Ok(self.make_cache_entry(meta, creator))
    }
}
/*
pub struct BoomlingsClient {
    inner: reqwest::Client,
}

impl BoomlingsClient {
    pub async fn get_levels(&self, request: impl Into<LevelsRequest>, backing_storage: &mut Vec<u8>) -> Result<Vec<ListedLevel>> {
        let url = request.into().to_url();

        self.inner.post(url).send().await.unwrap();
    }
}*/

/*
pub struct GjData {
    object_count: i32,
    level_length: Duration,
    password: Password,
    description: Option<String>,
}

struct InternalGjData {
    object_count: i32,
    level_length: i16,
    password: i32,
    description: Option<String>,
    last_updated: NaiveDateTime,
}

impl From<InternalGjData> for GjData {
    fn from(internal: InternalGjData) -> Self {
        GjData {
            object_count: internal.object_count,
            level_length: Duration::from_secs(internal.level_length as u64),
            password: match internal.password {
                0 => Password::FreeCopy,
                -1 => Password::NoCopy,
                pw => Password::PasswordCopy(pw as u32),
            },
            description: internal.description,
        }
    }
}

pub enum GjIntegrationError {
    Malformed,
    ServersDown,
    NotFound,
}

struct GjDataError {
    demon_id: i32,
    what: String,
}

pub async fn gather_gj_data(state: PointercrateState, demon: &mut Demon) -> Result<std::result::Result<GjData, GjIntegrationError>> {
    let mut connection = state.connection().await?;
    if let Some(reason) = sqlx::query!("SELECT what FROM gj_data_error WHERE demon_id = $1", demon.base.id)
        .fetch_optional(&mut connection)
        .await?
    {
        return Ok(Err(match reason.what.as_ref() {
            "notfound" => GjIntegrationError::NotFound,
            "malformed" => GjIntegrationError::Malformed,
            _ => unreachable!(),
        }))
    }

    match demon.level_id {
        Some(level_id) => {
            let raw = sqlx::query_as!(
                InternalGjData,
                "SELECT object_count, level_length, password, description, last_updated FROM gj_data WHERE level_id = $1",
                level_id
            )
            .fetch_one(&mut connection)
            .await?;
        },
        None => {
            actix_rt::spawn(async move {
                let level = determine_level_id(&state.http_client, demon.base.name).await;
            });
        },
    }

    /*if let Some(level_id) = demon.level_id {
        sqlx::query_as!(
            GjData,
            "SELECT object_count, level_length, password, description, last_updated FROM gj_data WHERE level_id = $1",
            level_id
        );
    }*/
    unimplemented!()
}

pub async fn update_db_entry(connection: PoolConnection<PgConnection>, level_id: u64) {}

pub async fn download_level(http_client: &Client, level_id: u64) -> std::result::Result<GjData, GjIntegrationError> {
    let request = LevelRequest::new(level_id);

    match http_client.post(&request.to_url()).send().await {
        Ok(response) => {
            let bytes = &*response.bytes().await.unwrap();
            let data = std::str::from_utf8(bytes).unwrap();

            if data == "-1" {
                return Err(GjIntegrationError::NotFound)
            }

            match data.split('#').next() {
                Some(section) => {
                    let level = Level::from_robtop_str(section).map_err(|err| {
                        error!("dash-rs error: {:?}", err);
                        GjIntegrationError::Malformed
                    })?;

                    let description = match level.description {
                        Some(thunk) =>
                            Some(
                                thunk
                                    .into_processed()
                                    .map_err(|err| {
                                        error!("dash-rs error: {:?}", err);
                                        GjIntegrationError::Malformed
                                    })?
                                    .0
                                    .into_owned(),
                            ),
                        None => None,
                    };

                    match level.level_data {
                        Some(level_data) => {
                            let objects = level_data.level_data.into_processed().map_err(|err| {
                                error!("dash-rs error: {:?}", err);
                                GjIntegrationError::Malformed
                            })?;

                            let mut object_count = 0;
                            let mut portals = Vec::new();
                            let mut furthest_x = 0.0;

                            for object in &objects.objects {
                                object_count += 1;

                                if let ObjectData::SpeedPortal { checked: true, speed } = object.metadata {
                                    portals.push((object.x, speed))
                                }

                                furthest_x = f32::max(furthest_x, object.x);
                            }

                            portals.sort_by(|(x1, _), (x2, _)| x1.partial_cmp(x2).unwrap());

                            let duration = get_seconds_from_x_pos(furthest_x, objects.meta.starting_speed, &portals) as u64;

                            Ok(GjData {
                                object_count,
                                level_length: Duration::from_secs(duration),
                                password: level_data.password,
                                description,
                            })
                        },
                        None => Err(GjIntegrationError::Malformed),
                    }
                },
                None => Err(GjIntegrationError::Malformed),
            }
        },
        Err(_) => Err(GjIntegrationError::ServersDown),
    }
}

pub async fn determine_level_id(http_client: &Client, name: &str) -> std::result::Result<u64, GjIntegrationError> {
    let request = LevelsRequest::default()
        .request_type(LevelRequestType::MostLiked)
        .search(name)
        .with_rating(LevelRating::Demon(DemonRating::Hard))
        .search_filters(SearchFilters::default().rated());

    match http_client.post(&request.to_url()).send().await {
        Ok(response) => {
            let bytes = &*response.bytes().await.unwrap();
            let data = std::str::from_utf8(bytes).unwrap();

            if data == "-1" {
                return Err(GjIntegrationError::NotFound)
            }

            match data.split('#').next() {
                Some(section) =>
                    section
                        .split('|')
                        .map(|fragment| Level::from_robtop_str(fragment))
                        .filter_map(|result| result.ok())
                        .filter(|demon| demon.name.to_lowercase() == name.to_lowercase())
                        .max_by(|x, y| x.difficulty.cmp(&y.difficulty))
                        .map(|level| level.level_id)
                        .ok_or(GjIntegrationError::NotFound),
                None => Err(GjIntegrationError::Malformed),
            }
        },
        _ => Err(GjIntegrationError::ServersDown),
    }
}

fn get_seconds_from_x_pos(pos: f32, start_speed: Speed, portals: &[(f32, Speed)]) -> f32 {
    let mut speed: f32 = start_speed.into();

    if portals.is_empty() {
        return pos / speed
    }

    let mut last_obj_pos = 0.0;
    let mut total_time = 0.0;

    for (x, portal_speed) in portals {
        // distance between last portal and this one
        let current_segment = x - last_obj_pos;

        // break if we're past the position we want to calculate the position to
        if pos <= current_segment {
            break
        }

        // Calculate time spent in this segment and add to total time
        total_time += current_segment / speed;

        speed = (*portal_speed).into();

        last_obj_pos = *x;
    }

    // add the time spent between end and last portal to total time and return
    (pos - last_obj_pos) / speed + total_time
}
*/
