#![windows_subsystem = "windows"]

mod d2d;
mod models;
mod overlay;
mod pdf;
mod screen;
mod toolbar;
mod tray;
mod updater;
mod utils;

use windows::core::w;
use windows::Win32::Foundation::{ERROR_ALREADY_EXISTS, GetLastError};
use windows::Win32::System::Com::{CoInitializeEx, CoUninitialize, COINIT_APARTMENTTHREADED};
use windows::Win32::System::Threading::CreateMutexW;
use windows::Win32::UI::HiDpi::{
    SetProcessDpiAwarenessContext, DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2,
};
use windows::Win32::UI::WindowsAndMessaging::{
    DispatchMessageW, GetMessageW, MessageBoxW, TranslateMessage, MB_ICONINFORMATION, MB_OK, MSG,
};

fn main() {
    unsafe {
        // 1. Single Instance Protection via Global Named Mutex
        let mutex_name = w!("Global\\DrawIt_Rust_SingleInstance_Mutex");
        let _mutex = CreateMutexW(None, true, mutex_name);
        if GetLastError() == ERROR_ALREADY_EXISTS {
            let msg = w!("A DrawIt már fut a háttérben!\n\nA rajzolás elindításához nyomja meg a Ctrl + 2 billentyűkombinációt, vagy kattintson a tálca ikonra.");
            let title = w!("DrawIt");
            MessageBoxW(None, msg, title, MB_OK | MB_ICONINFORMATION);
            return;
        }

        // 2. High-DPI awareness (Per-Monitor v2)
        let _ = SetProcessDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2);

        // 3. COM Apartment initialization for Direct2D, DirectWrite, Shell & Dialogs
        let _ = CoInitializeEx(None, COINIT_APARTMENTTHREADED);

        // 3b. Cleanup any previous update artifacts and start background update check
        updater::cleanup_old_exe();
        std::thread::spawn(|| {
            std::thread::sleep(std::time::Duration::from_secs(3));
            if let Ok(Some(info)) = updater::check_for_update() {
                updater::set_available_update(Some(info));
            }
        });

        // 4. Create Fullscreen Overlay Window spanning all displays
        let overlay_hwnd = match overlay::create_overlay_window() {
            Some(h) => h,
            None => {
                let err_msg = w!("Nem sikerült létrehozni a DrawIt ablakot.");
                let err_title = w!("DrawIt Hiba");
                MessageBoxW(None, err_msg, err_title, MB_OK);
                CoUninitialize();
                return;
            }
        };

        // 5. Initialize System Tray Icon and Register Global Hotkeys (Ctrl + 2)
        let _tray_hwnd = match tray::init_tray_and_hotkeys(overlay_hwnd) {
            Some(h) => h,
            None => {
                let err_msg = w!("Nem sikerült regisztrálni a tálca ikont vagy a gyorsbillentyűt.");
                let err_title = w!("DrawIt Hiba");
                MessageBoxW(None, err_msg, err_title, MB_OK);
                CoUninitialize();
                return;
            }
        };

        // 6. Win32 Message Loop
        let mut msg = MSG::default();
        while GetMessageW(&mut msg, None, 0, 0).as_bool() {
            let _ = TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }

        CoUninitialize();
    }
}
