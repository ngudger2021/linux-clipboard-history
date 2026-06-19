use anyhow::{bail, Context, Result};
use std::{
    io::Write,
    process::{Command, Stdio},
};

pub fn read_text() -> Result<String> {
    let output = Command::new("xclip")
        .args(["-selection", "clipboard", "-o"])
        .output()
        .context("xclip is required for X11 clipboard access")?;
    if !output.status.success() {
        bail!("xclip could not read the clipboard");
    }
    Ok(String::from_utf8(output.stdout)?
        .trim_end_matches('\0')
        .to_string())
}

pub fn set_text(text: &str) -> Result<()> {
    let mut child = Command::new("xclip")
        .args(["-selection", "clipboard", "-in"])
        .stdin(Stdio::piped())
        .spawn()
        .context("xclip is required for X11 clipboard access")?;
    child
        .stdin
        .take()
        .context("open xclip stdin")?
        .write_all(text.as_bytes())?;
    if !child.wait()?.success() {
        bail!("xclip could not set the clipboard");
    }
    Ok(())
}
