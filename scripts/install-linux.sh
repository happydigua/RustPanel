#!/usr/bin/env bash
set -euo pipefail

WITH_NGINX=0
PUBLIC_ACCESS=0
RUSTPANEL_BIND="${RUSTPANEL_BIND:-}"
RUSTPANEL_BASE_PATH="${RUSTPANEL_BASE_PATH:-}"

for arg in "$@"; do
    case "$arg" in
        --with-nginx)
            WITH_NGINX=1
            ;;
        --public)
            PUBLIC_ACCESS=1
            ;;
        --local)
            PUBLIC_ACCESS=0
            ;;
        --help|-h)
            echo "Usage: sudo scripts/install-linux.sh [--with-nginx] [--public|--local]"
            exit 0
            ;;
        *)
            echo "unknown argument: $arg" >&2
            exit 2
            ;;
    esac
done

if [ -z "$RUSTPANEL_BIND" ]; then
    if [ "$PUBLIC_ACCESS" -eq 1 ]; then
        RUSTPANEL_BIND="0.0.0.0:7654"
    else
        RUSTPANEL_BIND="127.0.0.1:7654"
    fi
fi

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

detect_access_host() {
    if [ -n "${RUSTPANEL_PUBLIC_HOST:-}" ]; then
        echo "$RUSTPANEL_PUBLIC_HOST"
        return
    fi

    if command -v curl >/dev/null 2>&1; then
        public_ip="$(curl -fsS --max-time 4 https://api.ipify.org 2>/dev/null || true)"
        if [ -n "$public_ip" ]; then
            echo "$public_ip"
            return
        fi
    fi

    if command -v ip >/dev/null 2>&1; then
        route_ip="$(ip -4 route get 1.1.1.1 2>/dev/null | awk '{ for (i = 1; i <= NF; i++) if ($i == "src") { print $(i + 1); exit } }')"
        if [ -n "$route_ip" ]; then
            echo "$route_ip"
            return
        fi
    fi

    if command -v hostname >/dev/null 2>&1; then
        host_ip="$(hostname -I 2>/dev/null | awk '{ print $1 }')"
        if [ -n "$host_ip" ]; then
            echo "$host_ip"
            return
        fi
    fi

    echo "SERVER_IP"
}

generate_base_path() {
    if command -v openssl >/dev/null 2>&1; then
        token="$(openssl rand -hex 8)"
        echo "/rp-${token}"
        return
    fi

    if command -v od >/dev/null 2>&1; then
        token="$(od -An -N8 -tx1 /dev/urandom | tr -d ' \n')"
        echo "/rp-${token}"
        return
    fi

    echo "/rp-$(date +%s%N)"
}

normalize_base_path() {
    path="$1"

    if [ -z "$path" ] || [ "$path" = "/" ]; then
        echo "/"
        return
    fi

    path="/${path#/}"
    path="${path%/}"

    case "$path" in
        *[!abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789/_-]*)
            echo "RUSTPANEL_BASE_PATH may only contain letters, numbers, slash, hyphen, and underscore" >&2
            exit 1
            ;;
    esac

    echo "$path"
}

load_or_create_base_path() {
    if [ -n "$RUSTPANEL_BASE_PATH" ]; then
        normalize_base_path "$RUSTPANEL_BASE_PATH"
        return
    fi

    if [ -f /etc/rustpanel/rustpanel.env ]; then
        existing_path="$(awk -F= '/^RUSTPANEL_BASE_PATH=/{ print $2; exit }' /etc/rustpanel/rustpanel.env)"
        if [ -n "$existing_path" ]; then
            normalize_base_path "$existing_path"
            return
        fi
    fi

    generate_base_path
}

print_access_info() {
    bind_host="${RUSTPANEL_BIND%:*}"
    bind_port="${RUSTPANEL_BIND##*:}"

    echo
    echo "RustPanel installed."

    case "$bind_host" in
        0.0.0.0|::)
            access_host="$(detect_access_host)"
            echo "Access URL: http://${access_host}:${bind_port}${RUSTPANEL_BASE_PATH}"
            echo "Bind address: ${RUSTPANEL_BIND}"
            echo "Access path: ${RUSTPANEL_BASE_PATH}"
            echo "If it does not open, allow TCP ${bind_port} in the cloud firewall/security group."
            echo "Current public mode is for early debugging; add auth before leaving it exposed."
            ;;
        127.0.0.1|localhost)
            echo "Access URL on the server: http://${RUSTPANEL_BIND}${RUSTPANEL_BASE_PATH}"
            echo "Access path: ${RUSTPANEL_BASE_PATH}"
            echo "From your computer:"
            echo "  ssh -L ${bind_port}:127.0.0.1:${bind_port} root@SERVER_IP"
            echo "Then open:"
            echo "  http://127.0.0.1:${bind_port}${RUSTPANEL_BASE_PATH}"
            ;;
        *)
            echo "Access URL: http://${RUSTPANEL_BIND}${RUSTPANEL_BASE_PATH}"
            ;;
    esac
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

RUSTPANEL_BASE_PATH="$(load_or_create_base_path)"

cat >/etc/rustpanel/rustpanel.env <<ENV
RUSTPANEL_BIND=${RUSTPANEL_BIND}
RUSTPANEL_BASE_PATH=${RUSTPANEL_BASE_PATH}
ENV
chown root:rustpanel /etc/rustpanel/rustpanel.env
chmod 0640 /etc/rustpanel/rustpanel.env

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
EnvironmentFile=/etc/rustpanel/rustpanel.env
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

print_access_info
