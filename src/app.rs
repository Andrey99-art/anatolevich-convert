use gtk::prelude::*;
use gtk::{
    self, gdk, gio, glib, Align, Application, ApplicationWindow, Box as GtkBox, Button,
    CssProvider, DropDown, Entry, FileDialog, FileFilter, Label, ListBox, ListBoxRow, Orientation,
    ProgressBar, ScrolledWindow, SelectionMode, StringList,
};
use std::cell::RefCell;
use std::rc::Rc;

use crate::conversion;
use crate::converter::Format;
use crate::dialogs;
use crate::file_entry::{format_status, FileEntry};
use crate::formats;
use crate::history_window;
use crate::settings::{self, CSS_BADGES};
use crate::notifications;

pub fn build_ui(app: &Application) {
    let display = gdk::Display::default().expect("Could not get default display");

    let badge_css = CssProvider::new();
    badge_css.load_from_string(CSS_BADGES);
    gtk::style_context_add_provider_for_display(
        &display, &badge_css, gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );

    let theme_css = Rc::new(CssProvider::new());
    gtk::style_context_add_provider_for_display(
        &display, &*theme_css, gtk::STYLE_PROVIDER_PRIORITY_APPLICATION + 1,
    );

    let wallpaper_css = Rc::new(CssProvider::new());
    gtk::style_context_add_provider_for_display(
        &display, &*wallpaper_css, gtk::STYLE_PROVIDER_PRIORITY_APPLICATION + 2,
    );

    let app_settings = Rc::new(RefCell::new(settings::load_settings()));
    settings::apply_theme(app_settings.borrow().dark_theme, &theme_css);
    wallpaper_css.load_from_string(&settings::build_wallpaper_css(&app_settings.borrow().wallpaper_path));

    let files: Rc<RefCell<Vec<FileEntry>>> = Rc::new(RefCell::new(Vec::new()));

    // ── Header bar ──────────────────────────────────────────────────────
    let header = gtk::HeaderBar::builder().show_title_buttons(true).build();
    let title_label = Label::builder()
        .label("AnatolevichConvert")
        .css_classes(vec!["title".to_string()])
        .build();
    header.set_title_widget(Some(&title_label));

    let is_dark = app_settings.borrow().dark_theme;
    let btn_theme = Button::builder()
        .label(if is_dark { "☀️" } else { "🌙" })
        .tooltip_text("Переключить тему").build();
    let btn_history = Button::builder()
        .label("📋").tooltip_text("История конвертаций").build();
    let btn_wallpaper = Button::builder()
        .label("🖼️").tooltip_text("Установить фоновое изображение").build();
    let btn_clear_wallpaper = Button::builder()
        .label("🚫").tooltip_text("Убрать фоновое изображение")
        .visible(app_settings.borrow().wallpaper_path.is_some()).build();

    header.pack_end(&btn_theme);
    header.pack_end(&btn_history);
    header.pack_end(&btn_wallpaper);
    header.pack_end(&btn_clear_wallpaper);

    // ── Top bar ─────────────────────────────────────────────────────────
    let btn_select = Button::builder().label("📂 Выбрать файлы").build();
    let btn_clear = Button::builder().label("🗑 Очистить").sensitive(false).build();

    let top_bar = GtkBox::builder()
        .orientation(Orientation::Horizontal)
        .spacing(10).margin_top(12).margin_start(12).margin_end(12)
        .build();
    top_bar.append(&btn_select);
    top_bar.append(&btn_clear);

    let status_label = Label::builder()
        .label(&format_status(&[]))
        .halign(Align::Start).margin_top(8).margin_start(12)
        .build();

    let progress_bar = ProgressBar::builder()
        .margin_start(12).margin_end(12).visible(false).show_text(true)
        .build();

    // "Open folder" button — appears after conversion completes
    let btn_open_folder = Button::builder()
        .label("📂 Открыть папку")
        .margin_start(12)
        .margin_end(12)
        .visible(false)
        .build();

    let list_box = ListBox::builder()
        .selection_mode(SelectionMode::None)
        .margin_start(12).margin_end(12)
        .css_classes(vec!["boxed-list".to_string()])
        .build();

    let scrolled = ScrolledWindow::builder()
        .hscrollbar_policy(gtk::PolicyType::Never)
        .vscrollbar_policy(gtk::PolicyType::Automatic)
        .vexpand(true).child(&list_box)
        .build();

    // ── Bottom bar — format selector + rename template ──────────────────
    let format_strings = formats::build_format_list();
    let string_list = Rc::new(RefCell::new(
        StringList::new(&format_strings.iter().map(|s| s.as_str()).collect::<Vec<_>>())
    ));
    let format_dropdown = DropDown::builder()
        .model(&*string_list.borrow())
        .build();
    format_dropdown.set_selected(0);

    let format_label = Label::builder().label("Конвертировать в:").halign(Align::Start).build();

    // ── Rename template (user-friendly) ─────────────────────────────────
    let rename_box = GtkBox::builder()
        .orientation(Orientation::Horizontal)
        .spacing(4)
        .build();

    let rename_entry = Entry::builder()
        .placeholder_text("Имя файла (необязательно)")
        .tooltip_text("Переименовать файлы при конвертации.\nОставьте пустым — сохранятся оригинальные имена.")
        .hexpand(false)
        .width_chars(18)
        .build();

    // Quick-insert buttons — click to append template tag
    let btn_tag_name = Button::builder()
        .label("Имя")
        .tooltip_text("Вставить оригинальное имя файла")
        .css_classes(vec!["flat".to_string()])
        .build();
    let btn_tag_num = Button::builder()
        .label("№")
        .tooltip_text("Вставить порядковый номер (1, 2, 3...)")
        .css_classes(vec!["flat".to_string()])
        .build();
    let btn_tag_date = Button::builder()
        .label("Дата")
        .tooltip_text("Вставить текущую дату")
        .css_classes(vec!["flat".to_string()])
        .build();

    // Insert tag at cursor position when button is clicked
    btn_tag_name.connect_clicked(glib::clone!(
        #[weak] rename_entry,
        move |_| { insert_tag(&rename_entry, "{name}"); }
    ));
    btn_tag_num.connect_clicked(glib::clone!(
        #[weak] rename_entry,
        move |_| { insert_tag(&rename_entry, "{n}"); }
    ));
    btn_tag_date.connect_clicked(glib::clone!(
        #[weak] rename_entry,
        move |_| { insert_tag(&rename_entry, "{date}"); }
    ));

    rename_box.append(&rename_entry);
    rename_box.append(&btn_tag_name);
    rename_box.append(&btn_tag_num);
    rename_box.append(&btn_tag_date);

    // ── Folder name entry (visible when 5+ files) ───────────────────────
    let folder_entry = Entry::builder()
        .placeholder_text("Имя папки (авто)")
        .tooltip_text("Имя подпапки для результатов.\nОставьте пустым — создастся автоматически.\nПоявляется при 5+ файлах.")
        .hexpand(false)
        .width_chars(18)
        .visible(false)
        .build();

    // Smart format recommend button
    let btn_ai_recommend = Button::builder()
        .label("🧠")
        .tooltip_text("Умный подбор формата")
        .build();

    let btn_convert = Button::builder()
        .label("⚡ Конвертировать").sensitive(false)
        .css_classes(vec!["suggested-action".to_string()])
        .hexpand(true).build();

    let bottom_bar = GtkBox::builder()
        .orientation(Orientation::Horizontal)
        .spacing(8).margin_bottom(12).margin_start(12).margin_end(12).margin_top(8)
        .build();
    bottom_bar.append(&format_label);
    bottom_bar.append(&format_dropdown);
    bottom_bar.append(&rename_box);
    bottom_bar.append(&folder_entry);
    bottom_bar.append(&btn_ai_recommend);
    bottom_bar.append(&btn_convert);

    // ── Main layout ─────────────────────────────────────────────────────
    let main_box = GtkBox::builder()
        .orientation(Orientation::Vertical).spacing(4)
        .css_classes(vec!["main-container".to_string()])
        .build();
    main_box.append(&top_bar);
    main_box.append(&status_label);
    main_box.append(&progress_bar);
    main_box.append(&btn_open_folder);
    main_box.append(&scrolled);
    main_box.append(&bottom_bar);

    let window = ApplicationWindow::builder()
        .application(app)
        .title("AnatolevichConvert")
        .default_width(700).default_height(500)
        .child(&main_box)
        .build();
    window.set_titlebar(Some(&header));

    // ── Helper: update format dropdown based on selected files ───────────
    let update_formats = {
        let files = files.clone();
        let string_list = string_list.clone();
        let format_dropdown = format_dropdown.clone();
        let folder_entry = folder_entry.clone();
        move || {
            let fl = files.borrow();
            let extensions: Vec<String> = fl.iter()
                .map(|f| f.extension_upper())
                .collect();

            // Show folder name entry when 5+ files
            folder_entry.set_visible(fl.len() >= 5);

            let new_formats = if extensions.is_empty() {
                formats::build_format_list()
            } else {
                formats::build_filtered_format_list(Some(&extensions))
            };

            let new_string_list = StringList::new(
                &new_formats.iter().map(|s| s.as_str()).collect::<Vec<_>>()
            );
            format_dropdown.set_model(Some(&new_string_list));
            format_dropdown.set_selected(0);
            *string_list.borrow_mut() = new_string_list;
        }
    };

    // Wrap in Rc for sharing between closures
    let update_formats = Rc::new(update_formats);

    // ── Keyboard shortcuts ──────────────────────────────────────────────
    let key_controller = gtk::EventControllerKey::new();
    key_controller.connect_key_pressed(glib::clone!(
        #[weak] btn_select, #[weak] window,
        #[upgrade_or] glib::Propagation::Proceed,
        move |_, keyval, _, modifier| {
            let ctrl = modifier.contains(gdk::ModifierType::CONTROL_MASK);
            if ctrl {
                match keyval {
                    gdk::Key::o | gdk::Key::O => { btn_select.emit_clicked(); return glib::Propagation::Stop; }
                    gdk::Key::q | gdk::Key::Q => { window.close(); return glib::Propagation::Stop; }
                    _ => {}
                }
            }
            glib::Propagation::Proceed
        }
    ));
    window.add_controller(key_controller);

    // ── Theme toggle ────────────────────────────────────────────────────
    btn_theme.connect_clicked(glib::clone!(
        #[strong] app_settings, #[strong] theme_css,
        move |btn| {
            let mut s = app_settings.borrow_mut();
            s.dark_theme = !s.dark_theme;
            settings::apply_theme(s.dark_theme, &theme_css);
            btn.set_label(if s.dark_theme { "☀️" } else { "🌙" });
            settings::save_settings(&s);
        }
    ));

    btn_history.connect_clicked(glib::clone!(
        #[weak] window,
        move |_| { history_window::show_history_window(&window); }
    ));

    // ── Wallpaper ───────────────────────────────────────────────────────
    btn_wallpaper.connect_clicked(glib::clone!(
        #[weak] window, #[strong] app_settings, #[strong] wallpaper_css, #[weak] btn_clear_wallpaper,
        move |_| {
            let filter = FileFilter::new();
            filter.set_name(Some("Изображения"));
            for mime in ["image/jpeg", "image/png", "image/webp", "image/svg+xml", "image/bmp"] {
                filter.add_mime_type(mime);
            }
            let filters = gio::ListStore::new::<FileFilter>();
            filters.append(&filter);
            let dialog = FileDialog::builder()
                .title("Выберите фоновое изображение")
                .modal(true).filters(&filters).build();
            let s = app_settings.clone(); let wc = wallpaper_css.clone(); let cb = btn_clear_wallpaper.clone();
            dialog.open(Some(&window), gio::Cancellable::NONE, move |result| {
                if let Ok(file) = result {
                    if let Some(path) = file.path() {
                        let mut st = s.borrow_mut();
                        st.wallpaper_path = Some(path.to_string_lossy().to_string());
                        wc.load_from_string(&settings::build_wallpaper_css(&st.wallpaper_path));
                        settings::save_settings(&st);
                        cb.set_visible(true);
                    }
                }
            });
        }
    ));

    btn_clear_wallpaper.connect_clicked(glib::clone!(
        #[strong] app_settings, #[strong] wallpaper_css,
        move |btn| {
            let mut s = app_settings.borrow_mut();
            s.wallpaper_path = None;
            wallpaper_css.load_from_string(&settings::build_wallpaper_css(&None));
            settings::save_settings(&s);
            btn.set_visible(false);
        }
    ));

    // ── Drag & Drop ─────────────────────────────────────────────────────
    let drop_target = gtk::DropTarget::new(gdk::FileList::static_type(), gdk::DragAction::COPY);
    drop_target.connect_enter(glib::clone!(
        #[weak] scrolled, #[upgrade_or] gdk::DragAction::empty(),
        move |_, _, _| { scrolled.add_css_class("drop-highlight"); gdk::DragAction::COPY }
    ));
    drop_target.connect_leave(glib::clone!(
        #[weak] scrolled,
        move |_| { scrolled.remove_css_class("drop-highlight"); }
    ));
    drop_target.connect_drop(glib::clone!(
        #[weak] scrolled, #[weak] list_box, #[weak] status_label,
        #[weak] btn_clear, #[weak] btn_convert, #[strong] files, #[strong] update_formats,
        #[upgrade_or] false,
        move |_, value, _, _| {
            scrolled.remove_css_class("drop-highlight");
            if let Ok(file_list) = value.get::<gdk::FileList>() {
                let dropped = file_list.files();
                let mut paths = Vec::new();
                for file in dropped {
                    if let Some(path) = file.path() { paths.push(path); }
                }
                let mut fl = files.borrow_mut();
                for path in paths {
                    if fl.iter().any(|f| f.path == path) { continue; }
                    if let Some(entry) = FileEntry::from_path(path) {
                        let row = create_file_row(&entry, &files, &list_box, &status_label, &btn_clear, &btn_convert, &update_formats);
                        list_box.append(&row);
                        fl.push(entry);
                    }
                }
                status_label.set_label(&format_status(&fl));
                btn_clear.set_sensitive(!fl.is_empty());
                btn_convert.set_sensitive(!fl.is_empty());
                drop(fl); // release borrow before calling update_formats
                update_formats();
                return true;
            }
            false
        }
    ));
    window.add_controller(drop_target);

    // ── Select files ────────────────────────────────────────────────────
    btn_select.connect_clicked(glib::clone!(
        #[weak] window, #[weak] list_box, #[weak] status_label,
        #[weak] btn_clear, #[weak] btn_convert, #[strong] files, #[strong] update_formats,
        move |_| {
            let fc = files.clone(); let lc = list_box.clone();
            let sc = status_label.clone(); let cc = btn_clear.clone();
            let vc = btn_convert.clone(); let uf = update_formats.clone();
            dialogs::open_file_dialog(&window, move |new_files| {
                let mut fl = fc.borrow_mut();
                for path in new_files {
                    if fl.iter().any(|f| f.path == path) { continue; }
                    if let Some(entry) = FileEntry::from_path(path) {
                        let row = create_file_row(&entry, &fc, &lc, &sc, &cc, &vc, &uf);
                        lc.append(&row);
                        fl.push(entry);
                    }
                }
                sc.set_label(&format_status(&fl));
                cc.set_sensitive(!fl.is_empty());
                vc.set_sensitive(!fl.is_empty());
                drop(fl);
                uf();
            });
        }
    ));

   // ── Smart Format Recommend ──────────────────────────────────────
    btn_ai_recommend.connect_clicked(glib::clone!(
        #[strong] files, #[strong] string_list, #[weak] format_dropdown,
        #[weak] status_label,
        move |_| {
            let fl = files.borrow();
            if fl.is_empty() {
                status_label.set_label("⚠️ Сначала выберите файлы");
                return;
            }

            let extensions: Vec<String> = fl.iter().map(|f| f.extension_upper()).collect();
            drop(fl);

            let (recommended, reason) = formats::recommend_format(&extensions);

            // Select recommended format in dropdown
            let sl = string_list.borrow();
            for i in 0..sl.n_items() {
                if let Some(item) = sl.string(i) {
                    let item_fmt = item.split_whitespace().last().unwrap_or("").to_uppercase();
                    if item_fmt == recommended.to_uppercase() {
                        format_dropdown.set_selected(i);
                        break;
                    }
                }
            }

            status_label.set_label(&format!("🧠 {}", reason));
        }
    ));

    // ── Open folder button ──────────────────────────────────────────
    // Stores the output path, updated each conversion
    let output_path: Rc<RefCell<String>> = Rc::new(RefCell::new(String::new()));

    btn_open_folder.connect_clicked(glib::clone!(
        #[strong] output_path,
        move |_| {
            let path = output_path.borrow().clone();
            if !path.is_empty() {
                notifications::open_folder(&path);
            }
        }
    ));

    // ── Clear ───────────────────────────────────────────────────────────
    btn_clear.connect_clicked(glib::clone!(
        #[weak] list_box, #[weak] status_label, #[weak] btn_convert, #[weak] progress_bar,
        #[weak] btn_open_folder, #[weak] folder_entry, #[strong] files, #[strong] update_formats,
        move |btn| {
            files.borrow_mut().clear();
            while let Some(row) = list_box.first_child() { list_box.remove(&row); }
            status_label.set_label(&format_status(&[]));
            btn.set_sensitive(false);
            btn_convert.set_sensitive(false);
            progress_bar.set_visible(false);
            btn_open_folder.set_visible(false);
            folder_entry.set_visible(false);
            update_formats();
        }
    ));

    // ── Convert ─────────────────────────────────────────────────────────
    btn_convert.connect_clicked(glib::clone!(
        #[weak] window, #[strong] files, #[strong] string_list,
        #[weak] format_dropdown, #[weak] rename_entry,
        #[weak] status_label, #[weak] progress_bar,
        #[weak] btn_convert, #[weak] btn_select, #[weak] btn_clear,
        #[weak] btn_open_folder, #[strong] output_path,
        #[weak] folder_entry,
        move |_| {
            let file_list = files.borrow().clone();
            if file_list.is_empty() { return; }

            let selected_idx = format_dropdown.selected();
            let sl = string_list.borrow();
            let target_str = sl.string(selected_idx)
                .map(|s| formats::format_from_display(&s))
                .unwrap_or_else(|| "PDF".to_string());

            let target_format = match Format::from_extension(&target_str) {
                Some(f) => f,
                None => { status_label.set_label(&format!("❌ Неизвестный формат: {}", target_str)); return; }
            };

            let rename_template = rename_entry.text().to_string();
            let custom_folder = folder_entry.text().to_string();

            dialogs::pick_output_folder(&window, {
                let sl = status_label.clone(); let pb = progress_bar.clone();
                let bc = btn_convert.clone(); let bs = btn_select.clone(); let bl = btn_clear.clone();
                let bo = btn_open_folder.clone();
                let ts = target_str.clone();
                let op = output_path.clone();
                move |output_dir| {
                    *op.borrow_mut() = output_dir.to_string_lossy().to_string();
                    settings::save_last_dir(&output_dir);
                    conversion::start_conversion(
                        file_list, target_format, &ts, &rename_template,
                        &custom_folder, output_dir,
                        &sl, &pb, &bc, &bs, &bl, &bo,
                    );
                }
            });
        }
    ));

    window.present();
}

