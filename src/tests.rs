// ═══════════════════════════════════════════════════════════════════════════
// AnatolevichConvert — Unit Tests
// ═══════════════════════════════════════════════════════════════════════════
// Файл: src/tests.rs
// Подключение: добавить `#[cfg(test)] mod tests;` в src/main.rs
//
// Запуск: cargo test
// Запуск конкретного модуля: cargo test test_file_entry
// Запуск с выводом: cargo test -- --nocapture
// ═══════════════════════════════════════════════════════════════════════════

// ─── file_entry tests ───────────────────────────────────────────────────────
#[cfg(test)]
mod test_file_entry {
    use crate::file_entry::*;
    use std::path::PathBuf;

    // ── format_size ─────────────────────────────────────────────────────
    #[test]
    fn format_size_bytes() {
        assert_eq!(format_size(0), "0 B");
        assert_eq!(format_size(512), "512 B");
        assert_eq!(format_size(1023), "1023 B");
    }

    #[test]
    fn format_size_kilobytes() {
        assert_eq!(format_size(1024), "1.0 KB");
        assert_eq!(format_size(1536), "1.5 KB");
        assert_eq!(format_size(10240), "10.0 KB");
    }

    #[test]
    fn format_size_megabytes() {
        assert_eq!(format_size(1024 * 1024), "1.0 MB");
        assert_eq!(format_size(5 * 1024 * 1024), "5.0 MB");
        assert_eq!(format_size(1024 * 1024 + 512 * 1024), "1.5 MB");
    }

    #[test]
    fn format_size_gigabytes() {
        assert_eq!(format_size(1024 * 1024 * 1024), "1.0 GB");
        assert_eq!(format_size(2 * 1024 * 1024 * 1024), "2.0 GB");
    }

    // ── FileEntry creation ──────────────────────────────────────────────
    #[test]
    fn file_entry_from_real_file() {
        // Создаём временный файл
        let dir = std::env::temp_dir().join("anatolevich_test");
        let _ = std::fs::create_dir_all(&dir);
        let file_path = dir.join("test_document.docx");
        std::fs::write(&file_path, "hello world").unwrap();

        let entry = FileEntry::from_path(file_path.clone()).unwrap();
        assert_eq!(entry.name, "test_document.docx");
        assert_eq!(entry.size, 11); // "hello world" = 11 bytes
        assert_eq!(entry.path, file_path);

        let _ = std::fs::remove_file(&file_path);
        let _ = std::fs::remove_dir(&dir);
    }

    #[test]
    fn file_entry_from_directory_returns_none() {
        let dir = std::env::temp_dir();
        assert!(FileEntry::from_path(dir.to_path_buf()).is_none());
    }

    #[test]
    fn file_entry_from_nonexistent_returns_none() {
        let fake = PathBuf::from("/tmp/this_file_definitely_does_not_exist_12345.txt");
        assert!(FileEntry::from_path(fake).is_none());
    }

    // ── extension_upper ─────────────────────────────────────────────────
    #[test]
    fn extension_upper_various() {
        let make = |name: &str| FileEntry {
            path: PathBuf::from(name),
            name: name.to_string(),
            size: 0,
        };

        assert_eq!(make("report.pdf").extension_upper(), "PDF");
        assert_eq!(make("photo.JPG").extension_upper(), "JPG");
        assert_eq!(make("data.xlsx").extension_upper(), "XLSX");
        assert_eq!(make("slides.pptx").extension_upper(), "PPTX");
        assert_eq!(make("image.WebP").extension_upper(), "WEBP");
        assert_eq!(make("noext").extension_upper(), "???");
    }

