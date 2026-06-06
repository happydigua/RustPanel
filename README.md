# RustPanel

RustPanel is a small systemd-first server panel for developers who deploy
services directly on lightweight Linux servers.

The project intentionally starts narrow:

- systemd service generation and control
- systemd service discovery for the panel service view
- password-protected web login
- basic CPU load, memory, disk, and uptime monitoring
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
curl -fL --connect-timeout 15 --max-time 120 https://raw.githubusercontent.com/happydigua/RustPanel/main/scripts/bootstrap-linux.sh -o /tmp/rustpanel-bootstrap-linux.sh && sudo bash /tmp/rustpanel-bootstrap-linux.sh
```

The installer prints a server-IP based access URL:

```text
Username: admin
Password: 8b1f2d6e9a1c0f4b7d3a55c2
Access URL: http://SERVER_IP:28437/rp-a13f9c2d8e4b7a90
```

The port and path are both generated during install and reused during upgrades.
If the page does not open, allow the printed TCP port in the cloud
firewall/security group. If public IP detection fails, the installer prints
`PUBLIC_SERVER_IP`; replace it with the public IP shown by your cloud provider.

The one-command install downloads prebuilt release binaries. It does not install
Rust or run a Rust compiler on the server.

For local-only install:

```bash
curl -fL --connect-timeout 15 --max-time 120 https://raw.githubusercontent.com/happydigua/RustPanel/main/scripts/bootstrap-linux.sh -o /tmp/rustpanel-bootstrap-linux.sh && sudo bash /tmp/rustpanel-bootstrap-linux.sh --local
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
