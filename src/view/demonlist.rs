use super::Page;
use crate::{
    actor::{database::GetDemonlistOverview, http::GetDemon},
    api::PCResponder,
    config::{EXTENDED_LIST_SIZE, LIST_SIZE},
    error::PointercrateError,
    model::{
        demon::{self, Demon, DemonWithCreatorsAndRecords, PartialDemon},
        player::RankedPlayer,
        user::User,
    },
    state::PointercrateState,
    video,
};
use actix_web::{AsyncResponder, FromRequest, HttpRequest, Path, Responder};
use gdcf::model::{level::Password, Creator, Level};
use joinery::Joinable;
use maud::{html, Markup, PreEscaped};
use tokio::prelude::{Future, IntoFuture};

struct ListSection {
    name: &'static str,
    description: &'static str,
    id: &'static str,
    numbered: bool,
}

static MAIN_SECTION: ListSection = ListSection {
    name: "Main List",
    description: "The main section of the demonlist. These demons are the hardest rated levels in the game. Records are accepted above a given threshold and award a large amount of points!",
    id: "mainlist",
    numbered: true,
};

static EXTENDED_SECTION: ListSection = ListSection {
    name: "Extended List",
    description: "These are demons that dont qualify for the main section of the list, but are still of high relevance. Only 100% records are accepted for these demons! Note that non-100% that were submitted/approved before a demon fell off the main list will be retained",
    id: "extended",
    numbered: true
};

static LEGACY_SECTION: ListSection  = ListSection{
    name: "Legacy List",
    description: "These are demons that used to be in the top 100, but got pushed off as new demons were added. They are here for nostalgic reasons. This list is in no order whatsoever and will not be maintained any longer at all. This means no new records will be added for these demons.",
    id: "legacy",
    numbered: false,
};

#[derive(Debug)]
pub struct DemonlistOverview {
    pub demon_overview: Vec<PartialDemon>,
    pub admins: Vec<User>,
    pub mods: Vec<User>,
    pub helpers: Vec<User>,
    pub ranking: Vec<RankedPlayer>,
}

pub fn overview_handler(req: &HttpRequest<PointercrateState>) -> PCResponder {
    let req_clone = req.clone();

    req.state()
        .database(GetDemonlistOverview)
        .map(move |overview| overview.render(&req_clone).respond_to(&req_clone).unwrap())  // We can unwrap here since respond_to in the Responder implementation for PreEscaped<String> always returns an Ok value.
        .responder()
}

impl Page for DemonlistOverview {
    fn title(&self) -> String {
        "Geometry Dash Demonlist".to_string()
    }

    fn description(&self) -> String {
        "The official Geometry Dash Demonlist on pointercrate!".to_string()
    }

    fn scripts(&self) -> Vec<&str> {
        vec!["js/demonlist.v2.1.js"]
    }

    fn stylesheets(&self) -> Vec<&str> {
        vec!["css/demonlist.v2.1.css", "css/sidebar.css"]
    }

