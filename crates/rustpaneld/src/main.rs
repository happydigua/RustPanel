use std::{net::SocketAddr, sync::Arc};

use anyhow::{Context, bail};
use askama::Template;
use axum::{
    Json, Router,
    extract::State,
    http::StatusCode,
    response::{Html, IntoResponse},
    routing::get,
};
use rustpanel_core::{AppSpec, PanelPaths};
use serde::Serialize;
use serde_json::json;
use tokio::net::TcpListener;
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[derive(Clone)]
struct AppState {
    apps: Arc<Vec<AppSpec>>,
    paths: PanelPaths,
    base_path: String,
}

#[derive(Clone)]
struct AppRow {
    name: String,
    id: String,
    service_name: String,
    upstream: String,
    domain_count: usize,
    deploy_step_count: usize,
}

#[derive(Template)]
#[template(
    source = r#"
<!doctype html>
<html lang="en">
<head>
    <meta charset="utf-8">
    <meta name="viewport" content="width=device-width, initial-scale=1">
    <title>RustPanel</title>
    <style>
        :root {
            color-scheme: light;
            --bg: #f7f8fa;
            --panel: #ffffff;
            --line: #d8dee8;
            --text: #1d2430;
            --muted: #627084;
            --accent: #0f766e;
            --accent-strong: #0b5c56;
        }

        * { box-sizing: border-box; }

        body {
            margin: 0;
            background: var(--bg);
            color: var(--text);
            font: 14px/1.5 ui-sans-serif, system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif;
        }

        header.topbar {
            height: 56px;
            display: flex;
            align-items: center;
            justify-content: space-between;
            padding: 0 24px;
            border-bottom: 1px solid var(--line);
            background: var(--panel);
        }

        .brand {
            font-weight: 700;
            letter-spacing: 0;
        }

        .bind {
            color: var(--muted);
            font-size: 13px;
        }

        main {
            width: min(1120px, calc(100vw - 32px));
            margin: 28px auto;
        }

        .metrics {
            display: grid;
            grid-template-columns: repeat(3, minmax(0, 1fr));
            gap: 12px;
            margin-bottom: 20px;
        }

        .metric, .panel {
            background: var(--panel);
            border: 1px solid var(--line);
            border-radius: 8px;
        }

        .metric {
            padding: 16px;
        }

        .metric span {
            display: block;
            color: var(--muted);
            font-size: 13px;
        }

        .metric strong {
            display: block;
            margin-top: 6px;
            font-size: 24px;
            line-height: 1.2;
        }

        .panel-header {
            display: flex;
            align-items: center;
            justify-content: space-between;
            gap: 12px;
            padding: 16px 18px;
            border-bottom: 1px solid var(--line);
        }

        h1, h2 {
            margin: 0;
            font-size: 18px;
            line-height: 1.3;
        }

        .button {
            appearance: none;
            border: 1px solid var(--accent-strong);
            background: var(--accent);
            color: white;
            border-radius: 6px;
            padding: 8px 12px;
            font: inherit;
            text-decoration: none;
            cursor: pointer;
            white-space: nowrap;
        }

        table {
            width: 100%;
            border-collapse: collapse;
        }

        th, td {
            padding: 12px 18px;
            border-bottom: 1px solid var(--line);
            text-align: left;
            vertical-align: middle;
        }

        th {
            color: var(--muted);
            font-size: 12px;
            font-weight: 600;
            text-transform: uppercase;
        }

        tr:last-child td {
            border-bottom: 0;
        }

        code {
            padding: 2px 5px;
            border-radius: 4px;
            background: #eef2f6;
            color: #243244;
            font-family: ui-monospace, SFMono-Regular, Menlo, Consolas, monospace;
            font-size: 12px;
        }

        .muted { color: var(--muted); }

        @media (max-width: 760px) {
            header.topbar {
                align-items: flex-start;
                height: auto;
                padding: 14px 16px;
                flex-direction: column;
            }

            main {
                width: calc(100vw - 24px);
                margin: 18px auto;
            }

            .metrics {
                grid-template-columns: 1fr;
            }

            table, thead, tbody, tr, th, td {
                display: block;
            }

            thead {
                display: none;
            }

            tr {
                padding: 12px 0;
                border-bottom: 1px solid var(--line);
            }

            tr:last-child {
                border-bottom: 0;
            }

            td {
                border: 0;
                padding: 4px 16px;
            }
        }
    </style>