    // ── badge_css_class ─────────────────────────────────────────────────
    #[test]
    fn badge_classes() {
        let make = |name: &str| FileEntry {
            path: PathBuf::from(name),
            name: name.to_string(),
            size: 0,
        };

        assert_eq!(make("a.pdf").badge_css_class(), "badge-document");
        assert_eq!(make("a.docx").badge_css_class(), "badge-document");
        assert_eq!(make("a.txt").badge_css_class(), "badge-document");
        assert_eq!(make("a.md").badge_css_class(), "badge-document");
        assert_eq!(make("a.epub").badge_css_class(), "badge-document");

        assert_eq!(make("a.xlsx").badge_css_class(), "badge-spreadsheet");
        assert_eq!(make("a.csv").badge_css_class(), "badge-spreadsheet");
        assert_eq!(make("a.ods").badge_css_class(), "badge-spreadsheet");

        assert_eq!(make("a.pptx").badge_css_class(), "badge-presentation");
        assert_eq!(make("a.odp").badge_css_class(), "badge-presentation");

        assert_eq!(make("a.jpg").badge_css_class(), "badge-image");
        assert_eq!(make("a.png").badge_css_class(), "badge-image");
        assert_eq!(make("a.svg").badge_css_class(), "badge-image");
        assert_eq!(make("a.webp").badge_css_class(), "badge-image");

        assert_eq!(make("a.xyz").badge_css_class(), "badge-unknown");
        assert_eq!(make("noext").badge_css_class(), "badge-unknown");
    }

    // ── size_display ────────────────────────────────────────────────────
    #[test]
    fn size_display_formatting() {
        let make = |size: u64| FileEntry {
            path: PathBuf::from("test.txt"),
            name: "test.txt".to_string(),
            size,
        };

        assert_eq!(make(100).size_display(), "100 B");
        assert_eq!(make(2048).size_display(), "2.0 KB");
        assert_eq!(make(3 * 1024 * 1024).size_display(), "3.0 MB");
    }

    // ── format_status ───────────────────────────────────────────────────
    #[test]
    fn format_status_empty() {
        let status = format_status(&[]);
        assert!(status.contains("не выбраны"));
    }

    #[test]
    fn format_status_with_files() {
        let files = vec![
            FileEntry { path: PathBuf::from("a.pdf"), name: "a.pdf".to_string(), size: 1024 },
            FileEntry { path: PathBuf::from("b.docx"), name: "b.docx".to_string(), size: 2048 },
        ];
        let status = format_status(&files);
        assert!(status.contains("2"));
        assert!(status.contains("3.0 KB")); // 1024 + 2048 = 3072 = 3.0 KB
    }
}

// ─── formats tests ──────────────────────────────────────────────────────────
#[cfg(test)]
mod test_formats {
    use crate::formats;

    // ── format_from_display ─────────────────────────────────────────────
    #[test]
    fn format_from_display_extracts_format() {
        assert_eq!(formats::format_from_display("📄 PDF"), "PDF");
        assert_eq!(formats::format_from_display("📊 XLSX"), "XLSX");
        assert_eq!(formats::format_from_display("🖼 WebP"), "WebP");
        assert_eq!(formats::format_from_display("📽 PPTX"), "PPTX");
    }

    #[test]
    fn format_from_display_fallback() {
        assert_eq!(formats::format_from_display(""), "PDF");
    }

    // ── build_format_list ───────────────────────────────────────────────
    #[test]
    fn build_format_list_not_empty() {
        let list = formats::build_format_list();
        assert!(!list.is_empty());
        // Should contain all major formats
        assert!(list.iter().any(|s| s.contains("PDF")));
        assert!(list.iter().any(|s| s.contains("DOCX")));
        assert!(list.iter().any(|s| s.contains("XLSX")));
        assert!(list.iter().any(|s| s.contains("PPTX")));
        assert!(list.iter().any(|s| s.contains("JPG")));
        assert!(list.iter().any(|s| s.contains("PNG")));
    }

    #[test]
    fn build_format_list_has_icons() {
        let list = formats::build_format_list();
        for item in &list {
            // Each item should have "icon format" pattern
            assert!(item.contains(' '), "No space in: {}", item);
        }
    }

    // ── build_filtered_format_list ──────────────────────────────────────
    #[test]
    fn filtered_images_only() {
        let exts = vec!["JPG".to_string(), "PNG".to_string()];
        let list = formats::build_filtered_format_list(Some(&exts));
        // Should include image formats + PDF
        assert!(list.iter().any(|s| s.contains("PNG")));
        assert!(list.iter().any(|s| s.contains("WebP")));
        assert!(list.iter().any(|s| s.contains("PDF")));
        // Should NOT include DOCX, XLSX, PPTX
        assert!(!list.iter().any(|s| s.contains("DOCX")));
        assert!(!list.iter().any(|s| s.contains("XLSX")));
        assert!(!list.iter().any(|s| s.contains("PPTX")));
    }

