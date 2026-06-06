use anyhow::Context;
use axum::http::{
    HeaderMap,
    header::{COOKIE, SET_COOKIE},
};

#[derive(Clone)]
pub(crate) struct AuthConfig {
    pub(crate) username: String,
    password: String,
    session_secret: String,
}

impl AuthConfig {
    pub(crate) fn from_env() -> anyhow::Result<Self> {
        let username = std::env::var("RUSTPANEL_ADMIN_USER").unwrap_or_else(|_| "admin".to_owned());
        let password = std::env::var("RUSTPANEL_ADMIN_PASSWORD")
            .context("RUSTPANEL_ADMIN_PASSWORD must be set")?;
        let session_secret = std::env::var("RUSTPANEL_SESSION_SECRET")
            .context("RUSTPANEL_SESSION_SECRET must be set")?;

        Ok(Self {
            username,
            password,
            session_secret,
        })
    }

    pub(crate) fn verify_password(&self, username: &str, password: &str) -> bool {
        username == self.username && password == self.password
    }
}

pub(crate) fn is_authenticated(headers: &HeaderMap, auth: &AuthConfig) -> bool {
    let Some(cookie) = headers.get(COOKIE).and_then(|value| value.to_str().ok()) else {
        return false;
    };

    cookie.split(';').any(|part| {
        let trimmed = part.trim();
        trimmed
            .strip_prefix("rp_session=")
            .is_some_and(|value| value == auth.session_secret)
    })
}

pub(crate) fn set_session_cookie(headers: &mut HeaderMap, auth: &AuthConfig, base_path: &str) {
    let cookie = format!(
        "rp_session={}; Path={}; HttpOnly; SameSite=Strict",
        auth.session_secret,
        cookie_path(base_path)
    );
    headers.insert(SET_COOKIE, cookie.parse().expect("session cookie is valid"));
}

pub(crate) fn clear_session_cookie(headers: &mut HeaderMap, base_path: &str) {
    let cookie = format!(
        "rp_session=; Path={}; Max-Age=0; HttpOnly; SameSite=Strict",
        cookie_path(base_path)
    );
    headers.insert(SET_COOKIE, cookie.parse().expect("logout cookie is valid"));
}

fn cookie_path(base_path: &str) -> String {
    if base_path == "/" {
        "/".to_owned()
    } else {
        base_path.to_owned()
    }
}
