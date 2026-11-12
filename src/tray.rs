use std::mem;
use std::ptr;
use std::sync::atomic::Ordering;
use windows::core::{w, PCWSTR};
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, POINT, WPARAM};
use windows::Win32::System::Registry::{
    RegCloseKey, RegDeleteValueW, RegOpenKeyExW, RegQueryValueExW, RegSetValueExW,
    HKEY, HKEY_CURRENT_USER, KEY_READ, KEY_WRITE, REG_SZ,
};
use windows::Win32::UI::Input::KeyboardAndMouse::{
    RegisterHotKey, UnregisterHotKey, MOD_ALT, MOD_CONTROL, MOD_NOREPEAT, MOD_WIN,
};
use windows::Win32::UI::Shell::{
    Shell_NotifyIconW, NIF_ICON, NIF_MESSAGE, NIF_TIP, NIM_ADD, NIM_DELETE, NOTIFYICONDATAW,
};
use windows::Win32::UI::WindowsAndMessaging::{
    AppendMenuW, CreatePopupMenu, CreateWindowExW, DefWindowProcW, DestroyMenu,
    GetCursorPos, GetSystemMetrics, GetWindowLongPtrW, LoadImageW, PostQuitMessage,
    RegisterClassExW, SetForegroundWindow, SetMenuDefaultItem, SetWindowLongPtrW,
    TrackPopupMenu, GWLP_USERDATA, HICON, HMENU, IMAGE_ICON, LR_DEFAULTCOLOR, LR_SHARED,
    MF_CHECKED, MF_SEPARATOR, MF_STRING, MF_UNCHECKED, SM_CXSMICON, SM_CYSMICON,
    TPM_BOTTOMALIGN, TPM_RIGHTBUTTON, WM_COMMAND, WM_DESTROY, WM_HOTKEY,
    WM_LBUTTONDBLCLK, WM_LBUTTONUP, WM_RBUTTONUP, WM_USER, WNDCLASSEXW, WS_OVERLAPPEDWINDOW,
};

use crate::models::ActiveModal;
use crate::overlay::{hide_overlay, show_overlay, OverlayState, IS_OVERLAY_OPEN};

const WM_TRAY_CALLBACK: u32 = WM_USER + 100;
const HOTKEY_CTRL_2: i32 = 1;
const HOTKEY_CTRL_ALT_D: i32 = 2;
const HOTKEY_WIN_ALT_D: i32 = 3;

const ID_TRAY_START: usize = 1001;
const ID_TRAY_RESET: usize = 1002;
const ID_TRAY_HELP: usize = 1003;
const ID_TRAY_ABOUT: usize = 1004;
const ID_TRAY_AUTOSTART: usize = 1005;
const ID_TRAY_EXIT: usize = 1006;

pub struct TrayState {
    pub tray_hwnd: HWND,
    pub overlay_hwnd: HWND,
    pub nid: NOTIFYICONDATAW,
}

