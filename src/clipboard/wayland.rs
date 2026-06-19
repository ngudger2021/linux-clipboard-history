use anyhow::{bail, Context, Result};
use std::{
    io::Write,
    process::{Command, Stdio},
};

pub fn read_text() -> Result<String> {
    let output = Command::new("wl-paste")
        .args(["--no-newline", "--type", "text"])
        .output()
        .context("wl-clipboard is required for Wayland clipboard access")?;
    if !output.status.success() {
        bail!("wl-paste could not read a text clipboard");
    }
    Ok(String::from_utf8(output.stdout)?
        .trim_end_matches('\0')
        .to_string())
}

pub fn set_text(text: &str) -> Result<()> {
    let mut child = Command::new("wl-copy")
        .args(["--type", "text/plain;charset=utf-8"])
        .stdin(Stdio::piped())
        .spawn()
        .context("wl-clipboard is required for Wayland clipboard access")?;
    child
        .stdin
        .take()
        .context("open wl-copy stdin")?
        .write_all(text.as_bytes())?;
    if !child.wait()?.success() {
        bail!("wl-copy could not set the clipboard");
    }
    Ok(())
}
