use rocket::{response::Redirect, State};

use chrono::{DateTime, FixedOffset, NaiveDate, TimeZone};
use pointercrate_core::{audit::AuditLogEntryType, pool::PointercratePool};
use pointercrate_core_api::{error::Result, response::Page};
use pointercrate_demonlist::{
    demon::{
        audit::{audit_log_for_demon, DemonModificationData},
        current_list, list_at, FullDemon, MinimalDemon,
    },
    nationality::Nationality,
    LIST_ADMINISTRATOR, LIST_HELPER, LIST_MODERATOR,
};
use pointercrate_demonlist_pages::{
    components::{submitter::RecordSubmitter, team::Team, time_machine::Tardis},
    demon_page::{DemonMovement, DemonPage},
    overview::OverviewPage,
    statsviewer::{individual::IndividualStatsViewer, national::NationBasedStatsViewer},
};
use pointercrate_integrate::gd::GDIntegrationResult;
use pointercrate_user::User;
use rocket::http::CookieJar;

#[rocket::get("/?statsviewer=true")]
pub fn stats_viewer_redirect() -> Redirect {
    Redirect::to(rocket::uri!(stats_viewer))
}

#[rocket::get("/?<timemachine>&<submitter>")]
pub async fn overview(
    pool: &State<PointercratePool>, timemachine: Option<bool>, submitter: Option<bool>, cookies: &CookieJar<'_>,
) -> Result<Page<OverviewPage>> {
    // should be const, but chrono aint const :(
    let beginning_of_time: DateTime<FixedOffset> =
        FixedOffset::east(0).from_utc_datetime(&NaiveDate::from_ymd(2017, 1, 4).and_hms(0, 0, 0));

    let mut connection = pool.connection().await?;

    let demonlist = current_list(&mut connection).await?;

    let specified_when = cookies
        .get("when")
        .map(|cookie| DateTime::<FixedOffset>::parse_from_rfc3339(cookie.value()));

    let tardis = match specified_when {
        Some(Ok(destination)) =>
            Tardis::new(timemachine.unwrap_or(false)).activate(destination, list_at(&mut connection, destination).await?),
        _ => Tardis::new(timemachine.unwrap_or(false)),
    };

    Ok(Page(OverviewPage {
        team: Team {
            admins: User::by_permission(LIST_ADMINISTRATOR, &mut connection).await?,
            moderators: User::by_permission(LIST_MODERATOR, &mut connection).await?,
            helpers: User::by_permission(LIST_HELPER, &mut connection).await?,
        },
        demonlist,
        time_machine: tardis,
        submitter_initially_visible: submitter.unwrap_or(false),
    }))
}

#[rocket::get("/permalink/<demon_id>")]
pub async fn demon_permalink(demon_id: i32, pool: &State<PointercratePool>) -> Result<Redirect> {
    let mut connection = pool.connection().await?;

    let position = MinimalDemon::by_id(demon_id, &mut connection).await?.position;

    Ok(Redirect::to(rocket::uri!("/demonlist", demon_page(position))))
}

#[rocket::get("/<position>")]
pub async fn demon_page(position: i16, pool: &State<PointercratePool>) -> Result<Page<DemonPage>> {
    let mut connection = pool.connection().await?;

    let full_demon = FullDemon::by_position(position, &mut connection).await?;

    let audit_log = audit_log_for_demon(full_demon.demon.base.id, &mut connection).await?;

    let mut addition_time = None;

    let mut modifications = audit_log
        .iter()
        .filter_map(|entry| {
            match entry.r#type {
                AuditLogEntryType::Modification(ref modification) =>
                    match modification.position {
                        Some(old_position) if old_position > 0 =>
                            Some(DemonMovement {
                                from_position: old_position,
                                at: entry.time,
                            }),
                        _ => None,
                    },
                AuditLogEntryType::Addition => {
                    addition_time = Some(entry.time);

                    None
                },
                _ => None,
            }
        })
        .collect::<Vec<_>>();

    if let Some(addition) = addition_time {
        modifications.insert(0, DemonMovement {
            from_position: modifications
                .first()
                .map(|m| m.from_position)
                .unwrap_or(full_demon.demon.base.position),
            at: addition,
        });
    }

    Ok(Page(DemonPage {
        team: Team {
            admins: User::by_permission(LIST_ADMINISTRATOR, &mut connection).await?,
            moderators: User::by_permission(LIST_MODERATOR, &mut connection).await?,
            helpers: User::by_permission(LIST_HELPER, &mut connection).await?,
        },
        demonlist: current_list(&mut connection).await?,
        movements: modifications,
        data: full_demon,
        integration: GDIntegrationResult::DemonNotFoundByName,
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