    #[test]
    fn filtered_spreadsheets_only() {
        let exts = vec!["XLSX".to_string(), "CSV".to_string()];
        let list = formats::build_filtered_format_list(Some(&exts));
        assert!(list.iter().any(|s| s.contains("XLSX")));
        assert!(list.iter().any(|s| s.contains("CSV")));
        assert!(list.iter().any(|s| s.contains("ODS")));
        assert!(list.iter().any(|s| s.contains("PDF")));
        assert!(!list.iter().any(|s| s.contains("JPG")));
    }

    #[test]
    fn filtered_presentations_only() {
        let exts = vec!["PPTX".to_string()];
        let list = formats::build_filtered_format_list(Some(&exts));
        assert!(list.iter().any(|s| s.contains("PPTX")));
        assert!(list.iter().any(|s| s.contains("ODP")));
        assert!(list.iter().any(|s| s.contains("PDF")));
        assert!(!list.iter().any(|s| s.contains("XLSX")));
    }

    #[test]
    fn filtered_documents() {
        let exts = vec!["DOCX".to_string()];
        let list = formats::build_filtered_format_list(Some(&exts));
        assert!(list.iter().any(|s| s.contains("PDF")));
        assert!(list.iter().any(|s| s.contains("ODT")));
        assert!(list.iter().any(|s| s.contains("TXT")));
        assert!(!list.iter().any(|s| s.contains("JPG")));
    }

    #[test]
    fn filtered_none_returns_all() {
        let all = formats::build_format_list();
        let filtered = formats::build_filtered_format_list(None);
        assert_eq!(all.len(), filtered.len());
    }

    #[test]
    fn filtered_empty_fallback() {
        let exts: Vec<String> = vec![];
        let list = formats::build_filtered_format_list(Some(&exts));
        // Empty extensions → all formats (fallback)
        assert!(!list.is_empty());
    }

    // ── recommend_format ────────────────────────────────────────────────
    #[test]
    fn recommend_empty_files() {
        let (fmt, _reason) = formats::recommend_format(&[]);
        assert_eq!(fmt, "PDF");
    }

    #[test]
    fn recommend_images_png() {
        let exts = vec!["PNG".to_string(), "PNG".to_string()];
        let (fmt, reason) = formats::recommend_format(&exts);
        assert_eq!(fmt, "WebP");
        assert!(reason.contains("WebP"));
    }

    #[test]
    fn recommend_images_svg() {
        let exts = vec!["SVG".to_string()];
        let (fmt, _) = formats::recommend_format(&exts);
        assert_eq!(fmt, "PNG");
    }

    #[test]
    fn recommend_images_jpg() {
        let exts = vec!["JPG".to_string(), "JPEG".to_string()];
        let (fmt, _) = formats::recommend_format(&exts);
        assert_eq!(fmt, "WebP");
    }

    #[test]
    fn recommend_spreadsheets_csv() {
        let exts = vec!["CSV".to_string()];
        let (fmt, _) = formats::recommend_format(&exts);
        assert_eq!(fmt, "XLSX");
    }

    #[test]
    fn recommend_spreadsheets_xlsx() {
        let exts = vec!["XLSX".to_string()];
        let (fmt, _) = formats::recommend_format(&exts);
        assert_eq!(fmt, "PDF");
    }

    #[test]
    fn recommend_presentations() {
        let exts = vec!["PPTX".to_string(), "PPTX".to_string()];
        let (fmt, reason) = formats::recommend_format(&exts);
        assert_eq!(fmt, "PDF");
        assert!(reason.contains("PDF"));
    }

    #[test]
    fn recommend_documents_txt() {
        let exts = vec!["TXT".to_string()];
        let (fmt, _) = formats::recommend_format(&exts);
        assert_eq!(fmt, "PDF");
    }

