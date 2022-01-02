use crate::{endpoints::misc, ratelimits::DemonlistRatelimits};
use chrono::Duration;
use pointercrate_core::pool::PointercratePool;
use pointercrate_integrate::gd::PgCache;
use rocket::{Build, Rocket};

pub(crate) mod config;
mod endpoints;
pub(crate) mod pages;
pub(crate) mod ratelimits;

pub fn setup(rocket: Rocket<Build>) -> Rocket<Build> {
    let ratelimits = DemonlistRatelimits::new();
    let dash_rs = PgCache::new(rocket.state::<PointercratePool>().unwrap().clone_inner(), Duration::minutes(30));

    rocket
        .manage(ratelimits)
        .manage(dash_rs)
        .mount("/api/v1/list_information/", rocket::routes![misc::list_information])
        .mount("/api/v1/submitters/", rocket::routes![
            endpoints::submitter::paginate,
            endpoints::submitter::get,
            endpoints::submitter::patch
        ])
        .mount("/api/v1/records/", rocket::routes![
            endpoints::record::add_note,
            endpoints::record::audit,
            endpoints::record::delete,
            endpoints::record::delete_note,
            endpoints::record::get,
            endpoints::record::paginate,
            endpoints::record::unauthed_pagination,
            endpoints::record::patch,
            endpoints::record::patch_note,
            endpoints::record::submit
        ])
        .mount("/api/v1/players/", rocket::routes![
            endpoints::player::get,
            endpoints::player::paginate,
            endpoints::player::unauthed_paginate,
            endpoints::player::patch,
            endpoints::player::ranking,
            endpoints::player::put_claim,
            endpoints::player::patch_claim,
            endpoints::player::paginate_claims,
            endpoints::player::delete_claim,
            endpoints::player::geolocate_nationality
        ])
        .mount("/api/v1/nationalities/", rocket::routes![
            endpoints::nationality::subdivisions,
            endpoints::nationality::ranking,
            endpoints::nationality::nation
        ])
        .mount("/api/v2/demons/", rocket::routes![
            endpoints::demon::get,
            endpoints::demon::paginate,
            endpoints::demon::paginate_listed,
            endpoints::demon::audit,
            endpoints::demon::patch,
            endpoints::demon::post,
            endpoints::demon::post_creator,
            endpoints::demon::delete_creator
        ])
        .mount("/demonlist/", rocket::routes![
            pages::overview,
            pages::stats_viewer_redirect,
            pages::stats_viewer,
            pages::nation_stats_viewer,
            pages::demon_page,
            pages::demon_permalink,
            pages::heatmap_css
        ])
}
