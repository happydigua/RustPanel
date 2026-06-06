#!/usr/bin/env bash
set -euo pipefail

REPO_URL="${RUSTPANEL_REPO_URL:-https://github.com/happydigua/RustPanel.git}"
BRANCH="${RUSTPANEL_BRANCH:-main}"
INSTALL_DIR="${RUSTPANEL_SOURCE_DIR:-/opt/rustpanel-src}"
WITH_NGINX=1
PUBLIC_ACCESS=1
RUSTPANEL_VERSION="${RUSTPANEL_VERSION:-latest}"

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
            echo "Usage: curl -fsSL https://raw.githubusercontent.com/happydigua/RustPanel/main/scripts/bootstrap-linux.sh | sudo bash"
            echo "       curl -fsSL https://raw.githubusercontent.com/happydigua/RustPanel/main/scripts/bootstrap-linux.sh | sudo bash -s -- --minimal"
            echo "       curl -fsSL https://raw.githubusercontent.com/happydigua/RustPanel/main/scripts/bootstrap-linux.sh | sudo bash -s -- --local"
            echo "       curl -fsSL https://raw.githubusercontent.com/happydigua/RustPanel/main/scripts/bootstrap-linux.sh | sudo bash -s -- --version=v0.1.0"
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
        export DEBIAN_FRONTEND=noninteractive
        apt-get update
        apt-get install -y git curl ca-certificates tar gzip
        return
    fi

    if command -v dnf >/dev/null 2>&1; then
        dnf install -y git curl ca-certificates tar gzip
        return
    fi

    echo "unsupported package manager; install git, curl, ca-certificates, tar, and gzip manually" >&2
    exit 1
}

sync_source() {
    mkdir -p "$(dirname "$INSTALL_DIR")"

    if [ -d "$INSTALL_DIR/.git" ]; then
        git -C "$INSTALL_DIR" fetch origin "$BRANCH"
        git -C "$INSTALL_DIR" checkout "$BRANCH"
        git -C "$INSTALL_DIR" pull --ff-only origin "$BRANCH"
        return
    fi

    git clone --branch "$BRANCH" "$REPO_URL" "$INSTALL_DIR"
}

install_runtime_dependencies
sync_source

cd "$INSTALL_DIR"

install_args=("--version=${RUSTPANEL_VERSION}")

if [ "$WITH_NGINX" -eq 1 ]; then
    install_args+=(--with-nginx)
fi

if [ "$PUBLIC_ACCESS" -eq 1 ]; then
    install_args+=(--public)
else
    install_args+=(--local)
fi

scripts/install-binary-linux.sh "${install_args[@]}"
