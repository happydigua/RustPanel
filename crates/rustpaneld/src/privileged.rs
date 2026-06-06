use std::{
    fs,
    io::{BufRead, BufReader, Write},
    os::unix::net::UnixStream,
    path::PathBuf,
};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

const SOCKET_PATH: &str = "/run/rustpanel/helper.sock";
const STATUS_DIR: &str = "/var/lib/rustpanel/jobs";

#[derive(Clone)]
pub(crate) struct JobStatus {
    pub(crate) status: String,
    pub(crate) status_class: String,
    pub(crate) message: String,
    pub(crate) output: String,
}

#[derive(Clone, Copy)]
pub(crate) enum JobKind {
    Update,
    Certificate,
}

#[derive(Serialize)]
struct HelperRequest {
    action: HelperAction,
}

#[derive(Serialize)]
#[serde(rename_all = "snake_case", tag = "type")]
enum HelperAction {
    Update,
    IssueCertificate { domain: String, email: String },
}

#[derive(Deserialize)]
struct HelperResponse {
    ok: bool,
    message: String,
}

pub(crate) async fn start_update() -> Result<String> {
    send_request(HelperRequest {
        action: HelperAction::Update,
    })
    .await
}

pub(crate) async fn start_certificate(domain: String, email: String) -> Result<String> {
    send_request(HelperRequest {
        action: HelperAction::IssueCertificate { domain, email },
    })
    .await
}

async fn send_request(request: HelperRequest) -> Result<String> {
    tokio::task::spawn_blocking(move || send_request_blocking(request))
        .await
        .context("helper request task failed")?
}

fn send_request_blocking(request: HelperRequest) -> Result<String> {
    let mut stream = UnixStream::connect(SOCKET_PATH)
        .with_context(|| format!("failed to connect helper socket {SOCKET_PATH}"))?;
    let payload = serde_json::to_string(&request)?;
    writeln!(stream, "{payload}")?;

    let mut line = String::new();
    BufReader::new(stream).read_line(&mut line)?;
    let response: HelperResponse =
        serde_json::from_str(&line).context("invalid helper response JSON")?;
    if response.ok {
        Ok(response.message)
    } else {
        anyhow::bail!(response.message)
    }
}

pub(crate) fn read_job_status(kind: JobKind) -> JobStatus {
    let Ok(contents) = fs::read_to_string(status_path(kind)) else {
        return JobStatus {
            status: "idle".to_owned(),
            status_class: "idle".to_owned(),
            message: "尚未执行".to_owned(),
            output: String::new(),
        };
    };

    let status = field(&contents, "Status").unwrap_or("unknown").to_owned();
    let message = field(&contents, "Message").unwrap_or("").to_owned();
    let output = contents
        .split_once("Output:\n")
        .map(|(_, output)| output.trim().to_owned())
        .unwrap_or_default();
    let status_class = match status.as_str() {
        "completed" => "ok",
        "running" => "warn",
        "failed" => "error",
        _ => "idle",
    }
    .to_owned();

    JobStatus {
        status,
        status_class,
        message,
        output,
    }
}

fn field<'a>(contents: &'a str, key: &str) -> Option<&'a str> {
    contents
        .lines()
        .find_map(|line| line.strip_prefix(&format!("{key}:")))
        .map(str::trim)
}

fn status_path(kind: JobKind) -> PathBuf {
    match kind {
        JobKind::Update => PathBuf::from(STATUS_DIR).join("update.status"),
        JobKind::Certificate => PathBuf::from(STATUS_DIR).join("certificate.status"),
    }
}
