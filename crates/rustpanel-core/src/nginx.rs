use crate::{AppSpec, ConfigError, DomainSpec, PanelPaths};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NginxServerConfig {
    pub file_name: String,
    pub contents: String,
}

pub fn render_nginx_server(
    app: &AppSpec,
    domain: &DomainSpec,
    paths: &PanelPaths,
) -> Result<NginxServerConfig, ConfigError> {
    app.validate()?;
    if domain.name.trim().is_empty() {
        return Err(ConfigError::EmptyDomain);
    }

    let contents = if domain.https {
        render_https_server(app, domain, paths)
    } else {
        render_http_server(app, domain, paths)
    };

    Ok(NginxServerConfig {
        file_name: format!("{}-{}.conf", app.id, domain.name),
        contents,
    })
}

fn render_http_server(app: &AppSpec, domain: &DomainSpec, paths: &PanelPaths) -> String {
    format!(
        "\
server {{
    listen 80;
    server_name {domain};

    location /.well-known/acme-challenge/ {{
        root {acme_root};
    }}

    location / {{
        proxy_pass http://{upstream};
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }}
}}
",
        domain = domain.name,
        acme_root = paths.acme_webroot.display(),
        upstream = app.upstream_addr(),
    )
}

fn render_https_server(app: &AppSpec, domain: &DomainSpec, paths: &PanelPaths) -> String {
    format!(
        "\
server {{
    listen 80;
    server_name {domain};

    location /.well-known/acme-challenge/ {{
        root {acme_root};
    }}

    location / {{
        return 301 https://$host$request_uri;
    }}
}}

server {{
    listen 443 ssl;
    server_name {domain};

    ssl_certificate /etc/letsencrypt/live/{domain}/fullchain.pem;
    ssl_certificate_key /etc/letsencrypt/live/{domain}/privkey.pem;

    location / {{
        proxy_pass http://{upstream};
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }}
}}
",
        domain = domain.name,
        acme_root = paths.acme_webroot.display(),
        upstream = app.upstream_addr(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders_https_server() {
        let app = AppSpec::sample();
        let domain = &app.domains[0];
        let config = render_nginx_server(&app, domain, &PanelPaths::default()).unwrap();

        assert_eq!(config.file_name, "demo-api-api.example.com.conf");
        assert!(config.contents.contains("listen 443 ssl;"));
        assert!(
            config
                .contents
                .contains("proxy_pass http://127.0.0.1:8080;")
        );
    }

    #[test]
    fn renders_http_only_server() {
        let app = AppSpec {
            domains: vec![DomainSpec {
                name: "plain.example.com".to_owned(),
                https: false,
            }],
            ..AppSpec::sample()
        };

        let config = render_nginx_server(&app, &app.domains[0], &PanelPaths::default()).unwrap();

        assert!(config.contents.contains("listen 80;"));
        assert!(!config.contents.contains("listen 443 ssl;"));
    }
}