    #[test]
    fn recommend_documents_html() {
        let exts = vec!["HTML".to_string()];
        let (fmt, _) = formats::recommend_format(&exts);
        assert_eq!(fmt, "PDF");
    }

    #[test]
    fn recommend_documents_docx() {
        let exts = vec!["DOCX".to_string()];
        let (fmt, _) = formats::recommend_format(&exts);
        assert_eq!(fmt, "PDF");
    }

    #[test]
    fn recommend_mixed_files() {
        let exts = vec!["JPG".to_string(), "DOCX".to_string(), "XLSX".to_string()];
        let (fmt, reason) = formats::recommend_format(&exts);
        assert_eq!(fmt, "PDF");
        assert!(reason.contains("Смешанные"));
    }
}

// ─── converter tests ────────────────────────────────────────────────────────
#[cfg(test)]
mod test_converter {
    use crate::converter::*;
    use std::path::Path;

    // ── Format::from_extension ──────────────────────────────────────────
    #[test]
    fn format_from_extension_valid() {
        assert_eq!(Format::from_extension("pdf"), Some(Format::Pdf));
        assert_eq!(Format::from_extension("PDF"), Some(Format::Pdf));
        assert_eq!(Format::from_extension("Pdf"), Some(Format::Pdf));
        assert_eq!(Format::from_extension("docx"), Some(Format::Docx));
        assert_eq!(Format::from_extension("jpg"), Some(Format::Jpg));
        assert_eq!(Format::from_extension("jpeg"), Some(Format::Jpg));
        assert_eq!(Format::from_extension("png"), Some(Format::Png));
        assert_eq!(Format::from_extension("webp"), Some(Format::WebP));
        assert_eq!(Format::from_extension("md"), Some(Format::Md));
        assert_eq!(Format::from_extension("markdown"), Some(Format::Md));
        assert_eq!(Format::from_extension("htm"), Some(Format::Html));
        assert_eq!(Format::from_extension("html"), Some(Format::Html));
        assert_eq!(Format::from_extension("tif"), Some(Format::Tiff));
        assert_eq!(Format::from_extension("tiff"), Some(Format::Tiff));
    }

    #[test]
    fn format_from_extension_invalid() {
        assert_eq!(Format::from_extension(""), None);
        assert_eq!(Format::from_extension("xyz"), None);
        assert_eq!(Format::from_extension("mp3"), None);
        assert_eq!(Format::from_extension("avi"), None);
    }

    // ── Format::extension ───────────────────────────────────────────────
    #[test]
    fn format_extension_roundtrip() {
        let formats = [
            Format::Pdf, Format::Docx, Format::Odt, Format::Rtf,
            Format::Txt, Format::Md, Format::Html, Format::Epub,
            Format::Xlsx, Format::Csv, Format::Ods,
            Format::Pptx, Format::Odp,
            Format::Jpg, Format::Png, Format::WebP, Format::Svg, Format::Bmp, Format::Tiff,
        ];

        for fmt in formats {
            let ext = fmt.extension();
            let parsed = Format::from_extension(ext);
            assert_eq!(parsed, Some(fmt), "Roundtrip failed for {:?} → {} → {:?}", fmt, ext, parsed);
        }
    }

    // ── Format::detect ──────────────────────────────────────────────────
    #[test]
    fn format_detect_from_path() {
        assert_eq!(Format::detect(Path::new("report.pdf")), Some(Format::Pdf));
        assert_eq!(Format::detect(Path::new("/home/user/doc.docx")), Some(Format::Docx));
        assert_eq!(Format::detect(Path::new("photo.JPEG")), Some(Format::Jpg));
        assert_eq!(Format::detect(Path::new("image.TIF")), Some(Format::Tiff));
        assert_eq!(Format::detect(Path::new("noext")), None);
        assert_eq!(Format::detect(Path::new("video.mp4")), None);
    }

