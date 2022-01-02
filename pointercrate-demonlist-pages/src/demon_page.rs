use crate::{
    components::{
        submitter::{submit_panel, RecordSubmitter},
        team::Team,
    },
    statsviewer::stats_viewer_panel,
};
use chrono::NaiveDateTime;
use maud::{html, Markup, PreEscaped, Render};
use pointercrate_core_pages::{config as page_config, PageFragment, Script};
use pointercrate_demonlist::{
    config as list_config,
    demon::{Demon, FullDemon},
};
use pointercrate_integrate::gd::{DemonRating, GDIntegrationResult, LevelRating, Thunk};
use url::Url;

#[derive(Debug)]
pub struct DemonMovement {
    pub from_position: i16,
    pub at: NaiveDateTime,
}

pub struct DemonPage {
    pub team: Team,
    pub demonlist: Vec<Demon>,
    pub data: FullDemon,
    pub movements: Vec<DemonMovement>,
    pub integration: GDIntegrationResult,
}

impl PageFragment for DemonPage {
    fn title(&self) -> String {
        format!(
            "#{} - {} - Geometry Dash Demonlist",
            self.data.demon.base.position,
            self.data.demon.base.name // FIXME: flatten the structs, holy shit
        )
    }

    fn description(&self) -> String {
        if let GDIntegrationResult::Success(ref level, ..) = self.integration {
            if let Some(Thunk::Processed(ref description)) = level.description {
                return format!("{}: {}", self.title(), description.0)
            }
        }
        format!("{}: <No Description Provided>", self.title())
    }

    fn additional_scripts(&self) -> Vec<Script> {
        vec![
            Script::new("https://cdn.jsdelivr.net/chartist.js/latest/chartist.min.js"),
            Script::module("/static/js/modules/formv2.js"),
            Script::module("/static/js/modules/demonlistv2.js"),
            Script::module("/static/js/demonlist.v2.2.js"),
        ]
    }

    fn additional_stylesheets(&self) -> Vec<String> {
        vec![
            "https://cdn.jsdelivr.net/chartist.js/latest/chartist.min.css".to_string(),
            "/static/css/demonlist.v2.1.css".to_string(),
            "/static/css/sidebar.css".to_string(),
        ]
    }

