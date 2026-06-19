use crate::{
    clipboard::paste::{self, PasteResult},
    config::Config,
    db::Database,
    ui::settings,
};
use gtk::{gdk, glib, prelude::*};

#[derive(Clone)]
pub struct Popup {
    pub window: gtk::ApplicationWindow,
    list: gtk::ListBox,
    search: gtk::SearchEntry,
    status: gtk::Label,
    db: Database,
    config: Config,
}

impl Popup {
    pub fn new(application: &gtk::Application, db: Database, config: Config) -> Self {
        let window = gtk::ApplicationWindow::builder()
            .application(application)
            .title("Clipboard History")
            .default_width(480)
            .default_height(620)
            .resizable(true)
            .build();
        let root = gtk::Box::new(gtk::Orientation::Vertical, 8);
        root.set_margin_top(12);
        root.set_margin_bottom(12);
        root.set_margin_start(12);
        root.set_margin_end(12);

        let header = gtk::Box::new(gtk::Orientation::Horizontal, 8);
        let search = gtk::SearchEntry::builder()
            .placeholder_text("Search clipboard history")
            .hexpand(true)
            .build();
        let settings_button = gtk::Button::builder()
            .icon_name("emblem-system-symbolic")
            .tooltip_text("Settings")
            .build();
        header.append(&search);
        header.append(&settings_button);
        root.append(&header);

        let list = gtk::ListBox::new();
        list.set_selection_mode(gtk::SelectionMode::Single);
        list.add_css_class("boxed-list");
        let scroll = gtk::ScrolledWindow::builder()
            .vexpand(true)
            .hscrollbar_policy(gtk::PolicyType::Never)
            .child(&list)
            .build();
        root.append(&scroll);

        let footer = gtk::Box::new(gtk::Orientation::Horizontal, 8);
        let clear = gtk::Button::with_label("Clear unpinned");
        let status = gtk::Label::new(Some(""));
        status.set_hexpand(true);
        status.set_xalign(1.0);
        footer.append(&clear);
        footer.append(&status);
        root.append(&footer);
        window.set_child(Some(&root));

        let popup = Self {
            window,
            list,
            search,
            status,
            db,
            config,
        };
        popup.connect_actions(clear, settings_button);
        popup.refresh();
        popup
    }

    pub fn show(&self) {
        self.refresh();
        self.window.present();
        self.search.grab_focus();
    }

    fn connect_actions(&self, clear: gtk::Button, settings_button: gtk::Button) {
        let this = self.clone();
        self.search.connect_search_changed(move |_| this.refresh());

        let this = self.clone();
        self.list
            .connect_row_activated(move |_, row| this.restore_row(row));

        let this = self.clone();
        clear.connect_clicked(move |_| {
            if let Err(e) = this.db.clear_unpinned() {
                this.error(&e);
            }
            this.refresh();
        });

        let this = self.clone();
        settings_button.connect_clicked(move |_| settings::show(&this.window, this.db.clone()));

        let keys = gtk::EventControllerKey::new();
        let this = self.clone();
        keys.connect_key_pressed(move |_, key, _, modifiers| {
            if key == gdk::Key::Escape {
                this.window.close();
                return glib::Propagation::Stop;
            }
            if key == gdk::Key::Return || key == gdk::Key::KP_Enter {
                if let Some(row) = this.list.selected_row() {
                    this.restore_row(&row);
                }
                return glib::Propagation::Stop;
            }
            if key == gdk::Key::Delete {
                if let Some(row) = this.list.selected_row() {
                    this.delete_row(&row);
                }
                return glib::Propagation::Stop;
            }
            if key == gdk::Key::f && modifiers.contains(gdk::ModifierType::CONTROL_MASK) {
                this.search.grab_focus();
                return glib::Propagation::Stop;
            }
            glib::Propagation::Proceed
        });
        self.window.add_controller(keys);
    }

