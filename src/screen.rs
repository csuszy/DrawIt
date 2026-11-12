use std::mem;
use windows::Win32::Foundation::{BOOL, LPARAM, POINT, RECT};
use windows::Win32::Graphics::Gdi::{
    BitBlt, CreateCompatibleBitmap, CreateCompatibleDC, DeleteDC, DeleteObject,
    EnumDisplayMonitors, GetDC, GetDIBits, GetMonitorInfoW, ReleaseDC, SelectObject,
    BITMAPINFO, BITMAPINFOHEADER, BI_RGB, DIB_RGB_COLORS, HDC, HMONITOR, MONITORINFO,
    SRCCOPY, CAPTUREBLT,
};

const MONITORINFOF_PRIMARY: u32 = 1;
use windows::Win32::UI::WindowsAndMessaging::{
    GetCursorPos, GetSystemMetrics, SM_CXVIRTUALSCREEN, SM_CYVIRTUALSCREEN, SM_XVIRTUALSCREEN,
    SM_YVIRTUALSCREEN,
};

#[derive(Clone, Debug)]
pub struct MonitorInfo {
    pub index: usize,
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
    pub _is_primary: bool,
    pub name: String,
}

#[derive(Clone)]
pub struct ScreenCapture {
    pub left: i32,
    pub top: i32,
    pub width: u32,
    pub height: u32,
    pub monitors: Vec<MonitorInfo>,
    pub pixels: Vec<u8>,       // BGRA32
    pub blur_pixels: Vec<u8>,  // Pre-blurred BGRA32
}

/// Detects the display monitor currently containing the mouse cursor
pub fn get_focused_monitor<'a>(monitors: &'a [MonitorInfo]) -> Option<&'a MonitorInfo> {
    unsafe {
        let mut pt = POINT { x: 0, y: 0 };
        let _ = GetCursorPos(&mut pt);
        monitors
            .iter()
            .find(|m| {
                pt.x >= m.x && pt.x < m.x + m.width && pt.y >= m.y && pt.y < m.y + m.height
            })
            .or_else(|| monitors.first())
    }
}

extern "system" fn monitor_enum_proc(
    hmonitor: HMONITOR,
    _: HDC,
    _: *mut RECT,
    lparam: LPARAM,
) -> BOOL {
    unsafe {
        let list = &mut *(lparam.0 as *mut Vec<MonitorInfo>);
        let mut mi: MONITORINFO = mem::zeroed();
        mi.cbSize = mem::size_of::<MONITORINFO>() as u32;

        if GetMonitorInfoW(hmonitor, &mut mi).as_bool() {
            let idx = list.len();
            let is_primary = (mi.dwFlags & MONITORINFOF_PRIMARY) != 0;
            let w = mi.rcMonitor.right - mi.rcMonitor.left;
            let h = mi.rcMonitor.bottom - mi.rcMonitor.top;

            let name = if is_primary {
                format!("{}. Kijelző [Főképernyő] ({}×{})", idx + 1, w, h)
            } else {
                format!("{}. Kijelző ({}×{})", idx + 1, w, h)
            };

            list.push(MonitorInfo {
                index: idx,
                x: mi.rcMonitor.left,
                y: mi.rcMonitor.top,
                width: w,
                height: h,
                _is_primary: is_primary,
                name,
            });
        }
    }
    BOOL(1)
}

pub fn get_all_monitors() -> Vec<MonitorInfo> {
    let mut monitors = Vec::new();
    unsafe {
        let _ = EnumDisplayMonitors(
            None,
            None,
            Some(monitor_enum_proc),
            LPARAM(&mut monitors as *mut _ as isize),
        );
    }
    monitors
}