    fn body(&self, req: &HttpRequest<PointercrateState>) -> Markup {
        let dropdowns = dropdowns(req, &self.demon_overview, None);

        html! {
            (dropdowns)

            div.flex.m-center#container {
                div.left {
                    (submission_panel())
                    (stats_viewer(&self.ranking))
                    @for demon in &self.demon_overview {
                        @if demon.position <= *EXTENDED_LIST_SIZE {
                            div.panel.fade {
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

    fn head(&self, _: &HttpRequest<PointercrateState>) -> Vec<Markup> {
        vec![
            html! {
            (PreEscaped(r#"
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
                    "description": "The official Geometry Dash Demonlist on pointercrate!",
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
                    </script>", *LIST_SIZE, *EXTENDED_LIST_SIZE)
                ))
            },
        ]
    }
}

#[derive(Debug)]
pub struct Demonlist {
    overview: DemonlistOverview,
    data: DemonWithCreatorsAndRecords,
    server_level: Option<Level<u64, Creator>>,
}

pub fn handler(req: &HttpRequest<PointercrateState>) -> PCResponder {
    let req_clone = req.clone();
    let state = req.state().clone();

    Path::<i16>::extract(req)
        .map_err(|_| PointercrateError::bad_request("Demon position must be integer"))
        .into_future()
        .and_then(move |position| {
            state
                .get(position.into_inner())
                .and_then(move |data: DemonWithCreatorsAndRecords| {
                    state
                        .database(GetDemonlistOverview)
                        .and_then(move |overview| {
                            state
                                .gdcf
                                .send(GetDemon(data.demon.name.clone()))
                                .map_err(PointercrateError::internal)
                                .map(move |demon| {
                                    Demonlist {
                                        overview,
                                        data,
                                        server_level: demon,
                                    }
                                    .render(&req_clone)
                                    .respond_to(&req_clone)
                                    .unwrap()
                                })
                        })
                })
        })
        .responder()
}

impl Page for Demonlist {
    fn title(&self) -> String {
        format!(
            "#{} - {} - Geometry Dash Demonlist",
            self.data.demon.position, self.data.demon.name
        )
    }

    fn description(&self) -> String {
        if let Some(ref level) = self.server_level {
            if let Some(ref description) = level.base.description {
                return format!("{}: {}", self.title(), description)
            }
        }
        format!("{}: <No Description Provided>", self.title())
    }

    fn scripts(&self) -> Vec<&str> {
        vec!["js/demonlist.v2.1.js"]
    }

    fn stylesheets(&self) -> Vec<&str> {
        vec!["css/demonlist.v2.1.css", "css/sidebar.css"]
    }

    fn body(&self, req: &HttpRequest<PointercrateState>) -> Markup {
        let dropdowns = dropdowns(req, &self.overview.demon_overview, Some(&self.data.demon));
        let score100 = demon::score(
            self.data.demon.position,
            100,
            self.overview.demon_overview.len(),
        );
        let score_requirement = demon::score(
            self.data.demon.position,
            self.data.demon.requirement,
            self.overview.demon_overview.len(),
        );

        html! {
            (dropdowns)

            div.flex.m-center#container {
                div.left {
                    (submission_panel())
                    (stats_viewer(&self.overview.ranking))
                    div.panel.fade.js-scroll-anim data-anim = "fade" {
                        div.underlined {
                            h1 style = "overflow: hidden"{
                                (self.data.demon.name)
                            }
                            h3 {
                                @if self.data.creators.0.len() > 3 {
                                    "by " (self.data.creators.0[0].name) " and "
                                    div.tooltip {
                                        "more"
                                        div.tooltiptext.fade {
                                            (self.data.creators.0.iter().map(|player| &player.name).join_with(", ").to_string())
                                        }
                                    }
                                    ", " (self.data.short_headline())
                                }
                                @else {
                                    (self.data.headline())
                                }
                            }
                        }
                        @if let Some(ref level) = self.server_level {
                            @if let Some(ref description) = level.base.description {
                                div.underlined.pad {
                                    q {
                                        (description)
                                    }
                                }
                            }
                        }
                        @if let Some(ref video) = self.data.demon.video {
                            iframe."ratio-16-9"."js-delay-attr" style="width:90%; margin: 15px 5%" data-attr = "src" data-attr-value = (video::embed(video)) {"Verification Video"}
                        }
                        div.underlined.pad.flex.wrap#level-info {
                            @if let Some(ref level) = self.server_level {
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
                                    (level.base.length.to_string())
                                }
                                span {
                                    b {
                                        div.tooltip {
                                        "Reported object count: "
                                            div.tooltiptext.fade {
                                                "The object count reported by the Geometry Dash servers is mostly wrong. It is provided by the game client and not validated by the servers. Additionally to the official client having a set of bugs that can cause the count to be wrong, the servers also store the object count in a 16 bit unsigned integer. This means that if a level has more than 65535 objects, it'll simply display 65535. Furthermore, since the count isn't validated, it is possible to manually send fake requests to the servers to set the object count to arbitrary values."
                                            }
                                        }
                                    }
                                    br;
                                    (level.base.object_amount)
                                }
                            }
                            @if self.data.demon.position <= *EXTENDED_LIST_SIZE {
                                span {
                                    b {
                                        "Demonlist score (100%): "
                                    }
                                    br;
                                        (format!("{:.2}", score100))
                                }
                            }
                            @if self.data.demon.position <= *LIST_SIZE {
                                span {
                                    b {
                                        "Demonlist score (" (self.data.demon.requirement) "%)"
                                    }
                                    br;
                                    (format!("{:.2}", score_requirement))
                                }
                            }
                        }
                    }
                    (rules_panel())
                    @if !self.data.records.is_empty() || self.data.demon.position <= *EXTENDED_LIST_SIZE {
                        div.records.panel.fade.js-scroll-anim data-anim = "fade" {
                            div.underlined.pad {
                                h2 {
                                    "Records"
                                }
                                @if self.data.demon.position <= *LIST_SIZE {
                                    h3 {
                                        (self.data.demon.requirement) "% or better required to qualify"
                                    }
                                }
                                @else if self.data.demon.position <= *EXTENDED_LIST_SIZE {
                                    h3 {
                                        "100% required to qualify"
                                    }
                                }
                                @if !self.data.records.is_empty() {
                                    h4 {
                                        (self.data.records.len())
                                        " records registered, out of which "
                                        (self.data.records.iter().filter(|record| record.progress == 100).count())
                                        " are 100%"
                                    }
                                }
                            }
                            @if self.data.records.is_empty() {
                                h3 {
                                    @if self.data.demon.position > *EXTENDED_LIST_SIZE {
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
                                            th {
                                                "Record Holder"
                                            }
                                            th {
                                                "Progress"
                                            }
                                            th.video-link {
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
                                                         a.link href = (video) {
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

    fn head(&self, _: &HttpRequest<PointercrateState>) -> Vec<Markup> {
        vec![
            html! {
                (PreEscaped(format!(r#"
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
                "#, self.data.demon.position, self.data.demon.name, self.description())))
            },
            html! {
                (PreEscaped(format!("
                    <script>
                        window.list_length = {0};
                        window.extended_list_length = {1}
                    </script>", *LIST_SIZE, *EXTENDED_LIST_SIZE
                )))
            },
        ]
    }
}

fn dropdowns(
    req: &HttpRequest<PointercrateState>, all_demons: &[PartialDemon], current: Option<&Demon>,
) -> Markup {
    let (main, extended, legacy) = if all_demons.len() < *LIST_SIZE as usize {
        (&all_demons[..], Default::default(), Default::default())
    } else {
        let (extended, legacy) = if all_demons.len() < *EXTENDED_LIST_SIZE as usize {
            (&all_demons[*LIST_SIZE as usize..], Default::default())
        } else {
            (
                &all_demons[*LIST_SIZE as usize..*EXTENDED_LIST_SIZE as usize],
                &all_demons[*EXTENDED_LIST_SIZE as usize..],
            )
        };

        (&all_demons[..*LIST_SIZE as usize], extended, legacy)
    };

    html! {
        div.flex.wrap.m-center.fade#lists style="text-align: center;" {
            // The drop down for the main list:
            (dropdown(req, &MAIN_SECTION, main, current))
            // The drop down for the extended list:
            (dropdown(req, &EXTENDED_SECTION, extended, current))
            // The drop down for the legacy list:
            (dropdown(req, &LEGACY_SECTION, legacy, current))
        }
    }
}

fn dropdown(
    req: &HttpRequest<PointercrateState>, section: &ListSection, demons: &[PartialDemon],
    current: Option<&Demon>,
) -> Markup {
    let format = |demon: &PartialDemon| -> Markup {
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
                div.search.seperated {
                    input placeholder = "Filter..." type = "text" {}
                }
                p style = "margin: 10px" {
                    (section.description)
                }
                ul.flex.wrap.space {
                    @for demon in demons {
                        @match current {
                            Some(current) if current.position == demon.position =>
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
            div.underlined {
                h2 {"Record Submission"}
            }
            i {
                p#submission-output style = "margin: 5px auto; display: none" {}
            }
            div.flex {
                form#submission-form onsubmit = "return submitter.submit();" {
                    h3 {
                        "Demon:"
                    }
                    p {
                        "The demon the record was made on. Only demons in the top " (EXTENDED_LIST_SIZE) " are accepted. This excludes legacy demons!"
                    }
                    input#id_demon type = "text" name = "demon" required="" placeholder = "e. g. 'Bloodbath', 'Yatagarasu'" ;
                    h3 {
                        "Holder:"
                    }
                    p {
                        "The holder of the record. Please enter the holders Geometry Dash name here, even if their YouTube name differs!"
                    }
                    input#id_player type = "text" name = "demon" required="" placeholder="e. g. 'Slypp, 'Krazyman50'" maxlength="50" ;
                    h3 {
                        "Progress:"
                    }
                    p {
                        "The progress made as percentage. Only values greater than the demons record requirement and smaller than or equal to 100 are accepted!"
                    }
                    input#id_progress type = "number" name = "progress" required="" placeholder = "e. g. '50', '98'" min="0" max="100" ;
                    h3 {
                        "Video: "
                    }
                    p {
                        "A proof video of the legitimancy of the given record. If the record was achieved on stream, but wasn't uploaded anywhere else, please provide a twitch link to that stream."
                        br {}

                        i { "Note: " }
                        "Please pay attention to only submit well-formed URLs!"
                    }
                    input#id_video type = "url" name = "video" required = "" placeholder = "e.g. 'https://youtu.be/cHEGAqOgddA'" ;
                    input.button.blue.hover type = "submit" style = "margin: 15px auto 0px;" {
                        "Log in"
                    }
                }
            }
        }
    }
}

fn stats_viewer(players: &[RankedPlayer]) -> Markup {
    html! {
        div.panel.fade.closable#statsviewer style = "display:none" {
            span.plus.cross.hover {}
            h2.underlined.pad {
                "Stats Viewer"
            }
            div.flex#stats-viewer-cont {
                div.flex#player-selection style="flex-direction: column"{
                    div.search.seperated style="flex-grow:0" {
                        input placeholder = "Filter..." type = "text" style = "height: 1em";
                    }
                    div style="position:relative; margin: 0px 10px 10px; min-height: 400px" {
                        ul#players style = "position: absolute; top: 0px; bottom:0px; left: 0px; right:0px" {
                            @for player in players {
                                li.white.hover data-id = (player.id) data-rank = (player.rank) {
                                    (player.name)
                                    i {
                                        (format!("{:.2}", player.score))
                                    }
                                }
                            }
                        }
                    }
                }
                div {
                    p#error-output style = "text-align: center" {
                        "Click on a player's name on the left to get started!"
                    }
                    div#stats-data style = "display:none" {
                        div.flex.col {
                            div.container.flex.space {
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
                            div.container.flex.space {
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
                            div.container.flex.space {
                                span {
                                    b {
                                        "Demons completed:"
                                    }
                                    br;
                                    span#beaten {}
                                }
                            }
                            div.container.flex.space {
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
                            div.container.flex.space {
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
        did#rules.panel.fade.flex.js-scroll-anim data-anim = "fade" style = "flex-direction: column" {
            div.underlined {
                h2 {
                    "Rules:"
                }
            }
            ul.roman {
                li {
                    span {
                        "Anyone posting illegitimate recordings (hacked, cut, stolen, automated gameplay, no-clip, etc.) and passing them of as legit will have all their records removed from this list"
                    }
                }
                li {
                    span {
                        "Demons need to be rated to be included on this list"
                    }
                }
                li {
                    span {
                        "If you verified a level on this list, your record for it won't be included - You get points for your verification though"
                    }
                }
                li {
                    span {
                        "If a record has been added, it is legit and was either streamed or has a full video uploaded"
                    }
                }
                li {
                    span {
                        "The record holder must meet the percentage requirement in order to be added to the list for that level"
                    }
                }
                li {
                    span {
                        "Be polite about suggesting changes. We probably won't listed to you if you're rude or forceful about it"
                    }
                }
                li {
                    span {
                        "Being in a group in which people beat levels for the same channel, yet passing that channel of as being a single person's, can cause your records to be temporarily removed from this list"
                    }
                }
                li {
                    span {
                        "Records made using the FPS bypass are"
                        i { " not " }
                        "accepted"
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
                "Note: Please do not submit nonsense, it only makes it harder for us all and will get you banned. Also note that the form rejects duplicate submission"
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
            a.blue.hover.button.slightly-rounded.js-scroll data-destination = "statsviewer" data-reveal = "true" {
                "Open the stats viewer!"
            }
        }
    }
}

fn discord_panel() -> Markup {
    html! {
        div.panel.fade.js-scroll-anim#discord data-anim = "fade" {
            iframe.js-delay-attr style = "width: 100%; height: 400px;" allowtransparency="true" frameborder = "0" data-attr = "src" data-attr-value = "https://discordapp.com/widget?id=395654171422097420&theme=light" {}
            p {
                "Join the official demonlist discord server, where you can get in touch with the demonlist team!"
            }
        }
    }
}