impl TrayState {
    pub fn new(tray_hwnd: HWND, overlay_hwnd: HWND) -> Self {
        let mut nid: NOTIFYICONDATAW = unsafe { mem::zeroed() };
        nid.cbSize = mem::size_of::<NOTIFYICONDATAW>() as u32;
        nid.hWnd = tray_hwnd;
        nid.uID = 1;
        nid.uFlags = NIF_MESSAGE | NIF_ICON | NIF_TIP;
        nid.uCallbackMessage = WM_TRAY_CALLBACK;

        // Load the embedded DrawIt icon (resource ID 1 from app.ico)
        unsafe {
            if let Ok(mod_handle) = windows::Win32::System::LibraryLoader::GetModuleHandleW(None) {
                let h_instance = windows::Win32::Foundation::HINSTANCE(mod_handle.0);
                let ico = LoadImageW(
                    h_instance,
                    PCWSTR(1 as *const u16),
                    IMAGE_ICON,
                    GetSystemMetrics(SM_CXSMICON),
                    GetSystemMetrics(SM_CYSMICON),
                    LR_DEFAULTCOLOR | LR_SHARED,
                )
                .ok()
                .map(|h| HICON(h.0));

                nid.hIcon = ico.unwrap_or_else(|| {
                    windows::Win32::UI::WindowsAndMessaging::LoadIconW(
                        h_instance,
                        windows::Win32::UI::WindowsAndMessaging::IDI_APPLICATION,
                    )
                    .unwrap_or(HICON(ptr::null_mut()))
                });
            }
        }

        let tip_text = "DrawIt – Képernyő rajzoló és jegyzetelő (Ctrl + 2)";
        let utf16: Vec<u16> = tip_text.encode_utf16().collect();
        let copy_len = utf16.len().min(nid.szTip.len() - 1);
        nid.szTip[..copy_len].copy_from_slice(&utf16[..copy_len]);

        unsafe {
            let _ = Shell_NotifyIconW(NIM_ADD, &nid);
        }

        Self {
            tray_hwnd,
            overlay_hwnd,
            nid,
        }
    }
}

pub fn is_autostart_enabled() -> bool {
    unsafe {
        let mut hkey: HKEY = mem::zeroed();
        let subkey = w!("Software\\Microsoft\\Windows\\CurrentVersion\\Run");
        let status = RegOpenKeyExW(HKEY_CURRENT_USER, subkey, 0, KEY_READ, &mut hkey);
        if status.0 != 0 {
            return false;
        }

        let val_name = w!("DrawIt");
        let mut data_len = 0u32;
        let query_res = RegQueryValueExW(hkey, val_name, None, None, None, Some(&mut data_len));
        let _ = RegCloseKey(hkey);

        query_res.0 == 0
    }
}

pub fn set_autostart_enabled(enable: bool) -> bool {
    unsafe {
        let mut hkey: HKEY = mem::zeroed();
        let subkey = w!("Software\\Microsoft\\Windows\\CurrentVersion\\Run");
        let status = RegOpenKeyExW(HKEY_CURRENT_USER, subkey, 0, KEY_WRITE, &mut hkey);
        if status.0 != 0 {
            return false;
        }

        let val_name = w!("DrawIt");
        let success = if enable {
            if let Ok(exe_path) = std::env::current_exe() {
                let path_str = format!("\"{}\"", exe_path.to_string_lossy());
                let utf16: Vec<u16> = path_str.encode_utf16().chain(std::iter::once(0)).collect();
                let bytes_len = (utf16.len() * 2) as u32;
                RegSetValueExW(
                    hkey,
                    val_name,
                    0,
                    REG_SZ,
                    Some(std::slice::from_raw_parts(utf16.as_ptr() as *const u8, bytes_len as usize)),
                )
                .0 == 0
            } else {
                false
            }
        } else {
            RegDeleteValueW(hkey, val_name).0 == 0
        };

        let _ = RegCloseKey(hkey);
        success
    }
}

