# RustPanel

RustPanel is a small systemd-first server panel for developers who deploy
services directly on lightweight Linux servers.

The project intentionally starts narrow:

- systemd service generation and control
- journald-oriented logs
- Nginx reverse proxy configuration
- Let's Encrypt-ready ACME webroot layout
- a small Rust daemon with server-rendered UI
- a CLI for bootstrap and diagnostics

It is not intended to become a full software marketplace or a replacement for
every feature in traditional hosting panels.

## Workspace

```text
crates/rustpanel-core     Shared app model and config renderers
crates/rustpaneld         Web daemon
crates/rustpanel-helper   Future root helper for privileged actions
crates/rustpanel-cli      CLI for setup, preview, and diagnostics
```

## One-command server install

```bash
curl -fsSL https://raw.githubusercontent.com/happydigua/RustPanel/main/scripts/bootstrap-linux.sh | sudo bash
```

The installer prints a server-IP based access URL:

```text
Access URL: http://SERVER_IP:7654/rp-a13f9c2d8e4b7a90
```

If the page does not open, allow TCP `7654` in the cloud firewall/security
group. The random path is generated during install and reused during upgrades.
Public mode is still for early debugging until real authentication is added.

For local-only install:

```bash
curl -fsSL https://raw.githubusercontent.com/happydigua/RustPanel/main/scripts/bootstrap-linux.sh | sudo bash -s -- --local
```

## First vertical slice

```bash
cargo test
cargo run -p rustpanel -- render-sample
cargo run -p rustpanel -- version
cargo run -p rustpanel -- update-check --source-dir .
cargo run -p rustpaneld
```

The daemon listens on `127.0.0.1:7654` when run locally with `cargo run`.
