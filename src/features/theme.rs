use serde::{Deserialize, Serialize};
use topcoat::{
    Result,
    context::Cx,
    cookie::{Cookies, SameSite, UnparsedCookieStore, cookie_store, cookies, time::Duration},
    router::error::bad_request,
    runtime::procedure,
};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Theme {
    Light,
    Dark,
    #[default]
    Auto,
}

impl Theme {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Light => "light",
            Self::Dark => "dark",
            Self::Auto => "auto",
        }
    }

    fn parse(value: &str) -> Option<Self> {
        match value {
            "light" => Some(Self::Light),
            "dark" => Some(Self::Dark),
            "auto" => Some(Self::Auto),
            _ => None,
        }
    }
}

pub fn current(cx: &Cx) -> Theme {
    store(cx).parse_or_default().get()
}

#[procedure]
pub async fn save_theme(cx: &Cx, mode: String) -> Result<String> {
    let theme = Theme::parse(&mode).ok_or_else(|| bad_request("invalid theme mode"))?;
    store(cx)
        .parse_or_default()
        .update(|current| *current = theme)
        .commit()?;
    Ok(theme.as_str().to_owned())
}

fn store(cx: &Cx) -> UnparsedCookieStore<Theme, impl Cookies> {
    let jar = cookies(cx)
        .default_path("/")
        .default_http_only(true)
        .default_same_site(SameSite::Lax)
        .default_max_age(Duration::days(365));
    cookie_store(jar, "freenit-theme-mode")
}
