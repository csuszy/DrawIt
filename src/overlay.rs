use std::mem;
use std::ptr;
use std::sync::atomic::{AtomicBool, Ordering};
use windows::core::{w, PCWSTR};
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, POINT, WPARAM};
use windows::Win32::Graphics::Direct2D::ID2D1Bitmap;
use windows::Win32::Graphics::Gdi::{
    BitBlt, CreateCompatibleBitmap, CreateCompatibleDC, DeleteDC, DeleteObject, GetDC, GetDIBits,
    ReleaseDC, SelectObject, UpdateWindow, BITMAPINFO, BITMAPINFOHEADER, BI_RGB, DIB_RGB_COLORS,
    SRCCOPY,
};
use windows::Win32::UI::Input::KeyboardAndMouse::{
    GetKeyState, RegisterHotKey, ReleaseCapture, SetActiveWindow, SetCapture, SetFocus,
    UnregisterHotKey, HOT_KEY_MODIFIERS, VK_CONTROL, VK_DELETE, VK_ESCAPE, VK_F1,
};
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, GetCursorPos, GetSystemMetrics, LoadCursorW, LoadImageW,
    PostQuitMessage, RegisterClassExW, SendMessageW, SetCursor, SetForegroundWindow,
    SetWindowLongPtrW, GetWindowLongPtrW, SetWindowPos, ShowWindow, CS_HREDRAW, CS_VREDRAW, GWLP_USERDATA,
    HCURSOR, HICON, HWND_TOPMOST, IDC_ARROW, IDC_CROSS, IDC_HAND, IDC_IBEAM, IDC_SIZEALL,
    IMAGE_ICON, LR_DEFAULTCOLOR, LR_SHARED, SM_CXICON, SM_CXSMICON, SM_CYICON, SM_CYSMICON,
    SW_HIDE, SW_SHOW, WM_CHAR, WM_DESTROY, WM_ERASEBKGND, WM_HOTKEY,
    WM_KEYDOWN, WM_LBUTTONDOWN, WM_LBUTTONUP, WM_MOUSEMOVE, WM_PAINT, WM_RBUTTONDOWN,
    WM_SETCURSOR, WM_SETICON, WM_TIMER, WM_MOUSEWHEEL, WNDCLASSEXW, WS_EX_TOOLWINDOW, WS_EX_TOPMOST, WS_POPUP,
};

use crate::d2d::D2DContext;
use crate::models::{ActiveModal, CanvasMode, DrawAction, DrawingHistory, ToolMode};
use crate::screen::{bgra_to_rgba, capture_screen, crop_bgra, ScreenCapture};
use crate::toolbar::{HitTestResult, OpenPopup, PopupAction, Toolbar, ToolbarItem};
use crate::utils::{
    copy_bgra_to_clipboard, create_anchor_for_monitor_by_cursor,
    save_pdf_dialog, save_png_dialog,
};

