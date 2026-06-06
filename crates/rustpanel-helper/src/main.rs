use std::{
    fs::{self, OpenOptions},
    io::{BufRead, BufReader, Write},
    os::unix::{
        fs::PermissionsExt,
        net::{UnixListener, UnixStream},
    },
    path::{Path, PathBuf},
    process::Command as ProcessCommand,
    thread,
};

use anyhow::{Context, Result, bail};
use clap::{Parser, Subcommand};
use rustpanel_core::{AppSpec, PanelPaths, render_nginx_server, render_systemd_service};
use serde::{Deserialize, Serialize};

const SOCKET_PATH: &str = "/run/rustpanel/helper.sock";
const RUNTIME_DIR: &str = "/run/rustpanel";
const STATUS_DIR: &str = "/var/lib/rustpanel/jobs";

#[derive(Debug, Parser)]
#[command(name = "rustpanel-helper")]
#[command(about = "Privileged helper for RustPanel")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Print the privileged operations accepted by the helper daemon.
    Contract,

    /// Run the root helper daemon.
    Serve,

    /// Validate that core config rendering is available to the helper.
    ValidateSample,
}

#[derive(Debug, Deserialize)]
struct HelperRequest {
    action: HelperAction,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case", tag = "type")]
enum HelperAction {
    Update,
    IssueCertificate { domain: String, email: String },
}

#[derive(Debug, Serialize)]
struct HelperResponse {
    ok: bool,
    message: String,
}

#[derive(Clone, Copy)]
enum JobKind {
    Update,
    Certificate,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Command::Contract => print_contract(),
        Command::Serve => serve(),
        Command::ValidateSample => validate_sample(),
    }
}

fn print_contract() -> Result<()> {
    println!("rustpanel-helper accepts structured privileged operations only:");
    println!(r#"  {{"action":{{"type":"update"}}}}"#);
    println!(
        r#"  {{"action":{{"type":"issue_certificate","domain":"example.com","email":"admin@example.com"}}}}"#
    );
    Ok(())
}

fn serve() -> Result<()> {
    prepare_runtime_dir()?;
    let socket_path = Path::new(SOCKET_PATH);
    if socket_path.exists() {
        fs::remove_file(socket_path)
            .with_context(|| format!("failed to remove stale socket {SOCKET_PATH}"))?;
    }

    let listener = UnixListener::bind(socket_path)
        .with_context(|| format!("failed to bind helper socket {SOCKET_PATH}"))?;
    fs::set_permissions(socket_path, fs::Permissions::from_mode(0o660))
        .context("failed to chmod helper socket")?;
    let _ = ProcessCommand::new("chown")
        .args(["root:rustpanel", SOCKET_PATH])
        .status();

    println!("rustpanel-helper listening on {SOCKET_PATH}");
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                thread::spawn(move || {
                    if let Err(error) = handle_client(stream) {
                        eprintln!("helper client error: {error:#}");
                    }
                });
            }
            Err(error) => eprintln!("helper accept error: {error}"),
        }
    }

    Ok(())
}

fn prepare_runtime_dir() -> Result<()> {
    fs::create_dir_all(RUNTIME_DIR).context("failed to create helper runtime dir")?;
    fs::set_permissions(RUNTIME_DIR, fs::Permissions::from_mode(0o750))
        .context("failed to chmod helper runtime dir")?;
    let _ = ProcessCommand::new("chown")
        .args(["root:rustpanel", RUNTIME_DIR])
        .status();
    fs::create_dir_all(STATUS_DIR).context("failed to create helper status dir")?;
    fs::set_permissions(STATUS_DIR, fs::Permissions::from_mode(0o750))
        .context("failed to chmod helper status dir")?;
    let _ = ProcessCommand::new("chown")
        .args(["root:rustpanel", STATUS_DIR])
        .status();
    Ok(())
}

fn handle_client(mut stream: UnixStream) -> Result<()> {
    let mut line = String::new();
    BufReader::new(stream.try_clone()?).read_line(&mut line)?;
    let request: HelperRequest =
        serde_json::from_str(&line).context("invalid helper request JSON")?;
    let response = handle_request(request);
    let payload = serde_json::to_string(&response)?;
    writeln!(stream, "{payload}")?;
    Ok(())
}

fn handle_request(request: HelperRequest) -> HelperResponse {
    match request.action {
        HelperAction::Update => {
            start_background_job(JobKind::Update, "RustPanel update".to_owned())
        }
        HelperAction::IssueCertificate { domain, email } => {
            let domain = domain.trim().to_ascii_lowercase();
            let email = email.trim().to_owned();
            if let Err(error) = validate_domain(&domain).and_then(|_| validate_email(&email)) {
                return HelperResponse {
                    ok: false,
                    message: error.to_string(),
                };
            }

            start_certificate_job(domain, email)
        }
    }
}

