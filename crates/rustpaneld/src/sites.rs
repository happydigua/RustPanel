use std::{
    fs,
    path::{Path, PathBuf},
};

#[derive(Clone)]
pub(crate) struct SiteInfo {
    pub(crate) domain: String,
    pub(crate) upstream: String,
    pub(crate) ssl: String,
    pub(crate) config_path: String,
}

pub(crate) fn discover_sites() -> Vec<SiteInfo> {
    let mut sites = Vec::new();
    for dir in [
        Path::new("/etc/nginx/conf.d"),
        Path::new("/etc/nginx/sites-enabled"),
    ] {
        collect_from_dir(dir, 0, &mut sites);
    }

    sites.sort_by(|left, right| left.domain.cmp(&right.domain));
    sites.dedup_by(|left, right| {
        left.domain == right.domain
            && left.upstream == right.upstream
            && left.config_path == right.config_path
    });
    sites
}

fn collect_from_dir(dir: &Path, depth: usize, sites: &mut Vec<SiteInfo>) {
    if depth > 3 {
        return;
    }

    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };

    for entry in entries.filter_map(Result::ok) {
        let path = entry.path();
        if path.is_dir() {
            collect_from_dir(&path, depth + 1, sites);
            continue;
        }

        if path.extension().is_some_and(|ext| ext == "conf") {
            sites.extend(parse_nginx_config(&path));
        }
    }
}

fn parse_nginx_config(path: &Path) -> Vec<SiteInfo> {
    let Ok(contents) = fs::read_to_string(path) else {
        return Vec::new();
    };

    let server_names = parse_server_names(&contents);
    if server_names.is_empty() {
        return Vec::new();
    }

    let upstream = parse_proxy_pass(&contents).unwrap_or_else(|| "-".to_owned());
    let ssl = if contents.contains("listen 443") || contents.contains(" ssl") {
        "已启用".to_owned()
    } else {
        "未启用".to_owned()
    };
    let config_path = display_path(path);

    server_names
        .into_iter()
        .map(|domain| SiteInfo {
            domain,
            upstream: upstream.clone(),
            ssl: ssl.clone(),
            config_path: config_path.clone(),
        })
        .collect()
}

fn parse_server_names(contents: &str) -> Vec<String> {
    contents
        .lines()
        .map(strip_comment)
        .filter_map(|line| line.trim().strip_prefix("server_name").map(str::trim))
        .flat_map(|rest| {
            rest.trim_end_matches(';')
                .split_whitespace()
                .filter(|name| *name != "_" && !name.starts_with('$'))
                .map(str::to_owned)
                .collect::<Vec<_>>()
        })
        .collect()
}

fn parse_proxy_pass(contents: &str) -> Option<String> {
    contents.lines().map(strip_comment).find_map(|line| {
        line.split_once("proxy_pass")
            .map(|(_, value)| value)
            .map(str::trim)
            .map(|value| value.split(';').next().unwrap_or(value).trim().to_owned())
    })
}

fn strip_comment(line: &str) -> &str {
    line.split_once('#').map_or(line, |(before, _)| before)
}

fn display_path(path: &Path) -> String {
    path.to_string_lossy().to_string()
}

pub(crate) fn managed_config_dir() -> String {
    display_path(&PathBuf::from("/etc/nginx/conf.d/rustpanel"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_server_names_and_proxy() {
        let config = r#"
            server {
                listen 80;
                server_name example.com www.example.com;
                location / { proxy_pass http://127.0.0.1:8080; }
            }
        "#;

        assert_eq!(
            parse_server_names(config),
            vec!["example.com".to_owned(), "www.example.com".to_owned()]
        );
        assert_eq!(
            parse_proxy_pass(config),
            Some("http://127.0.0.1:8080".to_owned())
        );
    }
}
