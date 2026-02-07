use derive_more::Display;
use tokio::task_local;

#[derive(Debug, Display, Default, Clone, Copy)]
pub enum Theme {
    #[default]
    #[display("light")]
    Light,

    #[display("dark")]
    Dark,
}

impl Theme {
    pub fn cookie_name() -> &'static str {
        "theme"
    }

    pub fn from_cookie(s: &str) -> Theme {
        match s {
            "dark" => Theme::Dark,
            _ => Theme::Light,
        }
    }

    // https://developers.google.com/identity/gsi/web/reference/html-reference#data-theme
    pub fn as_gsi_theme(&self) -> &'static str {
        match self {
            Theme::Light => "outline",
            Theme::Dark => "filled_black",
        }
    }
}

task_local! {
    pub static THEME: Theme;
}

/// Utility function for easily retrieving the current [`Theme`] inside the
/// `task_local!` [`LocalKey`] scope of wherever this is called from.
pub fn task_theme() -> Theme {
    THEME.with(|theme| *theme)
}
