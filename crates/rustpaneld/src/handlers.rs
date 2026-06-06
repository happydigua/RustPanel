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
    privileged::{JobKind, JobStatus, read_job_status, start_certificate, start_update},
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
    state: Option<String>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct LoginForm {
    username: String,
    password: String,
    lang: Option<String>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct CertificateForm {
    domain: String,
    email: String,
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
    render_panel_page(headers, state, params, PanelPage::Overview, None, None).await
}

pub(crate) async fn processes_page(
    headers: HeaderMap,
    State(state): State<AppState>,
    Query(params): Query<PageParams>,
) -> Response {
    render_panel_page(headers, state, params, PanelPage::Processes, None, None).await
}

pub(crate) async fn services_page(
    headers: HeaderMap,
    State(state): State<AppState>,
    Query(params): Query<PageParams>,
) -> Response {
    render_panel_page(headers, state, params, PanelPage::Services, None, None).await
}

pub(crate) async fn sites_page(
    headers: HeaderMap,
    State(state): State<AppState>,
    Query(params): Query<PageParams>,
) -> Response {
    render_panel_page(headers, state, params, PanelPage::Sites, None, None).await
}

pub(crate) async fn ssl_page(
    headers: HeaderMap,
    State(state): State<AppState>,
    Query(params): Query<PageParams>,
) -> Response {
    render_panel_page(headers, state, params, PanelPage::Ssl, None, None).await
}

pub(crate) async fn update_check_page(
    headers: HeaderMap,
    State(state): State<AppState>,
    Query(params): Query<PageParams>,
) -> Response {
    let language = Language::from_param(params.lang.as_deref());
    let result = run_update_check(language).await;
    render_panel_page(
        headers,
        state,
        params,
        PanelPage::Update,
        Some(result),
        None,
    )
    .await
}

pub(crate) async fn update_run_page(
    headers: HeaderMap,
    State(state): State<AppState>,
    Query(params): Query<PageParams>,
) -> Response {
    let language = Language::from_param(params.lang.as_deref());
    if !is_authenticated(&headers, &state.auth) {
        return Redirect::to(&scoped_lang_path(&state.base_path, "/login", language))
            .into_response();
    }

    let flash = match start_update().await {
        Ok(message) => ("ok".to_owned(), message),
        Err(error) => ("error".to_owned(), format!("更新启动失败: {error:#}")),
    };

    render_panel_page(headers, state, params, PanelPage::Update, None, Some(flash)).await
}

pub(crate) async fn certificate_issue_page(
    headers: HeaderMap,
    State(state): State<AppState>,
    Form(form): Form<CertificateForm>,
) -> Response {
    let params = PageParams {
        lang: form.lang.clone(),
        state: None,
    };
    let language = Language::from_param(params.lang.as_deref());
    if !is_authenticated(&headers, &state.auth) {
        return Redirect::to(&scoped_lang_path(&state.base_path, "/login", language))
            .into_response();
    }

    let flash = match start_certificate(form.domain, form.email).await {
        Ok(message) => ("ok".to_owned(), message),
        Err(error) => ("error".to_owned(), format!("证书申请启动失败: {error:#}")),
    };

    render_panel_page(headers, state, params, PanelPage::Ssl, None, Some(flash)).await
}

async fn render_panel_page(
    headers: HeaderMap,
    state: AppState,
    params: PageParams,
    page: PanelPage,
    update_result: Option<UpdateCheckResult>,
    flash: Option<(String, String)>,
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
    let service_filter = ServiceFilter::from_param(params.state.as_deref());
    let service_rows = filtered_service_rows(language, services, service_filter);
    let sites = discover_sites();
    let ssl_info = collect_ssl_info();
    let active_page = page.key().to_owned();
    let (has_flash, flash_class, flash_message) = match flash {
        Some((class, message)) => (true, class, message),
        None => (false, String::new(), String::new()),
    };

    render(PanelTemplate {
        labels,
        page_title: page.title(language).to_owned(),
        active_page: active_page.clone(),
        lang_code: language.code().to_owned(),
        overview_class: nav_class(&active_page, "overview"),
        processes_class: nav_class(&active_page, "processes"),
        services_class: nav_class(&active_page, "services"),
        sites_class: nav_class(&active_page, "sites"),
        ssl_class: nav_class(&active_page, "ssl"),
        update_class: nav_class(&active_page, "update"),
        overview_path: scoped_lang_path(&state.base_path, "/", language),
        processes_path: scoped_lang_path(&state.base_path, "/processes", language),
        services_path: scoped_lang_path(&state.base_path, "/services", language),
        services_running_path: scoped_lang_state_path(
            &state.base_path,
            "/services",
            language,
            ServiceFilter::Running,
        ),
        services_failed_path: scoped_lang_state_path(
            &state.base_path,
            "/services",
            language,
            ServiceFilter::Failed,
        ),
        services_stopped_path: scoped_lang_state_path(
            &state.base_path,
            "/services",
            language,
            ServiceFilter::Stopped,
        ),
        services_all_path: scoped_lang_state_path(
            &state.base_path,
            "/services",
            language,
            ServiceFilter::All,
        ),
        services_running_class: segment_class(service_filter, ServiceFilter::Running),
        services_failed_class: segment_class(service_filter, ServiceFilter::Failed),
        services_stopped_class: segment_class(service_filter, ServiceFilter::Stopped),
        services_all_class: segment_class(service_filter, ServiceFilter::All),
        sites_path: scoped_lang_path(&state.base_path, "/sites", language),
        ssl_path: scoped_lang_path(&state.base_path, "/ssl", language),
        update_path: scoped_lang_path(&state.base_path, "/update-check", language),
        update_run_path: scoped_lang_path(&state.base_path, "/update-run", language),
        certificate_issue_path: scoped_path(&state.base_path, "/ssl/issue"),
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
        update_job_status: localized_job_status(read_job_status(JobKind::Update), language),
        certificate_job_status: localized_job_status(
            read_job_status(JobKind::Certificate),
            language,
        ),
        has_flash,
        flash_class,
        flash_message,
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

#[derive(Clone, Copy, Eq, PartialEq)]
enum ServiceFilter {
    Running,
    Failed,
    Stopped,
    All,
}

impl ServiceFilter {
    fn from_param(param: Option<&str>) -> Self {
        match param {
            Some("failed") => Self::Failed,
            Some("stopped") => Self::Stopped,
            Some("all") => Self::All,
            _ => Self::Running,
        }
    }

    fn param(self) -> &'static str {
        match self {
            Self::Running => "running",
            Self::Failed => "failed",
            Self::Stopped => "stopped",
            Self::All => "all",
        }
    }

    fn matches(self, service: &ServiceUnit) -> bool {
        match self {
            Self::Running => service.active_state == "active",
            Self::Failed => service.active_state == "failed",
            Self::Stopped => service.active_state == "inactive",
            Self::All => true,
        }
    }
}

fn filtered_service_rows(
    language: Language,
    services: Vec<ServiceUnit>,
    filter: ServiceFilter,
) -> Vec<ServiceRow> {
    services
        .into_iter()
        .filter(|service| filter.matches(service))
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

fn segment_class(active_filter: ServiceFilter, filter: ServiceFilter) -> String {
    if active_filter == filter {
        "active".to_owned()
    } else {
        String::new()
    }
}

fn scoped_lang_state_path(
    base_path: &str,
    path: &str,
    language: Language,
    filter: ServiceFilter,
) -> String {
    format!(
        "{}?lang={}&state={}",
        scoped_path(base_path, path),
        language.code(),
        filter.param()
    )
}

fn localized_job_status(mut status: JobStatus, language: Language) -> JobStatus {
    status.status = match (language, status.status.as_str()) {
        (Language::Zh, "idle") => "未执行",
        (Language::Zh, "running") => "执行中",
        (Language::Zh, "completed") => "已完成",
        (Language::Zh, "failed") => "失败",
        (Language::En, "idle") => "Idle",
        (Language::En, "running") => "Running",
        (Language::En, "completed") => "Completed",
        (Language::En, "failed") => "Failed",
        _ => status.status.as_str(),
    }
    .to_owned();

    if matches!(language, Language::En) && status.message == "尚未执行" {
        status.message = "Not run yet".to_owned();
    }

    status
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