pub static IS_OVERLAY_OPEN: AtomicBool = AtomicBool::new(false);

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum SnipAction {
    Copy,
    SavePng,
    SavePdf,
    Cancel,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum CurvePhase {
    Idle,
    Bending { start: (f32, f32), end: (f32, f32) },
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ModalButton {
    Close,
    Action,
}

pub struct OverlayState {
    pub hwnd: HWND,
    pub capture: ScreenCapture,
    pub d2d: Option<D2DContext>,
    pub bg_bitmap: Option<ID2D1Bitmap>,
    pub blur_bitmap: Option<ID2D1Bitmap>,
    pub toolbar: Toolbar,
    pub history: DrawingHistory,

    // Mouse interaction states
    pub is_drawing: bool,
    pub is_dragging_toolbar: bool,
    pub drag_start_mouse: (f32, f32),
    pub drag_start_bar: (f32, f32),

    // Active tool data
    pub current_points: Vec<(f32, f32)>,
    pub draw_start: (f32, f32),
    pub draw_current: (f32, f32),
    pub curve_phase: CurvePhase,

    // Inline text editing
    pub text_pos: Option<(f32, f32)>,
    pub text_buffer: String,
    pub text_cursor_visible: bool,

    // Snipping tool
    pub is_snipping: bool,
    pub snip_start: (f32, f32),
    pub snip_rect: Option<(f32, f32, f32, f32)>,
    pub hovered_snip_action: Option<SnipAction>,

    // Cursor handles
    pub h_cross: HCURSOR,
    pub h_arrow: HCURSOR,
    pub h_ibeam: HCURSOR,
    pub h_hand: HCURSOR,
    pub h_sizeall: HCURSOR,

    // Suppress toolbar rendering during screen capture export
    pub hide_toolbar_during_capture: bool,

    // On-screen HUD Toast notification
    pub toast_message: Option<String>,
    pub toast_until: std::time::Instant,

    // Custom color dragging & pipette
    pub is_dragging_hue: bool,
    pub is_dragging_lightness: bool,
    pub pipette_buffer: Option<Vec<u8>>,
    pub previous_tool_before_pipette: ToolMode,
    pub active_modal: ActiveModal,
    pub hovered_modal_btn: Option<ModalButton>,
}

impl OverlayState {
    pub fn new(hwnd: HWND, capture: ScreenCapture) -> Self {
        let (h_cross, h_arrow, h_ibeam, h_hand, h_sizeall) = unsafe {
            (
                LoadCursorW(None, IDC_CROSS).unwrap_or_default(),
                LoadCursorW(None, IDC_ARROW).unwrap_or_default(),
                LoadCursorW(None, IDC_IBEAM).unwrap_or_default(),
                LoadCursorW(None, IDC_HAND).unwrap_or_default(),
                LoadCursorW(None, IDC_SIZEALL).unwrap_or_default(),
            )
        };

        let toolbar = Toolbar::new(capture.width as f32, capture.monitors.clone());

        Self {
            hwnd,
            capture,
            d2d: None,
            bg_bitmap: None,
            blur_bitmap: None,
            toolbar,
            history: DrawingHistory::new(),
            is_drawing: false,
            is_dragging_toolbar: false,
            drag_start_mouse: (0.0, 0.0),
            drag_start_bar: (0.0, 0.0),
            current_points: Vec::new(),
            draw_start: (0.0, 0.0),
            draw_current: (0.0, 0.0),
            curve_phase: CurvePhase::Idle,
            text_pos: None,
            text_buffer: String::new(),
            text_cursor_visible: true,
            is_snipping: false,
            snip_start: (0.0, 0.0),
            snip_rect: None,
            h_cross,
            h_arrow,
            h_ibeam,
            h_hand,
            h_sizeall,
            hide_toolbar_during_capture: false,
            toast_message: None,
            toast_until: std::time::Instant::now(),
            hovered_snip_action: None,
            is_dragging_hue: false,
            is_dragging_lightness: false,
            pipette_buffer: None,
            previous_tool_before_pipette: ToolMode::Freehand,
            active_modal: ActiveModal::None,
            hovered_modal_btn: None,
        }
    }

    /// Displays an on-screen Toast HUD notification centered on the active focused monitor
    pub fn show_toast(&mut self, msg: &str) {
        self.toast_message = Some(msg.to_string());
        self.toast_until = std::time::Instant::now() + std::time::Duration::from_millis(2200);
        unsafe {
            let _ = windows::Win32::UI::WindowsAndMessaging::SetTimer(self.hwnd, 9999, 2200, None);
        }
        self.render();
    }

    pub fn commit_text_if_any(&mut self) {
        if let Some(pos) = self.text_pos.take() {
            let trimmed = self.text_buffer.trim();
            if !trimmed.is_empty() {
                let font_size = self.toolbar.thickness * 3.5 + 14.0;
                self.history.push(DrawAction::Text {
                    pos,
                    text: trimmed.to_string(),
                    color: self.toolbar.color,
                    font_size,
                });
            }
            self.text_buffer.clear();
        }
    }

    /// Computes the exact layout and interactive regions for the snip action buttons
    pub fn layout_snip_bar(&self) -> Option<((f32, f32, f32, f32), Vec<(SnipAction, (f32, f32, f32, f32))>)> {
        let (sl, st, sr, sb) = self.snip_rect?;
        if self.is_snipping {
            return None;
        }

        let w_px = (sr - sl).abs().round();
        let h_px = (sb - st).abs().round();
        if w_px <= 15.0 || h_px <= 15.0 {
            return None;
        }

        let sw = self.capture.width as f32;
        let sh = self.capture.height as f32;

        let bar_w = 368.0;
        let bar_h = 40.0;
        let bar_x = sl.clamp(10.0, sw - bar_w - 10.0);
        let bar_y = (sb + 8.0).min(sh - bar_h - 10.0);

        let btn_y = bar_y + 4.0;
        let btn_h = 32.0;

        let mut actions = Vec::new();
        actions.push((SnipAction::Copy, (bar_x + 6.0, btn_y, bar_x + 94.0, btn_y + btn_h)));
        actions.push((SnipAction::SavePng, (bar_x + 98.0, btn_y, bar_x + 182.0, btn_y + btn_h)));
        actions.push((SnipAction::SavePdf, (bar_x + 186.0, btn_y, bar_x + 270.0, btn_y + btn_h)));
        actions.push((SnipAction::Cancel, (bar_x + 274.0, btn_y, bar_x + 362.0, btn_y + btn_h)));

        Some(((bar_x, bar_y, bar_x + bar_w, bar_y + bar_h), actions))
    }

    /// Checks if mouse coordinates are over the floating snip action bar
    pub fn is_over_snip_bar(&self, mx: f32, my: f32) -> bool {
        if let Some((bar_rect, _)) = self.layout_snip_bar() {
            mx >= bar_rect.0 && mx <= bar_rect.2 && my >= bar_rect.1 && my <= bar_rect.3
        } else {
            false
        }
    }

    /// Renders custom brush cursor indicator representing size and color:
    /// - Small thickness (<= 5px): Colored '+' crosshair with size representation
    /// - Larger thickness (> 5px): Sized colored dot/circle showing exact stroke width
    pub fn render_brush_cursor(&self, d2d: &D2DContext) {
        let (mx, my) = self.draw_current;

        // Do not render brush cursor over toolbar, popups or snip controls
        if self.toolbar.is_inside(mx, my) || self.is_over_snip_bar(mx, my) {
            return;
        }

        // Only render for drawing tools (not Text or Snip)
        if !matches!(
            self.toolbar.active_tool,
            ToolMode::Freehand
                | ToolMode::Line
                | ToolMode::Rectangle
                | ToolMode::FilledRectangle
                | ToolMode::Circle
                | ToolMode::Arrow
                | ToolMode::NumberedBadge
                | ToolMode::Curve
                | ToolMode::Blur
        ) {
            return;
        }

        let th = self.toolbar.thickness;
        let col = if self.toolbar.active_tool == ToolMode::Blur {
            (0.2, 0.7, 1.0, 0.95)
        } else {
            (
                (self.toolbar.color.0 as f32) / 255.0,
                (self.toolbar.color.1 as f32) / 255.0,
                (self.toolbar.color.2 as f32) / 255.0,
                0.95,
            )
        };

        if th <= 5.0 {
            // "kis méret esetében egy szines + ami a méretet reprezentálja"
            let arm = (th * 1.5 + 4.0).clamp(6.0, 11.0);
            let stroke_w = (th * 0.75).clamp(1.2, 2.4);

            // Subtle high-contrast shadow behind crosshair
            d2d.draw_line(mx - arm, my, mx + arm, my, (0.0, 0.0, 0.0, 0.50), stroke_w + 1.2);
            d2d.draw_line(mx, my - arm, mx, my + arm, (0.0, 0.0, 0.0, 0.50), stroke_w + 1.2);

            // Colored crosshair arms representing active color and size
            d2d.draw_line(mx - arm, my, mx + arm, my, col, stroke_w);
            d2d.draw_line(mx, my - arm, mx, my + arm, col, stroke_w);

            // Center targeting point
            let dot_r = (th * 0.5).clamp(1.2, 2.5);
            d2d.fill_ellipse(mx, my, dot_r, dot_r, (1.0, 1.0, 1.0, 0.95));
        } else {
            // "egy kis pont jelenjen meg ... ami a méretet reprezentálja"
            let r = th / 2.0;

            // Semi-transparent colored brush preview
            d2d.fill_ellipse(mx, my, r, r, (col.0, col.1, col.2, 0.35));

            // Outer dark shadow ring for high contrast
            d2d.draw_ellipse(mx, my, r + 0.8, r + 0.8, (0.0, 0.0, 0.0, 0.45), 1.0);

            // Crisp colored ring representing exact brush perimeter
            d2d.draw_ellipse(mx, my, r, r, col, 1.5);

            // Inner white highlight ring for dark backgrounds
            if r > 5.0 {
                d2d.draw_ellipse(mx, my, (r - 1.2).max(1.0), (r - 1.2).max(1.0), (1.0, 1.0, 1.0, 0.70), 0.8);
            }

            // Center targeting dot
            d2d.fill_ellipse(mx, my, 1.5, 1.5, (1.0, 1.0, 1.0, 0.95));
            d2d.draw_ellipse(mx, my, 1.5, 1.5, (0.0, 0.0, 0.0, 0.6), 0.8);
        }
    }

    pub fn sample_color_at(&self, mx: f32, my: f32) -> (u8, u8, u8) {
        let w = self.capture.width as usize;
        let h = self.capture.height as usize;
        let px = mx.round().clamp(0.0, (w.saturating_sub(1)) as f32) as usize;
        let py = my.round().clamp(0.0, (h.saturating_sub(1)) as f32) as usize;
        let idx = (py * w + px) * 4;

        if let Some(buf) = &self.pipette_buffer {
            if idx + 2 < buf.len() {
                let b = buf[idx];
                let g = buf[idx + 1];
                let r = buf[idx + 2];
                return (r, g, b);
            }
        }
        if idx + 2 < self.capture.pixels.len() {
            let b = self.capture.pixels[idx];
            let g = self.capture.pixels[idx + 1];
            let r = self.capture.pixels[idx + 2];
            return (r, g, b);
        }
        (255, 255, 255)
    }

    pub fn render_pipette_loupe(&self, d2d: &D2DContext) {
        let (mx, my) = self.draw_current;
        let (r, g, b) = self.sample_color_at(mx, my);
        let cr = (r as f32) / 255.0;
        let cg = (g as f32) / 255.0;
        let cb = (b as f32) / 255.0;

        // 1. Crosshair at cursor point
        let ch_sz = 10.0;
        d2d.draw_line(mx - ch_sz, my, mx + ch_sz, my, (0.0, 0.0, 0.0, 0.8), 2.5);
        d2d.draw_line(mx, my - ch_sz, mx, my + ch_sz, (0.0, 0.0, 0.0, 0.8), 2.5);
        d2d.draw_line(mx - ch_sz, my, mx + ch_sz, my, (1.0, 1.0, 1.0, 1.0), 1.2);
        d2d.draw_line(mx, my - ch_sz, mx, my + ch_sz, (1.0, 1.0, 1.0, 1.0), 1.2);
        d2d.fill_ellipse(mx, my, 2.0, 2.0, (cr, cg, cb, 1.0));
        d2d.draw_ellipse(mx, my, 2.0, 2.0, (0.0, 0.0, 0.0, 0.9), 1.0);

        // 2. Loupe position (prevent clipping off edges)
        let sw = self.capture.width as f32;
        let sh = self.capture.height as f32;
        let mut lx = mx + 24.0;
        let mut ly = my - 70.0;
        if lx + 120.0 > sw {
            lx = mx - 130.0;
        }
        if ly < 20.0 {
            ly = my + 30.0;
        }
        if ly + 90.0 > sh {
            ly = sh - 95.0;
        }

        let radius = 24.0;
        let loupe_cx = lx + 55.0;
        let loupe_cy = ly + radius;

        // Drop shadow
        d2d.fill_ellipse(loupe_cx, loupe_cy + 2.0, radius + 2.0, radius + 2.0, (0.0, 0.0, 0.0, 0.40));
        // Outer dark casing
        d2d.fill_ellipse(loupe_cx, loupe_cy, radius, radius, (0.09, 0.11, 0.15, 0.96));
        // Outer white highlight ring
        d2d.draw_ellipse(loupe_cx, loupe_cy, radius, radius, (1.0, 1.0, 1.0, 0.85), 2.0);
        // Inner sampled color disc
        d2d.fill_ellipse(loupe_cx, loupe_cy, radius - 3.5, radius - 3.5, (cr, cg, cb, 1.0));
        d2d.draw_ellipse(loupe_cx, loupe_cy, radius - 3.5, radius - 3.5, (0.0, 0.0, 0.0, 0.35), 1.0);

        // 3. Information Pill below loupe
        let pill_w = 110.0;
        let pill_h = 32.0;
        let pill_l = loupe_cx - pill_w / 2.0;
        let pill_t = loupe_cy + radius + 4.0;
        let pill_r = pill_l + pill_w;
        let pill_b = pill_t + pill_h;

        d2d.fill_rounded_rectangle(pill_l - 1.0, pill_t + 1.0, pill_r + 1.0, pill_b + 2.0, 7.0, (0.0, 0.0, 0.0, 0.40));
        d2d.fill_rounded_rectangle(pill_l, pill_t, pill_r, pill_b, 7.0, (0.09, 0.11, 0.15, 0.96));
        d2d.draw_rounded_rectangle(pill_l, pill_t, pill_r, pill_b, 7.0, (1.0, 1.0, 1.0, 0.20), 1.0);

        let hex_str = format!("#{:02X}{:02X}{:02X}", r, g, b);
        d2d.draw_text_simple(&hex_str, pill_l, pill_t + 2.0, pill_r, pill_t + 17.0, 11.5, (1.0, 1.0, 1.0, 1.0), true, true);
        let rgb_str = format!("RGB({}, {}, {})", r, g, b);
        d2d.draw_text_simple(&rgb_str, pill_l, pill_t + 17.0, pill_r, pill_b - 1.0, 9.5, (0.75, 0.78, 0.84, 1.0), false, true);
    }

    pub fn refresh_screen_capture(&mut self) {
        self.capture = capture_screen();
        self.toolbar.monitors = self.capture.monitors.clone();
        self.toolbar.screen_width = self.capture.width as f32;
        if let Some(d2d) = &self.d2d {
            self.bg_bitmap = d2d
                .create_bitmap_from_bgra(self.capture.width, self.capture.height, &self.capture.pixels)
                .ok();
            self.blur_bitmap = d2d
                .create_bitmap_from_bgra(
                    self.capture.width,
                    self.capture.height,
                    &self.capture.blur_pixels,
                )
                .ok();
        }
    }

    /// Captures the rendered Direct2D window back-buffer into a BGRA buffer
    pub fn capture_rendered_canvas(&mut self) -> Vec<u8> {
        self.hide_toolbar_during_capture = true;
        self.render();
        self.hide_toolbar_during_capture = false;

        unsafe {
            let w = self.capture.width;
            let h = self.capture.height;
            let hdc_wnd = GetDC(self.hwnd);
            let hdc_mem = CreateCompatibleDC(hdc_wnd);
            let h_bmp = CreateCompatibleBitmap(hdc_wnd, w as i32, h as i32);
            let h_old = SelectObject(hdc_mem, h_bmp);

            let _ = BitBlt(hdc_mem, 0, 0, w as i32, h as i32, hdc_wnd, 0, 0, SRCCOPY);

            let mut bmi: BITMAPINFO = mem::zeroed();
            bmi.bmiHeader.biSize = mem::size_of::<BITMAPINFOHEADER>() as u32;
            bmi.bmiHeader.biWidth = w as i32;
            bmi.bmiHeader.biHeight = -(h as i32);
            bmi.bmiHeader.biPlanes = 1;
            bmi.bmiHeader.biBitCount = 32;
            bmi.bmiHeader.biCompression = BI_RGB.0;

            let mut pixels = vec![0u8; (w as usize) * (h as usize) * 4];
            GetDIBits(
                hdc_mem,
                h_bmp,
                0,
                h,
                Some(pixels.as_mut_ptr() as *mut _),
                &mut bmi,
                DIB_RGB_COLORS,
            );

            SelectObject(hdc_mem, h_old);
            let _ = DeleteObject(h_bmp);
            let _ = DeleteDC(hdc_mem);
            ReleaseDC(self.hwnd, hdc_wnd);

            // Re-render toolbar
            self.render();

            pixels
        }
    }

    /// Exports a specific monitor (or virtual desktop) to PNG, PDF, or Clipboard
    pub fn export_display_action(&mut self, monitor_idx: usize, format: &str) {
        let full_bgra = self.capture_rendered_canvas();
        let full_w = self.capture.width;
        let full_h = self.capture.height;

        let (crop_x, crop_y, crop_w, crop_h, label) = if monitor_idx == usize::MAX {
            (0, 0, full_w, full_h, "DrawIt_Teljes_Asztal".to_string())
        } else if let Some(m) = self.capture.monitors.iter().find(|m| m.index == monitor_idx) {
            let cx = (m.x - self.capture.left).max(0) as u32;
            let cy = (m.y - self.capture.top).max(0) as u32;
            (cx, cy, m.width as u32, m.height as u32, format!("DrawIt_{}_Kijelzo", m.index + 1))
        } else {
            (0, 0, full_w, full_h, "DrawIt_Képernyő".to_string())
        };

        let cropped_bgra = crop_bgra(&full_bgra, full_w, full_h, crop_x, crop_y, crop_w, crop_h);

        match format {
            "clipboard" => {
                if copy_bgra_to_clipboard(self.hwnd, crop_w, crop_h, &cropped_bgra) {
                    self.show_toast("✓ Kijelző képe másolva a vágólapra!");
                }
            }
            "png" => {
                let rgba = bgra_to_rgba(&cropped_bgra);
                let default_name = format!("{}_{}.png", label, chrono_timestamp());
                let anchor = create_anchor_for_monitor_by_cursor(&self.capture, self.hwnd);
                let saved = save_png_dialog(anchor, &default_name, crop_w, crop_h, &rgba);
                unsafe { let _ = windows::Win32::UI::WindowsAndMessaging::DestroyWindow(anchor); }
                if saved.is_some() {
                    self.show_toast("✓ Kijelző képe elmentve PNG fájlba!");
                }
            }
            "pdf" => {
                let rgba = bgra_to_rgba(&cropped_bgra);
                let default_name = format!("{}_{}.pdf", label, chrono_timestamp());
                let anchor = create_anchor_for_monitor_by_cursor(&self.capture, self.hwnd);
                let saved = save_pdf_dialog(anchor, &default_name, crop_w, crop_h, &rgba);
                unsafe { let _ = windows::Win32::UI::WindowsAndMessaging::DestroyWindow(anchor); }
                if saved.is_some() {
                    self.show_toast("✓ Kijelző elmentve PDF fájlba!");
                }
            }
            _ => {}
        }
    }

    pub fn render(&mut self) {
        let d2d = match &self.d2d {
            Some(d) => d,
            None => return,
        };

        d2d.begin_draw();

        // 1. Draw Canvas Background
        match self.toolbar.canvas_mode {
            CanvasMode::Transparent => {
                if let Some(bmp) = &self.bg_bitmap {
                    d2d.draw_bitmap(bmp, 0.0, 0.0, self.capture.width as f32, self.capture.height as f32, 1.0);
                } else {
                    d2d.clear(0.0, 0.0, 0.0, 1.0);
                }
            }
            CanvasMode::Blurred => {
                if let Some(bmp) = &self.blur_bitmap {
                    d2d.draw_bitmap(bmp, 0.0, 0.0, self.capture.width as f32, self.capture.height as f32, 1.0);
                } else {
                    d2d.clear(0.1, 0.12, 0.15, 1.0);
                }
            }
            CanvasMode::Whiteboard => {
                d2d.clear(1.0, 1.0, 1.0, 1.0);
            }
            CanvasMode::Blackboard => {
                d2d.clear(0.08, 0.09, 0.11, 1.0);
            }
        }

        // 2. Render Committed Drawing Actions
        for action in &self.history.actions {
            self.render_action(d2d, action);
        }

        // 3. Render Live Interactive Preview
        if self.is_drawing {
            let col = (
                (self.toolbar.color.0 as f32) / 255.0,
                (self.toolbar.color.1 as f32) / 255.0,
                (self.toolbar.color.2 as f32) / 255.0,
                self.toolbar.opacity,
            );
            let th = self.toolbar.thickness;

            match self.toolbar.active_tool {
                ToolMode::Freehand => {
                    d2d.draw_polyline(&self.current_points, col, th);
                }
                ToolMode::Line => {
                    d2d.draw_line(self.draw_start.0, self.draw_start.1, self.draw_current.0, self.draw_current.1, col, th);
                }
                ToolMode::Rectangle => {
                    let l = self.draw_start.0.min(self.draw_current.0);
                    let t = self.draw_start.1.min(self.draw_current.1);
                    let r = self.draw_start.0.max(self.draw_current.0);
                    let b = self.draw_start.1.max(self.draw_current.1);
                    d2d.draw_rectangle(l, t, r, b, col, th);
                }
                ToolMode::FilledRectangle => {
                    let l = self.draw_start.0.min(self.draw_current.0);
                    let t = self.draw_start.1.min(self.draw_current.1);
                    let r = self.draw_start.0.max(self.draw_current.0);
                    let b = self.draw_start.1.max(self.draw_current.1);
                    d2d.fill_rectangle(l, t, r, b, col);
                    let border_col = (col.0, col.1, col.2, (col.3 + 0.2).min(1.0));
                    d2d.draw_rectangle(l, t, r, b, border_col, 1.0);
                }
                ToolMode::Circle => {
                    let cx = (self.draw_start.0 + self.draw_current.0) / 2.0;
                    let cy = (self.draw_start.1 + self.draw_current.1) / 2.0;
                    let rx = (self.draw_current.0 - self.draw_start.0).abs() / 2.0;
                    let ry = (self.draw_current.1 - self.draw_start.1).abs() / 2.0;
                    d2d.draw_ellipse(cx, cy, rx, ry, col, th);
                }
                ToolMode::Arrow => {
                    d2d.draw_arrow(self.draw_start.0, self.draw_start.1, self.draw_current.0, self.draw_current.1, col, th);
                }
                ToolMode::NumberedBadge => {
                    // NUMBERED ARROW LIVE PREVIEW
                    let dx = self.draw_current.0 - self.draw_start.0;
                    let dy = self.draw_current.1 - self.draw_start.1;
                    let dist = (dx * dx + dy * dy).sqrt();

                    if dist > 8.0 {
                        d2d.draw_arrow(self.draw_start.0, self.draw_start.1, self.draw_current.0, self.draw_current.1, col, th);
                    }

                    // Number badge at arrow start
                    self.render_badge(d2d, self.draw_start.0, self.draw_start.1, self.history.next_number, col, th);
                }
                ToolMode::Curve => {
                    // PHASE 1 LIVE LINE PREVIEW
                    d2d.draw_line(self.draw_start.0, self.draw_start.1, self.draw_current.0, self.draw_current.1, col, th);
                    d2d.fill_ellipse(self.draw_start.0, self.draw_start.1, 4.0, 4.0, (1.0, 1.0, 1.0, 0.8));
                    d2d.fill_ellipse(self.draw_current.0, self.draw_current.1, 4.0, 4.0, (1.0, 1.0, 1.0, 0.8));
                }
                ToolMode::Blur => {
                    let l = self.draw_start.0.min(self.draw_current.0);
                    let t = self.draw_start.1.min(self.draw_current.1);
                    let r = self.draw_start.0.max(self.draw_current.0);
                    let b = self.draw_start.1.max(self.draw_current.1);
                    if let Some(blur_bmp) = &self.blur_bitmap {
                        d2d.draw_bitmap_crop(blur_bmp, l, t, r, b, l, t, r, b, 1.0);
                    }
                    d2d.draw_rectangle(l, t, r, b, (0.04, 0.52, 1.0, 0.7), 1.5);
                }
                _ => {}
            }
        }

        // Live preview for Curve bending (Phase 2)
        if let CurvePhase::Bending { start, end } = self.curve_phase {
            let col = (
                (self.toolbar.color.0 as f32) / 255.0,
                (self.toolbar.color.1 as f32) / 255.0,
                (self.toolbar.color.2 as f32) / 255.0,
                self.toolbar.opacity,
            );
            // In quadratic Bezier, to pass directly through mouse cursor:
            let ctrl_x = 2.0 * self.draw_current.0 - 0.5 * (start.0 + end.0);
            let ctrl_y = 2.0 * self.draw_current.1 - 0.5 * (start.1 + end.1);

            d2d.draw_quadratic_curve(start.0, start.1, ctrl_x, ctrl_y, end.0, end.1, col, self.toolbar.thickness);

            // Anchor points
            d2d.fill_ellipse(start.0, start.1, 4.0, 4.0, (1.0, 1.0, 1.0, 0.8));
            d2d.fill_ellipse(end.0, end.1, 4.0, 4.0, (1.0, 1.0, 1.0, 0.8));
            d2d.fill_ellipse(self.draw_current.0, self.draw_current.1, 3.0, 3.0, (0.04, 0.52, 1.0, 0.9));
        }

        // 4. Render Active Inline Text Editing
        if let Some(pos) = self.text_pos {
            let col = (
                (self.toolbar.color.0 as f32) / 255.0,
                (self.toolbar.color.1 as f32) / 255.0,
                (self.toolbar.color.2 as f32) / 255.0,
                1.0,
            );
            let font_size = self.toolbar.thickness * 3.5 + 14.0;
            let display_text = if self.text_cursor_visible {
                format!("{}|", self.text_buffer)
            } else {
                format!("{} ", self.text_buffer)
            };
            d2d.draw_text_simple(&display_text, pos.0, pos.1, pos.0 + 800.0, pos.1 + font_size + 10.0, font_size, col, true, false);
        }

        // 5. Render Snipping Tool Overlay
        if self.toolbar.active_tool == ToolMode::Snip {
            self.render_snip_overlay(d2d);
        }

        // 6. Render Floating Toolbar and HUD Toast Notifications
        if !self.hide_toolbar_during_capture {
            self.toolbar.can_undo = !self.history.actions.is_empty();
            self.toolbar.can_redo = !self.history.redo_stack.is_empty();
            self.toolbar.render(d2d);
            self.render_toast(d2d);
        }

        // 7. Render Pipette Loupe (if in pipette eyedropper mode)
        if !self.hide_toolbar_during_capture && self.toolbar.active_tool == ToolMode::Pipette {
            self.render_pipette_loupe(d2d);
        }

        // 8. Render Custom Brush Cursor Indicator (Small colored + or sized dot)
        if !self.hide_toolbar_during_capture && self.toolbar.active_tool != ToolMode::Pipette {
            self.render_brush_cursor(d2d);
        }

        // 9. Render In-App Modal Dialog (Help / About)
        match self.active_modal {
            ActiveModal::Help => self.render_help_modal(d2d),
            ActiveModal::About => self.render_about_modal(d2d),
            ActiveModal::None => {}
        }

        let _ = d2d.end_draw();
    }

    /// Renders an Apple-clean glassmorphic toast notification centered horizontally on the focused monitor
    fn render_toast(&self, d2d: &D2DContext) {
        if let Some(msg) = &self.toast_message {
            if std::time::Instant::now() >= self.toast_until {
                return;
            }

            let active_mon = crate::screen::get_focused_monitor(&self.capture.monitors)
                .or_else(|| self.capture.monitors.first());

            let (cx, toast_y) = if let Some(m) = active_mon {
                let mon_left = (m.x - self.capture.left) as f32;
                let mon_top = (m.y - self.capture.top) as f32;
                let mon_w = m.width as f32;
                let mon_h = m.height as f32;
                (mon_left + mon_w / 2.0, mon_top + mon_h - 90.0)
            } else {
                let sw = self.capture.width as f32;
                let sh = self.capture.height as f32;
                (sw / 2.0, sh - 90.0)
            };

            let text_len = msg.chars().count() as f32;
            let pill_w = (text_len * 9.5 + 44.0).clamp(180.0, 580.0);
            let pill_h = 38.0;
            let pill_l = cx - pill_w / 2.0;
            let pill_t = toast_y - pill_h / 2.0;
            let pill_r = pill_l + pill_w;
            let pill_b = pill_t + pill_h;
            let radius = 19.0;

            // Drop shadow
            d2d.fill_rounded_rectangle(pill_l - 2.0, pill_t + 2.0, pill_r + 2.0, pill_b + 4.0, radius, (0.0, 0.0, 0.0, 0.35));
            // Translucent dark frosted background
            d2d.fill_rounded_rectangle(pill_l, pill_t, pill_r, pill_b, radius, (0.07, 0.09, 0.14, 0.94));
            // Soft border highlight
            d2d.draw_rounded_rectangle(pill_l, pill_t, pill_r, pill_b, radius, (0.22, 0.74, 0.97, 0.70), 1.2);
            // Centered text
            d2d.draw_text_simple(msg, pill_l + 12.0, pill_t + 2.0, pill_r - 12.0, pill_b - 2.0, 13.5, (1.0, 1.0, 1.0, 0.98), true, true);
        }
    }

    pub fn get_active_monitor_rect(&self) -> (f32, f32, f32, f32) {
        let mut cursor_pt = windows::Win32::Foundation::POINT { x: 0, y: 0 };
        unsafe {
            let _ = windows::Win32::UI::WindowsAndMessaging::GetCursorPos(&mut cursor_pt);
        }
        let active_mon = self.capture.monitors.iter().find(|m| {
            cursor_pt.x >= m.x && cursor_pt.x < m.x + m.width &&
            cursor_pt.y >= m.y && cursor_pt.y < m.y + m.height
        }).or_else(|| self.capture.monitors.first());

        if let Some(m) = active_mon {
            ((m.x - self.capture.left) as f32, (m.y - self.capture.top) as f32, m.width as f32, m.height as f32)
        } else {
            (0.0, 0.0, self.capture.width as f32, self.capture.height as f32)
        }
    }

    pub fn get_modal_layout(&self) -> Option<((f32, f32, f32, f32), (f32, f32, f32, f32), (f32, f32, f32, f32))> {
        let (mon_x, mon_y, mon_w, mon_h) = self.get_active_monitor_rect();

        match self.active_modal {
            ActiveModal::None => None,
            ActiveModal::Help => {
                let mw = 620.0;
                let mh = 530.0;
                let mx = (mon_x + (mon_w - mw) / 2.0).max(mon_x + 10.0);
                let my = (mon_y + (mon_h - mh) / 2.0).max(mon_y + 10.0);

                let card_rect = (mx, my, mx + mw, my + mh);
                let close_btn = (mx + mw - 38.0, my + 10.0, mx + mw - 10.0, my + 38.0);
                let action_btn = (mx + (mw - 130.0) / 2.0, my + mh - 42.0, mx + (mw + 130.0) / 2.0, my + mh - 12.0);

                Some((card_rect, close_btn, action_btn))
            }
            ActiveModal::About => {
                let mw = 460.0;
                let mh = 290.0;
                let mx = (mon_x + (mon_w - mw) / 2.0).max(mon_x + 10.0);
                let my = (mon_y + (mon_h - mh) / 2.0).max(mon_y + 10.0);

                let card_rect = (mx, my, mx + mw, my + mh);
                let close_btn = (mx + mw - 38.0, my + 10.0, mx + mw - 10.0, my + 38.0);
                let action_btn = (mx + (mw - 130.0) / 2.0, my + mh - 42.0, mx + (mw + 130.0) / 2.0, my + mh - 12.0);

                Some((card_rect, close_btn, action_btn))
            }
        }
    }

    fn render_help_modal(&self, d2d: &D2DContext) {
        let (card, close, act) = match self.get_modal_layout() {
            Some(l) => l,
            None => return,
        };

        let (mx, my, mx2, my2) = card;
        let mw = mx2 - mx;

        // 1. Semi-transparent dark overlay dimming
        d2d.fill_rectangle(0.0, 0.0, self.capture.width as f32, self.capture.height as f32, (0.0, 0.0, 0.0, 0.55));

        // 2. Drop shadows
        d2d.fill_rounded_rectangle(mx - 4.0, my + 4.0, mx2 + 4.0, my2 + 12.0, 16.0, (0.0, 0.0, 0.0, 0.45));
        d2d.fill_rounded_rectangle(mx - 2.0, my + 2.0, mx2 + 2.0, my2 + 6.0, 16.0, (0.0, 0.0, 0.0, 0.30));

        // 3. Card body: dark frosted glass
        d2d.fill_rounded_rectangle(mx, my, mx2, my2, 16.0, (0.10, 0.12, 0.16, 0.97));
        d2d.draw_rounded_rectangle(mx, my, mx2, my2, 16.0, (1.0, 1.0, 1.0, 0.16), 1.2);

        // 4. Header title & close button
        d2d.draw_help_book_icon(mx + 30.0, my + 25.0, (0.22, 0.74, 0.97, 1.0), 1.6);
        d2d.draw_text_simple("DrawIt – Útmutató és Gyorsbillentyűk", mx + 44.0, my + 12.0, mx + mw - 50.0, my + 38.0, 15.0, (1.0, 1.0, 1.0, 1.0), true, false);

        let is_close_hov = self.hovered_modal_btn == Some(ModalButton::Close);
        let close_cx = (close.0 + close.2) / 2.0;
        let close_cy = (close.1 + close.3) / 2.0;
        if is_close_hov {
            d2d.fill_rounded_rectangle(close.0, close.1, close.2, close.3, 7.0, (1.0, 0.23, 0.19, 0.85));
            d2d.draw_close_icon(close_cx, close_cy, 4.5, (1.0, 1.0, 1.0, 1.0), 1.8);
        } else {
            d2d.draw_close_icon(close_cx, close_cy, 4.5, (0.75, 0.77, 0.80, 1.0), 1.6);
        }

        // Header separator
        d2d.draw_line(mx + 16.0, my + 46.0, mx + mw - 16.0, my + 46.0, (1.0, 1.0, 1.0, 0.12), 1.0);

        // Content Lines
        let left_x = mx + 22.0;
        let right_x = mx + mw - 22.0;
        let mut cur_y = my + 56.0;

        // Group 1: Aktiválás
        d2d.draw_text_simple("Aktiválás:", left_x, cur_y, right_x, cur_y + 18.0, 12.5, (0.22, 0.74, 0.97, 1.0), true, false);
        cur_y += 20.0;
        d2d.draw_text_simple("• Ctrl + 2 (vagy Ctrl + Alt + D) bárhonnan a rendszerből", left_x + 8.0, cur_y, right_x, cur_y + 18.0, 12.0, (0.85, 0.88, 0.92, 1.0), false, false);
        cur_y += 26.0;

        // Group 2: Rajzeszközök
        d2d.draw_text_simple("Rajzeszközök:", left_x, cur_y, right_x, cur_y + 18.0, 12.5, (0.22, 0.74, 0.97, 1.0), true, false);
        cur_y += 20.0;
        let tool_lines = [
            "• Rajz / Vonal / Nyíl: Szabadkézi toll, egyenes vonalak és folyamatnyilak",
            "• Téglalap / Kör: Területek keretezése és fókusz kiemelése (körvonal)",
            "• Takarás: Kitöltött téglalap érzékeny adatok, jelszavak azonnali maszkolására",
            "• Számozás: Húzzon egy nyilat, a számozott körjelvény automatikusan a nyíl tövére kerül",
            "• Ív: Húzzon egy vonalat, engedje fel az egeret, hajlítsa meg a kurzorral, majd kattintson",
            "• Szöveg: Kattintson bárhová és gépeljen közvetlenül a vászonra (Enter lezár)",
            "• Elmosás: Érzékeny adatok, képernyőterületek azonnali elhomályosítása",
            "• Kivágás: Képernyőrészlet kijelölése másoláshoz vagy fájlba mentéshez",
        ];
        for line in tool_lines {
            d2d.draw_text_simple(line, left_x + 8.0, cur_y, right_x, cur_y + 17.0, 11.5, (0.85, 0.88, 0.92, 1.0), false, false);
            cur_y += 19.0;
        }
        cur_y += 8.0;

        // Group 3: Színek és Gyorsbillentyűk
        d2d.draw_text_simple("Színek és Gyorsbillentyűk:", left_x, cur_y, right_x, cur_y + 18.0, 12.5, (0.22, 0.74, 0.97, 1.0), true, false);
        cur_y += 20.0;
        let shortcut_lines = [
            "• Színválasztó: 18 gyorsszín, Beépített színkeverő és Pipetta képernyő-mintavevő",
            "• Ctrl + Görgő (vagy görgő): Vonalvastagság és szövegméret gyors állítása (1–32 px)",
            "• Esc: Kilépés / Mégse   |   Ctrl + Z / Ctrl + Y: Visszavonás / Újra",
            "• Delete / Radír: Rajzok azonnali letörlése (Tiszta lap, számláló nullázása 1-re)",
            "• F1: Ezen súgóablak megjelenítése vagy bezárása",
        ];
        for line in shortcut_lines {
            d2d.draw_text_simple(line, left_x + 8.0, cur_y, right_x, cur_y + 17.0, 11.5, (0.85, 0.88, 0.92, 1.0), false, false);
            cur_y += 19.0;
        }

        // Action button "Rendben"
        let is_act_hov = self.hovered_modal_btn == Some(ModalButton::Action);
        let act_bg = if is_act_hov { (0.04, 0.52, 1.0, 1.0) } else { (0.04, 0.52, 1.0, 0.88) };
        d2d.fill_rounded_rectangle(act.0, act.1, act.2, act.3, 8.0, act_bg);
        d2d.draw_rounded_rectangle(act.0, act.1, act.2, act.3, 8.0, (1.0, 1.0, 1.0, 0.3), 1.0);
        d2d.draw_text_simple("Rendben", act.0, act.1, act.2, act.3, 13.0, (1.0, 1.0, 1.0, 1.0), true, true);
    }

    fn render_about_modal(&self, d2d: &D2DContext) {
        let (card, close, act) = match self.get_modal_layout() {
            Some(l) => l,
            None => return,
        };

        let (mx, my, mx2, my2) = card;
        let mw = mx2 - mx;

        // 1. Semi-transparent dark overlay dimming
        d2d.fill_rectangle(0.0, 0.0, self.capture.width as f32, self.capture.height as f32, (0.0, 0.0, 0.0, 0.55));

        // 2. Drop shadows
        d2d.fill_rounded_rectangle(mx - 4.0, my + 4.0, mx2 + 4.0, my2 + 12.0, 16.0, (0.0, 0.0, 0.0, 0.45));
        d2d.fill_rounded_rectangle(mx - 2.0, my + 2.0, mx2 + 2.0, my2 + 6.0, 16.0, (0.0, 0.0, 0.0, 0.30));

        // 3. Card body
        d2d.fill_rounded_rectangle(mx, my, mx2, my2, 16.0, (0.10, 0.12, 0.16, 0.97));
        d2d.draw_rounded_rectangle(mx, my, mx2, my2, 16.0, (1.0, 1.0, 1.0, 0.16), 1.2);

        // 4. Header title & close button
        d2d.draw_info_icon(mx + 30.0, my + 25.0, (0.22, 0.74, 0.97, 1.0), 1.6);
        d2d.draw_text_simple("DrawIt – Névjegy", mx + 44.0, my + 12.0, mx + mw - 50.0, my + 38.0, 15.0, (1.0, 1.0, 1.0, 1.0), true, false);

        let is_close_hov = self.hovered_modal_btn == Some(ModalButton::Close);
        let close_cx = (close.0 + close.2) / 2.0;
        let close_cy = (close.1 + close.3) / 2.0;
        if is_close_hov {
            d2d.fill_rounded_rectangle(close.0, close.1, close.2, close.3, 7.0, (1.0, 0.23, 0.19, 0.85));
            d2d.draw_close_icon(close_cx, close_cy, 4.5, (1.0, 1.0, 1.0, 1.0), 1.8);
        } else {
            d2d.draw_close_icon(close_cx, close_cy, 4.5, (0.75, 0.77, 0.80, 1.0), 1.6);
        }

        // Header separator
        d2d.draw_line(mx + 16.0, my + 46.0, mx + mw - 16.0, my + 46.0, (1.0, 1.0, 1.0, 0.12), 1.0);

        // Logo & Title (Vector SVG direct render)
        d2d.draw_svg_logo(mx + 22.0, my + 58.0, mx + 68.0, my + 104.0);
        d2d.draw_text_simple("DrawIt v0.1", mx + 78.0, my + 62.0, mx + mw - 20.0, my + 84.0, 15.0, (1.0, 1.0, 1.0, 1.0), true, false);
        d2d.draw_text_simple("Gyors, hordozható képernyőrajzoló és annotáló segédprogram", mx + 78.0, my + 85.0, mx + mw - 20.0, my + 104.0, 11.5, (0.75, 0.78, 0.84, 1.0), false, false);

        // Divider
        d2d.draw_line(mx + 20.0, my + 116.0, mx + mw - 20.0, my + 116.0, (1.0, 1.0, 1.0, 0.08), 1.0);

        // Feature bullet points
        let bullets = [
            "• 100% natív Rust kód hardveres Direct2D gyorsítással",
            "• Azonnali indulás, elhanyagolható memóriahasználat (~15-25 MB)",
            "• Nem igényel telepítést, .NET keretrendszert vagy admin jogot",
            "• Többmonitoros intelligens elhelyezés és kijelző-mentés",
        ];
        let mut cur_y = my + 128.0;
        for b in bullets {
            d2d.draw_text_simple(b, mx + 24.0, cur_y, mx + mw - 24.0, cur_y + 18.0, 11.5, (0.85, 0.88, 0.92, 1.0), false, false);
            cur_y += 22.0;
        }

        // Action button "Bezárás"
        let is_act_hov = self.hovered_modal_btn == Some(ModalButton::Action);
        let act_bg = if is_act_hov { (0.04, 0.52, 1.0, 1.0) } else { (0.04, 0.52, 1.0, 0.88) };
        d2d.fill_rounded_rectangle(act.0, act.1, act.2, act.3, 8.0, act_bg);
        d2d.draw_rounded_rectangle(act.0, act.1, act.2, act.3, 8.0, (1.0, 1.0, 1.0, 0.3), 1.0);
        d2d.draw_text_simple("Bezárás", act.0, act.1, act.2, act.3, 13.0, (1.0, 1.0, 1.0, 1.0), true, true);
    }

    fn render_badge(&self, d2d: &D2DContext, cx: f32, cy: f32, number: usize, color: (f32, f32, f32, f32), thickness: f32) {
        let radius = (thickness * 1.5 + 12.0).clamp(13.0, 26.0);
        // Shadow
        d2d.fill_ellipse(cx, cy + 1.5, radius + 1.0, radius + 1.0, (0.0, 0.0, 0.0, 0.35));
        // Badge body
        d2d.fill_ellipse(cx, cy, radius, radius, (color.0, color.1, color.2, 1.0));
        // Outer ring
        d2d.draw_ellipse(cx, cy, radius, radius, (1.0, 1.0, 1.0, 0.95), 2.0);
        // Number text
        let num_str = format!("{}", number);
        let font_sz = radius * 1.15;
        d2d.draw_text_simple(&num_str, cx - radius, cy - radius, cx + radius, cy + radius, font_sz, (1.0, 1.0, 1.0, 1.0), true, true);
    }

    fn render_action(&self, d2d: &D2DContext, action: &DrawAction) {
        match action {
            DrawAction::Freehand { points, color, thickness, opacity } => {
                let col = ((color.0 as f32)/255.0, (color.1 as f32)/255.0, (color.2 as f32)/255.0, *opacity);
                d2d.draw_polyline(points, col, *thickness);
            }
            DrawAction::Line { start, end, color, thickness, opacity } => {
                let col = ((color.0 as f32)/255.0, (color.1 as f32)/255.0, (color.2 as f32)/255.0, *opacity);
                d2d.draw_line(start.0, start.1, end.0, end.1, col, *thickness);
            }
            DrawAction::Rectangle { start, end, color, thickness, opacity, fill } => {
                let col = ((color.0 as f32)/255.0, (color.1 as f32)/255.0, (color.2 as f32)/255.0, *opacity);
                let l = start.0.min(end.0);
                let t = start.1.min(end.1);
                let r = start.0.max(end.0);
                let b = start.1.max(end.1);
                if *fill {
                    d2d.fill_rectangle(l, t, r, b, col);
                } else {
                    d2d.draw_rectangle(l, t, r, b, col, *thickness);
                }
            }
            DrawAction::Circle { start, end, color, thickness, opacity } => {
                let col = ((color.0 as f32)/255.0, (color.1 as f32)/255.0, (color.2 as f32)/255.0, *opacity);
                let cx = (start.0 + end.0) / 2.0;
                let cy = (start.1 + end.1) / 2.0;
                let rx = (end.0 - start.0).abs() / 2.0;
                let ry = (end.1 - start.1).abs() / 2.0;
                d2d.draw_ellipse(cx, cy, rx, ry, col, *thickness);
            }
            DrawAction::Arrow { start, end, color, thickness, opacity } => {
                let col = ((color.0 as f32)/255.0, (color.1 as f32)/255.0, (color.2 as f32)/255.0, *opacity);
                d2d.draw_arrow(start.0, start.1, end.0, end.1, col, *thickness);
            }
            DrawAction::NumberedArrow { start, end, number, color, thickness, opacity } => {
                let col = ((color.0 as f32)/255.0, (color.1 as f32)/255.0, (color.2 as f32)/255.0, *opacity);
                let dx = end.0 - start.0;
                let dy = end.1 - start.1;
                let dist = (dx * dx + dy * dy).sqrt();

                if dist > 8.0 {
                    d2d.draw_arrow(start.0, start.1, end.0, end.1, col, *thickness);
                }
                self.render_badge(d2d, start.0, start.1, *number, col, *thickness);
            }
            DrawAction::Curve { start, ctrl, end, color, thickness, opacity } => {
                let col = ((color.0 as f32)/255.0, (color.1 as f32)/255.0, (color.2 as f32)/255.0, *opacity);
                d2d.draw_quadratic_curve(start.0, start.1, ctrl.0, ctrl.1, end.0, end.1, col, *thickness);
            }
            DrawAction::Text { pos, text, color, font_size } => {
                let col = ((color.0 as f32)/255.0, (color.1 as f32)/255.0, (color.2 as f32)/255.0, 1.0);
                d2d.draw_text_simple(text, pos.0, pos.1, pos.0 + 1200.0, pos.1 + font_size + 10.0, *font_size, col, true, false);
            }
            DrawAction::Blur { start, end } => {
                let l = start.0.min(end.0);
                let t = start.1.min(end.1);
                let r = start.0.max(end.0);
                let b = start.1.max(end.1);
                if let Some(blur_bmp) = &self.blur_bitmap {
                    d2d.draw_bitmap_crop(blur_bmp, l, t, r, b, l, t, r, b, 1.0);
                }
            }
        }
    }

    fn render_snip_overlay(&self, d2d: &D2DContext) {
        let sw = self.capture.width as f32;
        let sh = self.capture.height as f32;

        let rect = match self.snip_rect {
            Some(r) => r,
            None => {
                if self.is_snipping {
                    (
                        self.snip_start.0.min(self.draw_current.0),
                        self.snip_start.1.min(self.draw_current.1),
                        self.snip_start.0.max(self.draw_current.0),
                        self.snip_start.1.max(self.draw_current.1),
                    )
                } else {
                    d2d.fill_rectangle(0.0, 0.0, sw, sh, (0.0, 0.0, 0.0, 0.35));
                    let active_mon = crate::screen::get_focused_monitor(&self.capture.monitors)
                        .or_else(|| self.capture.monitors.first());
                    let (cx, top_y) = if let Some(m) = active_mon {
                        let mon_left = (m.x - self.capture.left) as f32;
                        let mon_top = (m.y - self.capture.top) as f32;
                        (mon_left + (m.width as f32) / 2.0, mon_top + 90.0)
                    } else {
                        (sw / 2.0, 90.0)
                    };
                    d2d.draw_text_simple("Jelölje ki a kívánt területet az egér húzásával...", cx - 250.0, top_y, cx + 250.0, top_y + 40.0, 16.0, (1.0, 1.0, 1.0, 0.9), true, true);
                    return;
                }
            }
        };

        let (sl, st, sr, sb) = rect;
        let dim_color = (0.0, 0.0, 0.0, 0.45);

        d2d.fill_rectangle(0.0, 0.0, sw, st, dim_color);
        d2d.fill_rectangle(0.0, sb, sw, sh, dim_color);
        d2d.fill_rectangle(0.0, st, sl, sb, dim_color);
        d2d.fill_rectangle(sr, st, sw, sb, dim_color);

        d2d.draw_rectangle(sl, st, sr, sb, (0.04, 0.52, 1.0, 0.95), 2.0);

        let w_px = (sr - sl).abs().round();
        let h_px = (sb - st).abs().round();
        let dim_text = format!("{:.0} × {:.0} px", w_px, h_px);
        let badge_w = 110.0;
        let badge_h = 24.0;
        let badge_y = if st >= 30.0 { st - badge_h - 4.0 } else { sb + 4.0 };
        d2d.fill_rounded_rectangle(sl, badge_y, sl + badge_w, badge_y + badge_h, 6.0, (0.1, 0.12, 0.16, 0.9));
        d2d.draw_text_simple(&dim_text, sl, badge_y, sl + badge_w, badge_y + badge_h, 11.0, (1.0, 1.0, 1.0, 1.0), true, true);

        if let Some((bar_rect, actions)) = self.layout_snip_bar() {
            // Drop shadow
            d2d.fill_rounded_rectangle(bar_rect.0 - 2.0, bar_rect.1 + 2.0, bar_rect.2 + 2.0, bar_rect.3 + 4.0, 10.0, (0.0, 0.0, 0.0, 0.35));
            // Frosted dark pill container
            d2d.fill_rounded_rectangle(bar_rect.0, bar_rect.1, bar_rect.2, bar_rect.3, 10.0, (0.09, 0.11, 0.15, 0.96));
            // Subtle highlight border
            d2d.draw_rounded_rectangle(bar_rect.0, bar_rect.1, bar_rect.2, bar_rect.3, 10.0, (1.0, 1.0, 1.0, 0.16), 1.0);

            for (act, (bl, bt, br, bb)) in actions {
                let is_hov = self.hovered_snip_action == Some(act);
                let (btn_l, btn_t, btn_r, btn_b) = if is_hov {
                    (bl - 1.5, bt - 1.5, br + 1.5, bb + 1.5)
                } else {
                    (bl, bt, br, bb)
                };

                match act {
                    SnipAction::Copy => {
                        let (bg, border, col) = if is_hov {
                            ((0.10, 0.60, 0.35, 0.45), (0.25, 0.85, 0.45, 0.90), (0.40, 1.0, 0.60, 1.0))
                        } else {
                            ((1.0, 1.0, 1.0, 0.06), (1.0, 1.0, 1.0, 0.10), (0.85, 0.88, 0.92, 1.0))
                        };
                        d2d.fill_rounded_rectangle(btn_l, btn_t, btn_r, btn_b, 7.0, bg);
                        d2d.draw_rounded_rectangle(btn_l, btn_t, btn_r, btn_b, 7.0, border, 1.2);
                        d2d.draw_clipboard_icon(btn_l + 16.0, (btn_t + btn_b) / 2.0, col, 1.5);
                        d2d.draw_text_simple("Másolás", btn_l + 26.0, btn_t, btn_r - 4.0, btn_b, 12.0, col, is_hov, true);
                    }
                    SnipAction::SavePng => {
                        let (bg, border, col) = if is_hov {
                            ((0.08, 0.48, 0.90, 0.45), (0.20, 0.70, 1.0, 0.90), (0.40, 0.85, 1.0, 1.0))
                        } else {
                            ((1.0, 1.0, 1.0, 0.06), (1.0, 1.0, 1.0, 0.10), (0.85, 0.88, 0.92, 1.0))
                        };
                        d2d.fill_rounded_rectangle(btn_l, btn_t, btn_r, btn_b, 7.0, bg);
                        d2d.draw_rounded_rectangle(btn_l, btn_t, btn_r, btn_b, 7.0, border, 1.2);
                        d2d.draw_save_icon(btn_l + 16.0, (btn_t + btn_b) / 2.0, col, 1.5);
                        d2d.draw_text_simple("PNG", btn_l + 26.0, btn_t, btn_r - 4.0, btn_b, 12.0, col, is_hov, true);
                    }
                    SnipAction::SavePdf => {
                        let (bg, border, col) = if is_hov {
                            ((0.85, 0.38, 0.12, 0.45), (1.0, 0.55, 0.22, 0.90), (1.0, 0.70, 0.40, 1.0))
                        } else {
                            ((1.0, 1.0, 1.0, 0.06), (1.0, 1.0, 1.0, 0.10), (0.85, 0.88, 0.92, 1.0))
                        };
                        d2d.fill_rounded_rectangle(btn_l, btn_t, btn_r, btn_b, 7.0, bg);
                        d2d.draw_rounded_rectangle(btn_l, btn_t, btn_r, btn_b, 7.0, border, 1.2);
                        d2d.draw_pdf_doc_icon(btn_l + 16.0, (btn_t + btn_b) / 2.0, col, 1.5);
                        d2d.draw_text_simple("PDF", btn_l + 26.0, btn_t, btn_r - 4.0, btn_b, 12.0, col, is_hov, true);
                    }
                    SnipAction::Cancel => {
                        let (bg, border, col) = if is_hov {
                            ((0.85, 0.20, 0.25, 0.45), (1.0, 0.35, 0.40, 0.90), (1.0, 0.55, 0.55, 1.0))
                        } else {
                            ((1.0, 1.0, 1.0, 0.06), (1.0, 1.0, 1.0, 0.10), (0.85, 0.88, 0.92, 1.0))
                        };
                        d2d.fill_rounded_rectangle(btn_l, btn_t, btn_r, btn_b, 7.0, bg);
                        d2d.draw_rounded_rectangle(btn_l, btn_t, btn_r, btn_b, 7.0, border, 1.2);
                        d2d.draw_close_icon(btn_l + 16.0, (btn_t + btn_b) / 2.0, 4.0, col, 1.5);
                        d2d.draw_text_simple("Mégse", btn_l + 26.0, btn_t, btn_r - 4.0, btn_b, 12.0, col, is_hov, true);
                    }
                }
            }
        }
    }

    pub fn handle_snip_click(&mut self, mx: f32, my: f32) -> bool {
        let (sl, st, sr, sb) = match self.snip_rect {
            Some(r) => r,
            None => return false,
        };

        let (_, actions) = match self.layout_snip_bar() {
            Some(res) => res,
            None => return false,
        };

        let clicked_act = actions.into_iter().find(|(_, r)| {
            mx >= r.0 && mx <= r.2 && my >= r.1 && my <= r.3
        }).map(|(act, _)| act);

        let act = match clicked_act {
            Some(a) => a,
            None => return false,
        };

        let full_bgra = self.capture_rendered_canvas();
        let full_w = self.capture.width;
        let full_h = self.capture.height;

        let cx = sl.round().max(0.0) as u32;
        let cy = st.round().max(0.0) as u32;
        let cw = (sr - sl).abs().round().max(1.0) as u32;
        let ch = (sb - st).abs().round().max(1.0) as u32;

        let cropped_bgra = crop_bgra(&full_bgra, full_w, full_h, cx, cy, cw, ch);

        match act {
            SnipAction::Copy => {
                copy_bgra_to_clipboard(self.hwnd, cw, ch, &cropped_bgra);
                self.show_toast("✓ Kivágott terület másolva a vágólapra!");
                self.snip_rect = None;
                self.hovered_snip_action = None;
                self.toolbar.active_tool = ToolMode::Freehand;
                true
            }
            SnipAction::SavePng => {
                let rgba = bgra_to_rgba(&cropped_bgra);
                let def_name = format!("Kivagas_{}.png", chrono_timestamp());
                let anchor = create_anchor_for_monitor_by_cursor(&self.capture, self.hwnd);
                let saved = save_png_dialog(anchor, &def_name, cw, ch, &rgba);
                unsafe { let _ = windows::Win32::UI::WindowsAndMessaging::DestroyWindow(anchor); }
                if saved.is_some() {
                    self.show_toast("✓ Kivágás elmentve PNG fájlba!");
                }
                self.snip_rect = None;
                self.hovered_snip_action = None;
                self.toolbar.active_tool = ToolMode::Freehand;
                true
            }
            SnipAction::SavePdf => {
                let rgba = bgra_to_rgba(&cropped_bgra);
                let def_name = format!("Kivagas_{}.pdf", chrono_timestamp());
                let anchor = create_anchor_for_monitor_by_cursor(&self.capture, self.hwnd);
                let saved = save_pdf_dialog(anchor, &def_name, cw, ch, &rgba);
                unsafe { let _ = windows::Win32::UI::WindowsAndMessaging::DestroyWindow(anchor); }
                if saved.is_some() {
                    self.show_toast("✓ Kivágás elmentve PDF fájlba!");
                }
                self.snip_rect = None;
                self.hovered_snip_action = None;
                self.toolbar.active_tool = ToolMode::Freehand;
                true
            }
            SnipAction::Cancel => {
                self.show_toast("Képmetszés megszakítva");
                self.snip_rect = None;
                self.hovered_snip_action = None;
                self.toolbar.active_tool = ToolMode::Freehand;
                true
            }
        }
    }
}

fn chrono_timestamp() -> String {
    use std::time::SystemTime;
    let dur = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default();
    format!("{}", dur.as_secs())
}

fn handle_escape_press(state: &mut OverlayState) {
    if state.active_modal != ActiveModal::None {
        state.active_modal = ActiveModal::None;
        state.hovered_modal_btn = None;
        state.render();
    } else if state.toolbar.active_tool == ToolMode::Pipette {
        state.toolbar.active_tool = state.previous_tool_before_pipette;
        state.pipette_buffer = None;
        state.show_toast("Pipetta megszakítva");
        state.render();
    } else if state.toolbar.dismiss_popup() {
        state.render();
    } else if state.snip_rect.is_some() {
        state.snip_rect = None;
        state.render();
    } else if state.text_pos.is_some() {
        state.text_pos = None;
        state.text_buffer.clear();
        state.render();
    } else if state.curve_phase != CurvePhase::Idle {
        state.curve_phase = CurvePhase::Idle;
        state.render();
    } else {
        hide_overlay(state);
    }
}

pub unsafe extern "system" fn overlay_wndproc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    let ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut OverlayState;
    if ptr.is_null() {
        return DefWindowProcW(hwnd, msg, wparam, lparam);
    }
    let state = &mut *ptr;

    match msg {
        WM_HOTKEY => {
            if wparam.0 == 9998 {
                handle_escape_press(state);
                return LRESULT(0);
            }
            LRESULT(0)
        }

        WM_ERASEBKGND => LRESULT(1),

        WM_PAINT => {
            state.render();
            LRESULT(0)
        }

        WM_TIMER => {
            if wparam.0 == 9999 {
                if std::time::Instant::now() >= state.toast_until {
                    state.toast_message = None;
                    let _ = windows::Win32::UI::WindowsAndMessaging::KillTimer(hwnd, 9999);
                    state.render();
                }
            }
            LRESULT(0)
        }

        WM_MOUSEWHEEL => {
            let delta = ((wparam.0 >> 16) as i16) as i32;
            let step = if delta > 0 { 1.0 } else { -1.0 };
            let mult = (delta.abs() / 120).max(1) as f32;
            state.toolbar.thickness = (state.toolbar.thickness + step * mult).clamp(1.0, 32.0);

            if state.toolbar.active_tool == ToolMode::Text || state.text_pos.is_some() {
                let font_size = state.toolbar.thickness * 3.5 + 14.0;
                state.show_toast(&format!("Betűméret: {:.0} pt  |  Vonalvastagság: {:.0} px", font_size, state.toolbar.thickness));
            } else {
                state.show_toast(&format!("Vonalvastagság: {:.0} px", state.toolbar.thickness));
            }
            state.render();
            LRESULT(0)
        }

        WM_MOUSEMOVE => {
            let mx = (lparam.0 & 0xFFFF) as i16 as f32;
            let my = ((lparam.0 >> 16) & 0xFFFF) as i16 as f32;
            let prev_pos = state.draw_current;
            state.draw_current = (mx, my);

            // Active in-app modal (Help / About) button hover check
            if state.active_modal != ActiveModal::None {
                let prev_hov = state.hovered_modal_btn;
                state.hovered_modal_btn = None;
                if let Some((_, close, act)) = state.get_modal_layout() {
                    if mx >= close.0 && mx <= close.2 && my >= close.1 && my <= close.3 {
                        state.hovered_modal_btn = Some(ModalButton::Close);
                    } else if mx >= act.0 && mx <= act.2 && my >= act.1 && my <= act.3 {
                        state.hovered_modal_btn = Some(ModalButton::Action);
                    }
                }
                if state.hovered_modal_btn != prev_hov {
                    state.render();
                }
                return LRESULT(0);
            }

            // Dragging Hue bar
            if state.is_dragging_hue {
                if let Some(r) = state.toolbar.get_hue_bar_rect() {
                    let frac = ((mx - r.0) / (r.2 - r.0)).clamp(0.0, 1.0);
                    state.toolbar.set_hue_fraction(frac);
                    state.render();
                    return LRESULT(0);
                }
            }

            // Dragging Lightness bar
            if state.is_dragging_lightness {
                if let Some(r) = state.toolbar.get_lightness_bar_rect() {
                    let frac = ((mx - r.0) / (r.2 - r.0)).clamp(0.0, 1.0);
                    state.toolbar.set_lightness_fraction(frac);
                    state.render();
                    return LRESULT(0);
                }
            }

            // Pipette mode: render loupe and hide cursor
            if state.toolbar.active_tool == ToolMode::Pipette {
                SetCursor(HCURSOR(ptr::null_mut()));
                state.render();
                return LRESULT(0);
            }

            // Snip bar button hover check
            let prev_snip_act = state.hovered_snip_action;
            if let Some((_, actions)) = state.layout_snip_bar() {
                state.hovered_snip_action = actions.into_iter().find(|(_, r)| {
                    mx >= r.0 && mx <= r.2 && my >= r.1 && my <= r.3
                }).map(|(act, _)| act);
            } else {
                state.hovered_snip_action = None;
            }
            if prev_snip_act != state.hovered_snip_action {
                state.render();
            }

            // 1. Toolbar dragging (ONLY from GripHandle)
            if state.is_dragging_toolbar {
                state.toolbar.x = state.drag_start_bar.0 + (mx - state.drag_start_mouse.0);
                state.toolbar.y = state.drag_start_bar.1 + (my - state.drag_start_mouse.1);
                state.render();
                return LRESULT(0);
            }

            // 2. Toolbar & popup hover update (pure, no mutation of open popups)
            let prev_item = state.toolbar.hovered_item;
            let prev_act = state.toolbar.hovered_action;
            state.toolbar.update_hover(mx, my);
            if prev_item != state.toolbar.hovered_item || prev_act != state.toolbar.hovered_action {
                state.render();
            }

            // 3. Drawing interactions
            if state.is_drawing {
                if state.toolbar.active_tool == ToolMode::Freehand {
                    state.current_points.push((mx, my));
                }
                state.render();
                return LRESULT(0);
            }

            // 4. Snipping in progress
            if state.is_snipping {
                state.render();
                return LRESULT(0);
            }

            // 5. Curve bending in progress
            if let CurvePhase::Bending { .. } = state.curve_phase {
                state.render();
                return LRESULT(0);
            }

            let is_drawing_tool = matches!(
                state.toolbar.active_tool,
                ToolMode::Freehand
                    | ToolMode::Line
                    | ToolMode::Rectangle
                    | ToolMode::FilledRectangle
                    | ToolMode::Circle
                    | ToolMode::Arrow
                    | ToolMode::NumberedBadge
                    | ToolMode::Curve
                    | ToolMode::Blur
            );
            let is_over_toolbar = state.toolbar.is_inside(mx, my);
            let is_over_snip = state.is_over_snip_bar(mx, my);

            let was_on_canvas_drawing = is_drawing_tool
                && !state.toolbar.is_inside(prev_pos.0, prev_pos.1)
                && !state.is_over_snip_bar(prev_pos.0, prev_pos.1);
            let is_on_canvas_drawing = is_drawing_tool && !is_over_toolbar && !is_over_snip;

            // When hovering canvas in drawing mode, animate custom brush cursor (+ / dot)
            if is_on_canvas_drawing {
                if prev_pos != (mx, my) {
                    state.render();
                }
                SetCursor(HCURSOR(ptr::null_mut()));
                return LRESULT(0);
            } else if was_on_canvas_drawing {
                // Left canvas drawing area, re-render to remove brush indicator
                state.render();
            }

            // Update standard mouse cursor when outside drawing canvas
            if state.hovered_snip_action.is_some() {
                SetCursor(state.h_hand);
            } else if is_over_snip {
                SetCursor(state.h_arrow);
            } else if state.toolbar.hovered_item == Some(ToolbarItem::GripHandle) {
                SetCursor(state.h_sizeall);
            } else if state.toolbar.hovered_action.is_some() || state.toolbar.hovered_item.is_some() {
                SetCursor(state.h_hand);
            } else if is_over_toolbar {
                SetCursor(state.h_arrow);
            } else if state.toolbar.active_tool == ToolMode::Text {
                SetCursor(state.h_ibeam);
            } else if state.toolbar.active_tool == ToolMode::Snip {
                SetCursor(state.h_cross);
            } else {
                SetCursor(state.h_arrow);
            }

            LRESULT(0)
        }

        WM_SETCURSOR => {
            let hit = (lparam.0 & 0xFFFF) as u32;
            if hit == 1 { // HTCLIENT
                if state.active_modal != ActiveModal::None {
                    if state.hovered_modal_btn.is_some() {
                        SetCursor(state.h_hand);
                    } else {
                        SetCursor(state.h_arrow);
                    }
                    return LRESULT(1);
                }
                let mx = state.draw_current.0;
                let my = state.draw_current.1;
                if state.toolbar.active_tool == ToolMode::Pipette {
                    SetCursor(HCURSOR(ptr::null_mut()));
                    return LRESULT(1);
                }
                let is_drawing_tool = matches!(
                    state.toolbar.active_tool,
                    ToolMode::Freehand
                        | ToolMode::Line
                        | ToolMode::Rectangle
                        | ToolMode::FilledRectangle
                        | ToolMode::Circle
                        | ToolMode::Arrow
                        | ToolMode::NumberedBadge
                        | ToolMode::Curve
                        | ToolMode::Blur
                );
                let is_over_toolbar = state.toolbar.is_inside(mx, my);
                let is_over_snip = state.is_over_snip_bar(mx, my);

                if state.hovered_snip_action.is_some() {
                    SetCursor(state.h_hand);
                } else if is_over_snip {
                    SetCursor(state.h_arrow);
                } else if state.toolbar.hovered_item == Some(ToolbarItem::GripHandle) {
                    SetCursor(state.h_sizeall);
                } else if state.toolbar.hovered_item == Some(ToolbarItem::Undo) && !state.toolbar.can_undo {
                    SetCursor(state.h_arrow);
                } else if state.toolbar.hovered_item == Some(ToolbarItem::Redo) && !state.toolbar.can_redo {
                    SetCursor(state.h_arrow);
                } else if state.toolbar.hovered_action.is_some() || state.toolbar.hovered_item.is_some() {
                    SetCursor(state.h_hand);
                } else if is_over_toolbar {
                    SetCursor(state.h_arrow);
                } else if state.toolbar.active_tool == ToolMode::Text {
                    SetCursor(state.h_ibeam);
                } else if state.toolbar.active_tool == ToolMode::Snip {
                    SetCursor(state.h_cross);
                } else if is_drawing_tool {
                    SetCursor(HCURSOR(ptr::null_mut()));
                } else {
                    SetCursor(state.h_arrow);
                }
                return LRESULT(1);
            }
            DefWindowProcW(hwnd, msg, wparam, lparam)
        }

        WM_LBUTTONDOWN => {
            let _ = SetFocus(hwnd);
            let mx = (lparam.0 & 0xFFFF) as i16 as f32;
            let my = ((lparam.0 >> 16) & 0xFFFF) as i16 as f32;

            // Handle active in-app modal (Help / About)
            if state.active_modal != ActiveModal::None {
                if let Some((card, close, act)) = state.get_modal_layout() {
                    // Click on close button (✕) or action button ("Rendben" / "Bezárás")
                    if (mx >= close.0 && mx <= close.2 && my >= close.1 && my <= close.3)
                        || (mx >= act.0 && mx <= act.2 && my >= act.1 && my <= act.3)
                    {
                        state.active_modal = ActiveModal::None;
                        state.hovered_modal_btn = None;
                        state.render();
                        return LRESULT(0);
                    }
                    // Click inside card (consume)
                    if mx >= card.0 && mx <= card.2 && my >= card.1 && my <= card.3 {
                        return LRESULT(0);
                    }
                    // Click outside card (backdrop dismiss)
                    state.active_modal = ActiveModal::None;
                    state.hovered_modal_btn = None;
                    state.render();
                    return LRESULT(0);
                }
            }

            // Pipette mode click to sample color
            if state.toolbar.active_tool == ToolMode::Pipette {
                let (r, g, b) = state.sample_color_at(mx, my);
                state.toolbar.set_color_rgb((r, g, b));
                let prev_tool = state.previous_tool_before_pipette;
                state.toolbar.active_tool = prev_tool;
                state.pipette_buffer = None;
                state.show_toast(&format!("✓ Szín mintavételezve: #{:02X}{:02X}{:02X} ({})", r, g, b, prev_tool.label_hu()));
                state.render();
                return LRESULT(0);
            }

            // Check snip action bar click first if snip rect is visible
            if state.toolbar.active_tool == ToolMode::Snip && state.handle_snip_click(mx, my) {
                state.render();
                return LRESULT(0);
            }

            // Curve Phase 2: If we were bending, clicking commits the curve!
            if let CurvePhase::Bending { start, end } = state.curve_phase {
                let ctrl_x = 2.0 * mx - 0.5 * (start.0 + end.0);
                let ctrl_y = 2.0 * my - 0.5 * (start.1 + end.1);
                state.history.push(DrawAction::Curve {
                    start,
                    ctrl: (ctrl_x, ctrl_y),
                    end,
                    color: state.toolbar.color,
                    thickness: state.toolbar.thickness,
                    opacity: state.toolbar.opacity,
                });
                state.curve_phase = CurvePhase::Idle;
                state.show_toast("✓ Ív rögzítve");
                state.render();
                return LRESULT(0);
            }

            // Hit-test toolbar and open popups
            let hit = state.toolbar.hit_test(mx, my);

            match hit {
                HitTestResult::OutsidePopupDismissed => {
                    // Clicked on blank canvas outside: popup was dismissed, do not draw or drag
                    state.render();
                    return LRESULT(0);
                }

                HitTestResult::ToolbarBackground => {
                    return LRESULT(0);
                }

                HitTestResult::Item(ToolbarItem::GripHandle) => {
                    state.is_dragging_toolbar = true;
                    state.drag_start_mouse = (mx, my);
                    state.drag_start_bar = (state.toolbar.x, state.toolbar.y);
                    SetCapture(hwnd);
                    return LRESULT(0);
                }

                HitTestResult::Item(ToolbarItem::LogoButton) => {
                    state.toolbar.open_popup = if state.toolbar.open_popup == OpenPopup::LogoMenu {
                        OpenPopup::None
                    } else {
                        OpenPopup::LogoMenu
                    };
                    state.render();
                    return LRESULT(0);
                }

                HitTestResult::Item(ToolbarItem::Tool(mode)) => {
                    state.commit_text_if_any();
                    if mode == ToolMode::Rectangle && state.toolbar.active_tool == ToolMode::Rectangle {
                        state.toolbar.active_tool = ToolMode::FilledRectangle;
                        state.show_toast("■ Takarás (kitöltött téglalap) kiválasztva");
                    } else if mode == ToolMode::FilledRectangle && state.toolbar.active_tool == ToolMode::FilledRectangle {
                        state.toolbar.active_tool = ToolMode::Rectangle;
                        state.show_toast("▭ Téglalap (körvonal) kiválasztva");
                    } else {
                        state.toolbar.active_tool = mode;
                    }
                    state.snip_rect = None;
                    state.curve_phase = CurvePhase::Idle;
                    state.toolbar.dismiss_popup();
                    state.render();
                    return LRESULT(0);
                }

                HitTestResult::Item(ToolbarItem::ColorButton) => {
                    state.toolbar.open_popup = if state.toolbar.open_popup == OpenPopup::ColorPicker {
                        OpenPopup::None
                    } else {
                        OpenPopup::ColorPicker
                    };
                    state.render();
                    return LRESULT(0);
                }

                HitTestResult::Item(ToolbarItem::Undo) => {
                    if !state.history.actions.is_empty() {
                        state.history.undo();
                        state.show_toast("↶ Visszavonás");
                        state.render();
                    }
                    return LRESULT(0);
                }

                HitTestResult::Item(ToolbarItem::Redo) => {
                    if !state.history.redo_stack.is_empty() {
                        state.history.redo();
                        state.show_toast("↷ Újra");
                        state.render();
                    }
                    return LRESULT(0);
                }

                HitTestResult::Item(ToolbarItem::Clear) => {
                    state.history.clear();
                    state.history.reset_counter();
                    state.text_pos = None;
                    state.text_buffer.clear();
                    state.snip_rect = None;
                    state.curve_phase = CurvePhase::Idle;
                    state.show_toast("✓ Rajzok törölve (Tiszta vászon)");
                    state.render();
                    return LRESULT(0);
                }

                HitTestResult::Item(ToolbarItem::CanvasModeButton) => {
                    state.toolbar.open_popup = if state.toolbar.open_popup == OpenPopup::CanvasMode {
                        OpenPopup::None
                    } else {
                        OpenPopup::CanvasMode
                    };
                    state.render();
                    return LRESULT(0);
                }

                HitTestResult::Item(ToolbarItem::SnipButton) => {
                    state.commit_text_if_any();
                    state.toolbar.active_tool = ToolMode::Snip;
                    state.snip_rect = None;
                    state.curve_phase = CurvePhase::Idle;
                    state.toolbar.dismiss_popup();
                    state.render();
                    return LRESULT(0);
                }

                HitTestResult::Item(ToolbarItem::SaveDisplayButton) => {
                    state.toolbar.open_popup = if state.toolbar.open_popup == OpenPopup::SaveDisplay {
                        OpenPopup::None
                    } else {
                        OpenPopup::SaveDisplay
                    };
                    state.render();
                    return LRESULT(0);
                }

                HitTestResult::Item(ToolbarItem::CloseButton) => {
                    hide_overlay(state);
                    return LRESULT(0);
                }

                HitTestResult::Popup(act) => {
                    match act {
                        PopupAction::Help => {
                            state.toolbar.dismiss_popup();
                            state.active_modal = ActiveModal::Help;
                            state.hovered_modal_btn = None;
                            state.render();
                        }
                        PopupAction::CheckUpdates => {
                            state.toolbar.dismiss_popup();
                            crate::updater::prompt_and_update(state.hwnd, true);
                        }
                        PopupAction::About => {
                            state.toolbar.dismiss_popup();
                            state.active_modal = ActiveModal::About;
                            state.hovered_modal_btn = None;
                            state.render();
                        }
                        PopupAction::ExitApp => {
                            hide_overlay(state);
                        }
                        PopupAction::SelectColor(rgb) => {
                            state.toolbar.set_color_rgb(rgb);
                            state.show_toast("✓ Szín kiválasztva");
                        }
                        PopupAction::HueBar(frac) => {
                            state.is_dragging_hue = true;
                            state.toolbar.set_hue_fraction(frac);
                            unsafe { SetCapture(hwnd); }
                            state.render();
                            return LRESULT(0);
                        }
                        PopupAction::LightnessBar(frac) => {
                            state.is_dragging_lightness = true;
                            state.toolbar.set_lightness_fraction(frac);
                            unsafe { SetCapture(hwnd); }
                            state.render();
                            return LRESULT(0);
                        }
                        PopupAction::Eyedropper => {
                            state.previous_tool_before_pipette = if state.toolbar.active_tool != ToolMode::Pipette {
                                state.toolbar.active_tool
                            } else {
                                ToolMode::Freehand
                            };
                            state.toolbar.dismiss_popup();
                            state.pipette_buffer = Some(state.capture_rendered_canvas());
                            state.toolbar.active_tool = ToolMode::Pipette;
                            state.show_toast("💧 Pipetta: Kattints a képernyőre a szín mintavételezéséhez! (Esc: mégse)");
                            state.render();
                            return LRESULT(0);
                        }
                        PopupAction::SelectOpacity(op) => {
                            state.toolbar.opacity = op;
                            let pct = (op * 100.0).round() as i32;
                            state.show_toast(&format!("Átlátszatlanság: {}%", pct));
                        }
                        PopupAction::SelectThickness(th) => {
                            state.toolbar.thickness = th;
                            state.show_toast(&format!("Vonalvastagság: {:.0} px", th));
                        }
                        PopupAction::AdjustThickness(delta) => {
                            state.toolbar.thickness = (state.toolbar.thickness + delta).clamp(1.0, 32.0);
                            state.show_toast(&format!("Vonalvastagság: {:.0} px", state.toolbar.thickness));
                        }
                        PopupAction::SelectCanvasMode(mode) => {
                            state.toolbar.canvas_mode = mode;
                            let label = match mode {
                                CanvasMode::Transparent => "Átlátszó képernyő",
                                CanvasMode::Blurred => "Elmosott kijelző háttér",
                                CanvasMode::Whiteboard => "Fehér tábla mód",
                                CanvasMode::Blackboard => "Fekete tábla mód",
                            };
                            state.show_toast(label);
                            state.toolbar.dismiss_popup();
                        }
                        PopupAction::SaveMonitorPng(idx) => {
                            state.toolbar.dismiss_popup();
                            state.export_display_action(idx, "png");
                        }
                        PopupAction::SaveMonitorPdf(idx) => {
                            state.toolbar.dismiss_popup();
                            state.export_display_action(idx, "pdf");
                        }
                        PopupAction::CopyMonitorClipboard(idx) => {
                            state.toolbar.dismiss_popup();
                            state.export_display_action(idx, "clipboard");
                        }
                    }
                    state.render();
                    return LRESULT(0);
                }

                HitTestResult::Miss => {}
            }

            // Text Tool Click
            if state.toolbar.active_tool == ToolMode::Text {
                state.commit_text_if_any();
                state.text_pos = Some((mx, my));
                state.text_buffer.clear();
                state.text_cursor_visible = true;
                state.render();
                return LRESULT(0);
            }

            // Snip Tool Drag Start
            if state.toolbar.active_tool == ToolMode::Snip {
                state.is_snipping = true;
                state.snip_start = (mx, my);
                state.snip_rect = None;
                SetCapture(hwnd);
                state.render();
                return LRESULT(0);
            }

            // Standard / Arrow / NumberedArrow / Curve Drag Start
            state.is_drawing = true;
            state.draw_start = (mx, my);
            state.draw_current = (mx, my);
            state.current_points.clear();
            state.current_points.push((mx, my));
            SetCapture(hwnd);

            LRESULT(0)
        }

        WM_LBUTTONUP => {
            let mx = (lparam.0 & 0xFFFF) as i16 as f32;
            let my = ((lparam.0 >> 16) & 0xFFFF) as i16 as f32;

            if state.is_dragging_hue || state.is_dragging_lightness {
                state.is_dragging_hue = false;
                state.is_dragging_lightness = false;
                let _ = ReleaseCapture();
                state.render();
                return LRESULT(0);
            }

            if state.is_dragging_toolbar {
                state.is_dragging_toolbar = false;
                let _ = ReleaseCapture();
                return LRESULT(0);
            }

            if state.is_snipping {
                state.is_snipping = false;
                let _ = ReleaseCapture();
                let l = state.snip_start.0.min(mx);
                let t = state.snip_start.1.min(my);
                let r = state.snip_start.0.max(mx);
                let b = state.snip_start.1.max(my);
                if (r - l) > 10.0 && (b - t) > 10.0 {
                    state.snip_rect = Some((l, t, r, b));
                }
                state.render();
                return LRESULT(0);
            }

            if state.is_drawing {
                state.is_drawing = false;
                let _ = ReleaseCapture();

                let col = state.toolbar.color;
                let th = state.toolbar.thickness;
                let op = state.toolbar.opacity;

                match state.toolbar.active_tool {
                    ToolMode::Freehand => {
                        if state.current_points.len() > 1 {
                            state.history.push(DrawAction::Freehand {
                                points: state.current_points.clone(),
                                color: col,
                                thickness: th,
                                opacity: op,
                            });
                        }
                    }
                    ToolMode::Line => {
                        state.history.push(DrawAction::Line {
                            start: state.draw_start,
                            end: (mx, my),
                            color: col,
                            thickness: th,
                            opacity: op,
                        });
                    }
                    ToolMode::Rectangle => {
                        state.history.push(DrawAction::Rectangle {
                            start: state.draw_start,
                            end: (mx, my),
                            color: col,
                            thickness: th,
                            opacity: op,
                            fill: false,
                        });
                    }
                    ToolMode::FilledRectangle => {
                        state.history.push(DrawAction::Rectangle {
                            start: state.draw_start,
                            end: (mx, my),
                            color: col,
                            thickness: th,
                            opacity: op,
                            fill: true,
                        });
                    }
                    ToolMode::Circle => {
                        state.history.push(DrawAction::Circle {
                            start: state.draw_start,
                            end: (mx, my),
                            color: col,
                            thickness: th,
                            opacity: op,
                        });
                    }
                    ToolMode::Arrow => {
                        state.history.push(DrawAction::Arrow {
                            start: state.draw_start,
                            end: (mx, my),
                            color: col,
                            thickness: th,
                            opacity: op,
                        });
                    }
                    ToolMode::NumberedBadge => {
                        // NUMBERED ARROW COMMIT
                        state.history.push(DrawAction::NumberedArrow {
                            start: state.draw_start,
                            end: (mx, my),
                            number: state.history.next_number,
                            color: col,
                            thickness: th,
                            opacity: op,
                        });
                    }
                    ToolMode::Curve => {
                        let dx = mx - state.draw_start.0;
                        let dy = my - state.draw_start.1;
                        if (dx * dx + dy * dy).sqrt() > 5.0 {
                            state.curve_phase = CurvePhase::Bending {
                                start: state.draw_start,
                                end: (mx, my),
                            };
                            state.show_toast("Mozgasd az egeret az ív hajlításához, majd kattints a rögzítéshez!");
                        }
                    }
                    ToolMode::Blur => {
                        state.history.push(DrawAction::Blur {
                            start: state.draw_start,
                            end: (mx, my),
                        });
                    }
                    _ => {}
                }

                state.render();
            }

            LRESULT(0)
        }

        WM_RBUTTONDOWN => {
            let _ = SetFocus(hwnd);
            if state.is_drawing {
                state.is_drawing = false;
                state.current_points.clear();
                let _ = ReleaseCapture();
                state.render();
                return LRESULT(0);
            }
            handle_escape_press(state);
            LRESULT(0)
        }

        WM_CHAR => {
            if state.text_pos.is_some() {
                let ch = char::from_u32(wparam.0 as u32).unwrap_or('\0');
                if ch == '\r' {
                    state.commit_text_if_any();
                } else if ch == '\x08' {
                    state.text_buffer.pop();
                } else if ch == '\x1b' {
                    state.text_pos = None;
                    state.text_buffer.clear();
                } else if !ch.is_control() {
                    state.text_buffer.push(ch);
                }
                state.render();
                return LRESULT(0);
            }
            LRESULT(0)
        }

        WM_KEYDOWN => {
            let key = wparam.0 as u32;
            let ctrl_down = (GetKeyState(VK_CONTROL.0 as i32) as u16 & 0x8000) != 0;

            if key == VK_ESCAPE.0 as u32 {
                handle_escape_press(state);
                return LRESULT(0);
            }

            if key == VK_F1.0 as u32 {
                state.active_modal = if state.active_modal == ActiveModal::Help {
                    ActiveModal::None
                } else {
                    ActiveModal::Help
                };
                state.hovered_modal_btn = None;
                state.render();
                return LRESULT(0);
            }

            if key == VK_DELETE.0 as u32 {
                state.history.clear();
                state.history.reset_counter();
                state.text_pos = None;
                state.text_buffer.clear();
                state.snip_rect = None;
                state.curve_phase = CurvePhase::Idle;
                state.show_toast("✓ Rajzok törölve (Tiszta vászon)");
                state.render();
                return LRESULT(0);
            }

            if ctrl_down {
                if key == 'Z' as u32 {
                    state.history.undo();
                    state.show_toast("↶ Visszavonás");
                    state.render();
                    return LRESULT(0);
                }
                if key == 'Y' as u32 {
                    state.history.redo();
                    state.show_toast("↷ Újra");
                    state.render();
                    return LRESULT(0);
                }
                if key == 'C' as u32 {
                    if let Some(r) = state.snip_rect {
                        let full_bgra = state.capture_rendered_canvas();
                        let cropped = crop_bgra(
                            &full_bgra,
                            state.capture.width,
                            state.capture.height,
                            r.0.round() as u32,
                            r.1.round() as u32,
                            (r.2 - r.0).round() as u32,
                            (r.3 - r.1).round() as u32,
                        );
                        if copy_bgra_to_clipboard(hwnd, (r.2 - r.0).round() as u32, (r.3 - r.1).round() as u32, &cropped) {
                            state.show_toast("✓ Kivágott terület másolva a vágólapra!");
                        }
                    } else {
                        state.export_display_action(0, "clipboard");
                    }
                    return LRESULT(0);
                }
                if key == 'S' as u32 {
                    state.export_display_action(0, "png");
                    return LRESULT(0);
                }
            }

            LRESULT(0)
        }

        WM_DESTROY => {
            PostQuitMessage(0);
            LRESULT(0)
        }

        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}

pub fn create_overlay_window() -> Option<HWND> {
    unsafe {
        let class_name = w!("DrawItOverlayClass");
        let h_module = windows::Win32::System::LibraryLoader::GetModuleHandleW(None).ok()?;
        let h_instance = windows::Win32::Foundation::HINSTANCE(h_module.0);

        // Load embedded application icon (resource ID 1)
        let ico_big = LoadImageW(
            h_instance,
            PCWSTR(1 as *const u16),
            IMAGE_ICON,
            GetSystemMetrics(SM_CXICON),
            GetSystemMetrics(SM_CYICON),
            LR_DEFAULTCOLOR | LR_SHARED,
        )
        .ok()
        .map(|h| HICON(h.0))
        .unwrap_or(HICON(ptr::null_mut()));

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
            style: CS_HREDRAW | CS_VREDRAW,
            lpfnWndProc: Some(overlay_wndproc),
            cbClsExtra: 0,
            cbWndExtra: 0,
            hInstance: h_instance,
            hIcon: ico_big,
            hCursor: HCURSOR(ptr::null_mut()),
            hbrBackground: windows::Win32::Graphics::Gdi::HBRUSH(ptr::null_mut()),
            lpszMenuName: PCWSTR::null(),
            lpszClassName: class_name,
            hIconSm: ico_sm,
        };

        let _ = RegisterClassExW(&wnd_class);

        let initial_capture = capture_screen();

        let hwnd = CreateWindowExW(
            WS_EX_TOPMOST | WS_EX_TOOLWINDOW,
            class_name,
            w!("DrawIt"),
            WS_POPUP,
            initial_capture.left,
            initial_capture.top,
            initial_capture.width as i32,
            initial_capture.height as i32,
            None,
            None,
            h_instance,
            None,
        ).ok()?;

        let _ = SendMessageW(hwnd, WM_SETICON, WPARAM(1), LPARAM(ico_big.0 as isize));
        let _ = SendMessageW(hwnd, WM_SETICON, WPARAM(0), LPARAM(ico_sm.0 as isize));

        let mut state = Box::new(OverlayState::new(hwnd, initial_capture));

        if let Ok(d2d) = D2DContext::new(hwnd, state.capture.width, state.capture.height) {
            state.d2d = Some(d2d);
        }

        let state_ptr = Box::into_raw(state);
        SetWindowLongPtrW(hwnd, GWLP_USERDATA, state_ptr as isize);

        Some(hwnd)
    }
}

pub fn show_overlay(hwnd: HWND) {
    unsafe {
        let ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut OverlayState;
        if !ptr.is_null() {
            let state = &mut *ptr;

            // Refresh screen capture & displays
            state.refresh_screen_capture();

            // Reposition window over the entire virtual desktop
            let _ = SetWindowPos(
                hwnd,
                HWND_TOPMOST,
                state.capture.left,
                state.capture.top,
                state.capture.width as i32,
                state.capture.height as i32,
                windows::Win32::UI::WindowsAndMessaging::SET_WINDOW_POS_FLAGS(0),
            );

            if let Some(d2d) = &mut state.d2d {
                d2d.resize(state.capture.width, state.capture.height);
            }

            // Fresh canvas on open: clear all drawings and reset numbering to 1
            state.history.clear();
            state.history.reset_counter();
            state.curve_phase = CurvePhase::Idle;
            state.text_pos = None;
            state.text_buffer.clear();
            state.snip_rect = None;
            state.toast_message = None;

            // INTELLIGENT MULTI-MONITOR CENTERING:
            // Find which monitor the mouse cursor is currently on!
            let mut cursor_pt = POINT { x: 0, y: 0 };
            let _ = GetCursorPos(&mut cursor_pt);
            state.draw_current = ((cursor_pt.x - state.capture.left) as f32, (cursor_pt.y - state.capture.top) as f32);

            let active_mon = state.capture.monitors.iter().find(|m| {
                cursor_pt.x >= m.x && cursor_pt.x < m.x + m.width &&
                cursor_pt.y >= m.y && cursor_pt.y < m.y + m.height
            }).or_else(|| state.capture.monitors.first());

            if let Some(mon) = active_mon {
                let mon_client_x = (mon.x - state.capture.left) as f32;
                let mon_client_y = (mon.y - state.capture.top) as f32;
                let mon_w = mon.width as f32;

                state.toolbar.x = (mon_client_x + (mon_w - state.toolbar.width) / 2.0).max(mon_client_x + 10.0);
                state.toolbar.y = mon_client_y + 30.0;
            } else {
                let screen_w = state.capture.width as f32;
                state.toolbar.x = ((screen_w - state.toolbar.width) / 2.0).max(20.0);
                state.toolbar.y = 30.0;
            }

            state.toolbar.dismiss_popup();
            state.active_modal = ActiveModal::None;
            state.hovered_modal_btn = None;

            if let Some(update) = crate::updater::get_available_update() {
                state.show_toast(&format!("Új DrawIt verzió érhető el: {} (Logo menü)", update.tag));
            }

            IS_OVERLAY_OPEN.store(true, Ordering::SeqCst);
            let _ = ShowWindow(hwnd, SW_SHOW);
            let _ = SetForegroundWindow(hwnd);
            let _ = SetActiveWindow(hwnd);
            let _ = SetFocus(hwnd);
            let _ = RegisterHotKey(
                hwnd,
                9998,
                HOT_KEY_MODIFIERS(0),
                VK_ESCAPE.0 as u32,
            );
            let _ = UpdateWindow(hwnd);
            state.render();
        }
    }
}

pub fn hide_overlay(state: &mut OverlayState) {
    unsafe {
        let _ = UnregisterHotKey(state.hwnd, 9998);
        state.active_modal = ActiveModal::None;
        state.hovered_modal_btn = None;
        state.toolbar.dismiss_popup();
        state.commit_text_if_any();
        state.history.clear();
        state.history.reset_counter();
        state.curve_phase = CurvePhase::Idle;
        state.text_pos = None;
        state.text_buffer.clear();
        state.snip_rect = None;
        state.toast_message = None;
        state.pipette_buffer = None;
        state.is_dragging_hue = false;
        state.is_dragging_lightness = false;
        if state.toolbar.active_tool == ToolMode::Pipette {
            state.toolbar.active_tool = state.previous_tool_before_pipette;
        }
        IS_OVERLAY_OPEN.store(false, Ordering::SeqCst);
        let _ = ShowWindow(state.hwnd, SW_HIDE);
    }
}
