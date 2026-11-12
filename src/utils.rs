use std::mem;
use std::ptr;
use windows::core::{w, PCWSTR};
use windows::Win32::Foundation::{HANDLE, HWND};
use windows::Win32::Graphics::Gdi::{
    CreateCompatibleBitmap, CreateCompatibleDC, DeleteDC, GetDC, ReleaseDC, SetDIBits,
    BITMAPINFO, BITMAPINFOHEADER, BI_RGB, DIB_RGB_COLORS,
};
use windows::Win32::System::DataExchange::{CloseClipboard, EmptyClipboard, OpenClipboard, SetClipboardData};
use windows::Win32::System::Memory::{GlobalAlloc, GlobalLock, GlobalUnlock, GMEM_MOVEABLE};
use windows::Win32::UI::Controls::Dialogs::{
    GetSaveFileNameW, OFN_EXPLORER, OFN_OVERWRITEPROMPT, OFN_PATHMUSTEXIST, OPENFILENAMEW,
};
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, SetWindowPos, HWND_TOPMOST, SWP_NOACTIVATE, SWP_SHOWWINDOW,
    WS_EX_TOOLWINDOW, WS_EX_TOPMOST, WS_POPUP, WS_VISIBLE,
};

/// Creates a topmost anchor window centered on the active monitor and owned by parent
/// so Windows modal dialogs (GetSaveFileName, MessageBox) position themselves dead
/// center on the monitor the user is interacting with, always visible above the overlay.
pub fn create_anchor_for_monitor_by_cursor(capture: &crate::screen::ScreenCapture, parent: HWND) -> HWND {
    unsafe {
        let mon = crate::screen::get_focused_monitor(&capture.monitors);
        let (cx, cy) = if let Some(m) = mon {
            (m.x + m.width / 2, m.y + m.height / 2)
        } else {
            (capture.left + (capture.width as i32) / 2, capture.top + (capture.height as i32) / 2)
        };

        let hwnd = CreateWindowExW(
            WS_EX_TOPMOST | WS_EX_TOOLWINDOW,
            w!("STATIC"),
            w!("DrawItAnchor"),
            WS_POPUP | WS_VISIBLE,
            cx,
            cy,
            1,
            1,
            parent,
            None,
            None,
            None,
        ).unwrap_or(HWND(ptr::null_mut()));

        if !hwnd.0.is_null() {
            let _ = SetWindowPos(hwnd, HWND_TOPMOST, cx, cy, 1, 1, SWP_NOACTIVATE | SWP_SHOWWINDOW);
        }

        hwnd
    }
}

pub fn copy_bgra_to_clipboard(hwnd: HWND, width: u32, height: u32, bgra_pixels: &[u8]) -> bool {
    if width == 0 || height == 0 || bgra_pixels.is_empty() {
        return false;
    }

    unsafe {
        if OpenClipboard(hwnd).is_err() {
            return false;
        }
        let _ = EmptyClipboard();

        let header_size = mem::size_of::<BITMAPINFOHEADER>();
        let row_bytes = (width as usize) * 4;
        let image_size = row_bytes * (height as usize);
        let total_size = header_size + image_size;

        // 1. Standard Windows bottom-up DIB (CF_DIB = 8)
        // CRITICAL: Windows CF_DIB MUST have positive biHeight and rows in reverse order!
        if let Ok(handle) = GlobalAlloc(GMEM_MOVEABLE, total_size) {
            let ptr = GlobalLock(handle) as *mut u8;
            if !ptr.is_null() {
                let header = BITMAPINFOHEADER {
                    biSize: header_size as u32,
                    biWidth: width as i32,
                    biHeight: height as i32, // Positive = standard bottom-up DIB
                    biPlanes: 1,
                    biBitCount: 32,
                    biCompression: BI_RGB.0,
                    biSizeImage: image_size as u32,
                    biXPelsPerMeter: 3780,
                    biYPelsPerMeter: 3780,
                    biClrUsed: 0,
                    biClrImportant: 0,
                };

                ptr::copy_nonoverlapping(
                    &header as *const _ as *const u8,
                    ptr,
                    header_size,
                );

                // Copy rows in reverse vertical order (bottom-up)
                for y in 0..height {
                    let src_y = height - 1 - y;
                    let src_row = (src_y as usize) * row_bytes;
                    let dst_row = header_size + (y as usize) * row_bytes;
                    ptr::copy_nonoverlapping(
                        bgra_pixels.as_ptr().add(src_row),
                        ptr.add(dst_row),
                        row_bytes,
                    );
                }

                let _ = GlobalUnlock(handle);
                let _ = SetClipboardData(8, HANDLE(handle.0));
            }
        }

        // 2. Also register CF_BITMAP (2) using native GDI HBITMAP for universal compatibility
        let hdc_screen = GetDC(HWND(ptr::null_mut()));
        let hdc_mem = CreateCompatibleDC(hdc_screen);
        let h_bitmap = CreateCompatibleBitmap(hdc_screen, width as i32, height as i32);

        let mut bmi: BITMAPINFO = mem::zeroed();
        bmi.bmiHeader.biSize = header_size as u32;
        bmi.bmiHeader.biWidth = width as i32;
        bmi.bmiHeader.biHeight = -(height as i32); // Top-down for SetDIBits
        bmi.bmiHeader.biPlanes = 1;
        bmi.bmiHeader.biBitCount = 32;
        bmi.bmiHeader.biCompression = BI_RGB.0;

        let _ = SetDIBits(
            hdc_mem,
            h_bitmap,
            0,
            height,
            bgra_pixels.as_ptr() as *const _,
            &bmi,
            DIB_RGB_COLORS,
        );

        let _ = DeleteDC(hdc_mem);
        ReleaseDC(HWND(ptr::null_mut()), hdc_screen);

        let _ = SetClipboardData(2, HANDLE(h_bitmap.0));

        let _ = CloseClipboard();
        true
    }
}

