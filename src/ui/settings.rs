use crate::{config::Config, db::Database};
use gtk::prelude::*;

pub fn show(parent: &gtk::ApplicationWindow, db: Database) {
    let config = Config::load_or_create().unwrap_or_default();
    let window = gtk::Window::builder()
        .title("Clipboard History Settings")
        .transient_for(parent)
        .modal(true)
        .default_width(500)
        .default_height(600)
        .build();
    let root = gtk::Box::new(gtk::Orientation::Vertical, 10);
    root.set_margin_top(16);
    root.set_margin_bottom(16);
    root.set_margin_start(16);
    root.set_margin_end(16);

    let enabled = switch_row(&root, "Clipboard monitoring", config.enabled);
    let autostart = switch_row(&root, "Start on login", config.start_on_login);
    let auto_paste = switch_row(&root, "Auto-paste on X11", config.auto_paste);
    let max = gtk::SpinButton::with_range(1.0, 10_000.0, 1.0);
    max.set_value(config.max_history_items as f64);
    field(&root, "Maximum history items", &max);
    let shortcut = gtk::Entry::new();
    shortcut.set_text(&config.shortcut);
    field(&root, "Main shortcut", &shortcut);
    let fallback = gtk::Entry::new();
    fallback.set_text(&config.fallback_shortcut);
    field(&root, "Fallback shortcut", &fallback);
    let theme = gtk::ComboBoxText::new();
    for value in ["system", "light", "dark"] {
        theme.append(Some(value), value);
    }
    theme.set_active_id(Some(&config.theme));
    field(&root, "Theme", &theme);
    let apps = gtk::Entry::new();
    apps.set_text(&config.ignored_apps.join(", "));
    field(&root, "Ignored apps (comma-separated)", &apps);
    let patterns = gtk::Entry::new();
    patterns.set_text(&config.ignored_patterns.join("; "));
    field(&root, "Ignored regexes (semicolon-separated)", &patterns);

    let actions = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    let clear = gtk::Button::with_label("Clear unpinned");
    let clear_all = gtk::Button::with_label("Clear all");
    let save = gtk::Button::with_label("Save");
    save.add_css_class("suggested-action");
    actions.append(&clear);
    actions.append(&clear_all);
    actions.append(&save);
    root.append(&actions);
    window.set_child(Some(&root));

    let db_clear = db.clone();
    clear.connect_clicked(move |_| {
        let _ = db_clear.clear_unpinned();
    });
    clear_all.connect_clicked(move |_| {
        let _ = db.clear_all();
    });
    let window_save = window.clone();
    save.connect_clicked(move |_| {
        let mut updated = Config::load_or_create().unwrap_or_default();
        updated.enabled = enabled.is_active();
        updated.start_on_login = autostart.is_active();
        updated.auto_paste = auto_paste.is_active();
        updated.max_history_items = max.value() as usize;
        updated.shortcut = shortcut.text().to_string();
        updated.fallback_shortcut = fallback.text().to_string();
        updated.theme = theme
            .active_id()
            .map(|s| s.to_string())
            .unwrap_or_else(|| "system".into());
        updated.ignored_apps = split(&apps.text(), ',');
        updated.ignored_patterns = split(&patterns.text(), ';');
        if updated.save().is_ok() {
            let _ = updated.sync_autostart();
            window_save.close();
        }
    });
    window.present();
}

fn field(root: &gtk::Box, title: &str, widget: &impl IsA<gtk::Widget>) {
    let label = gtk::Label::new(Some(title));
    label.set_xalign(0.0);
    root.append(&label);
    root.append(widget);
}

fn switch_row(root: &gtk::Box, title: &str, active: bool) -> gtk::Switch {
    let row = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    let label = gtk::Label::new(Some(title));
    label.set_hexpand(true);
    label.set_xalign(0.0);
    let toggle = gtk::Switch::new();
    toggle.set_active(active);
    row.append(&label);
    row.append(&toggle);
    root.append(&row);
    toggle
}

fn split(value: &str, separator: char) -> Vec<String> {
    value
        .split(separator)
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .map(str::to_string)
        .collect()
}
