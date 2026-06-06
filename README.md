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

RustPanel listens on `127.0.0.1:7654` by default. Open it through an SSH
tunnel:

```bash
ssh -L 7654:127.0.0.1:7654 root@SERVER_IP
```

Then visit `http://127.0.0.1:7654`.

## First vertical slice

```bash
cargo test
cargo run -p rustpanel -- render-sample
cargo run -p rustpaneld
```

The daemon listens on `127.0.0.1:7654` by default.