</head>
<body>
    <header class="topbar">
        <div class="brand">RustPanel</div>
        <div class="bind">Local daemon on 127.0.0.1:7654</div>
    </header>

    <main>
        <section class="metrics" aria-label="Overview">
            <div class="metric">
                <span>Applications</span>
                <strong>{{ app_count }}</strong>
            </div>
            <div class="metric">
                <span>Domains</span>
                <strong>{{ domain_count }}</strong>
            </div>
            <div class="metric">
                <span>Proxy</span>
                <strong>Nginx</strong>
            </div>
        </section>

        <section class="panel">
            <div class="panel-header">
                <div>
                    <h1>Applications</h1>
                    <div class="muted">Managed Nginx config: <code>{{ nginx_conf_dir }}</code></div>
                </div>
                <a class="button" href="{{ apps_path }}">Open list</a>
            </div>
            <table>
                <thead>
                    <tr>
                        <th>Name</th>
                        <th>Service</th>
                        <th>Upstream</th>
                        <th>Domains</th>
                        <th>Deploy</th>
                    </tr>
                </thead>
                <tbody>
                    {% for app in rows %}
                    <tr>
                        <td><strong>{{ app.name }}</strong><br><span class="muted">{{ app.id }}</span></td>
                        <td><code>{{ app.service_name }}</code></td>
                        <td><code>{{ app.upstream }}</code></td>
                        <td>{{ app.domain_count }}</td>
                        <td>{{ app.deploy_step_count }} steps</td>
                    </tr>
                    {% endfor %}
                </tbody>
            </table>
        </section>
    </main>
</body>
</html>
"#,
    ext = "html"
)]
struct DashboardTemplate {
    rows: Vec<AppRow>,
    app_count: usize,
    domain_count: usize,
    nginx_conf_dir: String,
    apps_path: String,
}

#[derive(Template)]
#[template(
    source = r#"
<table>
    <thead>
        <tr>
            <th>Name</th>
            <th>Service</th>
            <th>Upstream</th>
            <th>Domains</th>
        </tr>
    </thead>
    <tbody>
        {% for app in rows %}
        <tr>
            <td>{{ app.name }}</td>
            <td>{{ app.service_name }}</td>
            <td>{{ app.upstream }}</td>
            <td>{{ app.domain_count }}</td>
        </tr>
        {% endfor %}
    </tbody>
</table>
"#,
    ext = "html"
)]
struct AppsPartialTemplate {
    rows: Vec<AppRow>,
}

#[derive(Serialize)]
struct HealthResponse {
    status: &'static str,
    service: &'static str,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "rustpaneld=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let addr = std::env::var("RUSTPANEL_BIND")
        .unwrap_or_else(|_| "127.0.0.1:7654".to_owned())
        .parse::<SocketAddr>()
        .context("RUSTPANEL_BIND must be a socket address")?;
    let base_path = base_path_from_env()?;

    let state = AppState {
        apps: Arc::new(vec![AppSpec::sample()]),
        paths: PanelPaths::default(),
        base_path: base_path.clone(),
    };

    let panel_routes = Router::new()
        .route("/", get(dashboard))
        .route("/apps", get(apps_partial))
        .route("/healthz", get(healthz));

    let app = if base_path == "/" {
        panel_routes
    } else {
        Router::new()
            .nest(&base_path, panel_routes)
            .route("/healthz", get(healthz))
    }
    .with_state(state)
    .layer(TraceLayer::new_for_http());

    let listener = TcpListener::bind(addr).await?;
    tracing::info!(%addr, %base_path, "rustpaneld listening");

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    Ok(())
}

async fn dashboard(State(state): State<AppState>) -> impl IntoResponse {
    render(DashboardTemplate {
        rows: app_rows(&state.apps),
        app_count: state.apps.len(),
        domain_count: state.apps.iter().map(|app| app.domains.len()).sum(),
        nginx_conf_dir: state.paths.nginx_conf_dir.display().to_string(),
        apps_path: scoped_path(&state.base_path, "/apps"),
    })
}

async fn apps_partial(State(state): State<AppState>) -> impl IntoResponse {
    render(AppsPartialTemplate {
        rows: app_rows(&state.apps),
    })
}

async fn healthz() -> impl IntoResponse {
    Json(json!(HealthResponse {
        status: "ok",
        service: "rustpaneld",
    }))
}

fn render<T: Template>(template: T) -> impl IntoResponse {
    match template.render() {
        Ok(html) => Html(html).into_response(),
        Err(error) => (StatusCode::INTERNAL_SERVER_ERROR, error.to_string()).into_response(),
    }
}

fn app_rows(apps: &[AppSpec]) -> Vec<AppRow> {
    apps.iter()
        .map(|app| AppRow {
            name: app.name.clone(),
            id: app.id.clone(),
            service_name: app.service_name(),
            upstream: app.upstream_addr(),
            domain_count: app.domains.len(),
            deploy_step_count: app.deploy.steps.len(),
        })
        .collect()
}

fn base_path_from_env() -> anyhow::Result<String> {
    let raw = std::env::var("RUSTPANEL_BASE_PATH").unwrap_or_else(|_| "/".to_owned());
    normalize_base_path(&raw)
}

fn normalize_base_path(raw: &str) -> anyhow::Result<String> {
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

fn scoped_path(base_path: &str, path: &str) -> String {
    if base_path == "/" {
        return path.to_owned();
    }

    format!("{base_path}{path}")
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
    }
}

async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("failed to install SIGTERM handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}
