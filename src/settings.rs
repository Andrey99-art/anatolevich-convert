use gtk::CssProvider;
use std::path::{Path, PathBuf};

const CONFIG_DIR: &str = "anatolevich-convert";
const LAST_DIR_FILE: &str = "last_dir.txt";
const SETTINGS_FILE: &str = "settings.conf";

pub const CSS_BADGES: &str = include_str!("../styles/badges.css");
pub const CSS_LIGHT: &str = include_str!("../styles/light.css");
pub const CSS_DARK: &str = include_str!("../styles/dark.css");

#[derive(Debug, Clone)]
pub struct AppSettings {
    pub dark_theme: bool,
    pub wallpaper_path: Option<String>,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self { dark_theme: false, wallpaper_path: None }
    }
}

pub fn config_dir() -> Option<PathBuf> {
    std::env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".config")))
        .map(|d| d.join(CONFIG_DIR))
}

pub fn save_last_dir(dir: &Path) {
    if let Some(config) = config_dir() {
        let _ = std::fs::create_dir_all(&config);
        let _ = std::fs::write(config.join(LAST_DIR_FILE), dir.to_string_lossy().as_bytes());
    }
}

pub fn load_last_dir() -> Option<PathBuf> {
    let config = config_dir()?;
    let path = std::fs::read_to_string(config.join(LAST_DIR_FILE)).ok()?;
    let path = PathBuf::from(path.trim());
    if path.is_dir() { Some(path) } else { None }
}

pub fn save_settings(settings: &AppSettings) {
    if let Some(config) = config_dir() {
        let _ = std::fs::create_dir_all(&config);
        let content = format!(
            "dark_theme={}\nwallpaper_path={}\n",
            settings.dark_theme,
            settings.wallpaper_path.as_deref().unwrap_or("")
        );
        let _ = std::fs::write(config.join(SETTINGS_FILE), content);
    }
}

pub fn load_settings() -> AppSettings {
    let config = match config_dir() { Some(c) => c, None => return AppSettings::default() };
    let content = match std::fs::read_to_string(config.join(SETTINGS_FILE)) {
        Ok(c) => c, Err(_) => return AppSettings::default(),
    };
    let mut settings = AppSettings::default();
    for line in content.lines() {
        if let Some((key, value)) = line.split_once('=') {
            match key.trim() {
                "dark_theme" => settings.dark_theme = value.trim() == "true",
                "wallpaper_path" => {
                    let v = value.trim();
                    if !v.is_empty() && PathBuf::from(v).exists() {
                        settings.wallpaper_path = Some(v.to_string());
                    }
                }
                _ => {}
            }
        }
    }
    settings
}

pub fn apply_theme(dark: bool, theme_css: &CssProvider) {
    theme_css.load_from_string(if dark { CSS_DARK } else { CSS_LIGHT });
    if let Some(s) = gtk::Settings::default() {
        s.set_gtk_application_prefer_dark_theme(dark);
    }
}

pub fn build_wallpaper_css(wallpaper_path: &Option<String>) -> String {
    match wallpaper_path {
        Some(path) => {
            let escaped = path.replace('\\', "\\\\").replace('\'', "\\'");
            format!(
                r#".main-container {{
                    background-image: url('file://{}');
                    background-size: cover;
                    background-position: center;
                    background-repeat: no-repeat;
                }}
                .main-container > * {{
                    background-color: alpha(@window_bg_color, 0.85);
                    border-radius: 8px;
                    margin: 4px;
                }}"#, escaped
            )
        }
        None => ".main-container { }".to_string(),
    }
}