# Deployment

This is the current source-based deployment flow for lightweight Linux servers.

## One-command install

```bash
curl -fsSL https://raw.githubusercontent.com/happydigua/RustPanel/main/scripts/bootstrap-linux.sh | sudo bash
```

This installs build dependencies, Rust, Nginx, certbot, clones the repository to
`/opt/rustpanel-src`, builds release binaries, installs them into
`/usr/local/bin`, and enables `rustpaneld`.

Use minimal mode when you do not want RustPanel to install Nginx/certbot:

```bash
curl -fsSL https://raw.githubusercontent.com/happydigua/RustPanel/main/scripts/bootstrap-linux.sh | sudo bash -s -- --minimal
```

RustPanel listens on `127.0.0.1:7654` by default.

```bash
ssh -L 7654:127.0.0.1:7654 root@SERVER_IP
```

Then open:

```text
http://127.0.0.1:7654
```

## Manual prerequisites

Ubuntu/Debian:

```bash
sudo apt update
sudo apt install -y build-essential pkg-config libssl-dev git curl
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source "$HOME/.cargo/env"
```

Rocky/Alma/CentOS with `dnf`:

```bash
sudo dnf groupinstall -y "Development Tools"
sudo dnf install -y git curl pkg-config openssl-devel
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source "$HOME/.cargo/env"
```

## Manual source install

```bash
git clone git@github.com:happydigua/RustPanel.git
cd RustPanel
sudo env "PATH=$PATH" scripts/install-linux.sh --with-nginx
```

## Debug

```bash
systemctl status rustpaneld --no-pager
journalctl -u rustpaneld -f
rustpanel render-sample
rustpanel-helper contract
```
