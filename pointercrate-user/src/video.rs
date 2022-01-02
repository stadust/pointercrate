use crate::error::{Result, UserError};
use pointercrate_core::error::CoreError;
use url::Url;

const SCHEMES: [&str; 2] = ["http", "https"];
const YOUTUBE_CHANNEL_FORMAT: &str =
    "'youtube.com/channel/{channel_id}' or 'youtube.com/c/{custom_channel_id}/' or 'youtube.com/user/{username}/";

pub fn validate_channel(url: &str) -> Result<String> {
    let url = Url::parse(url).map_err(|_| UserError::MalformedChannelUrl)?;

    if !SCHEMES.contains(&url.scheme()) {
        return Err(CoreError::InvalidUrlScheme.into())
    }

    if !url.username().is_empty() || url.password().is_some() {
        return Err(CoreError::UrlAuthenticated.into())
    }

    if let Some(host) = url.domain() {
        match host {
            "www.youtube.com" | "youtube.com" =>
                if let Some(path_segments) = url.path_segments() {
                    match &path_segments.collect::<Vec<_>>()[..] {
                        ["channel", _] | ["user", _] | ["c", _] => Ok(url.to_string()),
                        _ =>
                            Err(CoreError::InvalidUrlFormat {
                                expected: YOUTUBE_CHANNEL_FORMAT,
                            }
                            .into()),
                    }
                } else {
                    Err(CoreError::InvalidUrlFormat {
                        expected: YOUTUBE_CHANNEL_FORMAT,
                    }
                    .into())
                },
            _ => Err(UserError::NotYouTube),
        }
    } else {
        Err(CoreError::UnprocessableEntity.into())
    }
}