unsafe extern "system" fn tray_wndproc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    let ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut TrayState;

    match msg {
        WM_HOTKEY => {
            if !ptr.is_null() {
                let state = &*ptr;
                if IS_OVERLAY_OPEN.load(Ordering::SeqCst) {
                    let ov_ptr = GetWindowLongPtrW(state.overlay_hwnd, GWLP_USERDATA) as *mut OverlayState;
                    if !ov_ptr.is_null() {
                        hide_overlay(&mut *ov_ptr);
                    }
                } else {
                    show_overlay(state.overlay_hwnd);
                }
            }
            LRESULT(0)
        }

        WM_TRAY_CALLBACK => {
            let event = (lparam.0 & 0xFFFF) as u32;

            if event == WM_LBUTTONUP || event == WM_LBUTTONDBLCLK {
                if !ptr.is_null() {
                    let state = &*ptr;
                    if IS_OVERLAY_OPEN.load(Ordering::SeqCst) {
                        let ov_ptr = GetWindowLongPtrW(state.overlay_hwnd, GWLP_USERDATA) as *mut OverlayState;
                        if !ov_ptr.is_null() {
                            hide_overlay(&mut *ov_ptr);
                        }
                    } else {
                        show_overlay(state.overlay_hwnd);
                    }
                }
            } else if event == WM_RBUTTONUP {
                if !ptr.is_null() {
                    let state = &*ptr;
                    show_tray_context_menu(state);
                }
            }

            LRESULT(0)
        }

        WM_COMMAND => {
            let cmd = wparam.0 as usize;
            if !ptr.is_null() {
                let state = &*ptr;
                match cmd {
                    ID_TRAY_START => {
                        show_overlay(state.overlay_hwnd);
                    }
                    ID_TRAY_RESET => {
                        let ov_ptr = GetWindowLongPtrW(state.overlay_hwnd, GWLP_USERDATA) as *mut OverlayState;
                        if !ov_ptr.is_null() {
                            (*ov_ptr).history.reset_counter();
                        }
                    }
                    ID_TRAY_HELP => {
                        show_overlay(state.overlay_hwnd);
                        let ov_ptr = GetWindowLongPtrW(state.overlay_hwnd, GWLP_USERDATA) as *mut OverlayState;
                        if !ov_ptr.is_null() {
                            (*ov_ptr).active_modal = ActiveModal::Help;
                            (*ov_ptr).render();
                        }
                    }
                    ID_TRAY_ABOUT => {
                        show_overlay(state.overlay_hwnd);
                        let ov_ptr = GetWindowLongPtrW(state.overlay_hwnd, GWLP_USERDATA) as *mut OverlayState;
                        if !ov_ptr.is_null() {
                            (*ov_ptr).active_modal = ActiveModal::About;
                            (*ov_ptr).render();
                        }
                    }
                    ID_TRAY_AUTOSTART => {
                        let curr = is_autostart_enabled();
                        set_autostart_enabled(!curr);
                    }
                    ID_TRAY_EXIT => {
                        let _ = Shell_NotifyIconW(NIM_DELETE, &state.nid);
                        PostQuitMessage(0);
                    }
                    _ => {}
                }
            }
            LRESULT(0)
        }

        WM_DESTROY => {
            if !ptr.is_null() {
                let state = &*ptr;
                let _ = Shell_NotifyIconW(NIM_DELETE, &state.nid);
                let _ = UnregisterHotKey(hwnd, HOTKEY_CTRL_2);
                let _ = UnregisterHotKey(hwnd, HOTKEY_CTRL_ALT_D);
                let _ = UnregisterHotKey(hwnd, HOTKEY_WIN_ALT_D);
            }
            PostQuitMessage(0);
            LRESULT(0)
        }

        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}

unsafe fn show_tray_context_menu(state: &TrayState) {
    let menu = CreatePopupMenu().unwrap_or(HMENU(ptr::null_mut()));
    if menu.0.is_null() {
        return;
    }

    let _ = AppendMenuW(menu, MF_STRING, ID_TRAY_START, w!("Rajzolás indítása (Ctrl + 2)"));
    let _ = AppendMenuW(menu, MF_SEPARATOR, 0, PCWSTR::null());
    let _ = AppendMenuW(menu, MF_STRING, ID_TRAY_RESET, w!("Számláló nullázása (1)"));
    let _ = AppendMenuW(menu, MF_STRING, ID_TRAY_HELP, w!("Útmutató és billentyűk (F1)"));
    let _ = AppendMenuW(menu, MF_STRING, ID_TRAY_ABOUT, w!("Névjegy"));
    let _ = AppendMenuW(menu, MF_SEPARATOR, 0, PCWSTR::null());

    let autostart_flag = if is_autostart_enabled() { MF_CHECKED } else { MF_UNCHECKED };
    let _ = AppendMenuW(menu, MF_STRING | autostart_flag, ID_TRAY_AUTOSTART, w!("Indítás a Windows-zal"));
    let _ = AppendMenuW(menu, MF_SEPARATOR, 0, PCWSTR::null());
    let _ = AppendMenuW(menu, MF_STRING, ID_TRAY_EXIT, w!("Kilépés"));

    let _ = SetMenuDefaultItem(menu, ID_TRAY_START as u32, 0);

    let mut pt = POINT { x: 0, y: 0 };
    let _ = GetCursorPos(&mut pt);

    let _ = SetForegroundWindow(state.tray_hwnd);
    let _ = TrackPopupMenu(
        menu,
        TPM_BOTTOMALIGN | TPM_RIGHTBUTTON,
        pt.x,
        pt.y,
        0,
        state.tray_hwnd,
        None,
    );
    let _ = DestroyMenu(menu);
}

