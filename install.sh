#!/bin/bash
set -euo pipefail

BIN_DIR="${HOME}/.local/bin"
SYSTEMD_DIR="${HOME}/.config/systemd/user"
AUTOSTART_DIR="${HOME}/.config/autostart"

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
    echo ""
    enable_service
    echo ""
    echo "done. deskflow-auto-allow will start on next boot."
    echo "to start now: systemctl --user start deskflow-auto-allow"
}

enable_service() {
    # enable ydotool daemon first
    systemctl --user daemon-reload
    if systemctl --user enable --now ydotool 2>/dev/null; then
        echo "enabled ydotoold (systemd)"
    else
        echo "warning: could not enable ydotool service"
    fi

    # install and enable our service
    mkdir -p "$SYSTEMD_DIR"
    cp "$(dirname "$0")/deskflow-auto-allow.service" "$SYSTEMD_DIR/"
    systemctl --user daemon-reload
    systemctl --user enable --now deskflow-auto-allow
    echo "enabled deskflow-auto-allow service"
    echo ""
    echo "status: systemctl --user status deskflow-auto-allow"
    echo "logs:   journalctl --user -u deskflow-auto-allow -f"
}

enable_autostart() {
    # enable ydotool daemon if systemd available
    if systemctl --user enable --now ydotool 2>/dev/null; then
        echo "enabled ydotoold (systemd)"
    else
        echo "warning: ydotool daemon not autostarted"
    fi

    # install desktop autostart entry
    mkdir -p "$AUTOSTART_DIR"
    cp "$(dirname "$0")/deskflow-auto-allow.desktop" "$AUTOSTART_DIR/"
    chmod +x "${AUTOSTART_DIR}/deskflow-auto-allow.desktop"
    echo "enabled desktop autostart entry"
}

case "${1:-}" in
    enable-service)  enable_service ;;
    enable-autostart) enable_autostart ;;
    *) main ;;
esac
