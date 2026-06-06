use askama::Template;
use axum::{
    Form, Json,
    extract::{Query, State},
    http::{HeaderMap, StatusCode},
    response::{Html, IntoResponse, Redirect, Response},
};
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::{
    auth::{clear_session_cookie, is_authenticated, set_session_cookie},
    config::{scoped_lang_path, scoped_path},
    i18n::Language,
    processes::collect_processes,
    services::{ServiceUnit, collect_service_units},
    sites::{discover_sites, managed_config_dir},
    ssl::collect_ssl_info,
    state::AppState,
    system_metrics::collect_system_metrics,
    templates::{LoginTemplate, PanelTemplate, ServiceRow},
    updates::{UpdateCheckResult, run_update_check},
};

#[derive(Debug, Deserialize)]
pub(crate) struct PageParams {
    lang: Option<String>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct LoginForm {
    username: String,
    password: String,
    lang: Option<String>,
}

#[derive(Clone, Copy)]
pub(crate) enum PanelPage {
    Overview,
    Processes,
    Services,
    Sites,
    Ssl,
    Update,
}

#[derive(Serialize)]
struct HealthResponse {
    status: &'static str,
    service: &'static str,
}

pub(crate) async fn overview_page(
    headers: HeaderMap,
    State(state): State<AppState>,
    Query(params): Query<PageParams>,
) -> Response {
    render_panel_page(headers, state, params, PanelPage::Overview, None).await
}

pub(crate) async fn processes_page(
    headers: HeaderMap,
    State(state): State<AppState>,
    Query(params): Query<PageParams>,
) -> Response {
    render_panel_page(headers, state, params, PanelPage::Processes, None).await
}

pub(crate) async fn services_page(
    headers: HeaderMap,
    State(state): State<AppState>,
    Query(params): Query<PageParams>,
) -> Response {
    render_panel_page(headers, state, params, PanelPage::Services, None).await
}

pub(crate) async fn sites_page(
    headers: HeaderMap,
    State(state): State<AppState>,
    Query(params): Query<PageParams>,
) -> Response {
    render_panel_page(headers, state, params, PanelPage::Sites, None).await
}

pub(crate) async fn ssl_page(
    headers: HeaderMap,
    State(state): State<AppState>,
    Query(params): Query<PageParams>,
) -> Response {
    render_panel_page(headers, state, params, PanelPage::Ssl, None).await
}

pub(crate) async fn update_check_page(
    headers: HeaderMap,
    State(state): State<AppState>,
    Query(params): Query<PageParams>,
) -> Response {
    let language = Language::from_param(params.lang.as_deref());
    let result = run_update_check(language).await;
    render_panel_page(headers, state, params, PanelPage::Update, Some(result)).await
}

async fn render_panel_page(
    headers: HeaderMap,
    state: AppState,
    params: PageParams,
    page: PanelPage,
    update_result: Option<UpdateCheckResult>,
) -> Response {
    let language = Language::from_param(params.lang.as_deref());
    if !is_authenticated(&headers, &state.auth) {
        return Redirect::to(&scoped_lang_path(&state.base_path, "/login", language))
            .into_response();
    }

    let labels = language.labels();
    let processes = collect_processes(match page {
        PanelPage::Overview => 8,
        _ => 80,
    });
    let services = collect_service_units(match page {
        PanelPage::Overview => 8,
        _ => 200,
    });
    let service_rows = service_rows(language, services);
    let sites = discover_sites();
    let ssl_info = collect_ssl_info();
    let active_page = page.key().to_owned();

    render(PanelTemplate {
        labels,
        page_title: page.title(language).to_owned(),
        active_page: active_page.clone(),
        overview_class: nav_class(&active_page, "overview"),
        processes_class: nav_class(&active_page, "processes"),
        services_class: nav_class(&active_page, "services"),
        sites_class: nav_class(&active_page, "sites"),
        ssl_class: nav_class(&active_page, "ssl"),
        update_class: nav_class(&active_page, "update"),
        overview_path: scoped_lang_path(&state.base_path, "/", language),
        processes_path: scoped_lang_path(&state.base_path, "/processes", language),
        services_path: scoped_lang_path(&state.base_path, "/services", language),
        sites_path: scoped_lang_path(&state.base_path, "/sites", language),
        ssl_path: scoped_lang_path(&state.base_path, "/ssl", language),
        update_path: scoped_lang_path(&state.base_path, "/update-check", language),
        lang_zh_path: scoped_lang_path(&state.base_path, page.path(), Language::Zh),
        lang_en_path: scoped_lang_path(&state.base_path, page.path(), Language::En),
        logout_path: scoped_lang_path(&state.base_path, "/logout", language),
        version: current_version(),
        metrics: collect_system_metrics(),
        has_processes: !processes.is_empty(),
        process_count: processes.len(),
        processes,
        has_services: !service_rows.is_empty(),
        service_count: service_rows.len(),
        services: service_rows,
        has_sites: !sites.is_empty(),
        site_count: sites.len(),
        domain_count: sites.len(),
        sites,
        managed_nginx_config: managed_config_dir(),
        has_certificates: !ssl_info.certificates.is_empty(),
        ssl_info,
        update_result: update_result.unwrap_or_else(UpdateCheckResult::empty),
    })
    .into_response()
}

pub(crate) async fn login_page(
    State(state): State<AppState>,
    Query(params): Query<PageParams>,
) -> Response {
    let language = Language::from_param(params.lang.as_deref());
    render_login(&state, language, false)
}

pub(crate) async fn login_submit(
    State(state): State<AppState>,
    Form(form): Form<LoginForm>,
) -> Response {
    let language = Language::from_param(form.lang.as_deref());
    if !state.auth.verify_password(&form.username, &form.password) {
        return render_login(&state, language, true);
    }

    let mut response =
        Redirect::to(&scoped_lang_path(&state.base_path, "/", language)).into_response();
    set_session_cookie(response.headers_mut(), &state.auth, &state.base_path);
    response
}

pub(crate) async fn logout(
    State(state): State<AppState>,
    Query(params): Query<PageParams>,
) -> Response {
    let language = Language::from_param(params.lang.as_deref());
    let mut response =
        Redirect::to(&scoped_lang_path(&state.base_path, "/login", language)).into_response();
    clear_session_cookie(response.headers_mut(), &state.base_path);
    response
}

pub(crate) async fn healthz() -> impl IntoResponse {
    Json(json!(HealthResponse {
        status: "ok",
        service: "rustpaneld",
    }))
}

fn service_rows(language: Language, services: Vec<ServiceUnit>) -> Vec<ServiceRow> {
    services
        .into_iter()
        .map(|service| {
            let state_class = match service.active_state.as_str() {
                "active" => "ok",
                "failed" => "failed",
                _ => "idle",
            }
            .to_owned();
            let state_label = format!(
                "{} / {}",
                language.state_text(&service.active_state),
                language.state_text(&service.sub_state)
            );

            ServiceRow {
                name: service.name,
                load_state: language.state_text(&service.load_state),
                active_state: service.active_state,
                sub_state: service.sub_state,
                state_label,
                state_class,
                description: service.description,
            }
        })
        .collect()
}

fn render<T: Template>(template: T) -> impl IntoResponse {
    match template.render() {
        Ok(html) => Html(html).into_response(),
        Err(error) => (StatusCode::INTERNAL_SERVER_ERROR, error.to_string()).into_response(),
    }
}

fn render_login(state: &AppState, language: Language, login_failed: bool) -> Response {
    render(LoginTemplate {
        login_path: scoped_path(&state.base_path, "/login"),
        lang_zh_path: scoped_lang_path(&state.base_path, "/login", Language::Zh),
        lang_en_path: scoped_lang_path(&state.base_path, "/login", Language::En),
        lang_code: language.code().to_owned(),
        login_failed,
        labels: language.labels(),
    })
    .into_response()
}

fn nav_class(active_page: &str, page: &str) -> String {
    if active_page == page {
        "active".to_owned()
    } else {
        String::new()
    }
}

fn current_version() -> String {
    env!("CARGO_PKG_VERSION").to_owned()
}

impl PanelPage {
    fn key(self) -> &'static str {
        match self {
            Self::Overview => "overview",
            Self::Processes => "processes",
            Self::Services => "services",
            Self::Sites => "sites",
            Self::Ssl => "ssl",
            Self::Update => "update",
        }
    }

    fn path(self) -> &'static str {
        match self {
            Self::Overview => "/",
            Self::Processes => "/processes",
            Self::Services => "/services",
            Self::Sites => "/sites",
            Self::Ssl => "/ssl",
            Self::Update => "/update-check",
        }
    }

    fn title(self, language: Language) -> &'static str {
        let labels = language.labels();
        match self {
            Self::Overview => labels.overview,
            Self::Processes => labels.processes,
            Self::Services => labels.systemd_services,
            Self::Sites => labels.sites,
            Self::Ssl => labels.ssl,
            Self::Update => labels.update_check,
        }
    }
}
