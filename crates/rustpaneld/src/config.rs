use std::net::SocketAddr;

use anyhow::{Context, bail};

use crate::{auth::AuthConfig, i18n::Language};

#[derive(Clone)]
pub(crate) struct DaemonConfig {
    pub(crate) addr: SocketAddr,
    pub(crate) base_path: String,
    pub(crate) auth: AuthConfig,
}

impl DaemonConfig {
    pub(crate) fn from_env() -> anyhow::Result<Self> {
        let addr = std::env::var("RUSTPANEL_BIND")
            .unwrap_or_else(|_| "127.0.0.1:7654".to_owned())
            .parse::<SocketAddr>()
            .context("RUSTPANEL_BIND must be a socket address")?;

        Ok(Self {
            addr,
            base_path: base_path_from_env()?,
            auth: AuthConfig::from_env()?,
        })
    }
}

pub(crate) fn base_path_from_env() -> anyhow::Result<String> {
    let raw = std::env::var("RUSTPANEL_BASE_PATH").unwrap_or_else(|_| "/".to_owned());
    normalize_base_path(&raw)
}

pub(crate) fn normalize_base_path(raw: &str) -> anyhow::Result<String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() || trimmed == "/" {
        return Ok("/".to_owned());
    }

    let with_slash = format!("/{}", trimmed.trim_matches('/'));
    if !with_slash
        .bytes()
        .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'/' | b'-' | b'_'))
    {
        bail!(
            "RUSTPANEL_BASE_PATH may only contain letters, numbers, slash, hyphen, and underscore"
        );
    }

    Ok(with_slash)
}

pub(crate) fn scoped_path(base_path: &str, path: &str) -> String {
    if base_path == "/" {
        return path.to_owned();
    }

    if path == "/" {
        return base_path.to_owned();
    }

    format!("{base_path}{path}")
}

pub(crate) fn scoped_lang_path(base_path: &str, path: &str, language: Language) -> String {
    format!("{}?lang={}", scoped_path(base_path, path), language.code())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_base_path() {
        assert_eq!(normalize_base_path("rp-secret").unwrap(), "/rp-secret");
        assert_eq!(normalize_base_path("/rp-secret/").unwrap(), "/rp-secret");
        assert_eq!(normalize_base_path("/").unwrap(), "/");
    }

    #[test]
    fn rejects_unsafe_base_path() {
        assert!(normalize_base_path("/rp secret").is_err());
        assert!(normalize_base_path("/rp.secret").is_err());
    }

    #[test]
    fn scopes_child_paths() {
        assert_eq!(scoped_path("/", "/apps"), "/apps");
        assert_eq!(scoped_path("/rp-secret", "/apps"), "/rp-secret/apps");
        assert_eq!(scoped_path("/rp-secret", "/"), "/rp-secret");
    }

    #[test]
    fn scopes_language_paths() {
        assert_eq!(
            scoped_lang_path("/rp-secret", "/update-check", Language::Zh),
            "/rp-secret/update-check?lang=zh"
        );
    }
}
