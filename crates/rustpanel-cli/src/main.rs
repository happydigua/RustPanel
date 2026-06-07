use std::{
    fs,
    path::PathBuf,
    process::Command as ProcessCommand,
    time::{SystemTime, UNIX_EPOCH},
};

use anyhow::{Context, Result, bail};
use clap::{Parser, Subcommand, ValueEnum};
use rustpanel_core::{
    AppSpec, PanelPaths, render_env_file, render_nginx_server, render_systemd_service,
};
use serde::Deserialize;

const DEFAULT_RELEASE_API_URL: &str =
    "https://api.github.com/repos/happydigua/RustPanel/releases/latest";
const DEFAULT_BOOTSTRAP_URL: &str =
    "https://raw.githubusercontent.com/happydigua/RustPanel/main/scripts/bootstrap-linux.sh";

#[derive(Debug, Parser)]
#[command(name = "rustpanel")]
#[command(about = "A small systemd-first server panel")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Print the installed CLI version.
    Version,

    /// Check whether the installed binary is behind the latest GitHub Release.
    UpdateCheck {
        #[arg(long, default_value = DEFAULT_RELEASE_API_URL)]
        release_api_url: String,
    },

    /// Update RustPanel by downloading the current bootstrap script.
    Update {
        #[arg(long, default_value = DEFAULT_BOOTSTRAP_URL)]
        bootstrap_url: String,

        #[arg(long)]
        minimal: bool,

        #[arg(long, conflicts_with = "local")]
        public: bool,

        #[arg(long)]
        local: bool,
    },

    /// Render sample config artifacts for the first vertical slice.
    RenderSample {
        #[arg(default_value = "all")]
        artifact: Artifact,
    },
}

#[derive(Clone, Debug, ValueEnum)]
enum Artifact {
    All,
    Systemd,
    Env,
    Nginx,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Command::Version => print_version(),
        Command::UpdateCheck { release_api_url } => update_check(release_api_url),
        Command::Update {
            bootstrap_url,
            minimal,
            public,
            local,
        } => update(bootstrap_url, minimal, public, local),
        Command::RenderSample { artifact } => render_sample(artifact),
    }
}

fn print_version() -> Result<()> {
    println!("RustPanel {}", env!("CARGO_PKG_VERSION"));
    Ok(())
}

fn update_check(release_api_url: String) -> Result<()> {
    let latest = latest_release_tag(&release_api_url)?;
    let current = current_tag();

    println!("Version: {}", env!("CARGO_PKG_VERSION"));
    println!("Latest:  {latest}");

    if release_is_newer(&latest, &current) {
        println!("Status: update available");
        println!("Update: sudo rustpanel update");
    } else {
        println!("Status: up to date");
    }

    Ok(())
}

#[derive(Deserialize)]
struct GitHubRelease {
    tag_name: String,
}

fn latest_release_tag(release_api_url: &str) -> Result<String> {
    let output = curl_output(release_api_url)?;
    let release: GitHubRelease =
        serde_json::from_str(&output).context("failed to parse GitHub release response")?;
    Ok(release.tag_name)
}

fn current_tag() -> String {
    format!("v{}", env!("CARGO_PKG_VERSION"))
}

fn release_is_newer(latest: &str, current: &str) -> bool {
    match (
        parse_release_version(latest),
        parse_release_version(current),
    ) {
        (Some(latest), Some(current)) => latest > current,
        _ => latest != current,
    }
}

fn parse_release_version(tag: &str) -> Option<Vec<u64>> {
    tag.trim_start_matches('v')
        .split('.')
        .map(|part| part.parse::<u64>().ok())
        .collect()
}

