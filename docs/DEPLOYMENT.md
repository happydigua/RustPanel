# Deployment

This is the current deployment flow for lightweight Linux servers.

## One-command install

```bash
curl -fsSL https://raw.githubusercontent.com/happydigua/RustPanel/main/scripts/bootstrap-linux.sh | sudo bash
```

This installs runtime dependencies, downloads RustPanel Linux binaries from the
latest GitHub Release, installs them into `/usr/local/bin`, enables
`rustpaneld`, and prints login credentials plus a server-IP based URL:

```text
Username: admin
Password: 8b1f2d6e9a1c0f4b7d3a55c2
Access URL: http://SERVER_IP:28437/rp-a13f9c2d8e4b7a90
```

The port, path, admin password, and session secret are generated during install,
saved in `/etc/rustpanel/rustpanel.env`, and reused during upgrades. If the page
does not open, allow the printed TCP port in the cloud firewall/security group.
If public IP detection fails, the installer prints `PUBLIC_SERVER_IP`; replace
it with the public IP shown by your cloud provider.

The one-command install uses prebuilt binaries. It does not install Rust on the
server and does not compile the project on the server.

Use minimal mode when you do not want RustPanel to install Nginx/certbot:

```bash
curl -fsSL https://raw.githubusercontent.com/happydigua/RustPanel/main/scripts/bootstrap-linux.sh | sudo bash -s -- --minimal
```

Use local-only mode when you want SSH tunnel access instead of direct server-IP
access:

```bash
curl -fsSL https://raw.githubusercontent.com/happydigua/RustPanel/main/scripts/bootstrap-linux.sh | sudo bash -s -- --local
```

Then connect from your computer:

```bash
ssh -L PORT:127.0.0.1:PORT root@SERVER_IP
```

And open:

```text
http://127.0.0.1:PORT/rp-a13f9c2d8e4b7a90
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
rustpanel version
rustpanel update-check
sudo rustpanel update
rustpanel render-sample
rustpanel-helper contract
```
