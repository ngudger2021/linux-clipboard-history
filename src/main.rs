mod app;
mod clipboard;
mod config;
mod db;
mod shortcuts;
mod ui;
mod utils;

use anyhow::Result;
use config::Config;
use db::Database;
use std::io::Read;

fn main() {
    if let Err(error) = run() {
        eprintln!("linux-clipboard-history: {error:#}");
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let config = Config::load_or_create()?;
    let db = Database::open_default()?;
    let args: Vec<String> = std::env::args().collect();

    match args.get(1).map(String::as_str) {
        None => app::run_background(config, db),
        Some("show") => app::run_popup(config, db),
        Some("list") => {
            for item in db.list(None, config.max_history_items)? {
                println!("{}\t{}\t{}", item.id, item.created_at, item.preview());
            }
            Ok(())
        }
        Some("clear") => {
            db.clear_unpinned()?;
            Ok(())
        }
        Some("clear-all") => {
            db.clear_all()?;
            Ok(())
        }
        Some("pause") => {
            Config::set_enabled(false)?;
            println!("Clipboard monitoring paused.");
            Ok(())
        }
        Some("resume") => {
            Config::set_enabled(true)?;
            println!("Clipboard monitoring resumed.");
            Ok(())
        }
        Some("capture") => {
            let mut text = String::new();
            std::io::stdin()
                .take(config.ignore_larger_than_bytes as u64 + 1)
                .read_to_string(&mut text)?;
            clipboard::monitor::capture(&db, &config, &text)
        }
        Some("--help" | "-h") => {
            print_help();
            Ok(())
        }
        Some(command) => anyhow::bail!("unknown command '{command}' (try --help)"),
    }
}

fn print_help() {
    println!("linux-clipboard-history [show|list|clear|clear-all|pause|resume]");
}
