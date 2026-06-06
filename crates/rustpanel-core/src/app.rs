use serde::{Deserialize, Serialize};

use crate::ConfigError;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct AppSpec {
    pub id: String,
    pub name: String,
    pub workdir: String,
    pub run_user: String,
    pub exec_start: String,
    pub port: u16,
    pub env: Vec<EnvVar>,
    pub domains: Vec<DomainSpec>,
    pub deploy: DeploySpec,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct EnvVar {
    pub key: String,
    pub value: String,
    pub secret: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct DomainSpec {
    pub name: String,
    pub https: bool,
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct DeploySpec {
    pub steps: Vec<String>,
}

impl AppSpec {
    pub fn validate(&self) -> Result<(), ConfigError> {
        if self.id.is_empty() {
            return Err(ConfigError::EmptyAppId);
        }

        if !self
            .id
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
        {
            return Err(ConfigError::InvalidAppId);
        }

        if self.name.trim().is_empty() {
            return Err(ConfigError::EmptyAppName);
        }

        if self.workdir.trim().is_empty() {
            return Err(ConfigError::EmptyWorkdir);
        }

        if self.run_user.trim().is_empty() {
            return Err(ConfigError::EmptyRunUser);
        }

        if self.exec_start.trim().is_empty() {
            return Err(ConfigError::EmptyExecStart);
        }

        if self.port == 0 {
            return Err(ConfigError::InvalidPort);
        }

        for env in &self.env {
            if !is_env_key(&env.key) {
                return Err(ConfigError::InvalidEnvKey(env.key.clone()));
            }
        }

        for domain in &self.domains {
            if domain.name.trim().is_empty() {
                return Err(ConfigError::EmptyDomain);
            }
        }

        Ok(())
    }

    pub fn service_name(&self) -> String {
        format!("rustpanel-{}", self.id)
    }

    pub fn unit_name(&self) -> String {
        format!("{}.service", self.service_name())
    }

    pub fn upstream_addr(&self) -> String {
        format!("127.0.0.1:{}", self.port)
    }

    pub fn sample() -> Self {
        Self {
            id: "demo-api".to_owned(),
            name: "Demo API".to_owned(),
            workdir: "/srv/demo-api".to_owned(),
            run_user: "deploy".to_owned(),
            exec_start: "/srv/demo-api/target/release/demo-api".to_owned(),
            port: 8080,
            env: vec![EnvVar {
                key: "RUST_LOG".to_owned(),
                value: "info".to_owned(),
                secret: false,
            }],
            domains: vec![DomainSpec {
                name: "api.example.com".to_owned(),
                https: true,
            }],
            deploy: DeploySpec {
                steps: vec![
                    "git pull --ff-only".to_owned(),
                    "cargo build --release".to_owned(),
                ],
            },
        }
    }
}

fn is_env_key(key: &str) -> bool {
    let mut bytes = key.bytes();
    let Some(first) = bytes.next() else {
        return false;
    };

    if !(first.is_ascii_uppercase() || first == b'_') {
        return false;
    }

    bytes.all(|b| b.is_ascii_uppercase() || b.is_ascii_digit() || b == b'_')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validates_sample_app() {
        AppSpec::sample().validate().unwrap();
    }

    #[test]
    fn rejects_uppercase_app_id() {
        let mut app = AppSpec::sample();
        app.id = "Demo".to_owned();

        assert!(matches!(app.validate(), Err(ConfigError::InvalidAppId)));
    }

    #[test]
    fn rejects_invalid_env_key() {
        let mut app = AppSpec::sample();
        app.env[0].key = "rust_log".to_owned();

        assert!(matches!(
            app.validate(),
            Err(ConfigError::InvalidEnvKey(key)) if key == "rust_log"
        ));
    }
}