    // ── Format::is_image ────────────────────────────────────────────────
    #[test]
    fn format_is_image() {
        assert!(Format::Jpg.is_image());
        assert!(Format::Png.is_image());
        assert!(Format::WebP.is_image());
        assert!(Format::Svg.is_image());
        assert!(Format::Bmp.is_image());
        assert!(Format::Tiff.is_image());

        assert!(!Format::Pdf.is_image());
        assert!(!Format::Docx.is_image());
        assert!(!Format::Xlsx.is_image());
        assert!(!Format::Pptx.is_image());
        assert!(!Format::Txt.is_image());
    }

    // ── select_backend ──────────────────────────────────────────────────
    #[test]
    fn backend_image_to_image() {
        assert_eq!(select_backend(Format::Jpg, Format::Png).unwrap(), Backend::ImageCrate);
        assert_eq!(select_backend(Format::Png, Format::WebP).unwrap(), Backend::ImageCrate);
        assert_eq!(select_backend(Format::Bmp, Format::Tiff).unwrap(), Backend::ImageCrate);
    }

    #[test]
    fn backend_pandoc_sources() {
        assert_eq!(select_backend(Format::Md, Format::Pdf).unwrap(), Backend::Pandoc);
        assert_eq!(select_backend(Format::Html, Format::Docx).unwrap(), Backend::Pandoc);
        assert_eq!(select_backend(Format::Txt, Format::Html).unwrap(), Backend::Pandoc);
        assert_eq!(select_backend(Format::Epub, Format::Pdf).unwrap(), Backend::Pandoc);
    }

    #[test]
    fn backend_libreoffice() {
        assert_eq!(select_backend(Format::Docx, Format::Pdf).unwrap(), Backend::LibreOffice);
        assert_eq!(select_backend(Format::Xlsx, Format::Pdf).unwrap(), Backend::LibreOffice);
        assert_eq!(select_backend(Format::Pptx, Format::Pdf).unwrap(), Backend::LibreOffice);
        assert_eq!(select_backend(Format::Odt, Format::Docx).unwrap(), Backend::LibreOffice);
        assert_eq!(select_backend(Format::Csv, Format::Xlsx).unwrap(), Backend::LibreOffice);
    }

    #[test]
    fn backend_same_format_error() {
        assert!(select_backend(Format::Pdf, Format::Pdf).is_err());
        assert!(select_backend(Format::Jpg, Format::Jpg).is_err());
    }

    #[test]
    fn backend_unsupported_conversion_error() {
        // Image to document without going through LO
        assert!(select_backend(Format::Jpg, Format::Docx).is_err());
    }

    // ── convert_file error handling ─────────────────────────────────────
    #[test]
    fn convert_nonexistent_file() {
        let result = convert_file(
            Path::new("/nonexistent/file.docx"),
            Format::Pdf,
            Path::new("/tmp"),
        );
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("not found"), "Error was: {}", err);
    }
}

// ─── conversion module tests ────────────────────────────────────────────────
#[cfg(test)]
mod test_conversion {
    use crate::conversion::apply_rename_template;

    #[test]
    fn rename_empty_template() {
        assert_eq!(
            apply_rename_template("", "document.pdf", 0),
            "document.pdf"
        );
    }

    #[test]
    fn rename_whitespace_template() {
        assert_eq!(
            apply_rename_template("   ", "document.pdf", 0),
            "document.pdf"
        );
    }

    #[test]
    fn rename_name_placeholder() {
        assert_eq!(
            apply_rename_template("{name}_converted", "report.docx", 0),
            "report_converted"
        );
    }

    #[test]
    fn rename_number_placeholder() {
        assert_eq!(
            apply_rename_template("file_{n}", "anything.pdf", 0),
            "file_1"
        );
        assert_eq!(
            apply_rename_template("file_{n}", "anything.pdf", 4),
            "file_5"
        );
    }

    #[test]
    fn rename_combined_placeholders() {
        let result = apply_rename_template("{name}_{n}", "photo.jpg", 2);
        assert_eq!(result, "photo_3");
    }

    #[test]
    fn rename_date_placeholder() {
        let result = apply_rename_template("{date}_backup", "data.xlsx", 0);
        // Date should be YYYY-MM-DD format
        assert!(result.contains("_backup"));
        assert!(result.len() > 11); // "YYYY-MM-DD_backup" = 17 chars
    }

