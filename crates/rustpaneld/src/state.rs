use crate::{auth::AuthConfig, config::DaemonConfig};

#[derive(Clone)]
pub(crate) struct AppState {
    pub(crate) base_path: String,
    pub(crate) auth: AuthConfig,
}

impl AppState {
    pub(crate) fn from_config(config: &DaemonConfig) -> Self {
        Self {
            base_path: config.base_path.clone(),
            auth: config.auth.clone(),
        }
    }
}
