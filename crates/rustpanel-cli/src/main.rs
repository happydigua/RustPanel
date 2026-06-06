use std::{
    fs,
    path::{Path, PathBuf},
    process::Command as ProcessCommand,
};

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

    let local_commit = local_commit_from_source_dir(&source_dir)?;
    let remote_ref = format!("refs/heads/{branch}");
    let remote = git_output(["ls-remote".to_owned(), repo_url.clone(), remote_ref])?;
    let Some(remote_commit) = remote.split_whitespace().next() else {
        bail!("remote branch `{branch}` was not found at {repo_url}");
    };

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

fn local_commit_from_source_dir(source_dir: &Path) -> Result<String> {
    let git_dir = resolve_git_dir(source_dir)?;
    let head = read_trimmed(git_dir.join("HEAD"))?;

    let Some(ref_name) = head.strip_prefix("ref: ") else {
        return Ok(head);
    };

    let loose_ref = git_dir.join(ref_name);
    if loose_ref.exists() {
        return read_trimmed(loose_ref);
    }

    read_packed_ref(&git_dir, ref_name)
}

fn resolve_git_dir(source_dir: &Path) -> Result<PathBuf> {
    let dot_git = source_dir.join(".git");
    if dot_git.is_dir() {
        return Ok(dot_git);
    }

    let contents = fs::read_to_string(&dot_git)
        .with_context(|| format!("failed to read {}", dot_git.display()))?;
    let Some(raw_git_dir) = contents.trim().strip_prefix("gitdir:") else {
        bail!("invalid git metadata file: {}", dot_git.display());
    };

    let git_dir = PathBuf::from(raw_git_dir.trim());
    if git_dir.is_absolute() {
        Ok(git_dir)
    } else {
        Ok(source_dir.join(git_dir))
    }
}

fn read_trimmed(path: impl AsRef<Path>) -> Result<String> {
    let path = path.as_ref();
    Ok(fs::read_to_string(path)
        .with_context(|| format!("failed to read {}", path.display()))?
        .trim()
        .to_owned())
}

fn read_packed_ref(git_dir: &Path, ref_name: &str) -> Result<String> {
    let packed_refs_path = git_dir.join("packed-refs");
    let packed_refs = fs::read_to_string(&packed_refs_path)
        .with_context(|| format!("failed to read {}", packed_refs_path.display()))?;

    for line in packed_refs.lines() {
        if line.is_empty() || line.starts_with('#') || line.starts_with('^') {
            continue;
        }

        let mut parts = line.split_whitespace();
        let Some(commit) = parts.next() else {
            continue;
        };
        if parts.next() == Some(ref_name) {
            return Ok(commit.to_owned());
        }
    }

    bail!("ref `{ref_name}` was not found in {}", git_dir.display());
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

#[cfg(test)]
mod tests {
    use std::time::{SystemTime, UNIX_EPOCH};

    use super::*;

    #[test]
    fn reads_local_commit_from_loose_ref() {
        let source_dir = test_source_dir("loose-ref");
        let git_dir = source_dir.join(".git");
        fs::create_dir_all(git_dir.join("refs/heads")).unwrap();
        fs::write(git_dir.join("HEAD"), "ref: refs/heads/main\n").unwrap();
        fs::write(
            git_dir.join("refs/heads/main"),
            "1111111111111111111111111111111111111111\n",
        )
        .unwrap();

        assert_eq!(
            local_commit_from_source_dir(&source_dir).unwrap(),
            "1111111111111111111111111111111111111111"
        );

        fs::remove_dir_all(source_dir).unwrap();
    }

    #[test]
    fn reads_local_commit_from_packed_ref() {
        let source_dir = test_source_dir("packed-ref");
        let git_dir = source_dir.join(".git");
        fs::create_dir_all(&git_dir).unwrap();
        fs::write(git_dir.join("HEAD"), "ref: refs/heads/main\n").unwrap();
        fs::write(
            git_dir.join("packed-refs"),
            "# pack-refs\n2222222222222222222222222222222222222222 refs/heads/main\n",
        )
        .unwrap();

        assert_eq!(
            local_commit_from_source_dir(&source_dir).unwrap(),
            "2222222222222222222222222222222222222222"
        );

        fs::remove_dir_all(source_dir).unwrap();
    }

    #[test]
    fn reads_local_commit_from_gitdir_file() {
        let root = test_source_dir("gitdir-file-root");
        let source_dir = root.join("worktree");
        let git_dir = root.join("actual-git-dir");
        fs::create_dir_all(&source_dir).unwrap();
        fs::create_dir_all(git_dir.join("refs/heads")).unwrap();
        fs::write(
            source_dir.join(".git"),
            format!("gitdir: {}\n", git_dir.display()),
        )
        .unwrap();
        fs::write(git_dir.join("HEAD"), "ref: refs/heads/main\n").unwrap();
        fs::write(
            git_dir.join("refs/heads/main"),
            "3333333333333333333333333333333333333333\n",
        )
        .unwrap();

        assert_eq!(
            local_commit_from_source_dir(&source_dir).unwrap(),
            "3333333333333333333333333333333333333333"
        );

        fs::remove_dir_all(root).unwrap();
    }

    fn test_source_dir(name: &str) -> PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!("rustpanel-{name}-{unique}"));
        fs::create_dir_all(&path).unwrap();
        path
    }
}
