use crate::components::P;
use crate::submit_record::submit_record_panel;
use crate::{components::team::Team, statsviewer::stats_viewer_panel};
use chrono::NaiveDateTime;
use maud::{html, Markup, PreEscaped};
use pointercrate_core::{localization::tr, trp};
use pointercrate_core_pages::{head::HeadLike, trp_html, PageFragment};
use pointercrate_demonlist::{
    config::{self as list_config, extended_list_size},
    demon::{Demon, FullDemon},
};
use pointercrate_integrate::gd::{DemonRating, IntegrationLevel, LevelRating, Thunk};
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
    pub integration: Option<IntegrationLevel>,
}

impl From<DemonPage> for PageFragment {
    fn from(page: DemonPage) -> Self {
        PageFragment::new(page.title(), page.description())
            .script("https://cdn.jsdelivr.net/chartist.js/latest/chartist.min.js")
            .module("/static/core/js/modules/form.js")
            .module("/static/demonlist/js/modules/demonlist.js")
            .module("/static/demonlist/js/demonlist.js")
            .stylesheet("https://cdn.jsdelivr.net/chartist.js/latest/chartist.min.css")
            .stylesheet("/static/demonlist/css/demonlist.css")
            .stylesheet("/static/core/css/sidebar.css")
            .head(page.head())
            .body(page.body())
    }
}

impl DemonPage {
    fn title(&self) -> String {
        let mut title = format!(
            "{} - Geometry Dash Demonlist",
            self.data.demon.base.name // FIXME: flatten the structs, holy shit
        );

        if self.data.demon.base.position <= extended_list_size() {
            title = format!("#{} - {}", self.data.demon.base.position, title);
        }

        title
    }

    fn description(&self) -> String {
        if let Some(ref level) = self.integration {
            if let Some(Thunk::Processed(ref description)) = level.description {
                return format!("{}: {}", self.title(), description);
            }
        }
        format!("{}: <No Description Provided>", self.title())
    }

    fn head(&self) -> Markup {
        html! {
            (PreEscaped(r##"
                <script type="application/ld+json">
                {
                    "@context": "http://schema.org",
                    "@type": "WebPage",
                    "breadcrumb": {
                        "@type": "BreadcrumbList",
                        "itemListElement": [{
                                "@type": "ListItem",
                                "position": 1,
                                "item": {
                                    "@id": "https://pointercrate.com/",
                                    "name": "pointercrate"
                                }
                            },{
                                "@type": "ListItem",
                                "position": 2,
                                "item": {
                                    "@id": "https://pointercrate.com/demonlist/",
                                    "name": "demonlist"
                                }
                            },{
                                "@type": "ListItem",
                                "position": 3,
                                "item": {
                                    "@id": "https://pointercrate.com/demonlist/"##)) (self.data.position()) (PreEscaped(r##"/",
                                    "name": ""##)) (self.data.name()) (PreEscaped(r##""
                                }
                            }
                        ]
                    },
                    "name": "#"##)) (self.data.position()) " - " (self.data.name()) (PreEscaped(r##"","description": ""##)) (self.description().replace(r"\", r"\\")) (PreEscaped(r##"",
                    "url": "https://pointercrate.com/demonlist/{0}/"
                }
                </script>
            "##))
            (PreEscaped(format!("
                <script>
                    window.list_length = {0};
                    window.extended_list_length = {1};
                    window.demon_id = {2};
                </script>", list_config::list_size(), list_config::extended_list_size(), self.data.demon.base.id
            )))
        }
    }

    fn body(&self) -> Markup {
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
            (dropdowns)

            div.flex.m-center.container {
                main.left {
                    (self.demon_panel())
                    div.panel.fade.js-scroll-anim.js-collapse data-anim = "fade" {
                        h2.underlined.pad {
                            (tr("movements"))
                            span.arrow.hover #history-trigger {}
                        }
                        div.js-collapse-content style="display:none"  {
                            div.ct-chart.ct-perfect-fourth #position-chart style="display:none"{}

                            table #history-table{
                                tbody #history-table-body {
                                    tr {
                                        th.blue {
                                            (tr("movements.date"))
                                        }
                                        th.blue {
                                            (tr("movements.change"))
                                        }
                                        th.blue {
                                            (tr("movements-newposition"))
                                        }
                                        th.blue {
                                            (tr("movements-reason"))
                                        }
                                    }
                                }
                            }
                        }
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
                    (super::rules_panel())
                    (submit_record_panel(Some(self.data.position())))
                    (stats_viewer_panel())
                    (super::discord_panel())
                }
            }
        }
    }