pub fn capture_screen() -> ScreenCapture {
    unsafe {
        let x = GetSystemMetrics(SM_XVIRTUALSCREEN);
        let y = GetSystemMetrics(SM_YVIRTUALSCREEN);
        let mut width = GetSystemMetrics(SM_CXVIRTUALSCREEN);
        let mut height = GetSystemMetrics(SM_CYVIRTUALSCREEN);

        if width <= 0 {
            width = 1920;
        }
        if height <= 0 {
            height = 1080;
        }

        let hdc_screen = GetDC(None);
        let hdc_mem = CreateCompatibleDC(hdc_screen);
        let h_bitmap = CreateCompatibleBitmap(hdc_screen, width, height);
        let h_old = SelectObject(hdc_mem, h_bitmap);

        // Capture full virtual desktop including translucent windows
        let _ = BitBlt(
            hdc_mem,
            0,
            0,
            width,
            height,
            hdc_screen,
            x,
            y,
            SRCCOPY | CAPTUREBLT,
        );

        let mut bmi: BITMAPINFO = mem::zeroed();
        bmi.bmiHeader.biSize = mem::size_of::<BITMAPINFOHEADER>() as u32;
        bmi.bmiHeader.biWidth = width;
        bmi.bmiHeader.biHeight = -height; // Top-down DIB
        bmi.bmiHeader.biPlanes = 1;
        bmi.bmiHeader.biBitCount = 32;
        bmi.bmiHeader.biCompression = BI_RGB.0;

        let total_pixels = (width as usize) * (height as usize);
        let mut pixels = vec![0u8; total_pixels * 4];

        GetDIBits(
            hdc_mem,
            h_bitmap,
            0,
            height as u32,
            Some(pixels.as_mut_ptr() as *mut _),
            &mut bmi,
            DIB_RGB_COLORS,
        );

        // Ensure full alpha on all pixels
        for i in 0..total_pixels {
            pixels[i * 4 + 3] = 255;
        }

        SelectObject(hdc_mem, h_old);
        let _ = DeleteObject(h_bitmap);
        let _ = DeleteDC(hdc_mem);
        ReleaseDC(None, hdc_screen);

        let monitors = get_all_monitors();
        let blur_pixels = generate_blur_pixels(&pixels, width as usize, height as usize);

        ScreenCapture {
            left: x,
            top: y,
            width: width as u32,
            height: height as u32,
            monitors,
            pixels,
            blur_pixels,
        }
    }
}

/// Fast downsampled box blur for frosted glass backdrop and blur tool
fn generate_blur_pixels(src: &[u8], width: usize, height: usize) -> Vec<u8> {
    if width < 16 || height < 16 {
        return src.to_vec();
    }

    // Downsample by 8x for ultra-fast calculation
    let scale = 8;
    let sw = width / scale;
    let sh = height / scale;
    let mut small = vec![0u8; sw * sh * 4];

    for sy in 0..sh {
        let orig_y = sy * scale;
        let s_row = sy * sw * 4;
        let o_row = orig_y * width * 4;
        for sx in 0..sw {
            let orig_x = sx * scale;
            let s_idx = s_row + sx * 4;
            let o_idx = o_row + orig_x * 4;
            small[s_idx] = src[o_idx];
            small[s_idx + 1] = src[o_idx + 1];
            small[s_idx + 2] = src[o_idx + 2];
            small[s_idx + 3] = 255;
        }
    }

    // Apply 3-pass box blur on small buffer (radius = 3)
    let radius = 3;
    let mut temp = small.clone();
    for _ in 0..3 {
        box_blur_horizontal(&small, &mut temp, sw, sh, radius);
        box_blur_vertical(&temp, &mut small, sw, sh, radius);
    }

    // Upsample back with bilinear interpolation
    let mut out = vec![255u8; width * height * 4];
    for y in 0..height {
        let sy = ((y * sh) / height).min(sh - 1);
        let o_row = y * width * 4;
        let s_row = sy * sw * 4;
        for x in 0..width {
            let sx = ((x * sw) / width).min(sw - 1);
            let o_idx = o_row + x * 4;
            let s_idx = s_row + sx * 4;
            out[o_idx] = small[s_idx];
            out[o_idx + 1] = small[s_idx + 1];
            out[o_idx + 2] = small[s_idx + 2];
            out[o_idx + 3] = 255;
        }
    }

    out
}

