#!/usr/bin/env bash
set -euo pipefail

WITH_NGINX=0
RUSTPANEL_BIND="${RUSTPANEL_BIND:-127.0.0.1:7654}"

for arg in "$@"; do
    case "$arg" in
        --with-nginx)
            WITH_NGINX=1
            ;;
        --help|-h)
            echo "Usage: sudo scripts/install-linux.sh [--with-nginx]"
            exit 0
            ;;
        *)
            echo "unknown argument: $arg" >&2
            exit 2
            ;;
    esac
done

if [ "$(id -u)" -ne 0 ]; then
    echo "run this installer with sudo" >&2
    exit 1
fi

if ! command -v cargo >/dev/null 2>&1; then
    echo "cargo was not found; install Rust first with rustup" >&2
    exit 1
fi

install_nginx() {
    if command -v nginx >/dev/null 2>&1; then
        return
    fi

    if command -v apt-get >/dev/null 2>&1; then
        apt-get update
        apt-get install -y nginx certbot
        return
    fi

    if command -v dnf >/dev/null 2>&1; then
        dnf install -y nginx certbot
        return
    fi

    echo "unsupported package manager; install nginx and certbot manually" >&2
    exit 1
}

if [ "$WITH_NGINX" -eq 1 ]; then
    install_nginx
    systemctl enable --now nginx
fi

repo_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repo_dir"

cargo build --release

install -m 0755 target/release/rustpaneld /usr/local/bin/rustpaneld
install -m 0755 target/release/rustpanel /usr/local/bin/rustpanel
install -m 0755 target/release/rustpanel-helper /usr/local/bin/rustpanel-helper

if ! id rustpanel >/dev/null 2>&1; then
    useradd --system --home-dir /var/lib/rustpanel --shell /usr/sbin/nologin rustpanel
fi

install -d -o rustpanel -g rustpanel -m 0750 /var/lib/rustpanel
install -d -o rustpanel -g rustpanel -m 0750 /var/log/rustpanel
install -d -o rustpanel -g rustpanel -m 0750 /var/lib/rustpanel/acme
install -d -o root -g rustpanel -m 0750 /etc/rustpanel
install -d -o root -g rustpanel -m 0750 /etc/rustpanel/apps

if [ -d /etc/nginx/conf.d ]; then
    install -d -o root -g root -m 0755 /etc/nginx/conf.d/rustpanel
fi

cat >/etc/systemd/system/rustpaneld.service <<UNIT
[Unit]
Description=RustPanel web daemon
After=network-online.target
Wants=network-online.target

[Service]
Type=simple
User=rustpanel
Group=rustpanel
Environment=RUSTPANEL_BIND=${RUSTPANEL_BIND}
ExecStart=/usr/local/bin/rustpaneld
Restart=on-failure
RestartSec=3
NoNewPrivileges=true
PrivateTmp=true
ProtectSystem=strict
ProtectHome=true
ReadWritePaths=/var/lib/rustpanel /var/log/rustpanel /etc/rustpanel

[Install]
WantedBy=multi-user.target
UNIT

systemctl daemon-reload
systemctl enable --now rustpaneld

echo "RustPanel installed."
echo "Local access: http://${RUSTPANEL_BIND}"
echo "SSH tunnel: ssh -L 7654:127.0.0.1:7654 root@SERVER_IP"
