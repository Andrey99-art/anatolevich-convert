#[cfg(test)]
mod tests;
mod app;
mod conversion;
mod converter;
mod dialogs;
mod file_entry;
mod formats;
mod history;
mod history_window;
mod notifications;
mod settings;

use gtk::{self, glib, Application};
use gtk::prelude::*;

pub const APP_ID: &str = "com.anatolevich.convert";

fn main() -> glib::ExitCode {
    let app = Application::builder().application_id(APP_ID).build();
    app.connect_activate(app::build_ui);
    app.run()
}