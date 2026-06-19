use anyhow::{Context, Result};
use std::path::PathBuf;

fn ensure(path: PathBuf) -> Result<PathBuf> {
    std::fs::create_dir_all(&path).with_context(|| format!("create {}", path.display()))?;
    Ok(path)
}

pub fn data_dir() -> Result<PathBuf> {
    ensure(
        dirs::data_local_dir()
            .context("cannot determine local data directory")?
            .join("linux-clipboard-history"),
    )
}

pub fn config_dir() -> Result<PathBuf> {
    ensure(
        dirs::config_dir()
            .context("cannot determine config directory")?
            .join("linux-clipboard-history"),
    )
}

pub fn config_file() -> Result<PathBuf> {
    Ok(config_dir()?.join("config.toml"))
}
pub fn database_file() -> Result<PathBuf> {
    Ok(data_dir()?.join("history.db"))
}

pub fn autostart_file() -> Result<PathBuf> {
    let root = dirs::config_dir()
        .context("cannot determine config directory")?
        .join("autostart");
    std::fs::create_dir_all(&root)?;
    Ok(root.join("linux-clipboard-history.desktop"))
}
