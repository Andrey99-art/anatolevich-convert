use gtk::prelude::*;
use gtk::{gio, ApplicationWindow, FileDialog, FileFilter};
use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;

use crate::settings;

pub fn open_file_dialog(window: &ApplicationWindow, on_selected: impl Fn(Vec<PathBuf>) + 'static) {
    let filter = FileFilter::new();
    filter.set_name(Some("Все поддерживаемые форматы"));
    for mime in [
        "application/pdf",
        "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
        "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
        "application/vnd.openxmlformats-officedocument.presentationml.presentation",
        "application/vnd.oasis.opendocument.text",
        "application/vnd.oasis.opendocument.spreadsheet",
        "application/vnd.oasis.opendocument.presentation",
        "application/rtf", "application/epub+zip",
        "text/plain", "text/markdown", "text/html", "text/csv",
        "image/jpeg", "image/png", "image/webp", "image/svg+xml", "image/bmp", "image/tiff",
    ] { filter.add_mime_type(mime); }
    for p in [
        "*.pdf", "*.docx", "*.doc", "*.xlsx", "*.xls", "*.pptx", "*.ppt",
        "*.odt", "*.ods", "*.odp", "*.rtf", "*.txt", "*.md", "*.markdown",
        "*.html", "*.htm", "*.csv", "*.epub",
        "*.jpg", "*.jpeg", "*.png", "*.webp", "*.svg", "*.bmp", "*.tiff", "*.tif",
    ] { filter.add_pattern(p); }

    let filters = gio::ListStore::new::<FileFilter>();
    filters.append(&filter);
    let dialog = FileDialog::builder()
        .title("Выберите файлы для конвертации")
        .modal(true).filters(&filters).build();

    dialog.open_multiple(Some(window), gio::Cancellable::NONE, move |result| {
        if let Ok(files) = result {
            let mut paths = Vec::new();
            for i in 0..files.n_items() {
                if let Some(file) = files.item(i) {
                    let file = file.downcast::<gio::File>().expect("gio::File");
                    if let Some(path) = file.path() { paths.push(path); }
                }
            }
            if !paths.is_empty() { on_selected(paths); }
        }
    });
}

pub fn pick_output_folder(window: &ApplicationWindow, on_selected: impl FnOnce(PathBuf) + 'static) {
    let dialog = FileDialog::builder()
        .title("Выберите папку для сохранения")
        .modal(true).build();
    if let Some(last_dir) = settings::load_last_dir() {
        dialog.set_initial_folder(Some(&gio::File::for_path(&last_dir)));
    }
    let cb = Rc::new(RefCell::new(Some(on_selected)));
    dialog.select_folder(Some(window), gio::Cancellable::NONE, move |result| {
        if let Ok(folder) = result {
            if let Some(path) = folder.path() {
                if let Some(cb) = cb.borrow_mut().take() { cb(path); }
            }
        }
    });
}