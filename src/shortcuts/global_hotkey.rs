use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use x11rb::{
    connection::Connection,
    protocol::{
        xproto::{ConnectionExt, GrabMode, ModMask},
        Event,
    },
};

pub fn start(open_requested: Arc<AtomicBool>, shortcut: String, fallback: String) {
    if std::env::var_os("WAYLAND_DISPLAY").is_some() {
        return;
    }
    std::thread::spawn(move || {
        if let Err(error) = listen(open_requested, &[shortcut, fallback]) {
            eprintln!("global shortcut unavailable: {error}");
        }
    });
}

fn listen(
    open_requested: Arc<AtomicBool>,
    shortcuts: &[String],
) -> Result<(), Box<dyn std::error::Error>> {
    let (connection, screen_number) = x11rb::connect(None)?;
    let screen = &connection.setup().roots[screen_number];
    let mapping = connection
        .get_keyboard_mapping(
            connection.setup().min_keycode,
            connection.setup().max_keycode - connection.setup().min_keycode + 1,
        )?
        .reply()?;
    for shortcut in shortcuts {
        let (modifiers, keysym) = parse(shortcut).ok_or("unsupported shortcut syntax")?;
        let keycode = mapping
            .keysyms
            .chunks(mapping.keysyms_per_keycode as usize)
            .position(|syms| syms.iter().any(|value| *value == keysym))
            .map(|index| connection.setup().min_keycode + index as u8)
            .ok_or("shortcut key not found")?;
        connection
            .grab_key(
                false,
                screen.root,
                modifiers,
                keycode,
                GrabMode::ASYNC,
                GrabMode::ASYNC,
            )?
            .check()?;
    }
    connection.flush()?;
    loop {
        if let Event::KeyPress(_) = connection.wait_for_event()? {
            open_requested.store(true, Ordering::Release);
        }
    }
}

fn parse(value: &str) -> Option<(ModMask, u32)> {
    let parts: Vec<_> = value.split('+').map(|part| part.trim()).collect();
    let key = parts.last()?.chars().next()?;
    if parts.last()?.chars().count() != 1 || !key.is_ascii_alphanumeric() {
        return None;
    }
    let mut modifiers = ModMask::default();
    for part in &parts[..parts.len() - 1] {
        modifiers |= match part.to_ascii_lowercase().as_str() {
            "super" | "meta" => ModMask::M4,
            "ctrl" | "control" => ModMask::CONTROL,
            "alt" => ModMask::M1,
            "shift" => ModMask::SHIFT,
            _ => return None,
        };
    }
    Some((modifiers, key.to_ascii_lowercase() as u32))
}
