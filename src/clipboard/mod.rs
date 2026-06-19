pub mod monitor;
pub mod paste;
pub mod wayland;
pub mod x11;

use anyhow::Result;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Session {
    X11,
    Wayland,
}

pub fn session() -> Session {
    if std::env::var_os("WAYLAND_DISPLAY").is_some() {
        Session::Wayland
    } else {
        Session::X11
    }
}

pub fn read_text() -> Result<String> {
    match session() {
        Session::Wayland => wayland::read_text(),
        Session::X11 => x11::read_text(),
    }
}

pub fn set_text(text: &str) -> Result<()> {
    match session() {
        Session::Wayland => wayland::set_text(text),
        Session::X11 => x11::set_text(text),
    }
}
