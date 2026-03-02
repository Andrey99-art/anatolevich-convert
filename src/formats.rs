pub const TARGET_FORMATS: &[(&str, &[&str])] = &[
    ("📄", &["PDF", "DOCX", "ODT", "RTF", "TXT", "MD", "HTML", "EPUB"]),
    ("📊", &["XLSX", "CSV", "ODS"]),
    ("📽", &["PPTX", "ODP"]),
    ("🖼", &["JPG", "PNG", "WebP", "SVG", "BMP", "TIFF"]),
];

pub fn build_format_list() -> Vec<String> {
    build_filtered_format_list(None)
}

/// Build format list, optionally filtered by source file extensions.
/// If extensions is None, returns all formats.
pub fn build_filtered_format_list(extensions: Option<&[String]>) -> Vec<String> {
    let allowed = match extensions {
        Some(exts) => get_compatible_targets(exts),
        None => {
            // All formats
            let mut all = Vec::new();
            for (_, formats) in TARGET_FORMATS {
                for fmt in *formats {
                    all.push(fmt.to_string());
                }
            }
            all
        }
    };

    let mut list = Vec::new();
    for (icon, formats) in TARGET_FORMATS {
        for fmt in *formats {
            if allowed.iter().any(|a| a.eq_ignore_ascii_case(fmt)) {
                list.push(format!("{} {}", icon, fmt));
            }
        }
    }

    // If filtering removed everything, return all formats as fallback
    if list.is_empty() {
        return build_filtered_format_list(None);
    }
    list
}

/// Determine which target formats are compatible with the given source extensions.
///
/// Rules:
/// - Image files (JPG, PNG, etc.) → image formats + PDF
/// - Document files (DOCX, ODT, TXT, MD, etc.) → document formats
/// - Spreadsheet files (XLSX, CSV, etc.) → spreadsheet formats + PDF
/// - Presentation files (PPTX, ODP) → presentation formats + PDF
/// - Mixed files → union of all compatible formats
fn get_compatible_targets(extensions: &[String]) -> Vec<String> {
    let mut targets = std::collections::HashSet::new();

    for ext in extensions {
        let upper = ext.to_uppercase();
        match upper.as_str() {
            // Images → image formats + PDF
            "JPG" | "JPEG" | "PNG" | "WEBP" | "SVG" | "BMP" | "TIFF" | "TIF" => {
                for fmt in ["JPG", "PNG", "WebP", "SVG", "BMP", "TIFF", "PDF"] {
                    targets.insert(fmt.to_string());
                }
            }
            // Spreadsheets → spreadsheet formats + PDF
            "XLSX" | "XLS" | "CSV" | "ODS" => {
                for fmt in ["XLSX", "CSV", "ODS", "PDF"] {
                    targets.insert(fmt.to_string());
                }
            }
            // Presentations → presentation formats + PDF
            "PPTX" | "PPT" | "ODP" => {
                for fmt in ["PPTX", "ODP", "PDF"] {
                    targets.insert(fmt.to_string());
                }
            }
            // Documents (everything else) → document formats
            _ => {
                for fmt in ["PDF", "DOCX", "ODT", "RTF", "TXT", "MD", "HTML", "EPUB"] {
                    targets.insert(fmt.to_string());
                }
            }
        }
    }

    targets.into_iter().collect()
}

pub fn format_from_display(display: &str) -> String {
    display.split_whitespace().last().unwrap_or("PDF").to_string()
}

/// Smart format recommendation based on file extensions.
/// Returns (recommended_format, explanation).
pub fn recommend_format(extensions: &[String]) -> (String, String) {
    if extensions.is_empty() {
        return ("PDF".to_string(), "PDF — универсальный формат по умолчанию".to_string());
    }

    // Count file categories
    let mut docs = 0u32;
    let mut sheets = 0u32;
    let mut slides = 0u32;
    let mut images = 0u32;

    for ext in extensions {
        match ext.to_uppercase().as_str() {
            "PDF" | "DOCX" | "DOC" | "ODT" | "RTF" | "TXT" | "MD" | "HTML" | "HTM" | "EPUB" => docs += 1,
            "XLSX" | "XLS" | "CSV" | "ODS" => sheets += 1,
            "PPTX" | "PPT" | "ODP" => slides += 1,
            "JPG" | "JPEG" | "PNG" | "WEBP" | "SVG" | "BMP" | "TIFF" | "TIF" => images += 1,
            _ => docs += 1,
        }
    }

    let total = docs + sheets + slides + images;

    // Pure images
    if images == total {
        let has_png = extensions.iter().any(|e| e.eq_ignore_ascii_case("png"));
        let has_svg = extensions.iter().any(|e| e.eq_ignore_ascii_case("svg"));

        if has_svg {
            return ("PNG".to_string(), "SVG → PNG: растеризация для совместимости".to_string());
        }
        if has_png {
            return ("WebP".to_string(), "PNG → WebP: сжатие без потерь, меньший размер".to_string());
        }
        return ("WebP".to_string(), "WebP: лучшее сжатие при сохранении качества".to_string());
    }

    // Pure spreadsheets
    if sheets == total {
        let has_csv = extensions.iter().any(|e| e.eq_ignore_ascii_case("csv"));
        if has_csv {
            return ("XLSX".to_string(), "CSV → XLSX: сохранение структуры и форматирования".to_string());
        }
        return ("PDF".to_string(), "Таблицы → PDF: удобно для отправки и печати".to_string());
    }

    // Pure presentations
    if slides == total {
        return ("PDF".to_string(), "Презентации → PDF: удобно для просмотра без PowerPoint".to_string());
    }

    // Pure documents
    if docs == total {
        let has_txt = extensions.iter().any(|e| {
            let u = e.to_uppercase();
            u == "TXT" || u == "MD"
        });
        let has_docx = extensions.iter().any(|e| e.eq_ignore_ascii_case("docx"));
        let has_html = extensions.iter().any(|e| {
            let u = e.to_uppercase();
            u == "HTML" || u == "HTM"
        });

        if has_txt {
            return ("PDF".to_string(), "Текст → PDF: финальный формат для публикации".to_string());
        }
        if has_html {
            return ("PDF".to_string(), "HTML → PDF: сохранение вёрстки для печати".to_string());
        }
        if has_docx {
            return ("PDF".to_string(), "DOCX → PDF: универсальный формат для обмена".to_string());
        }
        return ("PDF".to_string(), "Документы → PDF: универсальный формат".to_string());
    }

    // Mixed files — PDF is always safe
    ("PDF".to_string(), "Смешанные файлы → PDF: единый формат для всех типов".to_string())
}