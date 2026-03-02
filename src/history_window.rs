use gtk::prelude::*;
use gtk::{
    self, glib, Align, ApplicationWindow, Box as GtkBox, Button, Label, ListBox, Orientation,
    ScrolledWindow, SelectionMode,
};

use crate::history;

pub fn show_history_window(parent: &ApplicationWindow) {
    let entries = history::read_history();

    let history_window = gtk::Window::builder()
        .title("📋 История конвертаций")
        .default_width(600).default_height(400)
        .transient_for(parent).modal(true)
        .build();

    let vbox = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .spacing(8).margin_top(12).margin_bottom(12).margin_start(12).margin_end(12)
        .build();

    let header_box = GtkBox::builder()
        .orientation(Orientation::Horizontal).spacing(8)
        .build();

    let count_label = Label::builder()
        .label(&format!("Записей: {}", entries.len()))
        .halign(Align::Start).hexpand(true)
        .build();

    let btn_clear_history = Button::builder()
        .label("🗑 Очистить историю")
        .build();

    header_box.append(&count_label);
    header_box.append(&btn_clear_history);
    vbox.append(&header_box);

    let list_box = ListBox::builder()
        .selection_mode(SelectionMode::None)
        .css_classes(vec!["boxed-list".to_string()])
        .build();

    if entries.is_empty() {
        let empty_label = Label::builder()
            .label("История пуста").margin_top(20).margin_bottom(20)
            .build();
        list_box.append(&empty_label);
    } else {
        for entry in &entries {
            let label = Label::builder()
                .label(entry).halign(Align::Start)
                .margin_top(4).margin_bottom(4).margin_start(8).margin_end(8)
                .wrap(true).build();
            list_box.append(&label);
        }
    }

    let scrolled = ScrolledWindow::builder()
        .vexpand(true).child(&list_box)
        .build();

    vbox.append(&scrolled);
    history_window.set_child(Some(&vbox));

    btn_clear_history.connect_clicked(glib::clone!(
        #[weak] list_box,
        #[weak] count_label,
        move |_| {
            history::clear_history();
            while let Some(row) = list_box.first_child() { list_box.remove(&row); }
            let empty = Label::builder()
                .label("История пуста").margin_top(20).margin_bottom(20)
                .build();
            list_box.append(&empty);
            count_label.set_label("Записей: 0");
        }
    ));

    history_window.present();
}