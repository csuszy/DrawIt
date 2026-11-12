use std::sync::Mutex;
use windows::core::{w, PCWSTR};
use windows::Win32::Foundation::HWND;
use windows::Win32::Networking::WinHttp::*;
use windows::Win32::Storage::FileSystem::{MoveFileExW, MOVEFILE_REPLACE_EXISTING};
use windows::Win32::UI::Shell::ShellExecuteW;
use windows::Win32::UI::WindowsAndMessaging::{
    MessageBoxW, IDYES, MB_ICONERROR, MB_ICONINFORMATION, MB_ICONQUESTION, MB_OK, MB_YESNO,
    SW_SHOWNORMAL,
};

#[derive(Clone, Debug)]
#[allow(dead_code)]
pub struct UpdateInfo {
    pub tag: String,
    pub name: String,
    pub html_url: String,
    pub download_url: Option<String>,
    pub body: String,
}

static AVAILABLE_UPDATE: Mutex<Option<UpdateInfo>> = Mutex::new(None);

pub fn get_available_update() -> Option<UpdateInfo> {
    AVAILABLE_UPDATE.lock().ok()?.clone()
}

pub fn set_available_update(info: Option<UpdateInfo>) {
    if let Ok(mut lock) = AVAILABLE_UPDATE.lock() {
        *lock = info;
    }
}

pub fn parse_version(tag: &str) -> (u32, u32, u32) {
    let clean = tag.trim().trim_start_matches(|c| c == 'v' || c == 'V');
    let mut parts = clean.split('.');
    let major = parts.next().and_then(|s| s.parse().ok()).unwrap_or(0);
    let minor = parts.next().and_then(|s| s.parse().ok()).unwrap_or(0);
    let patch = parts.next().and_then(|s| s.parse().ok()).unwrap_or(0);
    (major, minor, patch)
}

pub fn is_newer(remote_tag: &str, current_tag: &str) -> bool {
    let r = parse_version(remote_tag);
    let c = parse_version(current_tag);
    r > c
}

fn to_wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}

pub fn http_get_bytes(url_or_path: &str, is_full_url: bool) -> Result<Vec<u8>, String> {
    unsafe {
        let agent = to_wide("DrawIt-Updater/0.1.0");
        let session = WinHttpOpen(
            PCWSTR(agent.as_ptr()),
            WINHTTP_ACCESS_TYPE_DEFAULT_PROXY,
            PCWSTR::null(),
            PCWSTR::null(),
            0,
        );
        if session.is_null() {
            return Err("WinHttpOpen sikertelen".into());
        }

        let (host_str, path_str) = if is_full_url {
            let stripped = url_or_path.trim_start_matches("https://");
            if let Some(slash_idx) = stripped.find('/') {
                (&stripped[..slash_idx], &stripped[slash_idx..])
            } else {
                (stripped, "/")
            }
        } else {
            ("api.github.com", url_or_path)
        };

        let host_w = to_wide(host_str);
        let path_w = to_wide(path_str);

        let connect = WinHttpConnect(session, PCWSTR(host_w.as_ptr()), 443, 0);
        if connect.is_null() {
            let _ = WinHttpCloseHandle(session);
            return Err("WinHttpConnect sikertelen".into());
        }

        let request = WinHttpOpenRequest(
            connect,
            w!("GET"),
            PCWSTR(path_w.as_ptr()),
            PCWSTR::null(),
            PCWSTR::null(),
            std::ptr::null(),
            WINHTTP_FLAG_SECURE,
        );
        if request.is_null() {
            let _ = WinHttpCloseHandle(connect);
            let _ = WinHttpCloseHandle(session);
            return Err("WinHttpOpenRequest sikertelen".into());
        }

        let redirect_policy = WINHTTP_OPTION_REDIRECT_POLICY_ALWAYS;
        let redirect_bytes = redirect_policy.to_ne_bytes();
        let _ = WinHttpSetOption(
            Some(request as *const _),
            WINHTTP_OPTION_REDIRECT_POLICY,
            Some(&redirect_bytes),
        );

        let headers = to_wide("User-Agent: DrawIt-App\r\nAccept: application/vnd.github+json\r\n");
        let headers_slice = if headers.ends_with(&[0]) {
            &headers[..headers.len() - 1]
        } else {
            &headers[..]
        };
        let sent = WinHttpSendRequest(
            request,
            Some(headers_slice),
            None,
            0,
            0,
            0,
        );
        if sent.is_err() {
            let _ = WinHttpCloseHandle(request);
            let _ = WinHttpCloseHandle(connect);
            let _ = WinHttpCloseHandle(session);
            return Err("WinHttpSendRequest sikertelen".into());
        }

        let resp = WinHttpReceiveResponse(request, std::ptr::null_mut());
        if resp.is_err() {
            let _ = WinHttpCloseHandle(request);
            let _ = WinHttpCloseHandle(connect);
            let _ = WinHttpCloseHandle(session);
            return Err("WinHttpReceiveResponse sikertelen".into());
        }

        let mut data = Vec::new();
        loop {
            let mut bytes_avail: u32 = 0;
            if WinHttpQueryDataAvailable(request, &mut bytes_avail).is_err() || bytes_avail == 0 {
                break;
            }
            let mut buf = vec![0u8; bytes_avail as usize];
            let mut bytes_read: u32 = 0;
            if WinHttpReadData(
                request,
                buf.as_mut_ptr() as *mut _,
                bytes_avail,
                &mut bytes_read,
            )
            .is_ok()
                && bytes_read > 0
            {
                data.extend_from_slice(&buf[..bytes_read as usize]);
            } else {
                break;
            }
        }

        let _ = WinHttpCloseHandle(request);
        let _ = WinHttpCloseHandle(connect);
        let _ = WinHttpCloseHandle(session);

        Ok(data)
    }
}