fn box_blur_horizontal(src: &[u8], dst: &mut [u8], w: usize, h: usize, r: usize) {
    let div = (2 * r + 1) as u32;
    for y in 0..h {
        let row = y * w * 4;
        let mut b_acc = 0u32;
        let mut g_acc = 0u32;
        let mut r_acc = 0u32;

        let first = row;
        let _last = row + (w - 1) * 4;

        for i in 0..=r {
            let idx = row + i.min(w - 1) * 4;
            b_acc += src[idx] as u32;
            g_acc += src[idx + 1] as u32;
            r_acc += src[idx + 2] as u32;
        }
        b_acc += (src[first] as u32) * (r as u32);
        g_acc += (src[first + 1] as u32) * (r as u32);
        r_acc += (src[first + 2] as u32) * (r as u32);

        for x in 0..w {
            let d_idx = row + x * 4;
            dst[d_idx] = (b_acc / div) as u8;
            dst[d_idx + 1] = (g_acc / div) as u8;
            dst[d_idx + 2] = (r_acc / div) as u8;
            dst[d_idx + 3] = 255;

            let left_x = if x >= r { (x - r) * 4 } else { 0 };
            let right_x = if x + r + 1 < w { (x + r + 1) * 4 } else { (w - 1) * 4 };

            b_acc += src[row + right_x] as u32;
            b_acc -= src[row + left_x] as u32;

            g_acc += src[row + right_x + 1] as u32;
            g_acc -= src[row + left_x + 1] as u32;

            r_acc += src[row + right_x + 2] as u32;
            r_acc -= src[row + left_x + 2] as u32;
        }
    }
}

fn box_blur_vertical(src: &[u8], dst: &mut [u8], w: usize, h: usize, r: usize) {
    let div = (2 * r + 1) as u32;
    for x in 0..w {
        let col = x * 4;
        let mut b_acc = 0u32;
        let mut g_acc = 0u32;
        let mut r_acc = 0u32;

        let first = col;
        let _last = (h - 1) * w * 4 + col;

        for i in 0..=r {
            let idx = i.min(h - 1) * w * 4 + col;
            b_acc += src[idx] as u32;
            g_acc += src[idx + 1] as u32;
            r_acc += src[idx + 2] as u32;
        }
        b_acc += (src[first] as u32) * (r as u32);
        g_acc += (src[first + 1] as u32) * (r as u32);
        r_acc += (src[first + 2] as u32) * (r as u32);

        for y in 0..h {
            let d_idx = y * w * 4 + col;
            dst[d_idx] = (b_acc / div) as u8;
            dst[d_idx + 1] = (g_acc / div) as u8;
            dst[d_idx + 2] = (r_acc / div) as u8;
            dst[d_idx + 3] = 255;

            let top_y = if y >= r { (y - r) * w * 4 } else { 0 };
            let bot_y = if y + r + 1 < h { (y + r + 1) * w * 4 } else { (h - 1) * w * 4 };

            b_acc += src[bot_y + col] as u32;
            b_acc -= src[top_y + col] as u32;

            g_acc += src[bot_y + col + 1] as u32;
            g_acc -= src[top_y + col + 1] as u32;

            r_acc += src[bot_y + col + 2] as u32;
            r_acc -= src[top_y + col + 2] as u32;
        }
    }
}

/// Crops a rectangular region from BGRA32 buffer
pub fn crop_bgra(
    src: &[u8],
    src_w: u32,
    src_h: u32,
    crop_x: u32,
    crop_y: u32,
    crop_w: u32,
    crop_h: u32,
) -> Vec<u8> {
    let mut out = vec![0u8; (crop_w as usize) * (crop_h as usize) * 4];
    for y in 0..crop_h {
        let sy = crop_y + y;
        if sy >= src_h {
            break;
        }
        let s_row = (sy * src_w + crop_x) as usize * 4;
        let d_row = (y * crop_w) as usize * 4;
        let row_bytes = (crop_w as usize * 4).min((src_w - crop_x) as usize * 4);
        out[d_row..d_row + row_bytes].copy_from_slice(&src[s_row..s_row + row_bytes]);
    }
    out
}

/// Converts BGRA32 to RGBA32
pub fn bgra_to_rgba(bgra: &[u8]) -> Vec<u8> {
    let mut rgba = bgra.to_vec();
    for chunk in rgba.chunks_exact_mut(4) {
        chunk.swap(0, 2); // Swap B and R
    }
    rgba
}
