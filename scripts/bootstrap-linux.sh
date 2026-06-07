#!/usr/bin/env bash
set -euo pipefail

BRANCH="${RUSTPANEL_BRANCH:-main}"
INSTALL_SCRIPT_URL="${RUSTPANEL_INSTALL_SCRIPT_URL:-https://raw.githubusercontent.com/happydigua/RustPanel/${BRANCH}/scripts/install-binary-linux.sh}"
WITH_NGINX=1
PUBLIC_ACCESS=1
RUSTPANEL_VERSION="${RUSTPANEL_VERSION:-latest}"

log() {
    printf '[RustPanel] %s\n' "$*"
}

for arg in "$@"; do
    case "$arg" in
        --minimal)
            WITH_NGINX=0
            ;;
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
            echo "Usage: curl -fL --retry 5 --retry-all-errors --retry-delay 2 --connect-timeout 30 --max-time 300 https://raw.githubusercontent.com/happydigua/RustPanel/main/scripts/bootstrap-linux.sh -o /tmp/rustpanel-bootstrap-linux.sh && sudo bash /tmp/rustpanel-bootstrap-linux.sh"
            echo "       curl -fL --retry 5 --retry-all-errors --retry-delay 2 --connect-timeout 30 --max-time 300 https://raw.githubusercontent.com/happydigua/RustPanel/main/scripts/bootstrap-linux.sh -o /tmp/rustpanel-bootstrap-linux.sh && sudo bash /tmp/rustpanel-bootstrap-linux.sh --minimal"
            echo "       curl -fL --retry 5 --retry-all-errors --retry-delay 2 --connect-timeout 30 --max-time 300 https://raw.githubusercontent.com/happydigua/RustPanel/main/scripts/bootstrap-linux.sh -o /tmp/rustpanel-bootstrap-linux.sh && sudo bash /tmp/rustpanel-bootstrap-linux.sh --local"
            echo "       curl -fL --retry 5 --retry-all-errors --retry-delay 2 --connect-timeout 30 --max-time 300 https://raw.githubusercontent.com/happydigua/RustPanel/main/scripts/bootstrap-linux.sh -o /tmp/rustpanel-bootstrap-linux.sh && sudo bash /tmp/rustpanel-bootstrap-linux.sh --version=v0.1.7"
            exit 0
            ;;
        *)
            echo "unknown argument: $arg" >&2
            exit 2
            ;;
    esac
done

if [ "$(id -u)" -ne 0 ]; then
    echo "run this bootstrapper with sudo" >&2
    exit 1
fi

install_runtime_dependencies() {
    if command -v apt-get >/dev/null 2>&1; then
        log "Installing runtime dependencies with apt"
        export DEBIAN_FRONTEND=noninteractive
        apt-get update
        apt-get install -y curl ca-certificates tar gzip
        log "Runtime dependencies are ready"
        return
    fi

    if command -v dnf >/dev/null 2>&1; then
        log "Installing runtime dependencies with dnf"
        dnf install -y curl ca-certificates tar gzip
        log "Runtime dependencies are ready"
        return
    fi

    echo "unsupported package manager; install curl, ca-certificates, tar, and gzip manually" >&2
    exit 1
}

download_url() {
    url="$1"
    destination="$2"

    curl -fL \
        --retry 5 \
        --retry-all-errors \
        --retry-delay 2 \
        --connect-timeout 30 \
        --max-time 300 \
        "$url" \
        -o "$destination"
}

installer=""

download_installer() {
    installer="$(mktemp /tmp/rustpanel-install-binary.XXXXXX.sh)"
    log "Downloading binary installer"
    if ! download_url "$INSTALL_SCRIPT_URL" "$installer"; then
        rm -f "$installer"
        echo "failed to download RustPanel binary installer" >&2
        exit 1
    fi
    chmod 0755 "$installer"
}

log "Bootstrap started"
install_runtime_dependencies
download_installer
trap 'rm -f "$installer"' EXIT

install_args=("--version=${RUSTPANEL_VERSION}")

if [ "$WITH_NGINX" -eq 1 ]; then
    install_args+=(--with-nginx)
fi

if [ "$PUBLIC_ACCESS" -eq 1 ]; then
    install_args+=(--public)
else
    install_args+=(--local)
fi

log "Installing RustPanel binary release ${RUSTPANEL_VERSION}"
bash "$installer" "${install_args[@]}"
