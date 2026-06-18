#!/bin/bash
set -euo pipefail

BIN_DIR="${HOME}/.local/bin"
SYSTEMD_DIR="${HOME}/.config/systemd/user"
AUTOSTART_DIR="${HOME}/.config/autostart"

confirm() {
    echo "this will remove:"
    echo "  - ${BIN_DIR}/deskflow-auto-allow"
    echo "  - ${SYSTEMD_DIR}/deskflow-auto-allow.service"
    echo "  - ${AUTOSTART_DIR}/deskflow-auto-allow.desktop"
    echo "  - PATH additions in shell rc files"
    echo "  - kdotool (if installed locally)"
    echo ""
    echo "note: rust/cargo and the project source are NOT removed."
    echo ""
    read -rp "continue? [y/N] " reply
    case "$reply" in [yY]*) ;; *) echo "aborted"; exit 1 ;; esac
}

remove_binary() {
    if [ -f "${BIN_DIR}/deskflow-auto-allow" ]; then
        rm -f "${BIN_DIR}/deskflow-auto-allow"
        echo "removed: ${BIN_DIR}/deskflow-auto-allow"
    fi
}

remove_systemd_service() {
    if systemctl --user --quiet is-enabled deskflow-auto-allow 2>/dev/null; then
        systemctl --user disable --now deskflow-auto-allow 2>/dev/null || true
        echo "disabled systemd service"
    fi
    if [ -f "${SYSTEMD_DIR}/deskflow-auto-allow.service" ]; then
        rm -f "${SYSTEMD_DIR}/deskflow-auto-allow.service"
        systemctl --user daemon-reload
        echo "removed: ${SYSTEMD_DIR}/deskflow-auto-allow.service"
    fi
}

remove_desktop_autostart() {
    if [ -f "${AUTOSTART_DIR}/deskflow-auto-allow.desktop" ]; then
        rm -f "${AUTOSTART_DIR}/deskflow-auto-allow.desktop"
        echo "removed: ${AUTOSTART_DIR}/deskflow-auto-allow.desktop"
    fi
}

remove_path_additions() {
    local cleaned=0
    for rc in "${HOME}/.bashrc" "${HOME}/.zshrc" "${HOME}/.config/fish/config.fish"; do
        [ -f "$rc" ] || continue
        if grep -q "${BIN_DIR}" "$rc" 2>/dev/null; then
            sed -i "\|export PATH=\"\$PATH:${BIN_DIR}\"|d" "$rc" 2>/dev/null || true
            echo "cleaned PATH from $rc"
            cleaned=1
        fi
    done
    [ "$cleaned" -eq 0 ] && echo "no PATH additions found"
}

remove_deps() {
    if [ -f "${BIN_DIR}/kdotool" ] && [ ! -f "/usr/bin/kdotool" ]; then
        rm -f "${BIN_DIR}/kdotool"
        echo "removed: ${BIN_DIR}/kdotool"
    fi
    # clean up any leftover ydotool files from older versions
    rm -f "${BIN_DIR}/ydotool" "${BIN_DIR}/ydotoold" 2>/dev/null || true
}

cleanup() {
    echo "==> removing deskflow-auto-allow..."
    remove_systemd_service
    remove_desktop_autostart
    remove_binary
    remove_deps
    remove_path_additions
    echo ""
    echo "done. rust/cargo and project source are untouched."
}

case "${1:-}" in
    -y|--yes) cleanup ;;
    *) confirm; cleanup ;;
esac
