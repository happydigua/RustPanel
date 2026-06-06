use axum::{Router, routing::get};
use tower_http::trace::TraceLayer;

use crate::{
    handlers::{
        healthz, login_page, login_submit, logout, overview_page, processes_page, services_page,
        sites_page, ssl_page, update_check_page,
    },
    state::AppState,
};

pub(crate) fn router(base_path: &str, state: AppState) -> Router {
    let panel_routes = Router::new()
        .route("/", get(overview_page))
        .route("/login", get(login_page).post(login_submit))
        .route("/logout", get(logout))
        .route("/apps", get(sites_page))
        .route("/processes", get(processes_page))
        .route("/services", get(services_page))
        .route("/sites", get(sites_page))
        .route("/ssl", get(ssl_page))
        .route("/update-check", get(update_check_page))
        .route("/healthz", get(healthz));

    let app = if base_path == "/" {
        panel_routes
    } else {
        let base_path_with_slash = format!("{base_path}/");
        Router::new()
            .route(&base_path_with_slash, get(overview_page))
            .nest(base_path, panel_routes)
            .route("/healthz", get(healthz))
    };

    app.with_state(state).layer(TraceLayer::new_for_http())
}
