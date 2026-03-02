use gtk::prelude::*;
use gtk::{glib, Button, Label, ProgressBar};
use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::mpsc;
use std::time::Duration;

use crate::converter::{self, Format};
use crate::file_entry::FileEntry;
use crate::{history, notifications};

const AUTO_FOLDER_THRESHOLD: usize = 5;

struct AnimState {
    completed: usize,
    total: usize,
    success_total: usize,
    error_total: usize,
    error_list: Vec<(String, String)>,
    current_fraction: f64,
    target_fraction: f64,
    last_file: String,
    last_success: bool,
    done: bool,
    notification_sent: bool,
    output_dir_display: String,
}

pub fn apply_rename_template(template: &str, original_name: &str, index: usize) -> String {
    if template.trim().is_empty() {
        return original_name.to_string();
    }
    let base_name = std::path::Path::new(original_name)
        .file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| original_name.to_string());
    let date = get_date();
    let result = template
        .replace("{name}", &base_name)
        .replace("{n}", &format!("{}", index + 1))
        .replace("{date}", &date);

    // Security: sanitize result to prevent path traversal via template
    result
        .replace("..", "_")
        .replace('/', "_")
        .replace('\\', "_")
        .chars()
        .filter(|c| !c.is_control())
        .collect()
}

fn resolve_output_dir(base_dir: &PathBuf, file_count: usize, target_str: &str, custom_folder: &str) -> PathBuf {
    if file_count < AUTO_FOLDER_THRESHOLD {
        return base_dir.clone();
    }

    let folder_name = if custom_folder.trim().is_empty() {
        // Auto-generate folder name
        let timestamp = get_timestamp();
        format!("AnatolevichConvert_{}_{}", target_str.to_uppercase(), timestamp)
    } else {
        // User-specified folder name — sanitize
        custom_folder.trim()
            .replace("..", "_")
            .replace('/', "_")
            .replace('\\', "_")
            .chars()
            .filter(|c| !c.is_control())
            .collect()
    };

    let subfolder = base_dir.join(folder_name);
    if let Err(e) = std::fs::create_dir_all(&subfolder) {
        eprintln!("Failed to create subfolder: {}", e);
        return base_dir.clone();
    }
    subfolder
}

fn get_date() -> String {
    std::process::Command::new("date").arg("+%Y-%m-%d").output().ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| "unknown".to_string())
}

fn get_timestamp() -> String {
    std::process::Command::new("date").arg("+%Y-%m-%d_%H-%M-%S").output().ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| "unknown".to_string())
}

