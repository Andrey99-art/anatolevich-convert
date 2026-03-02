use std::path::PathBuf;

const HISTORY_FILE: &str = "history.log";
const MAX_ENTRIES: usize = 200;

fn history_path() -> Option<PathBuf> {
    let config = std::env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".config")))?;
    Some(config.join("anatolevich-convert").join(HISTORY_FILE))
}

/// Append a conversion record to history
pub fn log_conversion(
    source_name: &str,
    target_format: &str,
    success: bool,
    error_msg: Option<&str>,
) {
    let path = match history_path() {
        Some(p) => p,
        None => return,
    };

    let timestamp = chrono_now();
    let status = if success { "✅" } else { "❌" };
    let detail = error_msg.unwrap_or("");

    // Security: sanitize inputs to prevent log injection
    // Remove newlines and control characters from all fields
    let safe_name: String = source_name.chars()
        .filter(|c| !c.is_control())
        .collect();
    let safe_format: String = target_format.chars()
        .filter(|c| !c.is_control())
        .collect();
    let safe_detail: String = detail.chars()
        .filter(|c| !c.is_control())
        .collect();

    let line = format!(
        "{} {} {} → {} {}\n",
        timestamp, status, safe_name, safe_format, safe_detail
    );

    // Append to file
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }

    use std::io::Write;
    if let Ok(mut file) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
    {
        let _ = file.write_all(line.as_bytes());
    }

    // Trim old entries if file is too large
    trim_history(&path);
}

/// Read all history entries (newest first)
pub fn read_history() -> Vec<String> {
    let path = match history_path() {
        Some(p) => p,
        None => return Vec::new(),
    };

    match std::fs::read_to_string(&path) {
        Ok(content) => content
            .lines()
            .rev() // newest first
            .filter(|l| !l.trim().is_empty())
            .map(|l| l.to_string())
            .collect(),
        Err(_) => Vec::new(),
    }
}

/// Clear all history
pub fn clear_history() {
    if let Some(path) = history_path() {
        let _ = std::fs::write(&path, "");
    }
}

/// Keep only last MAX_ENTRIES lines
fn trim_history(path: &PathBuf) {
    let content = match std::fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => return,
    };

    let lines: Vec<&str> = content.lines().collect();
    if lines.len() > MAX_ENTRIES {
        let trimmed: Vec<&str> = lines[lines.len() - MAX_ENTRIES..].to_vec();
        let _ = std::fs::write(path, trimmed.join("\n") + "\n");
    }
}

/// Simple timestamp without external crate
fn chrono_now() -> String {
    // Read system time and format manually — no chrono dependency needed
    use std::process::Command;
    Command::new("date")
        .arg("+%Y-%m-%d %H:%M:%S")
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| "unknown".to_string())
}