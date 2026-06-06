#!/usr/bin/env bash
set -euo pipefail

REPO_URL="${RUSTPANEL_REPO_URL:-https://github.com/happydigua/RustPanel.git}"
BRANCH="${RUSTPANEL_BRANCH:-main}"
INSTALL_DIR="${RUSTPANEL_SOURCE_DIR:-/opt/rustpanel-src}"
WITH_NGINX=1
PUBLIC_ACCESS=1

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
        --help|-h)
            echo "Usage: curl -fsSL https://raw.githubusercontent.com/happydigua/RustPanel/main/scripts/bootstrap-linux.sh | sudo bash"
            echo "       curl -fsSL https://raw.githubusercontent.com/happydigua/RustPanel/main/scripts/bootstrap-linux.sh | sudo bash -s -- --minimal"
            echo "       curl -fsSL https://raw.githubusercontent.com/happydigua/RustPanel/main/scripts/bootstrap-linux.sh | sudo bash -s -- --local"
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

install_build_dependencies() {
    if command -v apt-get >/dev/null 2>&1; then
        export DEBIAN_FRONTEND=noninteractive
        apt-get update
        apt-get install -y build-essential pkg-config libssl-dev git curl ca-certificates
        return
    fi

    if command -v dnf >/dev/null 2>&1; then
        dnf groupinstall -y "Development Tools"
        dnf install -y git curl pkg-config openssl-devel ca-certificates
        return
    fi

    echo "unsupported package manager; install git, curl, pkg-config, OpenSSL headers, and a C toolchain manually" >&2
    exit 1
}

install_rust() {
    export CARGO_HOME="${CARGO_HOME:-/root/.cargo}"
    export RUSTUP_HOME="${RUSTUP_HOME:-/root/.rustup}"
    export PATH="${CARGO_HOME}/bin:${PATH}"

    if command -v cargo >/dev/null 2>&1; then
        return
    fi

    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \
        | sh -s -- -y --profile minimal

    export PATH="${CARGO_HOME}/bin:${PATH}"
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

install_build_dependencies
install_rust
sync_source

cd "$INSTALL_DIR"

install_args=()

if [ "$WITH_NGINX" -eq 1 ]; then
    install_args+=(--with-nginx)
fi

if [ "$PUBLIC_ACCESS" -eq 1 ]; then
    install_args+=(--public)
else
    install_args+=(--local)
fi

env "PATH=$PATH" scripts/install-linux.sh "${install_args[@]}"
