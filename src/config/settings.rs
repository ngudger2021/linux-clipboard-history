use crate::utils::paths;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::{fs, path::Path};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    pub enabled: bool,
    pub start_on_login: bool,
    pub max_history_items: usize,
    pub shortcut: String,
    pub fallback_shortcut: String,
    pub auto_paste: bool,
    pub theme: String,
    pub poll_interval_ms: u64,
    pub ignore_shorter_than: usize,
    pub ignore_larger_than_bytes: usize,
    pub clear_on_exit: bool,
    pub ignored_apps: Vec<String>,
    pub ignored_patterns: Vec<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            enabled: true,
            start_on_login: true,
            max_history_items: 100,
            shortcut: "Super+V".into(),
            fallback_shortcut: "Ctrl+Alt+V".into(),
            auto_paste: true,
            theme: "system".into(),
            poll_interval_ms: 300,
            ignore_shorter_than: 1,
            ignore_larger_than_bytes: 1_048_576,
            clear_on_exit: false,
            ignored_apps: vec![],
            ignored_patterns: vec![
                "(?i)password".into(),
                "(?i)secret".into(),
                "(?i)token".into(),
            ],
        }
    }
}

impl Config {
    pub fn load_or_create() -> Result<Self> {
        let path = paths::config_file()?;
        if !path.exists() {
            let value = Self::default();
            value.save()?;
            return Ok(value);
        }
        let text = fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
        toml::from_str(&text).with_context(|| format!("parse {}", path.display()))
    }

    pub fn save(&self) -> Result<()> {
        let path = paths::config_file()?;
        let text = toml::to_string_pretty(self)?;
        fs::write(&path, text).with_context(|| format!("write {}", path.display()))
    }

    pub fn set_enabled(enabled: bool) -> Result<()> {
        let mut config = Self::load_or_create()?;
        config.enabled = enabled;
        config.save()
    }

    pub fn sync_autostart(&self) -> Result<()> {
        let path = paths::autostart_file()?;
        if !self.start_on_login {
            if path.exists() {
                fs::remove_file(path)?;
            }
            return Ok(());
        }
        let executable =
            std::env::current_exe().unwrap_or_else(|_| Path::new("linux-clipboard-history").into());
        let desktop = format!("[Desktop Entry]\nType=Application\nName=Clipboard History\nComment=Monitor clipboard history\nExec={}\nIcon=edit-paste\nTerminal=false\nStartupNotify=false\nX-GNOME-Autostart-enabled=true\n", executable.display());
        fs::write(path, desktop)?;
        Ok(())
    }
}
