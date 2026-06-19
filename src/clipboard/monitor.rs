use crate::{clipboard, config::Config, db::Database};
use anyhow::Result;
use regex::Regex;
use std::{
    process::{Command, Stdio},
    thread,
    time::Duration,
};

pub fn start(db: Database) {
    if clipboard::session() == clipboard::Session::Wayland {
        start_wayland_watch(db);
        return;
    }

    thread::spawn(move || poll(db));
}

fn poll(db: Database) -> ! {
    loop {
        let config = Config::load_or_create().unwrap_or_default();
        if config.enabled {
            if let Ok(text) = clipboard::read_text() {
                if let Err(error) = capture(&db, &config, &text) {
                    eprintln!("clipboard history write failed: {error:#}");
                }
            }
        }
        thread::sleep(Duration::from_millis(config.poll_interval_ms.max(100)));
    }
}

pub fn capture(db: &Database, config: &Config, text: &str) -> Result<()> {
    let ignored = !config.enabled
        || text.trim().chars().count() < config.ignore_shorter_than
        || text.len() > config.ignore_larger_than_bytes
        || config.ignored_patterns.iter().any(|pattern| {
            Regex::new(pattern)
                .map(|r| r.is_match(text))
                .unwrap_or(false)
        });
    if !ignored {
        db.insert_text(text, config)?;
    }
    Ok(())
}

fn start_wayland_watch(db: Database) {
    thread::spawn(move || {
        let executable = match std::env::current_exe() {
            Ok(path) => path,
            Err(error) => {
                eprintln!("cannot locate clipboard history executable: {error}");
                poll(db);
            }
        };
        let _ = Command::new("wl-paste")
            .args(["--type", "text", "--watch"])
            .arg(executable)
            .arg("capture")
            .stderr(Stdio::null())
            .status();
        eprintln!("Wayland watch unavailable; using quiet clipboard polling.");
        poll(db);
    });
}
