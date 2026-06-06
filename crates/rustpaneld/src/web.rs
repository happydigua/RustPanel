use axum::{Router, routing::get};
use tower_http::trace::TraceLayer;

use crate::{
    handlers::{
        apps_partial, dashboard, healthz, login_page, login_submit, logout, update_check_page,
    },
    state::AppState,
};

pub(crate) fn router(base_path: &str, state: AppState) -> Router {
    let panel_routes = Router::new()
        .route("/", get(dashboard))
        .route("/login", get(login_page).post(login_submit))
        .route("/logout", get(logout))
        .route("/apps", get(apps_partial))
        .route("/update-check", get(update_check_page))
        .route("/healthz", get(healthz));

    let app = if base_path == "/" {
        panel_routes
    } else {
        let base_path_with_slash = format!("{base_path}/");
        Router::new()
            .route(&base_path_with_slash, get(dashboard))
            .nest(base_path, panel_routes)
            .route("/healthz", get(healthz))
    };

    app.with_state(state).layer(TraceLayer::new_for_http())
}
