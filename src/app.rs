use crate::{
    clipboard::monitor, config::Config, db::Database, shortcuts::global_hotkey, ui::popup::Popup,
};
use anyhow::{Context, Result};
use gtk::prelude::*;
use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread,
    time::Duration,
};

/// Run the clipboard monitor without initializing GTK. Keeping the daemon out
/// of GTK prevents GNOME from treating it as an application waiting for its
/// first window.
pub fn run_background(config: Config, db: Database) -> Result<()> {
    config.sync_autostart()?;

    let open_requested = Arc::new(AtomicBool::new(false));
    global_hotkey::start(
        open_requested.clone(),
        config.shortcut.clone(),
        config.fallback_shortcut.clone(),
    );
    monitor::start(db);

    loop {
        if open_requested.swap(false, Ordering::AcqRel) {
            let executable = std::env::current_exe().context("locate running executable")?;
            if let Err(error) = std::process::Command::new(executable).arg("show").spawn() {
                eprintln!("could not open clipboard popup: {error}");
            }
        }
        thread::sleep(Duration::from_millis(100));
    }
}

/// Run a short-lived GTK process for the visible clipboard popup only.
pub fn run_popup(config: Config, db: Database) -> Result<()> {
    let application = gtk::Application::new(
        Some("io.github.linux_clipboard_history.Popup"),
        gtk::gio::ApplicationFlags::empty(),
    );
    let config_for_activate = config.clone();
    let db_for_activate = db.clone();
    application.connect_startup(move |_| apply_theme(&config.theme));
    application.connect_activate(move |app| {
        let popup = Popup::new(app, db_for_activate.clone(), config_for_activate.clone());
        popup.show();
    });
    application.run();
    Ok(())
}

fn apply_theme(theme: &str) {
    if let Some(settings) = gtk::Settings::default() {
        match theme {
            "dark" => settings.set_gtk_application_prefer_dark_theme(true),
            "light" => settings.set_gtk_application_prefer_dark_theme(false),
            _ => {}
        }
    }
}
