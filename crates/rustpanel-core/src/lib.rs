pub mod app;
pub mod nginx;
pub mod paths;
pub mod systemd;

pub use app::{AppSpec, DeploySpec, DomainSpec, EnvVar};
pub use nginx::{NginxServerConfig, render_nginx_server};
pub use paths::PanelPaths;
pub use systemd::{SystemdServiceConfig, render_env_file, render_systemd_service};

use thiserror::Error;

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("app id must contain only lowercase letters, numbers, and hyphens")]
    InvalidAppId,

    #[error("app id cannot be empty")]
    EmptyAppId,

    #[error("app name cannot be empty")]
    EmptyAppName,

    #[error("workdir cannot be empty")]
    EmptyWorkdir,

    #[error("run user cannot be empty")]
    EmptyRunUser,

    #[error("exec_start cannot be empty")]
    EmptyExecStart,

    #[error("port must be greater than zero")]
    InvalidPort,

    #[error("domain cannot be empty")]
    EmptyDomain,

    #[error("environment variable key `{0}` is invalid")]
    InvalidEnvKey(String),
}
