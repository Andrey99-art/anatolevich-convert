use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct FileEntry {
    pub path: PathBuf,
    pub name: String,
    pub size: u64,
}

impl FileEntry {
   pub fn from_path(path: PathBuf) -> Option<Self> {
        if path.is_dir() { return None; }

        // Security: reject symlinks to prevent symlink attacks
        if path.is_symlink() { return None; }

        // Security: reject paths with traversal components
        let name = path.file_name()?.to_string_lossy().to_string();
        if name.contains("..") || name.contains('/') || name.contains('\\') {
            return None;
        }

        let size = std::fs::metadata(&path).ok()?.len();
        Some(Self { path, name, size })
    }

    pub fn size_display(&self) -> String {
        format_size(self.size)
    }

    pub fn extension_upper(&self) -> String {
        self.path.extension()
            .map(|ext| ext.to_string_lossy().to_uppercase())
            .unwrap_or_else(|| "???".to_string())
    }

    pub fn badge_css_class(&self) -> &'static str {
        match self.extension_upper().as_str() {
            "PDF" | "DOCX" | "DOC" | "ODT" | "RTF" | "TXT" | "MD" | "HTML" | "HTM" | "EPUB" => "badge-document",
            "XLSX" | "XLS" | "CSV" | "ODS" => "badge-spreadsheet",
            "PPTX" | "PPT" | "ODP" => "badge-presentation",
            "JPG" | "JPEG" | "PNG" | "WEBP" | "SVG" | "BMP" | "TIFF" | "TIF" => "badge-image",
            _ => "badge-unknown",
        }
    }
}

pub fn format_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;
    match bytes {
        s if s >= GB => format!("{:.1} GB", s as f64 / GB as f64),
        s if s >= MB => format!("{:.1} MB", s as f64 / MB as f64),
        s if s >= KB => format!("{:.1} KB", s as f64 / KB as f64),
        s => format!("{} B", s),
    }
}

pub fn format_status(files: &[FileEntry]) -> String {
    let count = files.len();
    if count == 0 {
        return "Файлы не выбраны — перетащите файлы сюда или нажмите «Выбрать файлы»".to_string();
    }
    let total_bytes: u64 = files.iter().map(|f| f.size).sum();
    format!("Выбрано файлов: {} ({})", count, format_size(total_bytes))
}