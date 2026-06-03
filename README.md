# deskflow-auto-allow

Automatically accepts the deskflow input capture dialog. When a remote client requests input control, deskflow shows a dialog — this tool finds it via `kdotool` and presses Enter via `ydotool` to accept.

## How it works

1. Runs `kdotool search --title "Input Capture"` (and `"New Client"`) to find the deskflow dialog
2. Activates the dialog window
3. Simulates two Enter key presses via `ydotool` to accept
4. Exits, or continues watching if `--loop` is used

## Dependencies

- **kdotool** — KDE window management tool (for finding/activating windows)
- **ydotool** — generic Linux input simulation tool (for pressing keys)
- **Rust/Cargo** — only needed to build from source

The install script attempts to install these via your package manager (Arch, Fedora, Debian/Ubuntu, openSUSE, NixOS, Void, Alpine) or downloads prebuilt binaries from GitHub.

## Install

```sh
git clone https://github.com/YOUR_USER/deskflow-auto-allow.git
cd deskflow-auto-allow
./install.sh
```

This will:
- Build the Rust binary
- Copy it to `~/.local/bin/`
- Install dependencies (kdotool, ydotool)
- Install and enable a systemd user service
- Add `~/.local/bin` to your PATH in your shell rc file

Start the service immediately:

```sh
systemctl --user start deskflow-auto-allow
```

### Without systemd

```sh
./install.sh enable-autostart
```

This installs a desktop autostart file instead of a systemd service.

## Usage

```sh
deskflow-auto-allow [OPTIONS]
```

| Option | Description |
|--------|-------------|
| `--loop` | Keep watching after accepting a dialog (for multiple connections) |
| `--debug` | Verbose output |
| `--install-deps` | Install dependencies and exit |
| `--timeout SECS` | Exit after SECS seconds if no dialog is found |
| `--help` | Show help |

### Examples

Run once (exit after accepting the dialog):

```sh
deskflow-auto-allow
```

Keep watching for new dialogs:

```sh
deskflow-auto-allow --loop
```

Exit if no dialog appears within 30 seconds:

```sh
deskflow-auto-allow --timeout 30
```

## Uninstall

```sh
./uninstall.sh
```

Removes the binary, systemd service, desktop autostart entry, PATH additions, and any dependencies that were installed locally (kdotool, ydotool).
