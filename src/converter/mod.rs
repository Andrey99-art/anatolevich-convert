pub mod image_backend;
pub mod libreoffice;
pub mod pandoc;

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{mpsc, Arc};
use thiserror::Error;

// ── Supported formats ───────────────────────────────────────────────────────
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Format {
    Pdf, Docx, Odt, Rtf, Txt, Md, Html, Epub,
    Xlsx, Csv, Ods,
    Pptx, Odp,
    Jpg, Png, WebP, Svg, Bmp, Tiff,
}

impl Format {
    pub fn from_extension(ext: &str) -> Option<Self> {
        match ext.to_lowercase().as_str() {
            "pdf" => Some(Self::Pdf),
            "docx" => Some(Self::Docx),
            "odt" => Some(Self::Odt),
            "rtf" => Some(Self::Rtf),
            "txt" => Some(Self::Txt),
            "md" | "markdown" => Some(Self::Md),
            "html" | "htm" => Some(Self::Html),
            "epub" => Some(Self::Epub),
            "xlsx" => Some(Self::Xlsx),
            "csv" => Some(Self::Csv),
            "ods" => Some(Self::Ods),
            "pptx" => Some(Self::Pptx),
            "odp" => Some(Self::Odp),
            "jpg" | "jpeg" => Some(Self::Jpg),
            "png" => Some(Self::Png),
            "webp" => Some(Self::WebP),
            "svg" => Some(Self::Svg),
            "bmp" => Some(Self::Bmp),
            "tiff" | "tif" => Some(Self::Tiff),
            _ => None,
        }
    }

    pub fn extension(&self) -> &'static str {
        match self {
            Self::Pdf => "pdf", Self::Docx => "docx", Self::Odt => "odt",
            Self::Rtf => "rtf", Self::Txt => "txt", Self::Md => "md",
            Self::Html => "html", Self::Epub => "epub",
            Self::Xlsx => "xlsx", Self::Csv => "csv", Self::Ods => "ods",
            Self::Pptx => "pptx", Self::Odp => "odp",
            Self::Jpg => "jpg", Self::Png => "png", Self::WebP => "webp",
            Self::Svg => "svg", Self::Bmp => "bmp", Self::Tiff => "tiff",
        }
    }

    pub fn detect(path: &Path) -> Option<Self> {
        let ext = path.extension()?.to_str()?;
        Self::from_extension(ext)
    }

    pub fn is_image(&self) -> bool {
        matches!(self, Self::Jpg | Self::Png | Self::WebP | Self::Svg | Self::Bmp | Self::Tiff)
    }
}

