use rocket::{response::Redirect, State};

use chrono::{DateTime, FixedOffset, NaiveDate, TimeZone, Utc};
use pointercrate_core::{audit::AuditLogEntryType, pool::PointercratePool};
use pointercrate_core_api::{
    error::Result,
    response::{Page, Response2},
};
use pointercrate_demonlist::{
    demon::{audit::audit_log_for_demon, current_list, list_at, FullDemon, MinimalDemon},
    error::DemonlistError,
    nationality::Nationality,
    LIST_ADMINISTRATOR, LIST_HELPER, LIST_MODERATOR,
};
use pointercrate_demonlist_pages::{
    components::{team::Team, time_machine::Tardis},
    demon_page::{DemonMovement, DemonPage},
    overview::OverviewPage,
    statsviewer::{individual::IndividualStatsViewer, national::NationBasedStatsViewer},
};
use pointercrate_integrate::gd::{GDIntegrationResult, PgCache};
use pointercrate_user::User;
use rocket::{futures::StreamExt, http::CookieJar};

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

    let specified_when = match specified_when {
        Some(Ok(when)) if when < beginning_of_time => Some(beginning_of_time),
        Some(Ok(when)) if when >= Utc::now() => None,
        Some(Ok(when)) => Some(when),
        _ => None,
    };

    let tardis = match specified_when {
        Some(destination) => Tardis::new(timemachine.unwrap_or(false)).activate(destination, list_at(&mut connection, destination).await?),
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
pub async fn demon_page(position: i16, pool: &State<PointercratePool>, gd: &State<PgCache>) -> Result<Page<DemonPage>> {
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
        integration: gd
            .data_for_demon(
                reqwest::Client::new(),
                full_demon.demon.level_id,
                full_demon.demon.base.name.clone(),
                full_demon.demon.base.id,
            )
            .await
            .unwrap_or(GDIntegrationResult::LevelDataNotFound),
        data: full_demon,
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

macro_rules! heatmap_query {
    ($connection: expr, $query: expr, $($param:expr),*) => {
        {
            let mut css = String::new();
            let mut stream = sqlx::query!($query, $($param),*).fetch(&mut $connection);

            if let Some(firstrow) = stream.next().await {
                // first one is the one with most score
                let firstrow = firstrow.map_err(DemonlistError::from)?;
                let highest_score = firstrow.score * 1.5;

                css.push_str(&make_css_rule(&firstrow.code, firstrow.score, highest_score));

                while let Some(row) = stream.next().await {
                    let row = row.map_err(DemonlistError::from)?;

                    css.push_str(&make_css_rule(&row.code, row.score, highest_score));
                }
            }

            css
        }
    };
}

#[rocket::get("/statsviewer/heatmap.css")]
pub async fn heatmap_css(pool: &State<PointercratePool>) -> Result<Response2<String>> {
    let mut connection = pool.connection().await?;
    let mut css = heatmap_query!(
        connection,
        r#"SELECT LOWER(iso_country_code) as "code!", score as "score!" from nations_with_score order by score desc"#,
    );

    for nation in ["AU", "CA", "US", "GB"] {
        css.push_str(&heatmap_query!(
            connection,
            r#"SELECT CONCAT($1, '-', UPPER(subdivision_code)) AS "code!", score AS "score!" FROM subdivision_ranking_of($1) ORDER BY score DESC"#,
            nation
        ));
    }

    Ok(Response2::new(css).with_header("Content-Type", "text/css"))
}

fn make_css_rule(code: &str, score: f64, highest_score: f64) -> String {
    format!(
        ".heatmapped #{0}, .heatmapped #{0} > path {{ fill: rgb({1}, {2}, {3}); }}",
        code,
        0xda as f64 + (0x08 - 0xda) as f64 * (score / highest_score),
        0xdc as f64 + (0x81 - 0xdc) as f64 * (score / highest_score),
        0xe0 as f64 + (0xc6 - 0xe0) as f64 * (score / highest_score),
    )
}
