# Linux Clipboard History

`linux-clipboard-history` is a GTK4 desktop clipboard manager inspired by the
Windows 11 clipboard popup. It runs in the background, records plain-text
clipboard changes in SQLite, and lets you search, pin, delete, and restore old
items.

This first release intentionally supports plain text only. Images, HTML, copied
files, sync, and encryption are future work.

## Project status

This project is early-stage software and has known issues across different
Linux desktops, especially around Wayland clipboard access and global keyboard
shortcuts. Bug reports, testing, documentation improvements, and code
contributions would be greatly appreciated. Please use
[GitHub Issues](https://github.com/ngudger2021/linux-clipboard-history/issues)
to report reproducible problems and include your distribution, desktop
environment, and whether the session uses X11 or Wayland.

## Features

- Persistent SQLite history, newest first
- SHA-256 content identity and consecutive duplicate suppression
- Configurable size and 1 MiB default item limit
- Search, pin/unpin, delete, clear unpinned, and clear all
- X11 clipboard monitoring and automatic paste with `xclip` and `xdotool`
- Wayland clipboard monitoring with `wl-clipboard` and safe copy-only restore
- X11 `Super+V` and `Ctrl+Alt+V` global shortcuts
- Configurable settings and XDG autostart
- A GTK-free background monitor so login does not create a taskbar entry
- CLI list, clear, pause, and resume operations

## Dependencies

Ubuntu/Debian:

```sh
sudo apt update
sudo apt install -y build-essential pkg-config libgtk-4-dev sqlite3 xclip xdotool wl-clipboard
```

Fedora:

```sh
sudo dnf install cargo rust gtk4-devel sqlite xclip xdotool wl-clipboard
```

Arch:

```sh
sudo pacman -S rust gtk4 sqlite xclip xdotool wl-clipboard
```

Rust 1.75 or newer is recommended. The current UI uses GTK4 directly;
libadwaita is not required.

## Install the Debian package

Download or build the `.deb`, then install it with APT so dependencies are
installed automatically:

```sh
sudo apt install ./dist/linux-clipboard-history_0.1.0_amd64.deb
```

The package installs a native binary to `/usr/bin`; Docker is not required to
install or run the application.

To build the Debian package from source:

```sh
cargo build --release
./packaging/deb/build-deb.sh
```

## Build and install manually

```sh
cargo build --release
install -Dm755 target/release/linux-clipboard-history ~/.local/bin/linux-clipboard-history
install -Dm644 packaging/linux-clipboard-history.desktop ~/.local/share/applications/linux-clipboard-history.desktop
```

Run the background process:

```sh
linux-clipboard-history
```

Open the popup explicitly:

```sh
linux-clipboard-history show
```

The app creates:

- `~/.local/share/linux-clipboard-history/history.db`
- `~/.config/linux-clipboard-history/config.toml`
- `~/.config/autostart/linux-clipboard-history.desktop` when autostart is enabled

## Shortcuts

On X11 the app grabs `Super+V` and the fallback `Ctrl+Alt+V`. Some desktop
environments reserve `Super+V`; use the fallback when that happens.

Wayland intentionally prevents unrestricted global key grabs. In GNOME, KDE,
or another Wayland desktop, create a custom keyboard shortcut for:

```sh
linux-clipboard-history show
```

Assign `Super+V` or `Ctrl+Alt+V` in the desktop's keyboard settings. The
shortcut text fields support one letter or number with `Super`, `Ctrl`, `Alt`,
and `Shift` modifiers. Changing an X11 grab requires restarting the app.

Keyboard controls in the popup:

- Up/Down: select an item
- Enter: restore the selected item
- Escape: close
- Delete: delete selected item
- Ctrl+F: focus search

## X11 and Wayland behaviour

X11 uses `xclip` to read and write the clipboard. After restoring an item it
waits briefly and invokes `xdotool key --clearmodifiers ctrl+v`.

Wayland uses `wl-paste` and `wl-copy`. Automatic paste is not attempted because
many compositors reject synthetic keyboard input. The item is copied and the UI
reports that the user should press Ctrl+V.

## Configuration

Settings are stored as TOML. Defaults include:

```toml
enabled = true
start_on_login = true
max_history_items = 100
shortcut = "Super+V"
fallback_shortcut = "Ctrl+Alt+V"
auto_paste = true
theme = "system"
poll_interval_ms = 300
ignore_shorter_than = 1
ignore_larger_than_bytes = 1048576
clear_on_exit = false
ignored_apps = []
ignored_patterns = ["(?i)password", "(?i)secret", "(?i)token"]
```

Invalid ignored regular expressions are skipped. Application-based exclusion
is stored in configuration but is not enforced yet because determining the
clipboard owner is not portable across X11 and Wayland.

## CLI

```text
linux-clipboard-history show
linux-clipboard-history list
linux-clipboard-history clear
linux-clipboard-history clear-all
linux-clipboard-history pause
linux-clipboard-history resume
```

Pause and resume update the shared config, which the running monitor reloads.

## Autostart and systemd

Enabling “Start on login” writes the XDG autostart desktop file automatically.
As an alternative, copy `packaging/systemd-user.service` to
`~/.config/systemd/user/linux-clipboard-history.service`, adjust `ExecStart` if
needed, and run:

```sh
systemctl --user enable --now linux-clipboard-history.service
```

Do not enable both mechanisms.

## Tray support

The MVP has no hard dependency on AppIndicator. GNOME does not expose legacy
tray icons without an extension, and status icon support differs across KDE and
XFCE. The app therefore runs as a background service and exposes its actions in
the popup and CLI. Native StatusNotifier support is planned after the core
clipboard flow has broader desktop testing.

## Privacy and security

Clipboard history is sensitive. Entries are stored unencrypted in a local
SQLite database readable by your user account. Default regular expressions
exclude text containing `password`, `secret`, or `token`, but pattern matching
cannot identify every credential. Clear sensitive entries and protect your
Linux account and home directory. There is no network or synchronization code.

## Known issues

- Plain text only; HTML, image, and file payloads are not captured.
- Wayland requires a desktop-configured shortcut and manual Ctrl+V.
- Tray/status icon integration is not included in the MVP.
- Ignored application names are saved but not yet enforced.
- Clipboard helper failures are logged and monitoring continues.
- The app must run inside the graphical user session with `DISPLAY` or
  `WAYLAND_DISPLAY` set.

## Development status

Milestones 1–7 are represented in the current source: configuration, SQLite,
CLI, clipboard monitoring, GTK popup, X11 shortcut and paste, search, item
actions, settings, and autostart. Milestone 8 includes desktop/systemd packaging
and documentation; tray and formal distribution packages remain documented
follow-up work.

Run checks with:

```sh
cargo fmt --check
cargo test
```

## License

This project is open-source software licensed under the [MIT License](LICENSE).