pub fn prompt_save_file_path(
    owner_hwnd: HWND,
    default_filename: &str,
    filter: PCWSTR,
    default_ext: PCWSTR,
) -> Option<String> {
    unsafe {
        let mut file_buf = [0u16; 512];
        let def_u16: Vec<u16> = default_filename.encode_utf16().collect();
        let copy_len = def_u16.len().min(500);
        file_buf[..copy_len].copy_from_slice(&def_u16[..copy_len]);

        let mut ofn: OPENFILENAMEW = mem::zeroed();
        ofn.lStructSize = mem::size_of::<OPENFILENAMEW>() as u32;
        ofn.hwndOwner = owner_hwnd;
        ofn.lpstrFilter = filter;
        ofn.lpstrFile = windows::core::PWSTR(file_buf.as_mut_ptr());
        ofn.nMaxFile = file_buf.len() as u32;
        ofn.lpstrDefExt = default_ext;
        ofn.Flags = OFN_EXPLORER | OFN_OVERWRITEPROMPT | OFN_PATHMUSTEXIST;

        if GetSaveFileNameW(&mut ofn).as_bool() {
            let len = file_buf.iter().position(|&c| c == 0).unwrap_or(file_buf.len());
            Some(String::from_utf16_lossy(&file_buf[..len]))
        } else {
            None
        }
    }
}

pub fn save_png_dialog(
    hwnd: HWND,
    default_filename: &str,
    width: u32,
    height: u32,
    rgba_pixels: &[u8],
) -> Option<String> {
    if let Some(path) = prompt_save_file_path(
        hwnd,
        default_filename,
        w!("PNG kép (*.png)\0*.png\0Minden fájl (*.*)\0*.*\0\0"),
        w!("png"),
    ) {
        if image::save_buffer(
            &path,
            rgba_pixels,
            width,
            height,
            image::ExtendedColorType::Rgba8,
        )
        .is_ok()
        {
            return Some(path);
        }
    }
    None
}

pub fn save_pdf_dialog(
    hwnd: HWND,
    default_filename: &str,
    width: u32,
    height: u32,
    rgba_pixels: &[u8],
) -> Option<String> {
    if let Some(path) = prompt_save_file_path(
        hwnd,
        default_filename,
        w!("PDF dokumentum (*.pdf)\0*.pdf\0Minden fájl (*.*)\0*.*\0\0"),
        w!("pdf"),
    ) {
        if let Ok(pdf_data) = crate::pdf::create_pdf_from_rgba(width, height, rgba_pixels) {
            if std::fs::write(&path, pdf_data).is_ok() {
                return Some(path);
            }
        }
    }
    None
}

pub fn hsv_to_rgb(h: f32, s: f32, v: f32) -> (u8, u8, u8) {
    let s = s.clamp(0.0, 1.0);
    let v = v.clamp(0.0, 1.0);
    let h_clean = ((h % 360.0) + 360.0) % 360.0;
    let c = v * s;
    let h_prime = h_clean / 60.0;
    let x = c * (1.0 - (h_prime % 2.0 - 1.0).abs());
    let m = v - c;

    let (r1, g1, b1) = if (0.0..1.0).contains(&h_prime) {
        (c, x, 0.0)
    } else if (1.0..2.0).contains(&h_prime) {
        (x, c, 0.0)
    } else if (2.0..3.0).contains(&h_prime) {
        (0.0, c, x)
    } else if (3.0..4.0).contains(&h_prime) {
        (0.0, x, c)
    } else if (4.0..5.0).contains(&h_prime) {
        (x, 0.0, c)
    } else {
        (c, 0.0, x)
    };

    (
        ((r1 + m) * 255.0).round().clamp(0.0, 255.0) as u8,
        ((g1 + m) * 255.0).round().clamp(0.0, 255.0) as u8,
        ((b1 + m) * 255.0).round().clamp(0.0, 255.0) as u8,
    )
}

pub fn rgb_to_hsv(r: u8, g: u8, b: u8) -> (f32, f32, f32) {
    let rf = (r as f32) / 255.0;
    let gf = (g as f32) / 255.0;
    let bf = (b as f32) / 255.0;

    let cmax = rf.max(gf).max(bf);
    let cmin = rf.min(gf).min(bf);
    let delta = cmax - cmin;

    let h = if delta == 0.0 {
        0.0
    } else if cmax == rf {
        60.0 * (((gf - bf) / delta) % 6.0)
    } else if cmax == gf {
        60.0 * (((bf - rf) / delta) + 2.0)
    } else {
        60.0 * (((rf - gf) / delta) + 4.0)
    };
    let h = if h < 0.0 { h + 360.0 } else { h };

    let s = if cmax == 0.0 { 0.0 } else { delta / cmax };
    let v = cmax;

    (h, s, v)
}
