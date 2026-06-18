# deskflow-auto-allow

Automatically accepts the [deskflow](https://github.com/deskflow/deskflow) input capture dialog. When a remote client requests input control, deskflow shows a prompt — this tool finds it via `kdotool` and presses Enter via `xdotool` to accept.

## Why

Deskflow (and its predecessors Barrier/Synergy) requires manually clicking "Accept" on an input capture dialog whenever a remote client connects. If you frequently switch between machines, this gets tedious fast. This project automates that click so the experience is seamless.

## How it works

1. **Searches** for the deskflow dialog using `kdotool search --title "Input Capture"` (and `"New Client"`)
2. **Activates** the dialog window with `kdotool windowactivate` — this brings the dialog to the foreground
3. **Waits 1.5 seconds** for KWin to fully focus the window
4. **Presses Enter** via `xdotool key Return` — sends the keystroke to accept the dialog
5. **Exits** (or keeps watching if `--loop` is used)

### Why kdotool + xdotool instead of ydotool

- **kdotool** interacts with KWin's window manager to find and activate KDE dialogs. Standard X11 tools like `xdotool` cannot see dialogs managed internally by KWin (they return KWin UUIDs, not X11 window IDs).
- **xdotool** sends keystrokes at the X11 level. After `kdotool windowactivate` gives the dialog focus, `xdotool key Return` presses the default Allow button.
- **ydotool** (the original approach) required a background daemon (`ydotoold`) that grabs `/dev/uinput` and evdev devices, which can conflict with Steam's controller/input handling and cause X session issues. `xdotool` has no daemon and no such conflicts.

## Requirements

- **KDE Plasma** — uses `kdotool` (KWin's window manager tool) to find and interact with the deskflow dialog. Other desktop environments are not supported.

## Dependencies

- **kdotool** — KDE window management tool (finds and activates windows)
- **xdotool** — X11 input simulation (sends keystrokes)
- **Rust/Cargo** — only needed to build from source

The install script attempts to install these via your package manager (Arch, Fedora, Debian/Ubuntu, openSUSE, NixOS, Void, Alpine).

## Install

```sh
git clone https://github.com/YOUR_USER/deskflow-auto-allow.git
cd deskflow-auto-allow
./install.sh
```

This will:
- Build the Rust binary
- Copy it to `~/.local/bin/`
- Install dependencies (kdotool, xdotool)
- Install a desktop autostart entry in `~/.config/autostart/` so it runs on login
- Add `~/.local/bin` to your PATH in your shell rc file

After rebooting (or logging out and back in), it will run automatically.

### Using systemd (not recommended)

```sh
./install.sh enable-service
```

The systemd service can cause issues because it runs before the graphical session is fully ready and may lack the `DISPLAY` environment variable. Desktop autostart is the preferred method.

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

Debug mode (see what the tool sees):

```sh
deskflow-auto-allow --debug
```

## Troubleshooting

### Steam "Unable to open a connection to X" error

This was caused by the `ydotoold` background daemon from the original version of this tool. If you installed an earlier version that used ydotool, make sure `ydotoold` is not running:

```sh
systemctl --user disable --now ydotoold ydotool 2>/dev/null
```

The current version uses `xdotool` which has no background daemon and does not conflict with Steam.

### Dialog not accepted automatically

If the autostart doesn't work, check that the desktop file is in place:

```sh
ls -la ~/.config/autostart/deskflow-auto-allow.desktop
cat ~/.config/autostart/deskflow-auto-allow.desktop
```

The `Exec` line should point to the full path of the binary. Run it manually with `--debug` to see what's happening:

```sh
deskflow-auto-allow --debug
```

## Uninstall

```sh
./uninstall.sh
```

Removes the binary, desktop autostart entry, PATH additions, and any locally installed dependencies (kdotool).
