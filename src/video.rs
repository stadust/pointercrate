use crate::{error::PointercrateError, Result};
use url::Url;

const SCHEMES: [&str; 2] = ["http", "https"];
const YOUTUBE_FORMAT: &str = "https://www.youtube.com/watch?v={video_id}' or \
                              'https://m.youtube.com/watch?v={video_id}' or \
                              'https://youtube.com/watch?v={video_id}' or \
                              'https://youtu.be/{video_id}";
const TWITCH_FORMAT: &str = "https://www.twitch.tv/videos/{video_id}' or \
                             'https://twitch.tv/videos/{video_id}' or\
                             'https://www.twitch.tv/{channel_name}/v/{video_id}' or\
                             'https://twitch.tv/{channel_name}/v/{video_id}";
const EVERYPLAY_FORMAT: &str = "https://everyplay.com/videos/{video_id}' or\
                                'https://www.everyplay.com/videos/{video_id}";
const VIMEO_FORMAT: &str = "https://vimeo.com/{video_id}' or\
                            'https://www.vimeo.com/{video_id}";
const BILIBILI_FORMAT: &str = "'https://www.bilibili.com/video/{video_id}' or\
                               'https://bilibili.com/video/{video_id}";

pub fn validate(url: &str) -> Result<String> {
    let url = Url::parse(url).map_err(|_| PointercrateError::bad_request("Malformed video URL"))?;

    if !SCHEMES.contains(&url.scheme()) {
        return Err(PointercrateError::InvalidUrlScheme)
    }

    if !url.username().is_empty() || url.password().is_some() {
        return Err(PointercrateError::UrlAuthenticated)
    }

    if let Some(host) = url.domain() {
        match host {
            "www.youtube.com" | "m.youtube.com" | "youtube.com" => {
                if url.path() == "/watch" {
                    if let Some(video_id) =
                        url.query_pairs()
                            .find_map(|(key, value)| if key == "v" { Some(value) } else { None })
                    {
                        return Ok(format!("https://www.youtube.com/watch?v={}", video_id))
                    }
                }

                Err(PointercrateError::InvalidUrlFormat {
                    expected: YOUTUBE_FORMAT,
                })
            },
            "youtu.be" =>
                if let Some(path_segments) = url.path_segments() {
                    match &path_segments.collect::<Vec<_>>()[..] {
                        [video_id] => Ok(format!("https://www.youtube.com/watch?v={}", video_id)),
                        _ =>
                            Err(PointercrateError::InvalidUrlFormat {
                                expected: YOUTUBE_FORMAT,
                            }),
                    }
                } else {
                    Err(PointercrateError::InvalidUrlFormat {
                        expected: YOUTUBE_FORMAT,
                    })
                },
            "www.twitch.tv" | "twitch.tv" =>
                if let Some(path_segments) = url.path_segments() {
                    match &path_segments.collect::<Vec<_>>()[..] {
                        ["videos", video_id] =>
                            Ok(format!("https://www.twitch.tv/videos/{}", video_id)),
                        [_, "v", video_id] =>
                            Ok(format!("https://www.twitch.tv/videos/{}", video_id)),
                        _ =>
                            Err(PointercrateError::InvalidUrlFormat {
                                expected: TWITCH_FORMAT,
                            }),
                    }
                } else {
                    Err(PointercrateError::InvalidUrlFormat {
                        expected: TWITCH_FORMAT,
                    })
                },
            "everyplay.com" | "www.everyplay.com" =>
                if let Some(path_segments) = url.path_segments() {
                    match &path_segments.collect::<Vec<_>>()[..] {
                        ["videos", video_id] =>
                            Ok(format!("https://everyplay.com/videos/{}", video_id)),
                        _ =>
                            Err(PointercrateError::InvalidUrlFormat {
                                expected: EVERYPLAY_FORMAT,
                            }),
                    }
                } else {
                    Err(PointercrateError::InvalidUrlFormat {
                        expected: EVERYPLAY_FORMAT,
                    })
                },
            "www.bilibili.com" | "bilibili.com" =>
                if let Some(path_segments) = url.path_segments() {
                    match &path_segments.collect::<Vec<_>>()[..] {
                        ["video", video_id] =>
                            Ok(format!("https://www.bilibili.com/video/{}", video_id)),
                        _ =>
                            Err(PointercrateError::InvalidUrlFormat {
                                expected: BILIBILI_FORMAT,
                            }),
                    }
                } else {
                    Err(PointercrateError::InvalidUrlFormat {
                        expected: BILIBILI_FORMAT,
                    })
                },
            "vimeo.com" | "www.vimeo.com" =>
                if let Some(path_segments) = url.path_segments() {
                    match &path_segments.collect::<Vec<_>>()[..] {
                        [video_id] => Ok(format!("https://vimeo.com/{}", video_id)),
                        _ =>
                            Err(PointercrateError::InvalidUrlFormat {
                                expected: VIMEO_FORMAT,
                            }),
                    }
                } else {
                    Err(PointercrateError::InvalidUrlFormat {
                        expected: VIMEO_FORMAT,
                    })
                },
            _ => Err(PointercrateError::UnsupportedVideoHost),
        }
    } else {
        Err(PointercrateError::UnprocessableEntity)
    }
}

pub fn embed(video: &str) -> String {
    // Videos need to be welformed once we get here!
    let url = Url::parse(video).unwrap();
    let video_id = url
        .query_pairs()
        .find_map(|(key, value)| if key == "v" { Some(value) } else { None })
        .unwrap();

    format!("https://www.youtube.com/embed/{}", video_id)
}

pub fn host(video: &str) -> &str {
    match Url::parse(video).unwrap().domain().unwrap() {
        "www.youtube.com" => "YouTube",
        "www.twitch.com" => "Twitch",
        "everyplay.com" => "Everyplay",
        "www.bilibili.com" => "Bilibili",
        "vimeo.com" => "Vimeo",
        _ => unreachable!(),
    }
}
