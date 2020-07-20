use crate::{
    compat, config,
    model::demonlist::demon::FullDemon,
    state::PointercrateState,
    video,
    view::{demonlist::overview::DemonlistOverview, Page},
    ViewResult,
};
use actix_web::{web::Path, HttpResponse};
use actix_web_codegen::get;
use chrono::NaiveDateTime;
use gdcf::cache::CacheEntry;
use gdcf_model::{
    level::{data::LevelInformationSource, Level, Password},
    user::Creator,
};
use log::error;
use maud::{html, Markup, PreEscaped, Render};

#[derive(Debug)]
pub struct DemonMovement {
    from_position: i16,
    at: NaiveDateTime,
}

#[derive(Debug)]
pub struct Demonlist {
    overview: DemonlistOverview,
    data: FullDemon,
    server_level: Option<CacheEntry<Level<Option<u64>, Option<Creator>>, gdcf_diesel::Entry>>,
    movements: Vec<DemonMovement>,
    link_banned: bool,
}

#[get("/demonlist/{position}/")]
pub async fn page(state: PointercrateState, position: Path<i16>) -> ViewResult<HttpResponse> {
    let mut connection = state.connection().await?;
    let overview = DemonlistOverview::load(&mut connection).await?;
    let demon = FullDemon::by_position(position.into_inner(), &mut connection).await?;
    let gd_demon = compat::gd_demon_by_name(&state.gdcf, &demon.demon.base.name);
    let link_banned = sqlx::query!("SELECT link_banned FROM players WHERE id = $1", demon.demon.verifier.id)
        .fetch_one(&mut connection)
        .await?
        .link_banned;

    let mut movements: Vec<DemonMovement> = sqlx::query_as!(
        DemonMovement,
        "SELECT position AS from_position, time AS at FROM demon_modifications WHERE position IS NOT NULL AND id = $1 AND position > 0 \
         ORDER BY time",
        demon.demon.base.id
    )
    .fetch_all(&mut connection)
    .await?;

    let addition = sqlx::query!("SELECT time FROM demon_additions WHERE id = $1", demon.demon.base.id)
        .fetch_optional(&mut connection)
        .await?;

    match addition {
        Some(time) => {
            let from_position = movements.first().map(|m| m.from_position).unwrap_or(demon.demon.base.position);

            movements.insert(0, DemonMovement {
                at: time.time,
                from_position,
            });
        },
        None => error!("No addition logged for demon {}!", demon),
    }

    Ok(HttpResponse::Ok().content_type("text/html; charset=utf-8").body(
        Demonlist {
            overview,
            data: demon,
            server_level: gd_demon.ok(),
            movements,
            link_banned,
        }
        .render()
        .0,
    ))
}