    fn demon_panel(&self) -> Markup {
        let position = self.data.demon.base.position;
        let name = &self.data.demon.base.name;

        let score100 = self.data.demon.score(100);
        let score_requirement = self.data.demon.score(self.data.demon.requirement);

        let verified_and_published = html! {
            @if self.data.demon.publisher == self.data.demon.verifier {
                (trp_html!(
                    "demon-headline.same-verifier-publisher",
                    "publisher" = html! {(P(&self.data.demon.publisher, None))}
                ))
            }
            @else {
                (trp_html!(
                    "demon-headline.unique-verifier-publisher",
                    "publisher" = html! {(P(&self.data.demon.publisher, None))},
                    "verifier" = html! {(P(&self.data.demon.verifier, None))}
                ))
            }
        };

        html! {
            section.panel.fade.js-scroll-anim data-anim = "fade" {
                div.underlined {
                    h1 #demon-heading style = "overflow: hidden"{
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
                    document.getElementById("demon-heading").addEventListener('click', () => navigator.clipboard.writeText('https://pointercrate.com/demonlist/permalink/{}/?redirect'))
                    </script>
                    "#, self.data.demon.base.id)))
                    h3 {
                        @match &self.data.creators[..] {
                            [] => { (trp_html!(
                                "demon-headline.no-creators",
                                "verified-and-published" = verified_and_published
                            )) },
                            [creator] => {
                                @if creator == &self.data.demon.publisher && creator == &self.data.demon.verifier {
                                    (trp_html!("demon-headline-by", "creator" = html!{(P(creator, None))}))
                                }
                                @else if creator != &self.data.demon.publisher && creator != &self.data.demon.verifier {
                                    (trp_html!(
                                        "demon-headline.one-creator",
                                        "creator" = html!{(P(creator, None))},
                                        "verified-and-published" = verified_and_published
                                    ))
                                }
                                @else if creator == &self.data.demon.publisher {
                                    (trp_html!(
                                        "demon-headline.one-creator-is-publisher",
                                        "creator" = html!{(P(creator, None))},
                                        "verifier" = html!{(P(&self.data.demon.verifier, None))}
                                    ))
                                }
                                @else {
                                    (trp_html!(
                                        "demon-headline.one-creator-is-verifier",
                                        "creator" = html!{(P(creator, None))},
                                        "publisher" = html!{(P(&self.data.demon.publisher, None))}
                                    ))
                                }
                            },
                            [creator1, creator2] => {
                                (trp_html!(
                                    "demon-headline.two-creators",
                                    "creator1" = html!{(P(creator1, None))},
                                    "creator2" = html!{(P(creator2, None))},
                                    "verified-and-published" = verified_and_published
                                ))
                            },
                            [creator1, rest @ ..] => {
                                (trp_html!(
                                    "demon-headline.more-creators",
                                    "creator" = html!{(P(creator1, None))},
                                    "more" = html! {
                                      div.tooltip.underdotted {
                                            (tr("demon-headline.more-creators-tooltip"))
                                            div.tooltiptext.fade {
                                                (rest.iter().map(|player| player.name.as_ref()).collect::<Vec<_>>().join(", "))
                                            }
                                        }
                                    },
                                    "verified-and-published" = verified_and_published
                                ))
                            }
                        }
                    }
                }
                @if let Some(ref level) = self.integration {
                    @if let Some(Thunk::Processed(ref description)) = level.description {
                        div.underlined.pad {
                            q {
                                (description)
                            }
                        }
                    }
                }
                @if let Some(ref video) = self.data.demon.video {
                    @if let Some(embedded_video) = embed(video) {
                        iframe."ratio-16-9"."js-delay-attr" style="width:90%; margin: 15px 5%" allowfullscreen="" data-attr = "src" data-attr-value = (embedded_video) {"Verification Video"}
                    }
                }
                div.underlined.pad.flex.wrap #level-info {
                    @if let Some(ref level) = self.integration {
                        span {
                            b {
                                (tr("demon-password"))
                            }
                            br;
                            (level.level_data.password.as_processed().map(|pw| pw.to_string()).unwrap_or("Unknown".to_string()))
                        }
                        span {
                            b {
                                (tr("demon-id"))
                            }
                            br;
                            (level.level_id)
                        }
                        span {
                            b {
                                (tr("demon-length"))
                            }
                            br;
                            @match level.level_data.level_data {
                                Thunk::Processed(ref objects) => {
                                    @let length_in_seconds = objects.length_in_seconds();

                                    (format!("{}m:{:02}s", (length_in_seconds as i32)/ 60, (length_in_seconds as i32) % 60))
                                }
                                _ => "unreachable!()"
                            }
                        }
                        span {
                            b {
                                (tr("demon-objects"))
                            }
                            br;
                            @match level.level_data.level_data {
                                Thunk::Processed(ref objects) => (objects.objects.len()),
                                _ => "unreachable!()"
                            }
                        }
                        span {
                            b {
                                (tr("demon-difficulty"))
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
                                (tr("demon-gdversion"))
                            }
                            br;
                            (level.gd_version)
                        }
                        @if let Some(ref song) = level.custom_song {
                            span style = "width: 100%"{
                                b {
                                    (tr("demon-ngsong"))
                                }
                                br;
                                @match song.link {
                                    Thunk::Processed(ref link) if link != "-" => a.link href = (link) {(song.name) " by " (song.artist) " (ID " (song.song_id) ")"},
                                    Thunk::Processed(_) => a.link href = {"https://www.newgrounds.com/audio/listen/" (song.song_id)} {
                                        (song.name) " by " (song.artist) " (ID " (song.song_id) ")"
                                    },
                                    _ => "unreachable!()"
                                }
                            }
                        }
                    }
                    @if position <= list_config::extended_list_size() {
                        span {
                            b {
                                (trp!("demon-score", "percent" = 100.0))
                            }
                            br;
                            (format!("{:.2}", score100))
                        }
                    }
                    @if position <= list_config::list_size(){
                        span {
                            b {
                                (trp!("demon-score", "percent" = self.data.demon.requirement))
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
                            (tr("demon-records"))
                        }
                        @if position <= list_config::list_size() {
                            h3 {
                                (trp!("demon-records-qualify", "percent" = self.data.demon.requirement))
                            }
                        }
                        @else if position <= list_config::extended_list_size() {
                            h3 {
                                (trp!("demon-records-qualify", "percent" = 100.0))
                            }
                        }
                        @if !self.data.records.is_empty() {
                            h4 {
                                @let records_registered_100_count = self.data.records.iter().filter(|record| record.progress == 100).count();
                                (trp!("demon-records-total", "num-records" = self.data.records.len(), "num-completions" = records_registered_100_count))
                            }
                        }
                    }
                    @if self.data.records.is_empty() {
                        h3 {
                            @if position > list_config::extended_list_size() {
                                (tr("demon-records.none"))
                            }
                            @else {
                                (tr("demon-records.none-yet"))
                            }
                        }
                    }
                    @else {
                        table {
                            tbody {
                                tr {
                                    th.blue {}
                                    th.blue {
                                        (tr("record-holder"))
                                    }
                                    th.blue {
                                        (tr("record-progress"))
                                    }
                                    th.video-link.blue {
                                        (tr("record-videoproof"))
                                    }
                                }
                                @for record in &self.data.records {
                                    tr style = { @if record.progress == 100 {"font-weight: bold"} @else {""} } {
                                        td {
                                            @if let Some(ref nationality) = record.nationality {
                                                span.flag-icon style={"background-image: url(/static/demonlist/images/flags/" (nationality.iso_country_code.to_lowercase()) ".svg)"} title = (nationality.nation) {}
                                            }
                                        }
                                        td {
                                            (P(&record.player, None))
                                        }
                                        td {
                                            @if let Some(ref video) = record.video {
                                                a.mobile-only-link href = (video) target = "_blank" {
                                                    (record.progress) "%"
                                                }
                                            } @else {
                                                (record.progress) "%"
                                            }
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