// ── Errors ──────────────────────────────────────────────────────────────────
#[derive(Debug, Error)]
pub enum ConvertError {
    #[error("Unsupported conversion: {from} → {to}")]
    UnsupportedConversion { from: String, to: String },
    #[error("Source file not found: {0}")]
    FileNotFound(PathBuf),
    #[error("Tool not installed: {0}. Install with: {1}")]
    ToolNotFound(String, String),
    #[error("Conversion failed: {0}")]
    ProcessFailed(String),
    #[error("Conversion timed out after {0} seconds")]
    Timeout(u64),
    #[error("Image conversion error: {0}")]
    ImageError(String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

// ── Backend selection ───────────────────────────────────────────────────────
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Backend {
    LibreOffice,
    Pandoc,
    ImageCrate,
}

pub fn select_backend(from: Format, to: Format) -> Result<Backend, ConvertError> {
    if from == to {
        return Err(ConvertError::UnsupportedConversion {
            from: from.extension().to_string(),
            to: to.extension().to_string(),
        });
    }

    if from.is_image() && to.is_image() {
        return Ok(Backend::ImageCrate);
    }

    let pandoc_sources = [Format::Md, Format::Html, Format::Txt, Format::Epub];
    if pandoc_sources.contains(&from) && !to.is_image() {
        return Ok(Backend::Pandoc);
    }

    let lo_sources = [
        Format::Docx, Format::Odt, Format::Rtf,
        Format::Xlsx, Format::Csv, Format::Ods,
        Format::Pptx, Format::Odp, Format::Pdf,
    ];
    let lo_targets = [
        Format::Pdf, Format::Docx, Format::Odt, Format::Rtf, Format::Txt, Format::Html,
        Format::Xlsx, Format::Csv, Format::Ods, Format::Pptx, Format::Odp,
    ];
    if lo_sources.contains(&from) && lo_targets.contains(&to) {
        return Ok(Backend::LibreOffice);
    }

    Err(ConvertError::UnsupportedConversion {
        from: from.extension().to_string(),
        to: to.extension().to_string(),
    })
}

// ── Single file conversion ──────────────────────────────────────────────────
pub fn convert_file(
    input: &Path,
    target_format: Format,
    output_dir: &Path,
) -> Result<PathBuf, ConvertError> {
    if !input.exists() {
        return Err(ConvertError::FileNotFound(input.to_path_buf()));
    }

    let source_format = Format::detect(input).ok_or_else(|| ConvertError::UnsupportedConversion {
        from: input.extension().map(|e| e.to_string_lossy().to_string()).unwrap_or_else(|| "unknown".to_string()),
        to: target_format.extension().to_string(),
    })?;

    let backend = select_backend(source_format, target_format)?;
    std::fs::create_dir_all(output_dir)?;

    let stem = input.file_stem().map(|s| s.to_string_lossy().to_string()).unwrap_or_else(|| "output".to_string());
    // Security: sanitize filename to prevent path traversal
    let safe_stem: String = stem
        .replace("..", "_")  // prevent directory traversal
        .replace('/', "_")   // prevent path injection
        .replace('\\', "_")  // prevent Windows-style path injection
        .chars()
        .map(|c| if c.is_alphanumeric() || c == '-' || c == '_' || c == '.' { c } else { '_' })
        .collect();
    // Ensure stem is not empty after sanitization
    let safe_stem = if safe_stem.trim_matches('.').trim_matches('_').is_empty() {
        "converted".to_string()
    } else {
        safe_stem
    };
    let output_path = output_dir.join(format!("{}.{}", safe_stem, target_format.extension()));

    // Security: verify output stays within output_dir
    let canonical_dir = output_dir.canonicalize().unwrap_or_else(|_| output_dir.to_path_buf());
    let canonical_output = output_path.canonicalize().unwrap_or_else(|_| output_path.clone());
    if !canonical_output.starts_with(&canonical_dir) {
        return Err(ConvertError::ProcessFailed(
            format!("Security: output path escapes target directory: {:?}", output_path)
        ));
    }

    match backend {
        Backend::LibreOffice => libreoffice::convert(input, target_format, output_dir, &output_path)?,
        Backend::Pandoc => pandoc::convert(input, &output_path)?,
        Backend::ImageCrate => image_backend::convert(input, target_format, &output_path)?,
    }

    Ok(output_path)
}

// ── Batch conversion with parallelism ───────────────────────────────────────
/// Progress callback data sent per file
pub struct BatchProgress {
    pub index: usize,
    pub total: usize,
    pub file_name: String,
    pub success: bool,
    pub error_msg: Option<String>,
}

/// Result of a batch conversion
pub struct BatchResult {
    pub success_count: usize,
    pub error_count: usize,
    pub errors: Vec<(String, String)>,
}

/// Convert multiple files, sending progress through mpsc channel.
/// Uses rayon for parallelism where safe:
/// - LibreOffice: sequential (single soffice instance limitation)
/// - Pandoc / image: parallel across CPU cores
pub fn convert_batch(
    files: Vec<(PathBuf, String)>, // (path, display_name)
    target_format: Format,
    output_dir: PathBuf,
    progress_tx: mpsc::Sender<BatchProgress>,
) -> BatchResult {
    use rayon::prelude::*;

    let total = files.len();
    let counter = Arc::new(AtomicUsize::new(0));
    let success_count = Arc::new(AtomicUsize::new(0));
    let error_count = Arc::new(AtomicUsize::new(0));
    let errors = Arc::new(std::sync::Mutex::new(Vec::<(String, String)>::new()));

    // Separate files by backend type for optimal parallelism
    let mut lo_files = Vec::new();
    let mut parallel_files = Vec::new();

    for (path, name) in &files {
        let source = Format::detect(path);
        let backend = source.and_then(|s| select_backend(s, target_format).ok());

        match backend {
            Some(Backend::LibreOffice) => lo_files.push((path.clone(), name.clone())),
            _ => parallel_files.push((path.clone(), name.clone())),
        }
    }

    // Helper closure to process one file and send progress
    let process_file = |path: &Path, name: &str,
                        counter: &AtomicUsize, success: &AtomicUsize,
                        errs: &Arc<std::sync::Mutex<Vec<(String, String)>>>,
                        err_count: &AtomicUsize,
                        tx: &mpsc::Sender<BatchProgress>| {
        let result = convert_file(path, target_format, &output_dir);
        let idx = counter.fetch_add(1, Ordering::SeqCst) + 1;
        let ok = result.is_ok();

        let error_msg = if let Err(ref e) = result {
            let msg = e.to_string();
            errs.lock().unwrap().push((name.to_string(), msg.clone()));
            err_count.fetch_add(1, Ordering::SeqCst);
            Some(msg)
        } else {
            success.fetch_add(1, Ordering::SeqCst);
            None
        };

        let _ = tx.send(BatchProgress {
            index: idx,
            total,
            file_name: name.to_string(),
            success: ok,
            error_msg,
        });
    };

    // 1) Process LibreOffice files in single batch (one soffice invocation)
    if !lo_files.is_empty() {
        // Prepare batch: (input_path, expected_output_path)
        let batch_inputs: Vec<(PathBuf, PathBuf)> = lo_files.iter().map(|(path, name)| {
            let safe_stem: String = name
                .replace("..", "_")
                .replace('/', "_")
                .replace('\\', "_")
                .chars()
                .map(|c| if c.is_alphanumeric() || c == '-' || c == '_' || c == '.' { c } else { '_' })
                .collect();
            let safe_stem = if safe_stem.trim_matches('.').trim_matches('_').is_empty() {
                "converted".to_string()
            } else {
                safe_stem
            };
            let expected = output_dir.join(format!("{}.{}", safe_stem, target_format.extension()));
            (path.clone(), expected)
        }).collect();

        let batch_results = libreoffice::convert_batch(&batch_inputs, target_format, &output_dir);

        for ((path, _expected), (_, result)) in lo_files.iter().zip(batch_results.iter()) {
            let name = &lo_files.iter().find(|(p, _)| p == path).unwrap().1;
            let idx = counter.fetch_add(1, Ordering::SeqCst) + 1;
            let ok = result.is_ok();

            let error_msg = if let Err(ref e) = result {
                let msg = e.to_string();
                errors.lock().unwrap().push((name.to_string(), msg.clone()));
                error_count.fetch_add(1, Ordering::SeqCst);
                Some(msg)
            } else {
                success_count.fetch_add(1, Ordering::SeqCst);
                None
            };

            let _ = progress_tx.send(BatchProgress {
                index: idx,
                total,
                file_name: name.to_string(),
                success: ok,
                error_msg,
            });
        }
    }

    // 2) Process Pandoc + image files in parallel via rayon
    // par_iter() — like Kotlin's .asSequence().asStream().parallel()
    // rayon automatically distributes work across CPU cores
    parallel_files.par_iter().for_each(|(path, name)| {
        process_file(path, name, &counter, &success_count, &errors, &error_count, &progress_tx);
    });

    BatchResult {
        success_count: success_count.load(Ordering::SeqCst),
        error_count: error_count.load(Ordering::SeqCst),
        errors: Arc::try_unwrap(errors).unwrap().into_inner().unwrap(),
    }
}