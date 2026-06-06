#!/usr/bin/env bash
set -euo pipefail

WITH_NGINX=0
PUBLIC_ACCESS=0
RUSTPANEL_BIND="${RUSTPANEL_BIND:-}"
RUSTPANEL_BASE_PATH="${RUSTPANEL_BASE_PATH:-}"
RUSTPANEL_VERSION="${RUSTPANEL_VERSION:-latest}"
RUSTPANEL_REPO_SLUG="${RUSTPANEL_REPO_SLUG:-happydigua/RustPanel}"

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
        --version=*)
            RUSTPANEL_VERSION="${arg#*=}"
            ;;
        --help|-h)
            echo "Usage: sudo scripts/install-binary-linux.sh [--with-nginx] [--public|--local] [--version=v0.1.0|latest]"
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

install_nginx() {
    if command -v nginx >/dev/null 2>&1; then
        return
    fi

    if command -v apt-get >/dev/null 2>&1; then
        export DEBIAN_FRONTEND=noninteractive
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

target_triple() {
    arch="$(uname -m)"

    case "$arch" in
        x86_64|amd64)
            echo "x86_64-unknown-linux-gnu"
            ;;
        aarch64|arm64)
            echo "aarch64-unknown-linux-gnu"
            ;;
        *)
            echo "unsupported CPU architecture: ${arch}" >&2
            exit 1
            ;;
    esac
}

release_asset_url() {
    target="$1"
    asset="rustpanel-${target}.tar.gz"

    if [ "$RUSTPANEL_VERSION" = "latest" ]; then
        echo "https://github.com/${RUSTPANEL_REPO_SLUG}/releases/latest/download/${asset}"
    else
        echo "https://github.com/${RUSTPANEL_REPO_SLUG}/releases/download/${RUSTPANEL_VERSION}/${asset}"
    fi
}

download_and_install_binaries() {
    if ! command -v curl >/dev/null 2>&1; then
        echo "curl is required" >&2
        exit 1
    fi

    if ! command -v tar >/dev/null 2>&1; then
        echo "tar is required" >&2
        exit 1
    fi

    target="$(target_triple)"
    url="$(release_asset_url "$target")"
    tmp_dir="$(mktemp -d)"
    archive="${tmp_dir}/rustpanel.tar.gz"

    cleanup() {
        rm -rf "$tmp_dir"
    }
    trap cleanup EXIT

    echo "Downloading RustPanel binary: ${url}"
    if ! curl -fL --retry 3 --connect-timeout 20 -o "$archive" "$url"; then
        echo "failed to download RustPanel binary release" >&2
        echo "check that a GitHub Release exists for target ${target}" >&2
        exit 1
    fi

    tar -xzf "$archive" -C "$tmp_dir"

    install -m 0755 "${tmp_dir}/rustpaneld" /usr/local/bin/rustpaneld
    install -m 0755 "${tmp_dir}/rustpanel" /usr/local/bin/rustpanel
    install -m 0755 "${tmp_dir}/rustpanel-helper" /usr/local/bin/rustpanel-helper
}

port_is_available() {
    port="$1"

    if command -v ss >/dev/null 2>&1; then
        if ss -ltn | awk '{ print $4 }' | grep -Eq "[:.]${port}$"; then
            return 1
        fi
        return 0
    fi

    if command -v netstat >/dev/null 2>&1; then
        if netstat -ltn | awk '{ print $4 }' | grep -Eq "[:.]${port}$"; then
            return 1
        fi
        return 0
    fi

    return 0
}

random_port_candidate() {
    if command -v shuf >/dev/null 2>&1; then
        shuf -i 20000-49999 -n 1
        return
    fi

    if command -v od >/dev/null 2>&1; then
        number="$(od -An -N2 -tu2 /dev/urandom | tr -d ' ')"
        echo $((20000 + number % 30000))
        return
    fi

    echo $((20000 + $(date +%s) % 30000))
}

generate_bind_port() {
    attempt=0

    while [ "$attempt" -lt 50 ]; do
        attempt=$((attempt + 1))
        port="$(random_port_candidate)"

        if port_is_available "$port"; then
            echo "$port"
            return
        fi
    done

    echo "could not find an available random port" >&2
    exit 1
}

load_existing_bind() {
    if [ -f /etc/rustpanel/rustpanel.env ]; then
        awk -F= '/^RUSTPANEL_BIND=/{ print $2; exit }' /etc/rustpanel/rustpanel.env
    fi
}

load_or_create_bind() {
    if [ -n "$RUSTPANEL_BIND" ]; then
        echo "$RUSTPANEL_BIND"
        return
    fi

    existing_bind="$(load_existing_bind)"
    if [ -n "$existing_bind" ]; then
        echo "$existing_bind"
        return
    fi

    port="$(generate_bind_port)"
    if [ "$PUBLIC_ACCESS" -eq 1 ]; then
        echo "0.0.0.0:${port}"
    else
        echo "127.0.0.1:${port}"
    fi
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

download_and_install_binaries

if ! id rustpanel >/dev/null 2>&1; then
    useradd --system --home-dir /var/lib/rustpanel --shell /usr/sbin/nologin rustpanel
fi

install -d -o rustpanel -g rustpanel -m 0750 /var/lib/rustpanel
install -d -o rustpanel -g rustpanel -m 0750 /var/log/rustpanel
install -d -o rustpanel -g rustpanel -m 0750 /var/lib/rustpanel/acme
install -d -o root -g rustpanel -m 0750 /etc/rustpanel
install -d -o root -g rustpanel -m 0750 /etc/rustpanel/apps

RUSTPANEL_BIND="$(load_or_create_bind)"
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