pub fn start_conversion(
    file_list: Vec<FileEntry>, target_format: Format, target_str: &str,
    rename_template: &str, custom_folder: &str, output_dir: PathBuf,
    status_label: &Label, progress_bar: &ProgressBar,
    btn_convert: &Button, btn_select: &Button, btn_clear: &Button,
    btn_open_folder: &Button,
) {
    let total = file_list.len();
    let actual_output = resolve_output_dir(&output_dir, total, target_str, custom_folder);
    let output_display = actual_output.to_string_lossy().to_string();

    if total >= AUTO_FOLDER_THRESHOLD {
        let folder_name = actual_output.file_name()
            .map(|n| n.to_string_lossy().to_string()).unwrap_or_default();
        status_label.set_label(&format!("⏳ Конвертация... → 📁 {}", folder_name));
    } else {
        status_label.set_label("⏳ Конвертация...");
    }

    btn_convert.set_sensitive(false);
    btn_select.set_sensitive(false);
    btn_clear.set_sensitive(false);
    btn_open_folder.set_visible(false);
    progress_bar.set_visible(true);
    progress_bar.set_fraction(0.0);
    progress_bar.set_text(Some(&format!("0/{}", total)));

    let batch_files: Vec<(PathBuf, String)> = file_list.iter().enumerate()
        .map(|(i, e)| {
            let display_name = apply_rename_template(rename_template, &e.name, i);
            (e.path.clone(), display_name)
        }).collect();

    let (tx, rx) = mpsc::channel::<converter::BatchProgress>();
    let target_str_owned = target_str.to_uppercase();

    std::thread::spawn(move || {
        let _result = converter::convert_batch(batch_files, target_format, actual_output, tx);
    });

    let state = Rc::new(RefCell::new(AnimState {
        completed: 0, total, success_total: 0, error_total: 0,
        error_list: Vec::new(), current_fraction: 0.0, target_fraction: 0.0,
        last_file: String::new(), last_success: true, done: false, notification_sent: false,
        output_dir_display: output_display,
    }));

    let pb = progress_bar.clone();
    let sl = status_label.clone();
    let bc = btn_convert.clone();
    let bs = btn_select.clone();
    let bl = btn_clear.clone();
    let bo = btn_open_folder.clone();

    glib::timeout_add_local(Duration::from_millis(16), move || {
        let mut s = state.borrow_mut();

        // Collect any completed files from channel
        while let Ok(msg) = rx.try_recv() {
            s.completed += 1;
            s.last_file = msg.file_name.clone();
            s.last_success = msg.success;

            history::log_conversion(
                &msg.file_name, &target_str_owned, msg.success,
                msg.error_msg.as_deref(),
            );

            if msg.success { s.success_total += 1; }
            else {
                s.error_total += 1;
                if let Some(err) = msg.error_msg {
                    s.error_list.push((msg.file_name, err));
                }
            }
        }

        if s.completed >= s.total { s.done = true; }

        // ── Smooth progress animation ───────────────────────────────
        // Instead of jumping to file-based fraction, progress bar
        // advances smoothly on its own, like an installation bar.
        //
        // Phase 1: Not done yet → slowly approach 0.85 (ease-out curve)
        // Phase 2: Done → quickly animate to 1.0
        //
        if s.done {
            // Quickly finish: jump toward 1.0
            s.target_fraction = 1.0;
            let diff = s.target_fraction - s.current_fraction;
            if diff > 0.001 {
                s.current_fraction += diff * 0.15; // fast finish
            } else {
                s.current_fraction = 1.0;
            }
        } else {
            // Smooth crawl toward 85% — slows down as it gets closer
            // Like: 0→50% fast, 50→70% medium, 70→85% slow
            let ceiling = 0.85;
            let remaining = ceiling - s.current_fraction;
            if remaining > 0.001 {
                // Speed decreases as we approach ceiling (ease-out)
                let speed = remaining * 0.005; 
                s.current_fraction += speed.max(0.0003); // minimum crawl speed
            }
        }

        pb.set_fraction(s.current_fraction);

        // Update text with last completed file
        if !s.last_file.is_empty() && s.completed > 0 {
            let icon = if s.last_success { "✅" } else { "❌" };
            let percent = (s.current_fraction * 100.0) as u32;
            pb.set_text(Some(&format!("{}% — {} {}", percent, icon, s.last_file)));
        } else {
            let percent = (s.current_fraction * 100.0) as u32;
            pb.set_text(Some(&format!("{}% — конвертация...", percent)));
        }

        // Final: animation reached 1.0 and all files done
        if s.done && s.current_fraction >= 0.999 && !s.notification_sent {
            s.notification_sent = true;
            pb.set_fraction(1.0);
            bc.set_sensitive(true);
            bs.set_sensitive(true);
            bl.set_sensitive(true);
            bo.set_visible(true);

            let total = s.success_total + s.error_total;
            let folder_info = if s.total >= AUTO_FOLDER_THRESHOLD {
                format!("\n📁 {}", s.output_dir_display)
            } else { String::new() };

            if s.error_total == 0 {
                sl.set_label(&format!("✅ Готово! {}/{} файлов конвертировано{}", s.success_total, total, folder_info));
                pb.set_text(Some(&format!("✅ {}/{} готово", s.success_total, total)));
                notifications::notify_success(s.success_total, total, &s.output_dir_display);
            } else {
                let names: Vec<&str> = s.error_list.iter().map(|(n, _)| n.as_str()).collect();
                sl.set_label(&format!("⚠️ {}/{} успешно, {} ошибок ({}){}", s.success_total, total, s.error_total, names.join(", "), folder_info));
                pb.set_text(Some(&format!("⚠️ {}/{} успешно", s.success_total, total)));
                notifications::notify_with_errors(s.success_total, total, &s.error_list, &s.output_dir_display);
            }
            return glib::ControlFlow::Break;
        }
        glib::ControlFlow::Continue
    });
}