fn update(bootstrap_url: String, minimal: bool, public: bool, local: bool) -> Result<()> {
    ensure_root()?;
    let script = download_bootstrap_script(&bootstrap_url)?;

    let mut args = Vec::new();
    if minimal {
        args.push("--minimal");
    } else {
        args.push("--with-nginx");
    }

    if public {
        args.push("--public");
    } else if local || current_install_is_local() {
        args.push("--local");
    } else {
        args.push("--public");
    }

    println!("Updating RustPanel with {bootstrap_url}");
    let status = ProcessCommand::new("bash")
        .arg(&script)
        .args(args)
        .status()
        .context("failed to execute bootstrap script")?;
    let _ = fs::remove_file(&script);

    if !status.success() {
        bail!("update failed with status {status}");
    }

    Ok(())
}

fn download_bootstrap_script(url: &str) -> Result<PathBuf> {
    let script = std::env::temp_dir().join(format!(
        "rustpanel-bootstrap-{}-{}.sh",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .context("system clock is before UNIX_EPOCH")?
            .as_nanos()
    ));

    let output = ProcessCommand::new("curl")
        .args([
            "-fL",
            "--connect-timeout",
            "15",
            "--max-time",
            "120",
            "-o",
            script
                .to_str()
                .context("bootstrap script path is not valid UTF-8")?,
            url,
        ])
        .output()
        .context("failed to execute curl")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("failed to download bootstrap script: {}", stderr.trim());
    }

    Ok(script)
}

fn ensure_root() -> Result<()> {
    let output = ProcessCommand::new("id")
        .arg("-u")
        .output()
        .context("failed to execute id -u")?;

    if !output.status.success() {
        bail!("failed to determine current uid");
    }

    let uid = String::from_utf8_lossy(&output.stdout);
    if uid.trim() != "0" {
        bail!("run update with sudo: sudo rustpanel update");
    }

    Ok(())
}

fn current_install_is_local() -> bool {
    let Ok(contents) = std::fs::read_to_string("/etc/rustpanel/rustpanel.env") else {
        return false;
    };

    contents
        .lines()
        .find_map(|line| line.strip_prefix("RUSTPANEL_BIND="))
        .is_some_and(|bind| bind.starts_with("127.0.0.1:") || bind.starts_with("localhost:"))
}

fn curl_output(url: &str) -> Result<String> {
    let output = ProcessCommand::new("curl")
        .args(["-fsSL", "--connect-timeout", "15", "--max-time", "120", url])
        .output()
        .context("failed to execute curl")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("curl command failed: {}", stderr.trim());
    }

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

fn render_sample(artifact: Artifact) -> Result<()> {
    let app = AppSpec::sample();
    let paths = PanelPaths::default();

    match artifact {
        Artifact::All => {
            print_systemd(&app, &paths)?;
            print_env(&app)?;
            print_nginx(&app, &paths)?;
        }
        Artifact::Systemd => print_systemd(&app, &paths)?,
        Artifact::Env => print_env(&app)?,
        Artifact::Nginx => print_nginx(&app, &paths)?,
    }

    Ok(())
}

fn print_systemd(app: &AppSpec, paths: &PanelPaths) -> Result<()> {
    let unit = render_systemd_service(app, paths)?;
    println!(
        "--- {} ---",
        paths.systemd_unit_for(&unit.unit_name).display()
    );
    print!("{}", unit.contents);
    println!();
    Ok(())
}

fn print_env(app: &AppSpec) -> Result<()> {
    println!("--- /etc/rustpanel/apps/{}.env ---", app.id);
    print!("{}", render_env_file(app)?);
    println!();
    Ok(())
}

fn print_nginx(app: &AppSpec, paths: &PanelPaths) -> Result<()> {
    for domain in &app.domains {
        let config = render_nginx_server(app, domain, paths)?;
        println!(
            "--- {} ---",
            paths.nginx_config_for(&app.id, &domain.name).display()
        );
        print!("{}", config.contents);
        println!();
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn current_tag_has_v_prefix() {
        assert!(current_tag().starts_with('v'));
    }

    #[test]
    fn compares_release_versions() {
        assert!(release_is_newer("v0.1.5", "v0.1.4"));
        assert!(!release_is_newer("v0.1.4", "v0.1.5"));
        assert!(!release_is_newer("v0.1.5", "v0.1.5"));
    }
}