    pub fn refresh(&self) {
        while let Some(child) = self.list.first_child() {
            self.list.remove(&child);
        }
        match self.db.list(
            Some(self.search.text().as_str()),
            self.config.max_history_items,
        ) {
            Ok(items) => {
                for item in items {
                    let row = gtk::ListBoxRow::new();
                    row.set_widget_name(&item.id.to_string());
                    let outer = gtk::Box::new(gtk::Orientation::Horizontal, 8);
                    outer.set_margin_top(8);
                    outer.set_margin_bottom(8);
                    outer.set_margin_start(8);
                    outer.set_margin_end(8);
                    let labels = gtk::Box::new(gtk::Orientation::Vertical, 3);
                    labels.set_hexpand(true);
                    let preview = gtk::Label::new(Some(&item.preview()));
                    preview.set_xalign(0.0);
                    preview.set_ellipsize(gtk::pango::EllipsizeMode::End);
                    preview.set_lines(2);
                    preview.set_wrap(true);
                    let metadata = gtk::Label::new(Some(&format!(
                        "{}  •  {}{}",
                        item.created_at,
                        item.content_type,
                        if item.is_pinned { "  •  Pinned" } else { "" }
                    )));
                    metadata.add_css_class("dim-label");
                    metadata.set_xalign(0.0);
                    labels.append(&preview);
                    labels.append(&metadata);
                    outer.append(&labels);
                    let pin = gtk::Button::builder()
                        .icon_name(if item.is_pinned {
                            "view-pin-symbolic"
                        } else {
                            "view-pin-symbolic"
                        })
                        .tooltip_text("Pin or unpin")
                        .build();
                    let delete = gtk::Button::builder()
                        .icon_name("user-trash-symbolic")
                        .tooltip_text("Delete")
                        .build();
                    let this = self.clone();
                    let id = item.id;
                    pin.connect_clicked(move |_| {
                        if let Err(e) = this.db.toggle_pin(id) {
                            this.error(&e);
                        }
                        this.refresh();
                    });
                    let this = self.clone();
                    delete.connect_clicked(move |_| {
                        if let Err(e) = this.db.delete(id) {
                            this.error(&e);
                        }
                        this.refresh();
                    });
                    outer.append(&pin);
                    outer.append(&delete);
                    row.set_child(Some(&outer));
                    self.list.append(&row);
                }
            }
            Err(error) => self.error(&error),
        }
        if let Some(row) = self.list.row_at_index(0) {
            self.list.select_row(Some(&row));
        }
    }

    fn row_id(row: &gtk::ListBoxRow) -> Option<i64> {
        row.widget_name().parse().ok()
    }
    fn delete_row(&self, row: &gtk::ListBoxRow) {
        if let Some(id) = Self::row_id(row) {
            if let Err(e) = self.db.delete(id) {
                self.error(&e);
            }
            self.refresh();
        }
    }

    fn restore_row(&self, row: &gtk::ListBoxRow) {
        let Some(id) = Self::row_id(row) else {
            return;
        };
        let Ok(Some(item)) = self.db.get(id) else {
            return;
        };
        let Some(text) = item.text_content else {
            return;
        };
        let db = self.db.clone();
        let auto_paste = self.config.auto_paste;
        let status = self.status.clone();
        let window = self.window.clone();
        self.window.hide();
        glib::timeout_add_local_once(std::time::Duration::from_millis(120), move || {
            match paste::restore(&text, auto_paste) {
                Ok(PasteResult::Pasted) => {
                    let _ = db.mark_used(id);
                }
                Ok(PasteResult::ClipboardOnly) => {
                    let _ = db.mark_used(id);
                    status.set_text("Copied — press Ctrl+V to paste");
                }
                Err(error) => status.set_text(&format!("Paste failed: {error}")),
            }
            window.close();
        });
    }

    fn error(&self, error: &dyn std::fmt::Display) {
        self.status.set_text(&format!("Error: {error}"));
    }
}
