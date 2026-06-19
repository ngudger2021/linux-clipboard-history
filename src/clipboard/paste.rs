use super::{session, set_text, Session};
use anyhow::{bail, Context, Result};
use std::{process::Command, thread, time::Duration};

pub enum PasteResult {
    Pasted,
    ClipboardOnly,
}

pub fn restore(text: &str, auto_paste: bool) -> Result<PasteResult> {
    set_text(text)?;
    if session() != Session::X11 || !auto_paste {
        return Ok(PasteResult::ClipboardOnly);
    }
    thread::sleep(Duration::from_millis(90));
    let status = Command::new("xdotool")
        .args(["key", "--clearmodifiers", "ctrl+v"])
        .status()
        .context("xdotool is required for X11 automatic paste")?;
    if !status.success() {
        bail!("xdotool failed to simulate Ctrl+V");
    }
    Ok(PasteResult::Pasted)
}
