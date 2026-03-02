use super::ConvertError;
use std::path::Path;
use std::process::Command;
use std::time::{Duration, Instant};
use std::thread;

const TIMEOUT_SECS: u64 = 60;

fn check_pandoc() -> Result<(), ConvertError> {
    Command::new("pandoc")
        .arg("--version")
        .output()
        .map_err(|_| {
            ConvertError::ToolNotFound(
                "Pandoc".to_string(),
                "sudo dnf install pandoc".to_string(),
            )
        })?;
    Ok(())
}

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

pub fn convert(input: &Path, output: &Path) -> Result<(), ConvertError> {
    check_pandoc()?;

    let mut child = Command::new("pandoc")
        .arg(input)
        .arg("-o")
        .arg(output)
        .arg("--standalone")
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .map_err(|e| ConvertError::ProcessFailed(format!("Failed to start pandoc: {}", e)))?;

    let status = wait_with_timeout(&mut child, Duration::from_secs(TIMEOUT_SECS))?;

    if !status.success() {
        let stderr = child.stderr.take().map(|mut s| {
            let mut buf = String::new();
            std::io::Read::read_to_string(&mut s, &mut buf).ok();
            buf
        }).unwrap_or_default();

        return Err(ConvertError::ProcessFailed(format!(
            "pandoc exited with code {}: {}",
            status.code().unwrap_or(-1),
            stderr.trim()
        )));
    }

    if !output.exists() {
        return Err(ConvertError::ProcessFailed(
            "Pandoc completed but output file not found".to_string(),
        ));
    }

    Ok(())
}