impl Demonlist {
    fn demon_panel(&self) -> Markup {
        let position = self.data.demon.base.position;
        let name = &self.data.demon.base.name;

        let score100 = self.data.demon.score(100);
        let score_requirement = self.data.demon.score(self.data.demon.requirement);

        html! {
            section.panel.fade.js-scroll-anim data-anim = "fade" {
                div.underlined {
                    h1 style = "overflow: hidden"{
                        @if self.data.demon.base.position != 1 {
                            a href=(format!("/demonlist/{:?}", self.data.demon.base.position - 1)) {
                                i class="fa fa-chevron-left" style="padding-right: 5%" {}
                            }
                        }
                        (name)
                        @if position as usize != self.overview.demon_overview.len() {
                            a href=(format!("/demonlist/{:?}", position + 1)) {
                                i class="fa fa-chevron-right" style="padding-left: 5%" {}
                            }
                        }
                    }
                    h3 {
                        @if self.data.creators.len() > 3 {
                            "by " (self.data.creators[0].name) " and "
                            div.tooltip {
                                "more"
                                div.tooltiptext.fade {
                                    (self.data.creators.iter().map(|player| player.name.to_string()).collect::<Vec<_>>().join(", "))
                                }
                            }
                            ", " (self.data.short_headline())
                        }
                        @else {
                            (self.data.headline())
                        }
                    }
                }
                @if let Some(CacheEntry::Cached(ref level, _)) = self.server_level {
                    @if let Some(ref description) = level.base.description {
                        div.underlined.pad {
                            q {
                                (description)
                            }
                        }
                    }
                }
                @if self.link_banned {
                    p {
                        "Due to the questionable nature of the verifier's youtube content, embedding of their videos has been disabled"
                    }
                }
                @else {
                    @if let Some(ref video) = self.data.demon.video {
                        @if let Some(embedded_video) = video::embed(video) {
                            h3 {
                                "Showcase video:"
                            }
                            iframe."ratio-16-9"."js-delay-attr" style="width:90%; margin: 15px 5%" allowfullscreen="" data-attr = "src" data-attr-value = (embedded_video) {"Verification Video"}
                        }
                    }
                }
                div.underlined.pad.flex.wrap#level-info {
                    @match self.server_level {
                        None => {
                            p.info-red {
                                "An internal error occured while trying to access the GDCF database, or while processing Geometry Dash data. This is a bug."
                            }
                        }
                        Some(CacheEntry::Missing) => {
                            p.info-yellow {
                                "The data from the Geometry Dash servers has not yet been cached. Please wait a bit and refresh the page."
                            }
                        },
                        Some(CacheEntry::MarkedAbsent(_)) => {
                            p.info-red {
                                "This demon has not been found on the Geometry Dash servers. Its name was most likely misspelled when entered into the database. Please contact a list moderator to fix this."
                            }
                        },
                        Some(CacheEntry::Cached(ref level, ref meta)) => {
                            @let level_data = level.decompress_data().ok();
                            @let level_data = level_data.as_ref().and_then(|data| gdcf_parse::level::data::parse_lazy_parallel(data).ok());
                            @let stats = level_data.map(LevelInformationSource::stats);

                            span {
                                b {
                                    "Level Password: "
                                }
                                br;
                                @match level.password {
                                    Password::NoCopy => "Not copyable",
                                    Password::FreeCopy => "Free to copy",
                                    Password::PasswordCopy(ref pw) => (pw)
                                }
                            }
                            span {
                                b {
                                    "Level ID: "
                                }
                                br;
                                (level.base.level_id)
                            }
                            span {
                                b {
                                    "Level length: "
                                }
                                br;
                                @match stats {
                                    Some(ref stats) => (format!("{}m:{:02}s", stats.duration.as_secs() / 60, stats.duration.as_secs() % 60)),
                                    _ => (level.base.length.to_string())
                                }
                            }
                            span {
                                b {
                                    "Object count: "
                                }
                                br;
                                @match stats {
                                    Some(ref stats) => (stats.object_count),
                                    _ => (level.base.object_amount.unwrap_or(0))
                                }
                            }
                        }
                    }
                    @if position <= config::extended_list_size() {
                        span {
                            b {
                                "Demonlist score (100%): "
                            }
                            br;
                                (format!("{:.2}", score100))
                        }
                    }
                    @if position <= config::list_size(){
                        span {
                            b {
                                "Demonlist score (" (self.data.demon.requirement) "%): "
                            }
                            br;
                            (format!("{:.2}", score_requirement))
                        }
                    }
                }
            }
        }
    }

    fn records_panel(&self) -> Markup {
        let position = self.data.demon.base.position;
        let name = &self.data.demon.base.name;

        html! {
            @if !self.data.records.is_empty() || position <= config::extended_list_size() {
                section.records.panel.fade.js-scroll-anim data-anim = "fade" {
                    div.underlined.pad {
                        h2 {
                            "Records"
                        }
                        @if position <= config::list_size() {
                            h3 {
                                (self.data.demon.requirement) "% or better required to qualify"
                            }
                        }
                        @else if position <= config::extended_list_size() {
                            h3 {
                                "100% required to qualify"
                            }
                        }
                        @if !self.data.records.is_empty() {
                            h4 {
                                @let records_registered_100_count = self.data.records.iter().filter(|record| record.progress == 100).count();
                                (self.data.records.len())
                                " records registered, out of which "
                                (records_registered_100_count)
                                @if records_registered_100_count == 1 { " is" } @else { " are" }
                                " 100%"
                            }
                        }
                    }
                    @if self.data.records.is_empty() {
                        h3 {
                            @if position > config::extended_list_size() {
                                "No records!"
                            }
                            @else {
                                "No records yet! Be the first to achieve one!"
                            }
                        }
                    }
                    @else {
                        table {
                            tbody {
                                tr {
                                    th.blue {
                                        "Record Holder"
                                    }
                                    th.blue {
                                        "Progress"
                                    }
                                    th.video-link.blue {
                                        "Video Proof"
                                    }
                                }
                                @for record in &self.data.records {
                                    tr style = { @if record.progress == 100 {"font-weight: bold"} @else {""} } {
                                        td {
                                            @if let Some(ref video) = record.video {
                                                 a href = (video) target = "_blank"{
                                                    (record.player.name)
                                                 }
                                            }
                                            @else {
                                                (record.player.name)
                                            }
                                        }
                                        td {
                                            (record.progress) "%"
                                        }
                                        td.video-link {
                                            @if let Some(ref video) = record.video {
                                                 a.link href = (video) target = "_blank"{
                                                     (video::host(video))
                                                 }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

impl Page for Demonlist {
    fn title(&self) -> String {
        format!(
            "#{} - {} - Geometry Dash Demonlist",
            self.data.demon.base.position,
            self.data.demon.base.name // FIXME: flatten the structs, holy shit
        )
    }

    fn description(&self) -> String {
        if let Some(CacheEntry::Cached(ref level, _)) = self.server_level {
            if let Some(ref description) = level.base.description {
                return format!("{}: {}", self.title(), description)
            }
        }
        format!("{}: <No Description Provided>", self.title())
    }

    fn scripts(&self) -> Vec<&str> {
        vec!["js/modules/form.mjs", "js/modules/demonlist.mjs", "js/demonlist.v2.2.js"]
    }

    fn stylesheets(&self) -> Vec<&str> {
        vec!["css/demonlist.v2.1.css", "css/sidebar.css"]
    }

    fn body(&self) -> Markup {
        let dropdowns = super::dropdowns(&self.overview.demon_overview, Some(&self.data.demon));

        let mut labels = Vec::new();

        let year_only = self.movements.len() > 30;
        let mut last_label = None;

        for movement in &self.movements {
            let would_be_label = if year_only {
                movement.at.date().format("%Y").to_string()
            } else {
                movement.at.date().format("%b %y").to_string()
            };

            match last_label {
                Some(ref label) if &would_be_label == label => labels.push(String::new()),
                _ => {
                    last_label = Some(would_be_label.clone());
                    if labels.is_empty() {
                        labels.push(format!("Added ({})", would_be_label))
                    } else {
                        labels.push(would_be_label)
                    }
                },
            }
        }

        html! {
            (dropdowns)

            div.flex.m-center.container {
                main.left {
                    (super::submission_panel())
                    (super::stats_viewer(&self.overview.nations))
                    (self.demon_panel())
                    div.panel.fade.js-scroll-anim.js-collapse data-anim = "fade" {
                        h2.underlined.pad {
                            "Position History"
                            span.arrow.hover {}
                        }
                        div.ct-chart.ct-perfect-fourth.js-collapse-content#position-chart style="display:none" {}
                    }
                    (super::rules_panel())
                    (self.records_panel())
                    (PreEscaped(format!("
                        <script>
                        window.positionChartLabels = ['{}', 'Now'];
                        window.positionChartData = [{},{}];
                        </script>",
                        labels.join("','"),
                        self.movements.iter().map(|movement| movement.from_position.to_string()).collect::<Vec<_>>().join(","), self.data.demon.base.position
                    ))) // FIXME: bad
                }
                aside.right {
                    (self.overview.team_panel())
                    (super::submit_panel())
                    (super::stats_viewer_panel())
                    (super::discord_panel())
                }
            }
        }
    }

    fn head(&self) -> Vec<Markup> {
        vec![
            html! {
                (PreEscaped(format!(r##"
                    <link href="https://cdnjs.cloudflare.com/ajax/libs/flag-icon-css/3.4.3/css/flag-icon.min.css" rel="stylesheet">
                    <script type="application/ld+json">
                    {{
                        "@context": "http://schema.org",
                        "@type": "WebPage",
                        "breadcrumb": {{
                            "@type": "BreadcrumbList",
                            "itemListElement": [{{
                                    "@type": "ListItem",
                                    "position": 1,
                                    "item": {{
                                        "@id": "https://pointercrate.com/",
                                        "name": "pointercrate"
                                    }}
                                }},{{
                                    "@type": "ListItem",<
                                    "position": 2,
                                    "item": {{
                                        "@id": "https://pointercrate.com/demonlist/",
                                        "name": "demonlist"
                                    }}
                                }},{{
                                    "@type": "ListItem",
                                    "position": 3,
                                    "item": {{
                                        "@id": "https://pointercrate.com/demonlist/{0}/",
                                        "name": "{1}"
                                    }}
                                }}
                            ]
                        }},
                        "name": "#{0} - {1}",
                        "description": "{2}",
                        "url": "https://pointercrate.com/demonlist/{0}/"
                    }}
                    </script>
                "##, self.data.position(), self.data.name(), self.description().render().0)))
            },
            html! {
                (PreEscaped(format!("
                    <script>
                        window.list_length = {0};
                        window.extended_list_length = {1}
                    </script>", config::list_size(), config::extended_list_size()
                )))
            },
            html! {
                   (PreEscaped("<link rel='stylesheet' href='//cdn.jsdelivr.net/chartist.js/latest/chartist.min.css'>
                    <script src='//cdn.jsdelivr.net/chartist.js/latest/chartist.min.js'></script>"))
            },
        ]
    }
}
