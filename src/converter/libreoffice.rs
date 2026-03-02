use super::{ConvertError, Format};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};
use std::thread;

const TIMEOUT_SECS: u64 = 120;
const BATCH_TIMEOUT_SECS: u64 = 300; // longer timeout for batch

// Cache soffice availability check — only verify once per session
static SOFFICE_CHECKED: AtomicBool = AtomicBool::new(false);
static SOFFICE_AVAILABLE: AtomicBool = AtomicBool::new(false);

fn check_soffice() -> Result<(), ConvertError> {
    if SOFFICE_CHECKED.load(Ordering::Relaxed) {
        return if SOFFICE_AVAILABLE.load(Ordering::Relaxed) {
            Ok(())
        } else {
            Err(ConvertError::ToolNotFound(
                "LibreOffice (soffice)".to_string(),
                "sudo dnf install libreoffice".to_string(),
            ))
        };
    }

    let result = Command::new("soffice")
        .arg("--version")
        .output();

    let available = result.is_ok();
    SOFFICE_AVAILABLE.store(available, Ordering::Relaxed);
    SOFFICE_CHECKED.store(true, Ordering::Relaxed);

    if available {
        Ok(())
    } else {
        Err(ConvertError::ToolNotFound(
            "LibreOffice (soffice)".to_string(),
            "sudo dnf install libreoffice".to_string(),
        ))
    }
}

fn lo_filter_name(format: Format) -> &'static str {
    match format {
        Format::Pdf => "pdf",
        Format::Docx => "docx",
        Format::Odt => "odt",
        Format::Rtf => "rtf",
        Format::Txt => "txt",
        Format::Html => "html",
        Format::Xlsx => "xlsx",
        Format::Csv => "csv",
        Format::Ods => "ods",
        Format::Pptx => "pptx",
        Format::Odp => "odp",
        _ => "pdf",
    }
}

/// Wait for a child process with timeout.
fn wait_with_timeout(
    child: &mut std::process::Child,
    timeout: Duration,
) -> Result<std::process::ExitStatus, ConvertError> {
    let start = Instant::now();
    loop {
        match child.try_wait() {
            Ok(Some(status)) => return Ok(status),
            Ok(None) => {
                if start.elapsed() >= timeout {
                    let _ = child.kill();
                    return Err(ConvertError::Timeout(timeout.as_secs()));
                }
                thread::sleep(Duration::from_millis(100));
            }
            Err(e) => {
                return Err(ConvertError::ProcessFailed(format!("Wait error: {}", e)));
            }
        }
    }
}

/// Convert a single file via LibreOffice
pub fn convert(
    input: &Path,
    target_format: Format,
    output_dir: &Path,
    expected_output: &Path,
) -> Result<(), ConvertError> {
    check_soffice()?;

    let filter = lo_filter_name(target_format);

    let mut child = Command::new("soffice")
        .arg("--headless")
        .arg("--convert-to")
        .arg(filter)
        .arg("--outdir")
        .arg(output_dir)
        .arg(input)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .map_err(|e| ConvertError::ProcessFailed(format!("Failed to start soffice: {}", e)))?;

    let status = wait_with_timeout(&mut child, Duration::from_secs(TIMEOUT_SECS))?;

    if !status.success() {
        let stderr = child.stderr.take().map(|mut s| {
            let mut buf = String::new();
            std::io::Read::read_to_string(&mut s, &mut buf).ok();
            buf
        }).unwrap_or_default();

        return Err(ConvertError::ProcessFailed(format!(
            "soffice exited with code {}: {}",
            status.code().unwrap_or(-1),
            stderr.trim()
        )));
    }

    resolve_output(input, output_dir, filter, expected_output)
}

/// Batch convert multiple files in a single soffice invocation.
/// Much faster than calling convert() per file — one process startup instead of N.
///
/// Returns list of (input_path, Result) for each file.
pub fn convert_batch(
    inputs: &[(PathBuf, PathBuf)], // (input_path, expected_output_path)
    target_format: Format,
    output_dir: &Path,
) -> Vec<(PathBuf, Result<(), ConvertError>)> {
    if inputs.is_empty() {
        return Vec::new();
    }

    if let Err(e) = check_soffice() {
        return inputs.iter()
            .map(|(p, _)| (p.clone(), Err(ConvertError::ProcessFailed(e.to_string()))))
            .collect();
    }

    let filter = lo_filter_name(target_format);

    // Build command with all input files at once
    let mut cmd = Command::new("soffice");
    cmd.arg("--headless")
        .arg("--convert-to")
        .arg(filter)
        .arg("--outdir")
        .arg(output_dir);

    for (input_path, _) in inputs {
        cmd.arg(input_path);
    }

    let mut child = match cmd
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
    {
        Ok(c) => c,
        Err(e) => {
            let err_msg = format!("Failed to start soffice: {}", e);
            return inputs.iter()
                .map(|(p, _)| (p.clone(), Err(ConvertError::ProcessFailed(err_msg.clone()))))
                .collect();
        }
    };

    // Longer timeout for batch — scales with file count
    let timeout = Duration::from_secs(
        BATCH_TIMEOUT_SECS.max(inputs.len() as u64 * 30)
    );

    let status = match wait_with_timeout(&mut child, timeout) {
        Ok(s) => s,
        Err(e) => {
            return inputs.iter()
                .map(|(p, _)| (p.clone(), Err(ConvertError::Timeout(timeout.as_secs()))))
                .collect();
        }
    };

    if !status.success() {
        let stderr = child.stderr.take().map(|mut s| {
            let mut buf = String::new();
            std::io::Read::read_to_string(&mut s, &mut buf).ok();
            buf
        }).unwrap_or_default();

        let err_msg = format!("soffice batch exited with code {}: {}",
            status.code().unwrap_or(-1), stderr.trim());
        return inputs.iter()
            .map(|(p, _)| (p.clone(), Err(ConvertError::ProcessFailed(err_msg.clone()))))
            .collect();
    }

    // Check each output file individually
    inputs.iter().map(|(input_path, expected_output)| {
        let result = resolve_output(input_path, output_dir, filter, expected_output);
        (input_path.clone(), result)
    }).collect()
}

/// Resolve output file — rename if LibreOffice put it with a different name
fn resolve_output(
    input: &Path,
    output_dir: &Path,
    filter: &str,
    expected_output: &Path,
) -> Result<(), ConvertError> {
    if expected_output.exists() {
        return Ok(());
    }

    // LibreOffice names output as "original_stem.filter"
    let stem = input.file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_default();
    let lo_output = output_dir.join(format!("{}.{}", stem, filter));

    if lo_output.exists() && lo_output != expected_output {
        std::fs::rename(&lo_output, expected_output)?;
    } else if !expected_output.exists() {
        return Err(ConvertError::ProcessFailed(
            "LibreOffice completed but output file not found".to_string(),
        ));
    }

    Ok(())
}