    #[test]
    fn rename_name_strips_extension() {
        // {name} should be the stem, not including extension
        let result = apply_rename_template("{name}", "archive.tar.gz", 0);
        // file_stem of "archive.tar.gz" is "archive.tar"
        assert_eq!(result, "archive.tar");
    }

    #[test]
    fn rename_no_placeholders() {
        assert_eq!(
            apply_rename_template("fixed_name", "original.pdf", 0),
            "fixed_name"
        );
    }

    #[test]
    fn rename_all_placeholders() {
        let result = apply_rename_template("{name}_{n}_{date}", "test.txt", 9);
        assert!(result.starts_with("test_10_"));
        // Contains date
        assert!(result.len() > 10);
    }
}

// ─── history tests ──────────────────────────────────────────────────────────
#[cfg(test)]
mod test_history {
    use crate::history;
    use std::sync::Mutex;

    // History tests share the same file, so they must run sequentially
    static HISTORY_LOCK: Mutex<()> = Mutex::new(());

    #[test]
    fn log_and_read_history() {
        let _lock = HISTORY_LOCK.lock().unwrap();
        history::clear_history();

        history::log_conversion("test.docx", "PDF", true, None);
        history::log_conversion("fail.xlsx", "CSV", false, Some("timeout"));

        let entries = history::read_history();
        assert!(entries.len() >= 2);

        // Newest first
        assert!(entries[0].contains("fail.xlsx"));
        assert!(entries[0].contains("❌"));
        assert!(entries[1].contains("test.docx"));
        assert!(entries[1].contains("✅"));

        history::clear_history();
    }

    #[test]
    fn clear_history_works() {
        let _lock = HISTORY_LOCK.lock().unwrap();
        history::log_conversion("tmp.pdf", "DOCX", true, None);
        history::clear_history();
        let entries = history::read_history();
        assert!(entries.is_empty() || entries.iter().all(|e| e.trim().is_empty()));
    }

    #[test]
    fn history_success_format() {
        let _lock = HISTORY_LOCK.lock().unwrap();
        history::clear_history();
        history::log_conversion("report.docx", "PDF", true, None);
        let entries = history::read_history();
        assert!(!entries.is_empty());
        assert!(entries[0].contains("✅"));
        assert!(entries[0].contains("report.docx"));
        assert!(entries[0].contains("PDF"));
        history::clear_history();
    }

    #[test]
    fn history_error_format() {
        let _lock = HISTORY_LOCK.lock().unwrap();
        history::clear_history();
        history::log_conversion("broken.pptx", "PDF", false, Some("LibreOffice timeout"));
        let entries = history::read_history();
        assert!(!entries.is_empty());
        assert!(entries[0].contains("❌"), "Entry was: {}", entries[0]);
        assert!(entries[0].contains("broken.pptx"));
        assert!(entries[0].contains("LibreOffice timeout"));
        history::clear_history();
    }
}

// ─── settings tests (without GTK) ──────────────────────────────────────────
#[cfg(test)]
mod test_settings {
    use crate::settings::*;

    #[test]
    fn default_settings() {
        let s = AppSettings::default();
        assert!(!s.dark_theme);
        assert!(s.wallpaper_path.is_none());
    }

    #[test]
    fn config_dir_exists() {
        let dir = config_dir();
        assert!(dir.is_some());
        let dir = dir.unwrap();
        assert!(dir.to_string_lossy().contains("anatolevich-convert"));
    }

    #[test]
    fn wallpaper_css_none() {
        let css = build_wallpaper_css(&None);
        assert!(css.contains(".main-container"));
        assert!(!css.contains("background-image"));
    }

    #[test]
    fn wallpaper_css_with_path() {
        let css = build_wallpaper_css(&Some("/home/user/wallpaper.jpg".to_string()));
        assert!(css.contains("background-image"));
        assert!(css.contains("wallpaper.jpg"));
        assert!(css.contains("cover"));
    }

