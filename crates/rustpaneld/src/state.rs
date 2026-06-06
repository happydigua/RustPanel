use std::sync::Arc;

use rustpanel_core::{AppSpec, PanelPaths};

use crate::{auth::AuthConfig, config::DaemonConfig};

#[derive(Clone)]
pub(crate) struct AppState {
    pub(crate) apps: Arc<Vec<AppSpec>>,
    pub(crate) paths: PanelPaths,
    pub(crate) base_path: String,
    pub(crate) auth: AuthConfig,
}

impl AppState {
    pub(crate) fn from_config(config: &DaemonConfig) -> Self {
        Self {
            apps: Arc::new(vec![AppSpec::sample()]),
            paths: PanelPaths::default(),
            base_path: config.base_path.clone(),
            auth: config.auth.clone(),
        }
    }
}
