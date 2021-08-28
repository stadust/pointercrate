use rocket::{response::Redirect, State};

use pointercrate_core::pool::PointercratePool;
use pointercrate_core_api::{error::Result, response::Page};
use pointercrate_demonlist::{nationality::Nationality, LIST_ADMINISTRATOR, LIST_HELPER, LIST_MODERATOR};
use pointercrate_demonlist_pages::{
    components::{submitter::RecordSubmitter, time_machine::TimeMachine},
    overview::OverviewPage,
    statsviewer::{individual::IndividualStatsViewer, national::NationBasedStatsViewer},
    DemonlistData, OverviewDemon,
};
use pointercrate_user::User;

#[rocket::get("/?statsviewer=true")]
pub fn stats_viewer_redirect() -> Redirect {
    Redirect::to(rocket::uri!(stats_viewer))
}

#[rocket::get("/?<timemachine>&<submitter>")]
pub async fn overview(pool: &State<PointercratePool>, timemachine: Option<bool>, submitter: Option<bool>) -> Result<Page<OverviewPage>> {
    let mut connection = pool.connection().await?;

    Ok(Page(OverviewPage {
        data: DemonlistData {
            demon_overview: vec![],
            admins: User::by_permission(LIST_ADMINISTRATOR, &mut connection).await?,
            mods: User::by_permission(LIST_MODERATOR, &mut connection).await?,
            helpers: User::by_permission(LIST_HELPER, &mut connection).await?,
        },
        time_machine: TimeMachine::new(timemachine.unwrap_or(false)),
        submitter: RecordSubmitter::new(submitter.unwrap_or(false), vec![]),
    }))
}

#[rocket::get("/statsviewer")]
pub async fn stats_viewer(pool: &State<PointercratePool>) -> Result<Page<IndividualStatsViewer>> {
    let mut connection = pool.connection().await?;

    Ok(Page(IndividualStatsViewer {
        nationalities_in_use: Nationality::used(&mut connection).await?,
    }))
}

#[rocket::get("/statsviewer/nations")]
pub async fn nation_stats_viewer() -> Page<NationBasedStatsViewer> {
    Page(NationBasedStatsViewer)
}
