use std::{path::PathBuf, process::Command as ProcessCommand};

use anyhow::{Context, Result, bail};
use clap::{Parser, Subcommand, ValueEnum};
use rustpanel_core::{
    AppSpec, PanelPaths, render_env_file, render_nginx_server, render_systemd_service,
};

const DEFAULT_REPO_URL: &str = "https://github.com/happydigua/RustPanel.git";
const DEFAULT_BRANCH: &str = "main";
const DEFAULT_SOURCE_DIR: &str = "/opt/rustpanel-src";

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

    /// Check whether the source install is behind the GitHub branch.
    UpdateCheck {
        #[arg(long, default_value = DEFAULT_SOURCE_DIR)]
        source_dir: PathBuf,

        #[arg(long, default_value = DEFAULT_REPO_URL)]
        repo_url: String,

        #[arg(long, default_value = DEFAULT_BRANCH)]
        branch: String,
    },

    /// Update RustPanel from the source install directory.
    Update {
        #[arg(long, default_value = DEFAULT_SOURCE_DIR)]
        source_dir: PathBuf,

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
        Command::UpdateCheck {
            source_dir,
            repo_url,
            branch,
        } => update_check(source_dir, repo_url, branch),
        Command::Update {
            source_dir,
            minimal,
            public,
            local,
        } => update(source_dir, minimal, public, local),
        Command::RenderSample { artifact } => render_sample(artifact),
    }
}

fn print_version() -> Result<()> {
    println!("RustPanel {}", env!("CARGO_PKG_VERSION"));
    Ok(())
}

fn update_check(source_dir: PathBuf, repo_url: String, branch: String) -> Result<()> {
    if !source_dir.exists() {
        bail!("source directory does not exist: {}", source_dir.display());
    }

    let local = git_output([
        "-C".to_owned(),
        source_dir.display().to_string(),
        "rev-parse".to_owned(),
        "HEAD".to_owned(),
    ])?;
    let remote_ref = format!("refs/heads/{branch}");
    let remote = git_output(["ls-remote".to_owned(), repo_url.clone(), remote_ref])?;
    let Some(remote_commit) = remote.split_whitespace().next() else {
        bail!("remote branch `{branch}` was not found at {repo_url}");
    };

    let local_commit = local.trim();

    println!("Version: {}", env!("CARGO_PKG_VERSION"));
    println!("Source: {}", source_dir.display());
    println!("Branch: {branch}");
    println!("Local:  {local_commit}");
    println!("Remote: {remote_commit}");

    if local_commit == remote_commit {
        println!("Status: up to date");
    } else {
        println!("Status: update available");
        println!("Update: sudo rustpanel update");
    }

    Ok(())
}

fn update(source_dir: PathBuf, minimal: bool, public: bool, local: bool) -> Result<()> {
    ensure_root()?;

    let script = source_dir.join("scripts/bootstrap-linux.sh");
    if !script.exists() {
        bail!("bootstrap script does not exist: {}", script.display());
    }

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

    println!("Updating RustPanel from {}", source_dir.display());
    let status = ProcessCommand::new(script)
        .args(args)
        .status()
        .context("failed to execute bootstrap script")?;

    if !status.success() {
        bail!("update failed with status {status}");
    }

    Ok(())
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

fn git_output<I>(args: I) -> Result<String>
where
    I: IntoIterator<Item = String>,
{
    let output = ProcessCommand::new("git")
        .args(args)
        .output()
        .context("failed to execute git")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("git command failed: {}", stderr.trim());
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
