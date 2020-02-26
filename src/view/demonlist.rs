use super::Page;
use crate::{
    compat, config,
    error::{HtmlError, PointercrateError},
    model::{
        demonlist::demon::{Demon, FullDemon},
        nationality::Nationality,
        user::User,
    },
    permissions::Permissions,
    state::PointercrateState,
    video, Result, ViewResult,
};
use actix_web::{web::Path, FromRequest, HttpRequest, HttpResponse, Responder};
use actix_web_codegen::get;
use gdcf::cache::CacheEntry;
use gdcf_model::{
    level::{data::LevelInformationSource, Level, Password},
    user::Creator,
};
use maud::{html, Markup, PreEscaped};
use sqlx_core::postgres::PgConnection;

mod manage;

struct ListSection {
    name: &'static str,
    description: &'static str,
    id: &'static str,
    numbered: bool,
}

static MAIN_SECTION: ListSection = ListSection {
    name: "Main List",
    description: "The main section of the Demonlist. These demons are the hardest rated levels in the game. Records are accepted above a \
                  given threshold and award a large amount of points!",
    id: "mainlist",
    numbered: true,
};

static EXTENDED_SECTION: ListSection = ListSection {
    name: "Extended List",
    description: "These are demons that dont qualify for the main section of the list, but are still of high relevance. Only 100% records \
                  are accepted for these demons! Note that non-100% that were submitted/approved before a demon fell off the main list \
                  will be retained",
    id: "extended",
    numbered: true,
};

static LEGACY_SECTION: ListSection = ListSection {
    name: "Legacy List",
    description: "These are demons that used to be on the list, but got pushed off as new demons were added. They are here for nostalgic \
                  reasons. This list is in no order whatsoever and will not be maintained any longer at all. This means no new records \
                  will be added for these demons.",
    id: "legacy",
    numbered: false,
};

#[derive(Debug)]
pub struct OverviewDemon {
    position: i16,
    name: String,
    publisher: String,
    video: Option<String>,
}

#[derive(Debug)]
pub struct DemonlistOverview {
    pub demon_overview: Vec<OverviewDemon>,
    pub admins: Vec<User>,
    pub mods: Vec<User>,
    pub helpers: Vec<User>,
    pub nations: Vec<Nationality>,
}

impl DemonlistOverview {
    async fn load(connection: &mut PgConnection) -> Result<DemonlistOverview> {
        let admins = User::by_permission(Permissions::ListAdministrator, connection).await?;
        let mods = User::by_permission(Permissions::ListModerator, connection).await?;
        let helpers = User::by_permission(Permissions::ListHelper, connection).await?;

        let nations = Nationality::all(connection).await?;
        let demon_overview = sqlx::query_as!(
            OverviewDemon,
            "SELECT position, demons.name::TEXT, video::TEXT, players.name::TEXT as publisher FROM demons INNER JOIN players ON \
             demons.publisher = players.id WHERE position IS NOT NULL ORDER BY position"
        )
        .fetch_all(connection)
        .await?;

        Ok(DemonlistOverview {
            admins,
            mods,
            helpers,
            nations,
            demon_overview,
        })
    }
}

#[get("/demonlist/")]
pub async fn index(state: PointercrateState) -> ViewResult<HttpResponse> {
    let mut connection = state.connection().await?;

    Ok(HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(DemonlistOverview::load(&mut connection).await?.render().0))
}

impl Page for DemonlistOverview {
    fn title(&self) -> String {
        "Geometry Dash Demonlist".to_string()
    }

    fn description(&self) -> String {
        "The official pointercrate Demonlist!".to_string()
    }

    fn scripts(&self) -> Vec<&str> {
        vec!["js/form.js", "js/demonlist.v2.1.js"]
    }

    fn stylesheets(&self) -> Vec<&str> {
        vec!["css/demonlist.v2.1.css", "css/sidebar.css"]
    }

