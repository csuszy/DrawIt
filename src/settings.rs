use std::path::PathBuf;
use std::sync::Mutex;

#[derive(Clone, Debug, PartialEq)]
pub struct AppSettings {
    pub global_hotkey: String, // "Ctrl+2", "F8", "Ctrl+Alt+D", "Win+Alt+D"
    pub autostart: bool,
    pub check_updates: bool,
    pub last_color: (u8, u8, u8),
    pub last_thickness: f32,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            global_hotkey: "Ctrl+2".to_string(),
            autostart: false,
            check_updates: true,
            last_color: (255, 59, 48),
            last_thickness: 4.0,
        }
    }
}

static CURRENT_SETTINGS: Mutex<Option<AppSettings>> = Mutex::new(None);

pub fn get_settings() -> AppSettings {
    if let Ok(guard) = CURRENT_SETTINGS.lock() {
        if let Some(s) = guard.as_ref() {
            return s.clone();
        }
    }
    let loaded = load_settings();
    if let Ok(mut guard) = CURRENT_SETTINGS.lock() {
        *guard = Some(loaded.clone());
    }
    loaded
}

#[allow(dead_code)]
pub fn update_settings<F: FnOnce(&mut AppSettings)>(f: F) {
    let mut current = get_settings();
    f(&mut current);
    save_settings(&current);
    if let Ok(mut guard) = CURRENT_SETTINGS.lock() {
        *guard = Some(current);
    }
}

fn get_portable_path() -> Option<PathBuf> {
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            return Some(dir.join("DrawIt.json"));
        }
    }
    None
}

fn get_appdata_path() -> Option<PathBuf> {
    if let Ok(appdata) = std::env::var("APPDATA") {
        let dir = PathBuf::from(appdata).join("DrawIt");
        let _ = std::fs::create_dir_all(&dir);
        return Some(dir.join("settings.json"));
    }
    None
}

fn parse_bool(text: &str, key: &str, default: bool) -> bool {
    let p = format!("\"{}\":", key);
    if let Some(idx) = text.find(&p) {
        let rest = text[idx + p.len()..].trim_start();
        if rest.starts_with("true") {
            return true;
        } else if rest.starts_with("false") {
            return false;
        }
    }
    default
}

fn parse_string(text: &str, key: &str, default: &str) -> String {
    let p = format!("\"{}\":", key);
    if let Some(idx) = text.find(&p) {
        let rest = text[idx + p.len()..].trim_start();
        if let Some(start_q) = rest.find('"') {
            let after_q = &rest[start_q + 1..];
            if let Some(end_q) = after_q.find('"') {
                return after_q[..end_q].to_string();
            }
        }
    }
    default.to_string()
}

fn parse_f32(text: &str, key: &str, default: f32) -> f32 {
    let p = format!("\"{}\":", key);
    if let Some(idx) = text.find(&p) {
        let rest = text[idx + p.len()..].trim_start();
        let end_idx = rest.find(|c: char| !c.is_digit(10) && c != '.' && c != '-').unwrap_or(rest.len());
        if let Ok(v) = rest[..end_idx].trim().parse::<f32>() {
            return v;
        }
    }
    default
}

fn parse_color(text: &str, default: (u8, u8, u8)) -> (u8, u8, u8) {
    let p = "\"last_color\":";
    if let Some(idx) = text.find(p) {
        let rest = text[idx + p.len()..].trim_start();
        if let Some(open) = rest.find('[') {
            if let Some(close) = rest[open..].find(']') {
                let inside = &rest[open + 1..open + close];
                let parts: Vec<u8> = inside.split(',').filter_map(|s| s.trim().parse::<u8>().ok()).collect();
                if parts.len() == 3 {
                    return (parts[0], parts[1], parts[2]);
                }
            }
        }
    }
    default
}

pub fn load_settings() -> AppSettings {
    let mut defaults = AppSettings::default();

    // Check autostart directly from system registry as well
    defaults.autostart = crate::tray::is_autostart_enabled();

    let paths = [get_portable_path(), get_appdata_path()];
    for p_opt in paths {
        if let Some(p) = p_opt {
            if p.exists() {
                if let Ok(text) = std::fs::read_to_string(&p) {
                    defaults.global_hotkey = parse_string(&text, "global_hotkey", &defaults.global_hotkey);
                    defaults.autostart = parse_bool(&text, "autostart", defaults.autostart);
                    defaults.check_updates = parse_bool(&text, "check_updates", defaults.check_updates);
                    defaults.last_color = parse_color(&text, defaults.last_color);
                    defaults.last_thickness = parse_f32(&text, "last_thickness", defaults.last_thickness);
                    return defaults;
                }
            }
        }
    }

    defaults
}

pub fn save_settings(s: &AppSettings) {
    if let Ok(mut guard) = CURRENT_SETTINGS.lock() {
        *guard = Some(s.clone());
    }

    let json = format!(
        "{{\n  \"global_hotkey\": \"{}\",\n  \"autostart\": {},\n  \"check_updates\": {},\n  \"last_color\": [{}, {}, {}],\n  \"last_thickness\": {:.1}\n}}\n",
        s.global_hotkey,
        s.autostart,
        s.check_updates,
        s.last_color.0, s.last_color.1, s.last_color.2,
        s.last_thickness
    );

    // Sync autostart with Windows Registry
    let curr_reg = crate::tray::is_autostart_enabled();
    if curr_reg != s.autostart {
        crate::tray::set_autostart_enabled(s.autostart);
    }

    // Try portable path first, fallback to AppData
    if let Some(p) = get_portable_path() {
        if std::fs::write(&p, &json).is_ok() {
            return;
        }
    }

    if let Some(p) = get_appdata_path() {
        let _ = std::fs::write(&p, &json);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_settings_json_parsing() {
        let json = r#"{
  "global_hotkey": "F8",
  "autostart": true,
  "check_updates": false,
  "last_color": [10, 200, 50],
  "last_thickness": 8.5
}"#;

        assert_eq!(parse_string(json, "global_hotkey", "Ctrl+2"), "F8");
        assert_eq!(parse_bool(json, "autostart", false), true);
        assert_eq!(parse_bool(json, "check_updates", true), false);
        assert_eq!(parse_color(json, (0, 0, 0)), (10, 200, 50));
        assert_eq!(parse_f32(json, "last_thickness", 4.0), 8.5);
    }
}