    fn head_fragment(&self) -> Markup {
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
            (PreEscaped(format!("
                <script>
                    window.list_length = {0};
                    window.extended_list_length = {1}
                </script>", list_config::list_size(), list_config::extended_list_size()
            )))
        }
    }

    fn body_fragment(&self) -> Markup {
        let dropdowns = super::dropdowns(&self.demonlist.iter().collect::<Vec<_>>()[..], Some(&self.data.demon));

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
            (super::besides_sidebar_ad())
            (dropdowns)

            div.flex.m-center.container {
                main.left {
                    div.panel.fade style = "padding: 0px; height: 90px" {
                        (PreEscaped(format!(r#"
                        <script async src="https://pagead2.googlesyndication.com/pagead/js/adsbygoogle.js?client={0}"
     crossorigin="anonymous"></script>
<!-- Demonpage Banner ad -->
<ins class="adsbygoogle"
     style="display:inline-block;width:728px;height:90px"
     data-ad-client="{0}"
     data-ad-slot="4829214686"></ins>
<script>
     (adsbygoogle = window.adsbygoogle || []).push({{}});
</script>
                        "#, page_config::adsense_publisher_id())))
                    }
                    (RecordSubmitter::new(false, &self.demonlist))
                    (self.demon_panel())
                    div.panel.fade.js-scroll-anim.js-collapse data-anim = "fade" {
                        h2.underlined.pad {
                            "Position History"
                            span.arrow.hover {}
                        }
                        div.ct-chart.ct-perfect-fourth.js-collapse-content#position-chart style="display:none" {}
                    }
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
                    (self.team)
                    (super::sidebar_ad())
                    (super::rules_panel())
                    (submit_panel())
                    (stats_viewer_panel())
                    (super::discord_panel())
                }
            }
        }
    }
}

impl DemonPage {
    fn demon_panel(&self) -> Markup {
        let position = self.data.demon.base.position;
        let name = &self.data.demon.base.name;

        let score100 = self.data.demon.score(100);
        let score_requirement = self.data.demon.score(self.data.demon.requirement);

        html! {
            section.panel.fade.js-scroll-anim data-anim = "fade" {
                div.underlined {
                    h1#demon-heading style = "overflow: hidden"{
                        @if self.data.demon.base.position != 1 {
                            a href=(format!("/demonlist/{:?}", self.data.demon.base.position - 1)) {
                                i class="fa fa-chevron-left" style="padding-right: 5%" {}
                            }
                        }
                        (name)
                        @if position as usize != self.demonlist.len() {
                            a href=(format!("/demonlist/{:?}", position + 1)) {
                                i class="fa fa-chevron-right" style="padding-left: 5%" {}
                            }
                        }
                    }
                    (PreEscaped(format!(r#"
                    <script>
                    document.getElementById("demon-heading").addEventListener('click', () => navigator.clipboard.writeText('https://pointercrate.com/demonlist/permalink/{}/'))
                    </script>
                    "#, self.data.demon.base.id)))
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
                @if let GDIntegrationResult::Success(ref level, ..) = self.integration {
                    @if let Some(Thunk::Processed(ref description)) = level.description {
                        div.underlined.pad {
                            q {
                                (description.0)
                            }
                        }
                    }
                }
                @if let Some(ref video) = self.data.demon.video {
                    @if let Some(embedded_video) = embed(video) {
                        iframe."ratio-16-9"."js-delay-attr" style="width:90%; margin: 15px 5%" allowfullscreen="" data-attr = "src" data-attr-value = (embedded_video) {"Verification Video"}
                    }
                }
                div.underlined.pad.flex.wrap#level-info {
                    @match &self.integration {
                        GDIntegrationResult::DemonNotFoundByName => {
                            p.info-red {
                                "A demon with this name was not found on the Geometry Dash servers. Please notify a list moderator of this, as it means they most likely misspelled the name!"
                            }
                        }
                        GDIntegrationResult::DemonNotYetCached => {
                            p.info-yellow {
                                "The data from the Geometry Dash servers has not yet been cached. Please wait a bit and refresh the page."
                            }
                        }
                        GDIntegrationResult::LevelDataNotFound => {
                            p.info-red {
                                "It seems like this level has been deleted from the Geometry Dash servers"
                            }
                        }
                        GDIntegrationResult::LevelDataNotCached => {
                            p.info-red {
                                "This demon's level data is not stored in our database, even though the demon ID was successfully resolved. This either indicates a (hopefully temporary) inconsistent database state, or an error in dash-rs' level data processing. If this error persists, please contact an administrator!"
                            }
                        }
                        GDIntegrationResult::Success(level, level_data, song) => {
                            span {
                                b {
                                    "Level Password: "
                                }
                                br;
                                (level_data.password)
                            }
                            span {
                                b {
                                    "Level ID: "
                                }
                                br;
                                (level.level_id)
                            }
                            span {
                                b {
                                    "Level length: "
                                }
                                br;
                                @match level_data.level_data {
                                    Thunk::Processed(ref objects) => {
                                        @let length_in_seconds = objects.length_in_seconds();

                                        (format!("{}m:{:02}s", (length_in_seconds as i32)/ 60, (length_in_seconds as i32) % 60))
                                    }
                                    _ => "unreachable!()"
                                }
                            }
                            span {
                                b {
                                    "Object count: "
                                }
                                br;
                                @match level_data.level_data {
                                    Thunk::Processed(ref objects) => (objects.objects.len()),
                                    _ => "unreachable!()"
                                }
                            }
                            span {
                                b {
                                    "In-Game Difficulty: "
                                }
                                br;
                                @match level.difficulty {
                                    LevelRating::NotAvailable => "Unrated",
                                    LevelRating::Demon(demon_rating) => @match demon_rating {
                                        DemonRating::Easy => "Easy Demon",
                                        DemonRating::Medium => "Medium Demon",
                                        DemonRating::Hard => "Hard Demon",
                                        DemonRating::Insane => "Insane Demon",
                                        DemonRating::Extreme => "Extreme Demon",
                                        _ => "???"
                                    },
                                    _ => "Level not rated demon, list mods fucked up"
                                }
                            }
                            span {
                                b {
                                    "Created in:"
                                }
                                br;
                                (level.gd_version)
                            }
                            @if let Some(song) = song {
                                span style = "width: 100%"{
                                    b {
                                        "Newgrounds Song:"
                                    }
                                    br;
                                    @match song.link {
                                        Thunk::Processed(ref link) => a.link href = (link.0) {(song.name) " by " (song.artist) " (ID " (song.song_id) ")"},
                                        _ => "unreachable!()"
                                    }
                                }
                            }
                        }
                    }
                    @if position <= list_config::extended_list_size() {
                        span {
                            b {
                                "Demonlist score (100%): "
                            }
                            br;
                            (format!("{:.2}", score100))
                        }
                    }
                    @if position <= list_config::list_size(){
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
        let _name = &self.data.demon.base.name;

        html! {
            @if !self.data.records.is_empty() || position <= list_config::extended_list_size() {
                section.records.panel.fade.js-scroll-anim data-anim = "fade" {
                    div.underlined.pad {
                        h2 {
                            "Records"
                        }
                        @if position <= list_config::list_size() {
                            h3 {
                                (self.data.demon.requirement) "% or better required to qualify"
                            }
                        }
                        @else if position <= list_config::extended_list_size() {
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
                            @if position > list_config::extended_list_size() {
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
                                    th.blue {}
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
                                            @if let Some(ref nationality) = record.nationality {
                                                span.flag-icon.{"flag-icon-"(nationality.iso_country_code.to_lowercase())} title = (nationality.nation) {}
                                            }
                                        }
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
                                                     (host(video))
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

fn host(video: &str) -> &str {
    match Url::parse(video).unwrap().domain().unwrap() {
        "www.youtube.com" => "YouTube",
        "www.twitch.tv" => "Twitch",
        "everyplay.com" => "Everyplay",
        "www.bilibili.com" => "Bilibili",
        "vimeo.com" => "Vimeo",
        host => panic!("{}", host),
    }
}

fn embed(video: &str) -> Option<String> {
    // Video URLs need to be wellformed once we get here!
    let url = Url::parse(video).unwrap();

    match url.domain()? {
        "www.youtube.com" => {
            let video_id = url
                .query_pairs()
                .find_map(|(key, value)| if key == "v" { Some(value) } else { None })?;

            Some(format!("https://www.youtube.com/embed/{}", video_id))
        },
        "www.twitch.tv" => {
            // per validation always of the form 'https://www.twitch.tv/videos/[video id]/'
            let mut url_segment = url.path_segments()?;
            url_segment.next()?;
            let video_id = url_segment.next()?;

            Some(format!("https://player.twitch.tv/?video={}&autoplay=false", video_id))
        },
        _ => None,
    }
}
