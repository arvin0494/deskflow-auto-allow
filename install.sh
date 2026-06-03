#!/bin/bash
set -euo pipefail

BIN_DIR="${HOME}/.local/bin"
SYSTEMD_DIR="${HOME}/.config/systemd/user"

main() {
    echo "==> checking dependencies..."
    if ! command -v cargo &>/dev/null; then
        echo "cargo not found. install rust first: https://rustup.rs"
        exit 1
    fi

    echo "==> building deskflow-auto-allow..."
    cargo build --release --manifest-path "$(dirname "$0")/Cargo.toml"
    mkdir -p "$BIN_DIR"
    cp "$(dirname "$0")/target/release/deskflow-auto-allow" "${BIN_DIR}/deskflow-auto-allow"

    echo "==> installing dependencies..."
    "${BIN_DIR}/deskflow-auto-allow" --install-deps || true

    # add to PATH
    if [[ ":$PATH:" != *":$BIN_DIR:"* ]]; then
        rc="${HOME}/.${SHELL##*/}rc"
        echo "export PATH=\"\$PATH:$BIN_DIR\"" >> "$rc"
        echo "added $BIN_DIR to PATH in $rc"
    fi

    echo ""
    echo "done. run: deskflow-auto-allow"
    echo ""
    echo "autostart (install systemd service):"
    echo "  mkdir -p ~/.config/systemd/user"
    echo "  cp $(dirname "$0")/deskflow-auto-allow.service ~/.config/systemd/user/"
    echo "  systemctl --user enable --now deskflow-auto-allow"
}

install_service() {
    mkdir -p "$SYSTEMD_DIR"
    cp "$(dirname "$0")/deskflow-auto-allow.service" "$SYSTEMD_DIR/"
    systemctl --user daemon-reload
    echo "installed systemd service"
    echo "enable:  systemctl --user enable --now deskflow-auto-allow"
}

case "${1:-}" in
    install_service) install_service ;;
    *) main ;;
esac
