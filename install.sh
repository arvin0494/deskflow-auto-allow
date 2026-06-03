#!/bin/bash
set -euo pipefail

KDOTOOL_URL="https://github.com/jinliu/kdotool/releases/download/v0.2.3/kdotool-0.2.3-x86_64-unknown-linux-gnu.tar.gz"
BIN_DIR="${HOME}/.local/bin"
CARGO_BIN="${HOME}/.cargo/bin"

detect_distro() {
    for f in /etc/os-release /usr/lib/os-release; do
        if [ -f "$f" ]; then
            . "$f"
            echo "${ID_LIKE:-$ID}" | tr '[:upper:]' '[:lower:]'
            return
        fi
    done
    echo "unknown"
}

install_kdotool() {
    if command -v kdotool &>/dev/null; then
        echo "kdotool already installed"
        return
    fi
    if [ -x "${BIN_DIR}/kdotool" ]; then
        echo "kdotool already installed at ${BIN_DIR}/kdotool"
        return
    fi

    echo "installing kdotool..."
    mkdir -p "$BIN_DIR"

    local tmp
    tmp="$(mktemp -d)"
    curl -fsSL -o "$tmp/kdotool.tar.gz" "$KDOTOOL_URL"
    tar xzf "$tmp/kdotool.tar.gz" -C "$tmp"

    local kdotool_bin
    kdotool_bin="$(find "$tmp" -name kdotool -type f 2>/dev/null | head -1)"
    if [ -z "$kdotool_bin" ]; then
        echo "error: kdotool binary not found in archive" >&2
        rm -rf "$tmp"
        exit 1
    fi

    cp "$kdotool_bin" "${BIN_DIR}/kdotool"
    chmod +x "${BIN_DIR}/kdotool"
    rm -rf "$tmp"
    echo "installed kdotool to ${BIN_DIR}/kdotool"
}

install_ydotool() {
    if command -v ydotool &>/dev/null; then
        echo "ydotool already installed"
        return
    fi

    echo "installing ydotool..."
    local distro
    distro="$(detect_distro)"

    case "$distro" in
        *arch*|*manjaro*|*cachyos*|*endeavouros*)
            sudo pacman -S --noconfirm ydotool
            ;;
        *fedora*)
            sudo dnf install -y ydotool
            ;;
        *debian*|*ubuntu*|*pop*|*mint*)
            sudo apt install -y ydotool
            ;;
        *opensuse*|*suse*)
            sudo zypper install -y ydotool
            ;;
        *nixos*)
            nix-env -iA nixos.ydotool
            ;;
        *void*)
            sudo xbps-install -y ydotool
            ;;
        *alpine*)
            sudo apk add ydotool
            ;;
        *)
            echo "unsupported distro '$distro'. install ydotool manually:"
            echo "  https://github.com/ReimuNotMoe/ydotool"
            exit 1
            ;;
    esac
}

install_rust_app() {
    if ! command -v cargo &>/dev/null; then
        echo "cargo not found. install rust first: https://rustup.rs"
        exit 1
    fi

    echo "building deskflow-auto-allow..."
    cargo build --release
    mkdir -p "$BIN_DIR"
    cp target/release/deskflow-auto-allow "${BIN_DIR}/deskflow-auto-allow"
    echo "installed deskflow-auto-allow to ${BIN_DIR}/deskflow-auto-allow"
}

add_to_path() {
    local added=0
    for dir in "$BIN_DIR" "$CARGO_BIN"; do
        if [[ ":$PATH:" != *":$dir:"* ]]; then
            local rc
            rc="$(echo "$SHELL" | grep -oP '[^/]+$')"
            rc="${HOME}/.${rc}rc"
            echo "export PATH=\"\$PATH:$dir\"" >> "$rc"
            echo "added $dir to PATH in $rc"
            added=1
        fi
    done
    if [ "$added" -eq 1 ]; then
        echo "restart your shell or run: source ~/.${SHELL##*/}rc"
    fi
}

main() {
    install_kdotool
    install_ydotool
    install_rust_app
    add_to_path
    echo "done. run: deskflow-auto-allow  (add --loop to keep watching)"
    echo ""
    echo "to start on every boot:"
    echo "  mkdir -p ~/.config/systemd/user"
    echo "  cp deskflow-auto-allow.service ~/.config/systemd/user/"
    echo "  systemctl --user enable --now deskflow-auto-allow"
}

main