fn extract_json_str(json: &str, key: &str) -> Option<String> {
    let pattern = format!("\"{}\":", key);
    let idx = json.find(&pattern)?;
    let rest = &json[idx + pattern.len()..];
    let quote_start = rest.find('"')?;
    let content_start = &rest[quote_start + 1..];
    let quote_end = content_start.find('"')?;
    Some(content_start[..quote_end].to_string())
}

fn extract_asset_download_url(json: &str) -> Option<String> {
    let mut search_idx = 0;
    while let Some(pos) = json[search_idx..].find("\"browser_download_url\"") {
        let abs_pos = search_idx + pos;
        let rest = &json[abs_pos..];
        if let Some(url) = extract_json_str(rest, "browser_download_url") {
            if url.ends_with("DrawIt.exe") {
                return Some(url);
            }
        }
        search_idx = abs_pos + 22;
    }
    None
}

pub fn check_for_update() -> Result<Option<UpdateInfo>, String> {
    let bytes = http_get_bytes("/repos/csuszy/DrawIt/releases/latest", false)?;
    let text = String::from_utf8_lossy(&bytes);

    let tag = match extract_json_str(&text, "tag_name") {
        Some(t) => t,
        None => return Ok(None),
    };

    let current = env!("CARGO_PKG_VERSION");
    if !is_newer(&tag, current) {
        return Ok(None);
    }

    let name = extract_json_str(&text, "name").unwrap_or_else(|| format!("DrawIt {}", tag));
    let html_url = extract_json_str(&text, "html_url").unwrap_or_else(|| format!("https://github.com/csuszy/DrawIt/releases/tag/{}", tag));
    let download_url = extract_asset_download_url(&text);
    let body = extract_json_str(&text, "body").unwrap_or_default();

    Ok(Some(UpdateInfo {
        tag,
        name,
        html_url,
        download_url,
        body,
    }))
}

pub fn apply_update(download_url: &str) -> Result<(), String> {
    let current_exe = std::env::current_exe().map_err(|e| e.to_string())?;
    let new_exe = current_exe.with_extension("exe.new");
    let old_exe = current_exe.with_extension("exe.old");

    let bytes = http_get_bytes(download_url, true)?;
    if bytes.len() < 200_000 || !bytes.starts_with(b"MZ") {
        return Err("A letöltött fájl érvénytelen".into());
    }

    std::fs::write(&new_exe, &bytes).map_err(|e| e.to_string())?;

    let cur_w = to_wide(current_exe.to_str().ok_or("Hibás útvonal")?);
    let old_w = to_wide(old_exe.to_str().ok_or("Hibás útvonal")?);
    let new_w = to_wide(new_exe.to_str().ok_or("Hibás útvonal")?);

    unsafe {
        let _ = MoveFileExW(PCWSTR(cur_w.as_ptr()), PCWSTR(old_w.as_ptr()), MOVEFILE_REPLACE_EXISTING);
        let _ = MoveFileExW(PCWSTR(new_w.as_ptr()), PCWSTR(cur_w.as_ptr()), MOVEFILE_REPLACE_EXISTING);
    }

    let _ = std::process::Command::new(&current_exe).spawn();
    std::process::exit(0);
}