    fn body(&self) -> Markup {
        let dropdowns = dropdowns(&self.demon_overview, None);

        html! {
            (dropdowns)

            div.flex.m-center.container {
                div.left {
                    (submission_panel())
                    (stats_viewer(&self.nations))
                    @for demon in &self.demon_overview {
                        @if demon.position <= config::extended_list_size() {
                            div.panel.fade style="overflow:hidden" {
                                div.underlined.flex style = "padding-bottom: 10px; align-items: center" {
                                    @if let Some(ref video) = demon.video {
                                        div.thumb."ratio-16-9"."js-delay-css" style = "position: relative" data-property = "background-image" data-property-value = {"url('" (video::thumbnail(video)) "')"} {
                                            a.play href = (video) {}
                                        }
                                        div.leftlined.pad {
                                            h2 style = "text-align: left; margin-bottom: 0px" {
                                                a href = {"/demonlist/" (demon.position)} {
                                                    "#" (demon.position) " - " (demon.name)
                                                }
                                            }
                                            h3 style = "text-align: left" {
                                                i {
                                                    "by " (demon.publisher)
                                                }
                                            }
                                        }
                                    }
                                    @else {
                                        h2 {
                                            a href = {"/demonlist/" (demon.position)} {
                                                "#" (demon.position) " - " (demon.name) " by " (demon.publisher)
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                div.right {
                    (team_panel(&self.admins, &self.mods, &self.helpers))
                    (rules_panel())
                    (submit_panel())
                    (stats_viewer_panel())
                    (discord_panel())
                }
            }

        }
    }

    fn head(&self) -> Vec<Markup> {
        vec![
            html! {
            (PreEscaped(r#"
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
                                "@type": "ListItem",
                                "position": 2,
                                "item": {{
                                    "@id": "https://pointercrate.com/demonlist/",
                                    "name": "demonlist"
                                }}
                            }}
                        ]
                    }},
                    "name": "Geometry Dash Demonlist",
                    "description": "The official pointercrate Demonlist!",
                    "url": "https://pointercrate.com/demonlist/"
                }}
                </script>
            "#))
            },
            html! {
                (PreEscaped(format!("
                    <script>
                        window.list_length = {0};
                        window.extended_list_length = {1}
                    </script>", config::list_size(), config::extended_list_size())
                ))
            },
        ]
    }
}

#[derive(Debug)]
pub struct Demonlist {
    overview: DemonlistOverview,
    data: FullDemon,
    server_level: Option<CacheEntry<Level<Option<u64>, Option<Creator>>, gdcf_diesel::Entry>>,
}

#[get("/demonlist/{position}/")]
pub async fn demon_page(state: PointercrateState, position: Path<i16>) -> ViewResult<HttpResponse> {
    let mut connection = state.connection().await?;
    let overview = DemonlistOverview::load(&mut connection).await?;
    let demon = FullDemon::by_position(position.into_inner(), &mut connection).await?;
    let gd_demon = compat::gd_demon_by_name(&state.gdcf, &demon.demon.base.name);

    Ok(HttpResponse::Ok().content_type("text/html; charset=utf-8").body(
        Demonlist {
            overview,
            data: demon,
            server_level: gd_demon.ok(),
        }
        .render()
        .0,
    ))
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
        vec!["js/form.js", "js/demonlist.v2.1.js"]
    }

    fn stylesheets(&self) -> Vec<&str> {
        vec!["css/demonlist.v2.1.css", "css/sidebar.css"]
    }

    fn body(&self) -> Markup {
        let dropdowns = dropdowns(&self.overview.demon_overview, Some(&self.data.demon));
        let score100 = self.data.demon.score(100);
        let score_requirement = self.data.demon.score(self.data.demon.requirement);

        let position = self.data.demon.base.position;
        let name = &self.data.demon.base.name;

        html! {
            (dropdowns)

            div.flex.m-center.container {
                div.left {
                    (submission_panel())
                    (stats_viewer(&self.overview.nations))
                    div.panel.fade.js-scroll-anim data-anim = "fade" {
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
                                            (self.data.creators.iter().map(|player| player.name.to_string()).collect::<Vec<_>>().join(", ").to_string())
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
                        @if let Some(ref embedded_video) = self.data.demon.video.as_ref().and_then(video::embed) {
                            iframe."ratio-16-9"."js-delay-attr" style="width:90%; margin: 15px 5%" allowfullscreen="" data-attr = "src_old" data-attr-value = (embedded_video) {"Verification Video"}
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
                    (rules_panel())
                    @if !self.data.records.is_empty() || position <= config::extended_list_size() {
                        div.records.panel.fade.js-scroll-anim data-anim = "fade" {
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
                                                    (record.player.name)
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
                div.right {
                    (team_panel(&self.overview.admins, &self.overview.mods, &self.overview.helpers))
                    (submit_panel())
                    (stats_viewer_panel())
                    (discord_panel())
                }
            }
        }
    }

    fn head(&self) -> Vec<Markup> {
        vec![
            html! {
                (PreEscaped(format!(r#"
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
                                    "@type": "ListItem",
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
                        "name": "\#{0} - {1}",
                        "description": {2},
                        "url": "https://pointercrate.com/demonlist/{0}/"
                    }}
                    </script>
                "#, self.data.position(), self.data.name(), self.description())))
            },
            html! {
                (PreEscaped(format!("
                    <script>
                        window.list_length = {0};
                        window.extended_list_length = {1}
                    </script>", config::list_size(), config::extended_list_size()
                )))
            },
        ]
    }
}

fn dropdowns(all_demons: &[OverviewDemon], current: Option<&Demon>) -> Markup {
    let (main, extended, legacy) = if all_demons.len() < config::list_size() as usize {
        (&all_demons[..], Default::default(), Default::default())
    } else {
        let (extended, legacy) = if all_demons.len() < config::extended_list_size() as usize {
            (&all_demons[config::list_size() as usize..], Default::default())
        } else {
            (
                &all_demons[config::list_size() as usize..config::extended_list_size() as usize],
                &all_demons[config::extended_list_size() as usize..],
            )
        };

        (&all_demons[..config::list_size() as usize], extended, legacy)
    };

    html! {
        div.flex.wrap.m-center.fade#lists style="text-align: center;" {
            // The drop down for the main list:
            (dropdown(&MAIN_SECTION, main, current))
            // The drop down for the extended list:
            (dropdown(&EXTENDED_SECTION, extended, current))
            // The drop down for the legacy list:
            (dropdown(&LEGACY_SECTION, legacy, current))
        }
    }
}

fn dropdown(section: &ListSection, demons: &[OverviewDemon], current: Option<&Demon>) -> Markup {
    let format = |demon: &OverviewDemon| -> Markup {
        html! {
            a href = {"/demonlist/" (demon.position)} {
                @if section.numbered {
                    {"#" (demon.position) " - " (demon.name)}
                    br ;
                    i {
                        (demon.publisher)
                    }
                }
                @else {
                    {(demon.name)}
                    br ;
                    i {
                        (demon.publisher)
                    }
                }
            }
        }
    };

    html! {
        div {
            div.button.white.hover.no-shadow.js-toggle data-toggle-group="0" onclick={"javascript:void(DropDown.toggleDropDown('" (section.id) "'))"} {
                (section.name)
            }

            div.see-through.fade.dropdown#(section.id) {
                div.search.js-search.seperated style = "margin: 10px" {
                    input placeholder = "Filter..." type = "text" {}
                }
                p style = "margin: 10px" {
                    (section.description)
                }
                ul.flex.wrap.space {
                    @for demon in demons {
                        @match current {
                            Some(current) if current.base.position == demon.position =>
                                li.hover.white.active title={"#" (demon.position) " - " (demon.name)} {
                                    (format(demon))
                                },
                            _ =>
                                li.hover.white title={"#" (demon.position) " - " (demon.name)} {
                                    (format(demon))
                                }
                        }
                    }
                }
            }
        }
    }
}

fn submission_panel() -> Markup {
    html! {
        div.panel.fade.closable#submitter style = "display: none" {
            span.plus.cross.hover {}
            div.flex {
                form#submission-form novalidate = "" {
                    div.underlined {
                        h2 {"Record Submission"}
                    }
                    p.info-red.output {}
                    p.info-green.output {}
                    h3 {
                        "Demon:"
                    }
                    p {
                        "The demon the record was made on. Only demons in the top " (config::extended_list_size()) " are accepted. This excludes legacy demons!"
                    }
                    span.form-input.flex.col#id_demon {
                        input type = "text" name = "demon" required="" placeholder = "e. g. 'Bloodbath', 'Yatagarasu'" ;
                        p.error {}
                    }
                    h3 {
                        "Holder:"
                    }
                    p {
                        "The holder of the record. Please enter the holders Geometry Dash name here, even if their YouTube name differs!"
                    }
                    span.form-input.flex.col#id_player {
                        input type = "text" name = "demon" required="" placeholder="e. g. 'Slypp, 'KrmaL'" maxlength="50" ;
                        p.error {}
                    }
                    h3 {
                        "Progress:"
                    }
                    p {
                        "The progress made as percentage. Only values greater than the demons record requirement and smaller than or equal to 100 are accepted!"
                    }
                    span.form-input.flex.col#id_progress {
                        input type = "number" name = "progress" required="" placeholder = "e. g. '50', '98'" min="0" max="100";
                        p.error {}
                    }
                    h3 {
                        "Video: "
                    }
                    p {
                        "A proof video of the legitimancy of the given record. If the record was achieved on stream, but wasn't uploaded anywhere else, please provide a twitch link to that stream."
                        br {}

                        i { "Note: " }
                        "Please pay attention to only submit well-formed URLs!"
                    }
                    span.form-input.flex.col#id_video {
                        input type = "url" name = "video" required = "" placeholder = "e.g. 'https://youtu.be/cHEGAqOgddA'" ;
                        p.error {}
                    }
                    input.button.blue.hover type = "submit" style = "margin: 15px auto 0px;" value="Submit record";
                }
            }
        }
    }
}

fn stats_viewer(nations: &[Nationality]) -> Markup {
    html! {
        div.panel.fade.closable#statsviewer style = "display:none" {
            span.plus.cross.hover {}
            h2.underlined.pad {
                "Stats Viewer"
                (super::dropdown("International",
                    html! {
                        li.white.hover.underlined data-value = "International" {
                            span.em.em-world_map {}
                            (PreEscaped("&nbsp;"))
                            b {"WORLD"}
                            br;
                            span style = "font-size: 90%; font-style: italic" { "International" }
                        }
                    },
                    nations.iter().map(|nation| html! {
                        li.white.hover data-code = {(nation.iso_country_code)} data-value = {(nation.nation)} {
                            span class = {"flag-icon flag-icon-" (nation.iso_country_code.to_lowercase())} {}
                            (PreEscaped("&nbsp;"))
                            b {(nation.iso_country_code)}
                            br;
                            span style = "font-size: 90%; font-style: italic" {(nation.nation)}
                        }
                    })
                ))
            }
            div.flex.viewer {
                (super::filtered_paginator("stats-viewer-pagination", "/players/ranking/"))
                p.viewer-welcome {
                    "Click on a player's name on the left to get started!"
                }
                div.viewer-content {
                    div {
                        div.flex.col {
                            h3#player-name style = "font-size:1.4em; overflow: hidden" {}
                            div.stats-container.flex.space {
                                span {
                                    b {
                                        "List demons completed:"
                                    }
                                    br;
                                    span#amount-beaten {}
                                }
                                span {
                                    b {
                                        "Legacy demons completed:"
                                    }
                                    br;
                                    span#amount-legacy {}
                                }
                                span {
                                    b {
                                        "Demonlist score:"
                                    }
                                    br;
                                    span#score {}
                                }
                            }
                            div.stats-container.flex.space {
                                span {
                                    b {
                                        "Demonlist rank:"
                                    }
                                    br;
                                    span#rank {}
                                }
                                span {
                                    b {
                                        "Hardest demon:"
                                    }
                                    br;
                                    span#hardest {}
                                }
                            }
                            div.stats-container.flex.space {
                                span {
                                    b {
                                        "Demons completed:"
                                    }
                                    br;
                                    span#beaten {}
                                }
                            }
                            div.stats-container.flex.space {
                                span {
                                    b {
                                        "List demons created:"
                                    }
                                    br;
                                    span#created {}
                                }
                                span {
                                    b {
                                        "List demons published:"
                                    }
                                    br;
                                    span#published {}
                                }
                                span {
                                    b {
                                        "List demons verified:"
                                    }
                                    br;
                                    span#verified {}
                                }
                            }
                            div.stats-container.flex.space {
                                span {
                                    b {
                                        "Progress on:"
                                    }
                                    br;
                                    span#progress {}
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

fn team_panel(admins: &[User], mods: &[User], helpers: &[User]) -> Markup {
    let maybe_link = |user: &User| -> Markup {
        html! {
            li {
                @match user.youtube_channel {
                    Some(ref channel) => a target = "_blank" href = (channel) {
                        (user.name())
                    },
                    None => (user.name())
                }
            }
        }
    };

    html! {
        div.panel.fade.js-scroll-anim#editors data-anim = "fade" {
            div.underlined {
                h2 {
                    "List Editors:"
                }
            }
            p {
                "Contact any of these people if you have problems with the list or want to see a specific thing changed."
            }
            ul style = "line-height: 30px" {
                @for admin in admins {
                    b {
                        (maybe_link(admin))
                    }
                }
                @for moderator in mods {
                    (maybe_link(moderator))
                }
            }
            div.underlined {
                h2 {
                    "List Helpers"
                }
            }
            p {
                "Contact these people if you have any questions regarding why a specific record was rejected. Do not needlessly bug them about checking submissions though!"
            }
            ul style = "line-height: 30px" {
                @for helper in helpers {
                    (maybe_link(helper))
                }
            }
        }
    }
}

fn rules_panel() -> Markup {
    html! {
        div#rules.panel.fade.js-scroll-anim.js-collapse data-anim = "fade" {
            h2.underlined.pad.clickable {
                "Rules:"
                span.arrow.hover {}
            }
            ul.roman.js-collapse-content style="display:none" {
                h3 {
                    "Demon rules:"
                }
                li {
                    span {
                        "Demons need to be rated to be included on this list"
                    }
                }
                li {
                    span {
                        "List demons that receive a hacked update changing difficulty will be moved to the legacy section of the list. Alternatively, if a demon gets a hacked update before being list-worthy, it will not get added. However, a demon whose original verification was hacked will still get on the list."
                    }
                }
                h3 {
                    "Submission rules:"
                }
                li {
                    span {
                        "Records must be legitimate and either uploaded on YouTube, Vimeo, Bilibili or streamed to be added to the list."
                    }
                }
                li {
                    span {
                        " Anyone posting illegitimate recordings and passing them off as legit will have their records removed from the list. Illegitimate records include, but aren't limited to, speedhacks, noclip, auto, nerfs, macros, fps bypass, etc."
                    }
                }
                li {
                    span {
                        "Records on a level must be in normal mode and on the live version of the level or on an appropriate bug fixed/low detail copy of said level. Please refer to the bugfix and LDM guidelines."
                    }
                }
                li {
                    span {
                        "The record holder must meet the percentage requirement of a level in order to be added to the list for said level."
                    }
                }
                h3 {
                    "General guidelines:"
                }
                li {
                    span {
                        "Verifications are not counted as records on the list, but still award points."
                    }
                }
                li {
                    span {
                        "Being in a group in which people beat levels for the same channel will cause your records to be temporarily removed from the list."
                    }
                }
                h3 {
                    "Bugfix and LDM guidelines:"
                }
                li {
                    span {
                        "Records using a level's built-in LDM are always eligible. "
                    }
                }
                li {
                    span {
                        "Records on appropriate LDM copies of levels are eligible. Please take contact with a List Moderator if you are unsure of which decorations can or cannot be removed. Generally speaking, a LDM copy should not remove decorations that obstruct the player's vision, blind transitions, flashes or boss fights, for example. Referring to the first guideline, if the previously stated decorations are removed in a level's built-in LDM though, it is perfectly fine to use it."
                    }
                }
                li {
                    span {
                        "Records on appropriate bugfix copies of levels for different refresh rates are eligible. Please take contact with a List Moderator if you are unsure of what is or isn't a bug."
                    }
                }
            }
        }
    }
}

fn submit_panel() -> Markup {
    html! {
        div#submit.panel.fade.js-scroll-anim data-anim = "fade" {
            div.underlined {
                h2 {
                    "Submit Records:"
                }
            }
            p {
                "Note: Please do not submit nonsense, it only makes it harder for us all and will get you banned. Also note that the form rejects duplicate submissions."
            }
            a.blue.hover.button.slightly-rounded.js-scroll data-destination = "submitter" data-reveal = "true" {
                "Submit a record!"
            }
        }
    }
}

fn stats_viewer_panel() -> Markup {
    html! {
        div#stats.panel.fade.js-scroll-anim data-anim = "fade" {
            div.underlined {
                h2 {
                    "Stats Viewer"
                }
            }
            p {
                "Get a detailed overview of who completed the most, created the most demons or beat the hardest demons! There is even a leaderboard to compare yourself to the very best!"
            }
            a.blue.hover.button.slightly-rounded.js-scroll#show-stats-viewer data-destination = "statsviewer" data-reveal = "true" {
                "Open the stats viewer!"
            }
        }
    }
}

fn discord_panel() -> Markup {
    html! {
        div.panel.fade.js-scroll-anim#discord data-anim = "fade" {
            iframe.js-delay-attr style = "width: 100%; height: 400px;" allowtransparency="true" frameborder = "0" data-attr = "src_old" data-attr-value = "https://discordapp.com/widget?id=395654171422097420&theme=light" {}
            p {
                "Join the official Demonlist discord server, where you can get in touch with the demonlist team!"
            }
        }
    }
}