fn start_background_job(kind: JobKind, message: String) -> HelperResponse {
    let lock_path = lock_path(kind);
    let lock = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&lock_path);

    if lock.is_err() {
        return HelperResponse {
            ok: false,
            message: "已有任务正在执行".to_owned(),
        };
    }

    write_status(kind, "running", &message, "");
    thread::spawn(move || {
        let result = run_update();

        if let Err(error) = result {
            write_status(kind, "failed", &error.to_string(), "");
        }

        let _ = fs::remove_file(lock_path);
    });

    HelperResponse { ok: true, message }
}

fn start_certificate_job(domain: String, email: String) -> HelperResponse {
    let kind = JobKind::Certificate;
    let lock_path = lock_path(kind);
    let lock = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&lock_path);

    if lock.is_err() {
        return HelperResponse {
            ok: false,
            message: "已有任务正在执行".to_owned(),
        };
    }

    let message = format!("正在为 {domain} 申请证书");
    write_status(kind, "running", &message, "");
    let response_message = message.clone();
    thread::spawn(move || {
        if let Err(error) = run_certificate(&domain, &email) {
            write_status(kind, "failed", &error.to_string(), "");
        }

        let _ = fs::remove_file(lock_path);
    });

    HelperResponse {
        ok: true,
        message: response_message,
    }
}

fn run_update() -> Result<()> {
    let output = ProcessCommand::new("/usr/local/bin/rustpanel")
        .arg("update")
        .output()
        .context("failed to run rustpanel update")?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let combined = format!("{stdout}{stderr}");

    if output.status.success() {
        write_status(
            JobKind::Update,
            "completed",
            "RustPanel update completed",
            &combined,
        );
    } else {
        write_status(
            JobKind::Update,
            "failed",
            "RustPanel update failed",
            &combined,
        );
    }

    Ok(())
}

fn run_certificate(domain: &str, email: &str) -> Result<()> {
    let output = ProcessCommand::new("certbot")
        .args([
            "certonly",
            "--nginx",
            "--non-interactive",
            "--agree-tos",
            "--keep-until-expiring",
            "--email",
            email,
            "-d",
            domain,
        ])
        .output()
        .context("failed to run certbot")?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let combined = format!("{stdout}{stderr}");

    if output.status.success() {
        write_status(
            JobKind::Certificate,
            "completed",
            &format!("证书申请完成: {domain}"),
            &combined,
        );
    } else {
        write_status(
            JobKind::Certificate,
            "failed",
            &format!("证书申请失败: {domain}"),
            &combined,
        );
    }

    Ok(())
}

fn validate_domain(domain: &str) -> Result<()> {
    let domain = domain.trim();
    if domain.is_empty() || domain.len() > 253 || !domain.contains('.') {
        bail!("domain is invalid");
    }

    for label in domain.split('.') {
        if label.is_empty()
            || label.len() > 63
            || label.starts_with('-')
            || label.ends_with('-')
            || !label
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-')
        {
            bail!("domain is invalid");
        }
    }

    Ok(())
}

fn validate_email(email: &str) -> Result<()> {
    if email.trim().is_empty() || email.contains(char::is_whitespace) || !email.contains('@') {
        bail!("email is invalid");
    }

    Ok(())
}

fn write_status(kind: JobKind, status: &str, message: &str, output: &str) {
    let contents = format!(
        "Status: {status}\nMessage: {message}\nOutput:\n{}\n",
        output.trim()
    );
    let _ = fs::write(status_path(kind), contents);
}

fn status_path(kind: JobKind) -> PathBuf {
    match kind {
        JobKind::Update => PathBuf::from(STATUS_DIR).join("update.status"),
        JobKind::Certificate => PathBuf::from(STATUS_DIR).join("certificate.status"),
    }
}

fn lock_path(kind: JobKind) -> PathBuf {
    match kind {
        JobKind::Update => PathBuf::from(STATUS_DIR).join("update.lock"),
        JobKind::Certificate => PathBuf::from(STATUS_DIR).join("certificate.lock"),
    }
}

fn validate_sample() -> Result<()> {
    let app = AppSpec::sample();
    let paths = PanelPaths::default();

    let _unit = render_systemd_service(&app, &paths)?;
    for domain in &app.domains {
        let _nginx = render_nginx_server(&app, domain, &paths)?;
    }

    println!("sample helper inputs are valid");
    Ok(())
}
