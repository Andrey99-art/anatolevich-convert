use std::process::Command;

/// Open a folder in the default file manager
pub fn open_folder(path: &str) {
    let _ = Command::new("xdg-open")
        .arg(path)
        .spawn();
}

/// Send success notification with "Open folder" action
pub fn notify_success(success: usize, total: usize, output_dir: &str) {
    let body = format!("{}/{} файлов успешно конвертировано", success, total);
    send_with_action(
        "✅ Конвертация завершена",
        &body,
        "document-save",
        5,
        output_dir,
    );
}

/// Send notification with warnings about errors
pub fn notify_with_errors(success: usize, total: usize, errors: &[(String, String)], output_dir: &str) {
    let error_names: Vec<&str> = errors.iter().map(|(name, _)| name.as_str()).collect();
    let body = format!(
        "{}/{} успешно, {} ошибок\n{}",
        success, total, errors.len(), error_names.join(", ")
    );
    send_with_action(
        "⚠️ Конвертация завершена с ошибками",
        &body,
        "dialog-warning",
        8,
        output_dir,
    );
}

/// Send critical error notification
pub fn notify_error(message: &str) {
    send("❌ Ошибка конвертации", message, "dialog-error", 10);
}

/// Send notification with "Open folder" action button.
/// notify-send --action prints the action key to stdout when clicked.
/// We wait for the output and open the folder if user clicked the action.
fn send_with_action(summary: &str, body: &str, icon: &str, timeout_secs: u32, folder_path: &str) {
    let timeout_ms = (timeout_secs * 1000).to_string();

    // Clone all &str to owned Strings for the spawned thread ('static requirement)
    let summary = summary.to_string();
    let body = body.to_string();
    let icon = icon.to_string();
    let folder = folder_path.to_string();

    std::thread::spawn(move || {
        let result = Command::new("notify-send")
            .arg("--app-name=AnatolevichConvert")
            .arg("--icon")
            .arg(&icon)
            .arg("--expire-time")
            .arg(&timeout_ms)
            .arg("--action=open=📂 Открыть папку")
            .arg(&summary)
            .arg(&body)
            .output();

        match result {
            Ok(output) => {
                // If user clicked the action, stdout contains "open"
                let stdout = String::from_utf8_lossy(&output.stdout);
                if stdout.trim() == "open" {
                    open_folder(&folder);
                }
            }
            Err(_) => {
                // Fallback: send without action if --action is not supported
                let _ = Command::new("notify-send")
                    .arg("--app-name=AnatolevichConvert")
                    .arg("--icon")
                    .arg(&icon)
                    .arg("--expire-time")
                    .arg(&timeout_ms)
                    .arg(&summary)
                    .arg(&body)
                    .spawn();
            }
        }
    });
}

/// Simple notification without actions
fn send(summary: &str, body: &str, icon: &str, timeout_secs: u32) {
    let timeout_ms = (timeout_secs * 1000).to_string();
    let _ = Command::new("notify-send")
        .arg("--app-name=AnatolevichConvert")
        .arg("--icon")
        .arg(icon)
        .arg("--expire-time")
        .arg(&timeout_ms)
        .arg(summary)
        .arg(body)
        .spawn();
}