pub fn init_tray_and_hotkeys(overlay_hwnd: HWND) -> Option<HWND> {
    unsafe {
        let class_name = w!("DrawItTrayMessageClass");
        let h_module = windows::Win32::System::LibraryLoader::GetModuleHandleW(None).ok()?;
        let h_instance = windows::Win32::Foundation::HINSTANCE(h_module.0);

        let ico_sm = LoadImageW(
            h_instance,
            PCWSTR(1 as *const u16),
            IMAGE_ICON,
            GetSystemMetrics(SM_CXSMICON),
            GetSystemMetrics(SM_CYSMICON),
            LR_DEFAULTCOLOR | LR_SHARED,
        )
        .ok()
        .map(|h| HICON(h.0))
        .unwrap_or(HICON(ptr::null_mut()));

        let wnd_class = WNDCLASSEXW {
            cbSize: mem::size_of::<WNDCLASSEXW>() as u32,
            style: windows::Win32::UI::WindowsAndMessaging::WNDCLASS_STYLES(0),
            lpfnWndProc: Some(tray_wndproc),
            cbClsExtra: 0,
            cbWndExtra: 0,
            hInstance: h_instance,
            hIcon: ico_sm,
            hCursor: windows::Win32::UI::WindowsAndMessaging::HCURSOR(ptr::null_mut()),
            hbrBackground: windows::Win32::Graphics::Gdi::HBRUSH(ptr::null_mut()),
            lpszMenuName: PCWSTR::null(),
            lpszClassName: class_name,
            hIconSm: ico_sm,
        };

        let _ = RegisterClassExW(&wnd_class);

        let hwnd = CreateWindowExW(
            windows::Win32::UI::WindowsAndMessaging::WINDOW_EX_STYLE(0),
            class_name,
            w!("DrawIt Tray"),
            WS_OVERLAPPEDWINDOW,
            0,
            0,
            0,
            0,
            None,
            None,
            h_instance,
            None,
        ).ok()?;

        let state = Box::new(TrayState::new(hwnd, overlay_hwnd));
        SetWindowLongPtrW(hwnd, GWLP_USERDATA, Box::into_raw(state) as isize);

        // Register Global Hotkeys:
        // 1. Primary: Ctrl + 2
        let _ = RegisterHotKey(hwnd, HOTKEY_CTRL_2, MOD_CONTROL | MOD_NOREPEAT, 0x32);
        // 2. Fallback: Ctrl + Alt + D
        let _ = RegisterHotKey(hwnd, HOTKEY_CTRL_ALT_D, MOD_CONTROL | MOD_ALT | MOD_NOREPEAT, 0x44);
        // 3. Fallback: Win + Alt + D
        let _ = RegisterHotKey(hwnd, HOTKEY_WIN_ALT_D, MOD_WIN | MOD_ALT | MOD_NOREPEAT, 0x44);

        Some(hwnd)
    }
}
