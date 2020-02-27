//! Module for dealing with gdcf in compat mode, since that crate still uses futures 0.1

use gdcf::{
    api::request::{LevelRequestType, LevelsRequest, SearchFilters},
    cache::CacheEntry,
    future::CloneablePeekFuture,
    Gdcf,
};
use gdcf_diesel::Cache;
use gdcf_model::{
    level::{DemonRating, Level, LevelRating, PartialLevel},
    user::Creator,
};
use gdrs::BoomlingsClient;
use log::error;

pub fn gd_demon_by_name(
    gdcf: &Gdcf<BoomlingsClient, Cache>, name: &str,
) -> Result<CacheEntry<Level<Option<u64>, Option<Creator>>, gdcf_diesel::Entry>, ()> {
    let request = LevelsRequest::default()
        .request_type(LevelRequestType::MostLiked)
        .search(name.to_owned())
        .with_rating(LevelRating::Demon(DemonRating::Hard))
        .filter(SearchFilters::default().rated());

    let future = gdcf
        .levels(request, false)
        .map_err(|err| error!("GDCF database access failed: {:?}", err))?
        .upgrade_all::<PartialLevel<_, Option<Creator>>>()
        .upgrade_all::<Level<_, _>>();

    let cached_clone: CacheEntry<Vec<Level<Option<u64>, Option<Creator>>>, _> = future.clone_peek()?;

    // FIXME: this doesn't work because we're not in a tokio runtime
    /*actix_rt::spawn(async move {
        match future.compat().await {
            Ok(_) => info!("LevelsRequest successful"),
            Err(err) => error!("Error during GDCF cache refresh! {:?}", err),
        }
    });*/

    match cached_clone {
        CacheEntry::Missing => Ok(CacheEntry::Missing),
        CacheEntry::MarkedAbsent(meta) => Ok(CacheEntry::MarkedAbsent(meta)),
        CacheEntry::Cached(demons, meta) =>
            Ok(demons
                .into_iter()
                .filter(|demon| demon.base.name.to_lowercase() == name.to_lowercase())
                .max_by(|x, y| x.base.difficulty.cmp(&y.base.difficulty))
                .map(|demon| CacheEntry::Cached(demon, meta))
                .unwrap_or(CacheEntry::Missing)), //FIXME: this
    }
}
