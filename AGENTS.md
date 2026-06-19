# AGENTS.md

## Project

This project is `linux-clipboard-history`.

It is a Linux desktop clipboard history manager inspired by the Windows 11 clipboard manager.

The app should:

- Run in the background.
- Monitor copied clipboard content.
- Save clipboard history persistently.
- Open a clipboard history popup with `Super+V`.
- Allow users to select old clipboard items and paste them again.
- Support pinning, deleting, searching, and clearing history.
- Work on common Linux desktops.
- Support X11 and Wayland where possible.

The first version should focus on a reliable plain-text clipboard manager before adding advanced content types.

---

## Main Goal

Build a Linux app that behaves as closely as practical to the Windows 11 clipboard manager.

Expected behaviour:

1. User copies text with `Ctrl+C` or cuts text with `Ctrl+X`.
2. App detects the clipboard change.
3. App saves the copied text to SQLite history.
4. User presses `Super+V`.
5. Clipboard history popup opens.
6. User selects an old item.
7. App sets that item as the active clipboard content.
8. On X11, app auto-pastes using `Ctrl+V`.
9. On Wayland, app uses the safest available behaviour and falls back to “copied to clipboard, press Ctrl+V”.

---

## Tech Stack

Use:

- Rust
- GTK4
- libadwaita where useful
- SQLite
- TOML config
- `wl-clipboard` for Wayland clipboard helpers where needed
- `xclip` or `xsel` for X11 clipboard helpers where needed
- `xdotool` for X11 paste simulation

Prefer native Rust crates where stable and practical, but external helper commands are acceptable for clipboard and paste operations.

Do not over-engineer the first version.

---

## Supported Platforms

Target common Linux desktop environments:

- Ubuntu
- Debian
- Linux Mint
- Fedora
- Arch
- GNOME
- KDE Plasma
- XFCE

Support both:

- X11
- Wayland

Wayland has desktop security restrictions. Do not pretend Wayland can always support automatic paste. Implement graceful fallback behaviour.

---

## Core Features

### Clipboard Monitoring

The app must monitor clipboard changes and store history.

Minimum supported type:

- Plain text

Future optional types:

- HTML
- Images
- File paths / copied files

Rules:

- Do not store empty clipboard values.
- Do not store duplicate consecutive clipboard values.
- Do not store text larger than 1 MB by default.
- Store a content hash for each entry.
- Ignore content matching configured ignore rules.
- Keep pinned items permanently unless the user manually deletes them.
- Prune old unpinned entries when max history size is exceeded.

---

## Keyboard Shortcuts

Main shortcut:

```text
Super+V