// ── File row widget ─────────────────────────────────────────────────────────
fn create_file_row(
    entry: &FileEntry, files: &Rc<RefCell<Vec<FileEntry>>>,
    list_box: &ListBox, status_label: &Label, btn_clear: &Button, btn_convert: &Button,
    update_formats: &Rc<impl Fn() + 'static>,
) -> ListBoxRow {
    let badge = Label::builder()
        .label(&entry.extension_upper())
        .css_classes(vec![entry.badge_css_class().to_string()])
        .width_chars(6).build();

    // Editable name: Label by default, click to edit
    let name_stack = gtk::Stack::builder()
        .transition_type(gtk::StackTransitionType::Crossfade)
        .transition_duration(150)
        .hexpand(true)
        .build();

    let name_label = Label::builder()
        .label(&entry.name)
        .halign(Align::Start)
        .ellipsize(gtk::pango::EllipsizeMode::Middle)
        .build();

    // Show only filename stem (without extension) for editing
    let stem = std::path::Path::new(&entry.name)
        .file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| entry.name.clone());
    let ext = std::path::Path::new(&entry.name)
        .extension()
        .map(|s| format!(".{}", s.to_string_lossy()))
        .unwrap_or_default();

    let name_entry = Entry::builder()
        .text(&stem)
        .hexpand(true)
        .build();

    name_stack.add_named(&name_label, Some("label"));
    name_stack.add_named(&name_entry, Some("entry"));
    name_stack.set_visible_child_name("label");

    // Click on label → switch to edit mode
    let click = gtk::GestureClick::new();
    click.connect_released(glib::clone!(
        #[weak] name_stack, #[weak] name_entry,
        move |_, _, _, _| {
            name_stack.set_visible_child_name("entry");
            name_entry.grab_focus();
            // Select all text for easy replacement
            name_entry.select_region(0, -1);
        }
    ));
    name_label.add_controller(click);

    // Press Enter → save and switch back to label
    let ext_activate = ext.clone();
    name_entry.connect_activate(glib::clone!(
        #[weak] name_stack, #[weak] name_label, #[strong] files,
        move |entry| {
            let new_stem = entry.text().to_string().trim().to_string();
            if !new_stem.is_empty() {
                let old_name = name_label.label().to_string();
                let new_name = format!("{}{}", new_stem, ext_activate);
                name_label.set_label(&new_name);
                let mut fl = files.borrow_mut();
                for f in fl.iter_mut() {
                    if f.name == old_name {
                        f.name = new_name.clone();
                        break;
                    }
                }
            }
            name_stack.set_visible_child_name("label");
        }
    ));

    // Escape → cancel editing
    let key_controller = gtk::EventControllerKey::new();
    key_controller.connect_key_pressed(glib::clone!(
        #[weak] name_stack, #[weak] name_entry, #[weak] name_label,
        #[upgrade_or] glib::Propagation::Proceed,
        move |_, keyval, _, _| {
            if keyval == gdk::Key::Escape {
                // Revert to current label text
                name_entry.set_text(&name_label.label());
                name_stack.set_visible_child_name("label");
                return glib::Propagation::Stop;
            }
            glib::Propagation::Proceed
        }
    ));
    name_entry.add_controller(key_controller);

    // Focus lost → save and switch back
    let ext_focus = ext.clone();
    let focus_controller = gtk::EventControllerFocus::new();
    focus_controller.connect_leave(glib::clone!(
        #[weak] name_stack, #[weak] name_entry, #[weak] name_label, #[strong] files,
        move |_| {
            if name_stack.visible_child_name().as_deref() == Some("entry") {
                let new_stem = name_entry.text().to_string().trim().to_string();
                if !new_stem.is_empty() {
                    let old_name = name_label.label().to_string();
                    let new_name = format!("{}{}", new_stem, ext_focus);
                    name_label.set_label(&new_name);
                    let mut fl = files.borrow_mut();
                    for f in fl.iter_mut() {
                        if f.name == old_name {
                            f.name = new_name.clone();
                            break;
                        }
                    }
                }
                name_stack.set_visible_child_name("label");
            }
        }
    ));
    name_entry.add_controller(focus_controller);

    let size_label = Label::builder()
        .label(&entry.size_display())
        .css_classes(vec!["dim-label".to_string()]).build();
    let remove_btn = Button::builder()
        .label("✕")
        .css_classes(vec!["flat".to_string(), "remove-btn".to_string()])
        .valign(Align::Center).build();

    let content = GtkBox::builder()
        .orientation(Orientation::Horizontal)
        .spacing(12).margin_top(6).margin_bottom(6).margin_start(8).margin_end(8)
        .build();
    content.append(&badge);
    content.append(&name_stack);
    content.append(&size_label);
    content.append(&remove_btn);

    let row = ListBoxRow::builder().child(&content).build();
    let file_path = entry.path.clone();

    remove_btn.connect_clicked(glib::clone!(
        #[weak] row, #[weak] list_box, #[weak] status_label,
        #[weak] btn_clear, #[weak] btn_convert, #[strong] files, #[strong] update_formats,
        move |_| {
            files.borrow_mut().retain(|f| f.path != file_path);
            list_box.remove(&row);
            let fl = files.borrow();
            status_label.set_label(&format_status(&fl));
            btn_clear.set_sensitive(!fl.is_empty());
            btn_convert.set_sensitive(!fl.is_empty());
            drop(fl);
            update_formats();
        }
    ));

    row
}

/// Insert a template tag at the current cursor position in the entry
fn insert_tag(entry: &Entry, tag: &str) {
    let pos = entry.position();
    entry.insert_text(tag, &mut pos.clone());
    entry.set_position(pos + tag.len() as i32);
    entry.grab_focus();
}