    #[test]
    fn wallpaper_css_escapes_special_chars() {
        let css = build_wallpaper_css(&Some("/path/with spaces/wall'paper.jpg".to_string()));
        assert!(css.contains("background-image"));
        // Should handle the apostrophe
        assert!(!css.contains("wall'paper")); // should be escaped
    }

    #[test]
    fn save_and_load_settings_roundtrip() {
        let original = AppSettings {
            dark_theme: true,
            wallpaper_path: None, // Can't test with path since it checks exists()
        };
        save_settings(&original);
        let loaded = load_settings();
        assert_eq!(loaded.dark_theme, original.dark_theme);
    }
}

// ─── notifications tests ────────────────────────────────────────────────────
#[cfg(test)]
mod test_notifications {
    use crate::notifications;

    #[test]
    fn open_folder_does_not_panic() {
        // Just verify it doesn't crash — xdg-open may not be available in test env
        notifications::open_folder("/tmp");
    }
}

// ─── security tests ─────────────────────────────────────────────────────────
#[cfg(test)]
mod test_security {
    use crate::file_entry::FileEntry;
    use crate::conversion::apply_rename_template;
    use std::path::PathBuf;

    // ── Path traversal in filenames ─────────────────────────────────────
    #[test]
    fn reject_path_traversal_in_name() {
        let dir = std::env::temp_dir().join("anatolevich_sec_test");
        let _ = std::fs::create_dir_all(&dir);
        let bad_path = dir.join("..%2f..%2fetc%2fpasswd");
        let _ = std::fs::write(&bad_path, "test");
        // Should still create entry (% is valid in filenames)
        // but the name should not contain actual path separators
        if let Some(entry) = FileEntry::from_path(bad_path.clone()) {
            assert!(!entry.name.contains('/'));
            assert!(!entry.name.contains('\\'));
        }
        let _ = std::fs::remove_file(&bad_path);
        let _ = std::fs::remove_dir(&dir);
    }

    // ── Symlink rejection ───────────────────────────────────────────────
    #[test]
    fn reject_symlinks() {
        let dir = std::env::temp_dir().join("anatolevich_sym_test");
        let _ = std::fs::create_dir_all(&dir);
        let target = dir.join("real_file.txt");
        let link = dir.join("link_file.txt");
        std::fs::write(&target, "content").unwrap();

        #[cfg(unix)]
        {
            let _ = std::os::unix::fs::symlink(&target, &link);
            let entry = FileEntry::from_path(link.clone());
            assert!(entry.is_none(), "Symlinks should be rejected");
            let _ = std::fs::remove_file(&link);
        }

        let _ = std::fs::remove_file(&target);
        let _ = std::fs::remove_dir(&dir);
    }

    // ── Rename template path traversal ──────────────────────────────────
    #[test]
    fn rename_template_no_traversal() {
        let result = apply_rename_template("../../etc/{name}", "passwd", 0);
        assert!(!result.contains("../"));
        assert!(!result.contains("..\\"));
    }

    #[test]
    fn rename_template_no_slashes() {
        let result = apply_rename_template("{name}/../../secret", "test.txt", 0);
        assert!(!result.contains('/'));
    }

    #[test]
    fn rename_template_no_control_chars() {
        let result = apply_rename_template("test\n\r\x00inject", "file.txt", 0);
        assert!(!result.chars().any(|c| c.is_control()));
    }

    // ── History log injection ───────────────────────────────────────────
    #[test]
    fn history_no_newline_injection() {
        use crate::history;
        use std::sync::Mutex;
        static LOCK: Mutex<()> = Mutex::new(());
        let _lock = LOCK.lock().unwrap();

        history::clear_history();
        // Try to inject a fake log entry via filename
        history::log_conversion(
            "evil.pdf\n2025-01-01 00:00:00 ✅ FAKE_ENTRY → PDF",
            "PDF",
            true,
            None,
        );
        let entries = history::read_history();
        // Should be exactly 1 entry, not 2
        let non_empty: Vec<_> = entries.iter().filter(|e| !e.trim().is_empty()).collect();
        assert_eq!(non_empty.len(), 1, "Log injection detected! Entries: {:?}", entries);
        history::clear_history();
    }
}