pub fn cleanup_old_exe() {
    if let Ok(current_exe) = std::env::current_exe() {
        let old_exe = current_exe.with_extension("exe.old");
        if old_exe.exists() {
            let _ = std::fs::remove_file(old_exe);
        }
        let new_exe = current_exe.with_extension("exe.new");
        if new_exe.exists() {
            let _ = std::fs::remove_file(new_exe);
        }
    }
}

pub fn prompt_and_update(parent_hwnd: HWND, manual: bool) {
    let parent_raw = parent_hwnd.0 as usize;
    std::thread::spawn(move || {
        let parent = HWND(parent_raw as *mut _);
        let update_opt = match get_available_update() {
            Some(u) => Some(u),
            None => match check_for_update() {
                Ok(u) => {
                    set_available_update(u.clone());
                    u
                }
                Err(err) => {
                    if manual {
                        let msg = to_wide(&format!("A frissítések keresése során hiba lépett fel:\n{}", err));
                        let title = to_wide("DrawIt Frissítés");
                        unsafe {
                            let _ = MessageBoxW(parent, PCWSTR(msg.as_ptr()), PCWSTR(title.as_ptr()), MB_OK | MB_ICONERROR);
                        }
                    }
                    return;
                }
            },
        };

        match update_opt {
            Some(update) => {
                let msg = to_wide(&format!(
                    "Új DrawIt verzió érhető el: {}!\n\nJelenlegi verzió: v{}\nLegújabb kiadás: {}\n\nSzeretnéd most letölteni és azonnal frissíteni az alkalmazást?",
                    update.name,
                    env!("CARGO_PKG_VERSION"),
                    update.tag
                ));
                let title = to_wide("DrawIt Frissítés elérhető");
                let res = unsafe {
                    MessageBoxW(
                        parent,
                        PCWSTR(msg.as_ptr()),
                        PCWSTR(title.as_ptr()),
                        MB_YESNO | MB_ICONQUESTION,
                    )
                };

                if res == IDYES {
                    if let Some(dl_url) = update.download_url {
                        if let Err(e) = apply_update(&dl_url) {
                            let emsg = to_wide(&format!("A frissítés letöltése vagy telepítése sikertelen:\n{}", e));
                            let etitle = to_wide("Frissítési hiba");
                            unsafe {
                                let _ = MessageBoxW(parent, PCWSTR(emsg.as_ptr()), PCWSTR(etitle.as_ptr()), MB_OK | MB_ICONERROR);
                            }
                        }
                    } else {
                        let url_w = to_wide(&update.html_url);
                        unsafe {
                            let _ = ShellExecuteW(
                                parent,
                                windows::core::w!("open"),
                                PCWSTR(url_w.as_ptr()),
                                PCWSTR::null(),
                                PCWSTR::null(),
                                SW_SHOWNORMAL,
                            );
                        }
                    }
                }
            }
            None => {
                if manual {
                    let msg = to_wide(&format!(
                        "A legújabb DrawIt verziót használod (v{}).\nNincs újabb frissítés a GitHubon.",
                        env!("CARGO_PKG_VERSION")
                    ));
                    let title = to_wide("DrawIt Frissítés");
                    unsafe {
                        let _ = MessageBoxW(parent, PCWSTR(msg.as_ptr()), PCWSTR(title.as_ptr()), MB_OK | MB_ICONINFORMATION);
                    }
                }
            }
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_compare() {
        assert!(is_newer("v0.2", "0.1.0"));
        assert!(is_newer("v1.0.0", "0.1.0"));
        assert!(!is_newer("v0.1.0", "0.1.0"));
        assert!(!is_newer("v0.0.9", "0.1.0"));
    }

    #[test]
    fn test_github_release_api() {
        let res = http_get_bytes("/repos/csuszy/DrawIt/releases/latest", false);
        assert!(res.is_ok(), "WinHttp request failed: {:?}", res.err());
        let bytes = res.unwrap();
        let text = String::from_utf8_lossy(&bytes);
        assert!(text.contains("tag_name"), "Response doesn't contain tag_name: {}", text);
        let tag = extract_json_str(&text, "tag_name");
        assert_eq!(tag.as_deref(), Some("v0.1"));
        let dl = extract_asset_download_url(&text);
        assert!(dl.is_some(), "Download url not found in json");
        assert!(dl.unwrap().ends_with("DrawIt.exe"));
    }
}
