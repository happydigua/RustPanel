use askama::Template;
use axum::{
    Form, Json,
    extract::{Query, State},
    http::{HeaderMap, StatusCode},
    response::{Html, IntoResponse, Redirect, Response},
};
use rustpanel_core::AppSpec;
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::{
    auth::{clear_session_cookie, is_authenticated, set_session_cookie},
    config::{scoped_lang_path, scoped_path},
    i18n::Language,
    services::collect_service_units,
    state::AppState,
    system_metrics::collect_system_metrics,
    templates::{
        AppRow, AppsPartialTemplate, DashboardTemplate, LoginTemplate, UpdateCheckTemplate,
    },
    updates::run_update_check,
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

#[derive(Serialize)]
struct HealthResponse {
    status: &'static str,
    service: &'static str,
}

pub(crate) async fn dashboard(
    headers: HeaderMap,
    State(state): State<AppState>,
    Query(params): Query<PageParams>,
) -> Response {
    let language = Language::from_param(params.lang.as_deref());
    if !is_authenticated(&headers, &state.auth) {
        return Redirect::to(&scoped_lang_path(&state.base_path, "/login", language))
            .into_response();
    }

    let labels = language.labels();
    let service_units = collect_service_units(50);
    let service_count = service_units.len();

    render(DashboardTemplate {
        rows: app_rows(&state.apps),
        service_units,
        has_services: service_count > 0,
        app_count: state.apps.len(),
        domain_count: state.apps.iter().map(|app| app.domains.len()).sum(),
        service_count,
        nginx_conf_dir: state.paths.nginx_conf_dir.display().to_string(),
        apps_path: scoped_lang_path(&state.base_path, "/apps", language),
        update_check_path: scoped_lang_path(&state.base_path, "/update-check", language),
        lang_zh_path: scoped_lang_path(&state.base_path, "/", Language::Zh),
        lang_en_path: scoped_lang_path(&state.base_path, "/", Language::En),
        logout_path: scoped_lang_path(&state.base_path, "/logout", language),
        version: current_version(),
        metrics: collect_system_metrics(),
        labels,
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

pub(crate) async fn apps_partial(
    headers: HeaderMap,
    State(state): State<AppState>,
    Query(params): Query<PageParams>,
) -> Response {
    let language = Language::from_param(params.lang.as_deref());
    if !is_authenticated(&headers, &state.auth) {
        return Redirect::to(&scoped_lang_path(&state.base_path, "/login", language))
            .into_response();
    }

    render(AppsPartialTemplate {
        rows: app_rows(&state.apps),
        labels: language.labels(),
    })
    .into_response()
}

pub(crate) async fn update_check_page(
    headers: HeaderMap,
    State(state): State<AppState>,
    Query(params): Query<PageParams>,
) -> Response {
    let language = Language::from_param(params.lang.as_deref());
    if !is_authenticated(&headers, &state.auth) {
        return Redirect::to(&scoped_lang_path(&state.base_path, "/login", language))
            .into_response();
    }

    let labels = language.labels();
    let result = run_update_check(language).await;

    render(UpdateCheckTemplate {
        dashboard_path: scoped_lang_path(&state.base_path, "/", language),
        lang_zh_path: scoped_lang_path(&state.base_path, "/update-check", Language::Zh),
        lang_en_path: scoped_lang_path(&state.base_path, "/update-check", Language::En),
        logout_path: scoped_lang_path(&state.base_path, "/logout", language),
        version: current_version(),
        result,
        labels,
    })
    .into_response()
}

pub(crate) async fn healthz() -> impl IntoResponse {
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

fn current_version() -> String {
    env!("CARGO_PKG_VERSION").to_owned()
}
