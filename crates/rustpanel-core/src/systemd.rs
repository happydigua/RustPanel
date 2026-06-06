use crate::{AppSpec, ConfigError, PanelPaths};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SystemdServiceConfig {
    pub unit_name: String,
    pub contents: String,
}

pub fn render_systemd_service(
    app: &AppSpec,
    paths: &PanelPaths,
) -> Result<SystemdServiceConfig, ConfigError> {
    app.validate()?;

    let env_file = paths.env_file_for(&app.id);
    let contents = format!(
        "\
[Unit]
Description=RustPanel app: {name}
After=network-online.target
Wants=network-online.target

[Service]
Type=simple
User={user}
WorkingDirectory={workdir}
EnvironmentFile=-{env_file}
ExecStart={exec_start}
Restart=always
RestartSec=3
KillSignal=SIGINT
TimeoutStopSec=30

[Install]
WantedBy=multi-user.target
",
        name = app.name,
        user = app.run_user,
        workdir = app.workdir,
        env_file = env_file.display(),
        exec_start = app.exec_start,
    );

    Ok(SystemdServiceConfig {
        unit_name: app.unit_name(),
        contents,
    })
}

pub fn render_env_file(app: &AppSpec) -> Result<String, ConfigError> {
    app.validate()?;

    let mut output = String::new();
    for env in &app.env {
        output.push_str(&env.key);
        output.push('=');
        output.push_str(&render_env_value(&env.value));
        output.push('\n');
    }

    Ok(output)
}

fn render_env_value(value: &str) -> String {
    if value
        .bytes()
        .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'_' | b'-' | b'.' | b'/' | b':'))
    {
        return value.to_owned();
    }

    let escaped = value
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n");
    format!("\"{escaped}\"")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders_unit_with_expected_service_name() {
        let app = AppSpec::sample();
        let unit = render_systemd_service(&app, &PanelPaths::default()).unwrap();

        assert_eq!(unit.unit_name, "rustpanel-demo-api.service");
        assert!(unit.contents.contains("User=deploy"));
        assert!(
            unit.contents
                .contains("ExecStart=/srv/demo-api/target/release/demo-api")
        );
    }

    #[test]
    fn renders_env_file() {
        let mut app = AppSpec::sample();
        app.env.push(crate::EnvVar {
            key: "DATABASE_URL".to_owned(),
            value: "postgres://user:pass@127.0.0.1/app db".to_owned(),
            secret: true,
        });

        let env = render_env_file(&app).unwrap();

        assert!(env.contains("RUST_LOG=info\n"));
        assert!(env.contains("DATABASE_URL=\"postgres://user:pass@127.0.0.1/app db\"\n"));
    }
}
