use crate::d2d::D2DContext;
use crate::models::{CanvasMode, ToolMode, COLOR_PRESETS, OPACITY_PRESETS, THICKNESS_PRESETS};
use crate::screen::MonitorInfo;

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum ToolbarItem {
    GripHandle,
    LogoButton,
    Tool(ToolMode),
    ColorButton,
    Undo,
    Redo,
    Clear,
    CanvasModeButton,
    SnipButton,
    SaveDisplayButton,
    CloseButton,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum PopupAction {
    // Logo menu
    Help,
    CheckUpdates,
    About,
    ExitApp,
    // Color picker
    SelectColor((u8, u8, u8)),
    Eyedropper,
    HueBar(f32),
    LightnessBar(f32),
    SelectOpacity(f32),
    SelectThickness(f32),
    AdjustThickness(f32),
    // Canvas mode
    SelectCanvasMode(CanvasMode),
    // Save display
    SaveMonitorPng(usize), // usize::MAX for all monitors
    SaveMonitorPdf(usize),
    CopyMonitorClipboard(usize),
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum OpenPopup {
    None,
    LogoMenu,
    ColorPicker,
    CanvasMode,
    SaveDisplay,
}

pub struct Toolbar {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub active_tool: ToolMode,
    pub canvas_mode: CanvasMode,
    pub color: (u8, u8, u8),
    pub thickness: f32,
    pub opacity: f32,
    pub custom_hue: f32,
    pub custom_sat: f32,
    pub custom_val: f32,
    pub hovered_item: Option<ToolbarItem>,
    pub hovered_action: Option<PopupAction>,
    pub open_popup: OpenPopup,
    pub monitors: Vec<MonitorInfo>,
    pub screen_width: f32,
    pub can_undo: bool,
    pub can_redo: bool,
}

#[derive(PartialEq)]
pub enum HitTestResult {
    Item(ToolbarItem),
    Popup(PopupAction),
    OutsidePopupDismissed,
    ToolbarBackground,
    Miss,
}

impl Toolbar {
    pub fn new(screen_width: f32, monitors: Vec<MonitorInfo>) -> Self {
        let initial_width = 1110.0;
        let initial_x = ((screen_width - initial_width) / 2.0).max(20.0);
        let initial_y = 30.0;
        let init_col = (255, 59, 48); // Red
        let (init_h, init_s, init_v) = crate::utils::rgb_to_hsv(init_col.0, init_col.1, init_col.2);

        Self {
            x: initial_x,
            y: initial_y,
            width: initial_width,
            height: 48.0,
            active_tool: ToolMode::Freehand,
            canvas_mode: CanvasMode::Transparent,
            color: init_col,
            thickness: 4.0,
            opacity: 1.0,
            custom_hue: init_h,
            custom_sat: init_s,
            custom_val: init_v,
            hovered_item: None,
            hovered_action: None,
            open_popup: OpenPopup::None,
            monitors,
            screen_width,
            can_undo: false,
            can_redo: false,
        }
    }

    pub fn set_color_rgb(&mut self, rgb: (u8, u8, u8)) {
        self.color = rgb;
        let (h, s, v) = crate::utils::rgb_to_hsv(rgb.0, rgb.1, rgb.2);
        self.custom_hue = h;
        self.custom_sat = s;
        self.custom_val = v;
    }

    pub fn set_hue_fraction(&mut self, frac: f32) {
        self.custom_hue = (frac * 360.0).clamp(0.0, 360.0);
        if self.custom_sat < 0.15 {
            self.custom_sat = 0.95;
        }
        if self.custom_val < 0.15 {
            self.custom_val = 0.95;
        }
        self.color = crate::utils::hsv_to_rgb(self.custom_hue, self.custom_sat, self.custom_val);
    }

    pub fn set_lightness_fraction(&mut self, frac: f32) {
        let frac = frac.clamp(0.0, 1.0);
        if frac <= 0.5 {
            self.custom_val = (frac * 2.0).clamp(0.05, 1.0);
            self.custom_sat = 1.0;
        } else {
            self.custom_val = 1.0;
            self.custom_sat = ((1.0 - frac) * 2.0).clamp(0.0, 1.0);
        }
        self.color = crate::utils::hsv_to_rgb(self.custom_hue, self.custom_sat, self.custom_val);
    }

    pub fn dismiss_popup(&mut self) -> bool {
        if self.open_popup != OpenPopup::None {
            self.open_popup = OpenPopup::None;
            self.hovered_action = None;
            true
        } else {
            false
        }
    }

    /// Calculates all item bounding boxes and section dividers on the floating bar
    pub fn layout_all(&self) -> (Vec<(ToolbarItem, (f32, f32, f32, f32))>, Vec<f32>) {
        let mut items = Vec::new();
        let mut separators = Vec::new();

        let pad_x = 10.0;
        let btn_gap = 4.0;
        let sep_margin = 6.0;
        let btn_size = 36.0;

        let mut cur_x = self.x + pad_x;
        let bar_y = self.y;
        let btn_y1 = bar_y + 6.0;
        let btn_y2 = bar_y + self.height - 6.0; // 36px high

        // 1. Grip handle (width: 18px)
        let grip_w = 18.0;
        items.push((ToolbarItem::GripHandle, (cur_x, btn_y1, cur_x + grip_w, btn_y2)));
        cur_x += grip_w + btn_gap;

        // 2. Logo button (36x36)
        items.push((ToolbarItem::LogoButton, (cur_x, btn_y1, cur_x + btn_size, btn_y2)));
        cur_x += btn_size;

        // Separator 1
        cur_x += sep_margin;
        separators.push(cur_x);
        cur_x += 1.0 + sep_margin;

        // 3. Drawing Tools
        let tools = [
            ToolMode::Freehand,
            ToolMode::Line,
            ToolMode::Rectangle,
            ToolMode::FilledRectangle,
            ToolMode::Circle,
            ToolMode::Arrow,
            ToolMode::NumberedBadge,
            ToolMode::Curve,
            ToolMode::Text,
            ToolMode::Blur,
        ];

        for &t in &tools {
            let is_active = self.active_tool == t;
            let btn_w = if is_active {
                match t {
                    ToolMode::NumberedBadge => 112.0,
                    ToolMode::Rectangle => 100.0,
                    ToolMode::FilledRectangle => 96.0,
                    ToolMode::Freehand => 84.0,
                    ToolMode::Line => 84.0,
                    ToolMode::Circle => 80.0,
                    ToolMode::Arrow => 82.0,
                    ToolMode::Curve => 76.0,
                    ToolMode::Text => 90.0,
                    ToolMode::Blur => 104.0,
                    _ => 84.0,
                }
            } else {
                btn_size // 36.0
            };

            items.push((ToolbarItem::Tool(t), (cur_x, btn_y1, cur_x + btn_w, btn_y2)));
            cur_x += btn_w + btn_gap;
        }
        cur_x -= btn_gap; // remove trailing gap

        // Separator 2
        cur_x += sep_margin;
        separators.push(cur_x);
        cur_x += 1.0 + sep_margin;

        // 4. Color & Stroke button
        let color_w = 78.0;
        items.push((ToolbarItem::ColorButton, (cur_x, btn_y1, cur_x + color_w, btn_y2)));
        cur_x += color_w;

        // Separator 3
        cur_x += sep_margin;
        separators.push(cur_x);
        cur_x += 1.0 + sep_margin;

        // 5. History & Canvas Group
        // Undo (36x36)
        items.push((ToolbarItem::Undo, (cur_x, btn_y1, cur_x + btn_size, btn_y2)));
        cur_x += btn_size + btn_gap;

        // Redo (36x36)
        items.push((ToolbarItem::Redo, (cur_x, btn_y1, cur_x + btn_size, btn_y2)));
        cur_x += btn_size + btn_gap;

        // Clear canvas (36x36)
        items.push((ToolbarItem::Clear, (cur_x, btn_y1, cur_x + btn_size, btn_y2)));
        cur_x += btn_size + btn_gap;

        // Canvas Mode: "Kijelző ▾"
        let canvas_w = 88.0;
        items.push((ToolbarItem::CanvasModeButton, (cur_x, btn_y1, cur_x + canvas_w, btn_y2)));
        cur_x += canvas_w;

        // Separator 4
        cur_x += sep_margin;
        separators.push(cur_x);
        cur_x += 1.0 + sep_margin;

        // 6. Snip & Display Export
        let snip_w = 96.0;
        items.push((ToolbarItem::SnipButton, (cur_x, btn_y1, cur_x + snip_w, btn_y2)));
        cur_x += snip_w + btn_gap;

        let save_w = 148.0;
        items.push((ToolbarItem::SaveDisplayButton, (cur_x, btn_y1, cur_x + save_w, btn_y2)));
        cur_x += save_w;

        // Separator 5
        cur_x += sep_margin;
        separators.push(cur_x);
        cur_x += 1.0 + sep_margin;

        // 7. Close button (36x36)
        items.push((ToolbarItem::CloseButton, (cur_x, btn_y1, cur_x + btn_size, btn_y2)));

        (items, separators)
    }

    /// Calculates item bounding boxes on the main floating bar
    pub fn layout_items(&self) -> Vec<(ToolbarItem, (f32, f32, f32, f32))> {
        self.layout_all().0
    }

    /// Computes popover bounding box and interactive actions
    pub fn layout_popup(&self) -> Option<((f32, f32, f32, f32), Vec<(PopupAction, (f32, f32, f32, f32))>)> {
        let items = self.layout_items();

        match self.open_popup {
            OpenPopup::None => None,

            OpenPopup::LogoMenu => {
                let (_, l_rect) = items.iter().find(|(it, _)| *it == ToolbarItem::LogoButton)?;
                let pw = 220.0;
                let ph = 176.0;
                let px = l_rect.0.clamp(10.0, self.screen_width - pw - 10.0);
                let py = self.y + self.height + 4.0;

                let mut acts = Vec::new();
                let row_h = 36.0;
                let mut ry = py + 8.0;

                acts.push((PopupAction::Help, (px + 8.0, ry, px + pw - 8.0, ry + row_h)));
                ry += row_h + 4.0;

                acts.push((PopupAction::CheckUpdates, (px + 8.0, ry, px + pw - 8.0, ry + row_h)));
                ry += row_h + 4.0;

                acts.push((PopupAction::About, (px + 8.0, ry, px + pw - 8.0, ry + row_h)));
                ry += row_h + 4.0;

                acts.push((PopupAction::ExitApp, (px + 8.0, ry, px + pw - 8.0, ry + row_h)));

                Some(((px, py, px + pw, py + ph), acts))
            }

            OpenPopup::ColorPicker => {
                let (_, c_rect) = items.iter().find(|(it, _)| *it == ToolbarItem::ColorButton)?;
                let pw = 350.0;
                let ph = 330.0;
                let px = (c_rect.0 - 120.0).clamp(10.0, self.screen_width - pw - 10.0);
                let py = self.y + self.height + 4.0;

                let mut acts = Vec::new();
                let start_x = px + 16.0;

                // 1. Basic Preset colors (9 swatches)
                let swatch_size = 26.0;
                let gap = 7.0;
                let start_y = py + 34.0;

                for (idx, &(r, g, b, _)) in COLOR_PRESETS.iter().enumerate() {
                    let sx = start_x + (idx as f32) * (swatch_size + gap);
                    let sy = start_y;
                    acts.push((
                        PopupAction::SelectColor((r, g, b)),
                        (sx, sy, sx + swatch_size, sy + swatch_size),
                    ));
                }

                // 2. Extended Designer Palette (9 swatches)
                let ext_y = start_y + swatch_size + 6.0;
                for (idx, &(r, g, b, _)) in crate::models::COLOR_EXTENDED_PRESETS.iter().enumerate() {
                    let sx = start_x + (idx as f32) * (swatch_size + gap);
                    let sy = ext_y;
                    acts.push((
                        PopupAction::SelectColor((r, g, b)),
                        (sx, sy, sx + swatch_size, sy + swatch_size),
                    ));
                }

                // 3. Custom Hue Spectrum Bar + Pipetta Button
                let row3_y = py + 130.0;
                let pip_w = 88.0;
                let hue_w = 216.0;
                let hue_h = 24.0;
                let hue_r = (start_x, row3_y, start_x + hue_w, row3_y + hue_h);
                let pip_x = px + pw - 16.0 - pip_w;
                let pip_r = (pip_x, row3_y, pip_x + pip_w, row3_y + hue_h);

                acts.push((PopupAction::HueBar(self.custom_hue / 360.0), hue_r));
                acts.push((PopupAction::Eyedropper, pip_r));

                // 4. Lightness / Tone Bar (Hex code displayed next to it)
                let row4_y = row3_y + hue_h + 8.0;
                let light_h = 20.0;
                let light_r = (start_x, row4_y, start_x + hue_w, row4_y + light_h);
                acts.push((PopupAction::LightnessBar(0.5), light_r));

                // 5. Opacity presets: 25%, 50%, 75%, 100%
                let op_y = py + 220.0;
                let op_w = 72.0;
                let op_gap = 10.0;
                for (i, &op) in OPACITY_PRESETS.iter().enumerate() {
                    let ox = start_x + (i as f32) * (op_w + op_gap);
                    acts.push((
                        PopupAction::SelectOpacity(op),
                        (ox, op_y, ox + op_w, op_y + 26.0),
                    ));
                }

                // 6. Thickness presets: 2, 4, 8, 12 + minus/plus
                let th_y = py + 284.0;
                let th_w = 46.0;
                for (i, &th) in THICKNESS_PRESETS.iter().enumerate() {
                    let tx = start_x + (i as f32) * (th_w + 6.0);
                    acts.push((
                        PopupAction::SelectThickness(th),
                        (tx, th_y, tx + th_w, th_y + 26.0),
                    ));
                }
                let btn_minus_x = start_x + 4.0 * (th_w + 6.0);
                acts.push((
                    PopupAction::AdjustThickness(-1.0),
                    (btn_minus_x, th_y, btn_minus_x + 36.0, th_y + 26.0),
                ));
                let btn_plus_x = btn_minus_x + 42.0;
                acts.push((
                    PopupAction::AdjustThickness(1.0),
                    (btn_plus_x, th_y, btn_plus_x + 36.0, th_y + 26.0),
                ));

                Some(((px, py, px + pw, py + ph), acts))
            }

            OpenPopup::CanvasMode => {
                let (_, c_rect) = items.iter().find(|(it, _)| *it == ToolbarItem::CanvasModeButton)?;
                let pw = 210.0;
                let ph = 176.0;
                let px = (c_rect.0 - 40.0).clamp(10.0, self.screen_width - pw - 10.0);
                let py = self.y + self.height + 4.0;

                let mut acts = Vec::new();
                let row_h = 36.0;
                let mut ry = py + 8.0;

                let modes = [
                    CanvasMode::Transparent,
                    CanvasMode::Blurred,
                    CanvasMode::Whiteboard,
                    CanvasMode::Blackboard,
                ];

                for m in modes {
                    acts.push((
                        PopupAction::SelectCanvasMode(m),
                        (px + 8.0, ry, px + pw - 8.0, ry + row_h),
                    ));
                    ry += row_h + 4.0;
                }

                Some(((px, py, px + pw, py + ph), acts))
            }

            OpenPopup::SaveDisplay => {
                let (_, s_rect) = items.iter().find(|(it, _)| *it == ToolbarItem::SaveDisplayButton)?;
                let pw = 380.0;
                let px = (s_rect.0 - 60.0).clamp(10.0, self.screen_width - pw - 10.0);
                let py = self.y + self.height + 4.0;

                let mut acts = Vec::new();
                let mut ry = py + 48.0;

                let monitors_to_render = if self.monitors.is_empty() {
                    vec![MonitorInfo {
                        index: 0,
                        x: 0,
                        y: 0,
                        width: self.screen_width as i32,
                        height: 1080,
                        _is_primary: true,
                        name: "1. Kijelző".to_string(),
                    }]
                } else {
                    self.monitors.clone()
                };

                for mon in &monitors_to_render {
                    let card_y = ry;
                    let card_h = 70.0;
                    let card_l = px + 12.0;

                    let btn_w = 106.0;
                    let btn_h = 28.0;
                    let by = card_y + 32.0;

                    let b_png = card_l + 10.0;
                    let b_pdf = b_png + btn_w + 9.0;
                    let b_clip = b_pdf + btn_w + 9.0;

                    acts.push((
                        PopupAction::SaveMonitorPng(mon.index),
                        (b_png, by, b_png + btn_w, by + btn_h),
                    ));
                    acts.push((
                        PopupAction::SaveMonitorPdf(mon.index),
                        (b_pdf, by, b_pdf + btn_w, by + btn_h),
                    ));
                    acts.push((
                        PopupAction::CopyMonitorClipboard(mon.index),
                        (b_clip, by, b_clip + btn_w, by + btn_h),
                    ));

                    ry += card_h + 10.0;
                }

                if monitors_to_render.len() > 1 {
                    let card_y = ry;
                    let card_h = 70.0;
                    let card_l = px + 12.0;

                    let btn_w = 106.0;
                    let btn_h = 28.0;
                    let by = card_y + 32.0;

                    let b_png = card_l + 10.0;
                    let b_pdf = b_png + btn_w + 9.0;
                    let b_clip = b_pdf + btn_w + 9.0;

                    acts.push((
                        PopupAction::SaveMonitorPng(usize::MAX),
                        (b_png, by, b_png + btn_w, by + btn_h),
                    ));
                    acts.push((
                        PopupAction::SaveMonitorPdf(usize::MAX),
                        (b_pdf, by, b_pdf + btn_w, by + btn_h),
                    ));
                    acts.push((
                        PopupAction::CopyMonitorClipboard(usize::MAX),
                        (b_clip, by, b_clip + btn_w, by + btn_h),
                    ));

                    ry += card_h + 10.0;
                }

                let ph = ry - py + 4.0;
                Some(((px, py, px + pw, py + ph), acts))
            }
        }
    }

    pub fn get_hue_bar_rect(&self) -> Option<(f32, f32, f32, f32)> {
        let (_, actions) = self.layout_popup()?;
        actions.into_iter().find(|(act, _)| matches!(act, PopupAction::HueBar(_))).map(|(_, r)| r)
    }

    pub fn get_lightness_bar_rect(&self) -> Option<(f32, f32, f32, f32)> {
        let (_, actions) = self.layout_popup()?;
        actions.into_iter().find(|(act, _)| matches!(act, PopupAction::LightnessBar(_))).map(|(_, r)| r)
    }

    /// Checks if mouse coordinates are inside the toolbar capsule or its active popup
    pub fn is_inside(&self, mx: f32, my: f32) -> bool {
        let in_bar = mx >= self.x && mx <= self.x + self.width
            && my >= self.y && my <= self.y + self.height;
        if in_bar {
            return true;
        }
        if let Some((pop_rect, _)) = self.layout_popup() {
            if mx >= pop_rect.0 && mx <= pop_rect.2 && my >= pop_rect.1 && my <= pop_rect.3 {
                return true;
            }
        }
        false
    }

    /// Evaluates clicks on toolbar and popups (called strictly on WM_LBUTTONDOWN)
    pub fn hit_test(&mut self, mx: f32, my: f32) -> HitTestResult {
        // 1. If a popup is open:
        if let Some((pop_rect, actions)) = self.layout_popup() {
            // A. Clicked inside popup card bounds:
            if mx >= pop_rect.0 && mx <= pop_rect.2 && my >= pop_rect.1 && my <= pop_rect.3 {
                for (act, r) in actions {
                    if mx >= r.0 && mx <= r.2 && my >= r.1 && my <= r.3 {
                        let actual_act = match act {
                            PopupAction::HueBar(_) => {
                                let frac = ((mx - r.0) / (r.2 - r.0)).clamp(0.0, 1.0);
                                PopupAction::HueBar(frac)
                            }
                            PopupAction::LightnessBar(_) => {
                                let frac = ((mx - r.0) / (r.2 - r.0)).clamp(0.0, 1.0);
                                PopupAction::LightnessBar(frac)
                            }
                            other => other,
                        };
                        return HitTestResult::Popup(actual_act);
                    }
                }
                // Clicked blank area of popup card: keep popup open, consume click
                return HitTestResult::ToolbarBackground;
            }

            // B. Clicked directly on a toolbar button on the main bar:
            for (item, (l, t, r, b)) in self.layout_items() {
                if mx >= l && mx <= r && my >= t && my <= b {
                    return HitTestResult::Item(item);
                }
            }

            // C. Clicked on canvas outside both popup and toolbar: dismiss popup
            self.open_popup = OpenPopup::None;
            self.hovered_action = None;
            return HitTestResult::OutsidePopupDismissed;
        }

        // 2. Check main toolbar bounds
        let bar_left = self.x;
        let bar_top = self.y;
        let bar_right = self.x + self.width;
        let bar_bottom = self.y + self.height;

        if mx < bar_left || mx > bar_right || my < bar_top || my > bar_bottom {
            return HitTestResult::Miss;
        }

        // 3. Inside main bar: test items
        for (item, (l, t, r, b)) in self.layout_items() {
            if mx >= l && mx <= r && my >= t && my <= b {
                return HitTestResult::Item(item);
            }
        }

        HitTestResult::ToolbarBackground
    }

    /// Update hover state
    pub fn update_hover(&mut self, mx: f32, my: f32) {
        if let Some((pop_rect, actions)) = self.layout_popup() {
            if mx >= pop_rect.0 && mx <= pop_rect.2 && my >= pop_rect.1 && my <= pop_rect.3 {
                self.hovered_item = None;
                for (act, r) in actions {
                    if mx >= r.0 && mx <= r.2 && my >= r.1 && my <= r.3 {
                        self.hovered_action = Some(act);
                        return;
                    }
                }
                self.hovered_action = None;
                return;
            }
        }

        self.hovered_action = None;

        for (item, (l, t, r, b)) in self.layout_items() {
            if mx >= l && mx <= r && my >= t && my <= b {
                self.hovered_item = Some(item);
                return;
            }
        }
        self.hovered_item = None;
    }

    /// Renders the toolbar and any open popovers via Direct2D
    pub fn render(&mut self, d2d: &D2DContext) {
        let (items, separators) = self.layout_all();

        // Calculate dynamic width of bar based on expanded tools + pad_x (10.0)
        if let Some((_, last_rect)) = items.last() {
            self.width = last_rect.2 - self.x + 10.0;
        }

        let bar_l = self.x;
        let bar_t = self.y;
        let bar_r = self.x + self.width;
        let bar_b = self.y + self.height;
        let bar_radius = 24.0;

        // 1. Soft multi-layered drop shadow
        d2d.fill_rounded_rectangle(bar_l - 2.0, bar_t + 2.0, bar_r + 2.0, bar_b + 5.0, bar_radius, (0.0, 0.0, 0.0, 0.20));
        d2d.fill_rounded_rectangle(bar_l - 1.0, bar_t + 1.0, bar_r + 1.0, bar_b + 2.0, bar_radius, (0.0, 0.0, 0.0, 0.25));

        // 2. Bar glassmorphism capsule background: dark frosted glass
        d2d.fill_rounded_rectangle(bar_l, bar_t, bar_r, bar_b, bar_radius, (0.09, 0.11, 0.15, 0.92));

        // 3. Subtle highlight border: rgba(255, 255, 255, 0.14)
        d2d.draw_rounded_rectangle(bar_l, bar_t, bar_r, bar_b, bar_radius, (1.0, 1.0, 1.0, 0.14), 1.0);

        // 4. Subtle vertical hairline separators between groups
        let sep_y1 = bar_t + 12.0;
        let sep_y2 = bar_b - 12.0;
        for &sx in &separators {
            d2d.draw_line(sx, sep_y1, sx, sep_y2, (1.0, 1.0, 1.0, 0.14), 1.0);
        }

        // 5. Render items
        for (item, (l, t, r, b)) in items {
            let is_hovered = self.hovered_item == Some(item);

            match item {
                ToolbarItem::GripHandle => {
                    // 6-dot grip handle ⋮⋮
                    let dot_color = if is_hovered { (0.9, 0.9, 0.9, 0.8) } else { (0.6, 0.6, 0.65, 0.5) };
                    let cx1 = l + 6.0;
                    let cx2 = l + 12.0;
                    let cy_mid = (t + b) / 2.0;

                    for &cy in &[cy_mid - 7.0, cy_mid, cy_mid + 7.0] {
                        d2d.fill_ellipse(cx1, cy, 1.8, 1.8, dot_color);
                        d2d.fill_ellipse(cx2, cy, 1.8, 1.8, dot_color);
                    }
                }

                ToolbarItem::LogoButton => {
                    let is_active = self.open_popup == OpenPopup::LogoMenu;
                    if is_active || is_hovered {
                        d2d.fill_rounded_rectangle(l, t, r, b, 8.0, (1.0, 1.0, 1.0, 0.12));
                    }
                    // Hardware-accelerated direct vector SVG rendering
                    d2d.draw_svg_logo(l + 4.5, t + 4.5, r - 4.5, b - 4.5);
                }

                ToolbarItem::Tool(mode) => {
                    let is_active = self.active_tool == mode;

                    if is_active {
                        // Active Pill: Apple System Blue (#0A84FF)
                        d2d.fill_rounded_rectangle(l, t, r, b, 10.0, (0.04, 0.52, 1.0, 0.95));
                        d2d.draw_rounded_rectangle(l, t, r, b, 10.0, (1.0, 1.0, 1.0, 0.25), 1.0);

                        // Vector icon on left of active pill
                        let icon_cx = l + 16.0;
                        let icon_cy = (t + b) / 2.0;
                        mode.draw_icon(d2d, icon_cx, icon_cy, (1.0, 1.0, 1.0, 1.0), 1.8);

                        // Hungarian label cleanly on right
                        d2d.draw_text_simple(mode.label_hu(), l + 28.0, t, r - 6.0, b, 12.0, (1.0, 1.0, 1.0, 1.0), true, true);
                    } else {
                        if is_hovered {
                            d2d.fill_rounded_rectangle(l, t, r, b, 8.0, (1.0, 1.0, 1.0, 0.10));
                        }
                        // Compact vector icon centered
                        let col = if is_hovered { (1.0, 1.0, 1.0, 1.0) } else { (0.78, 0.80, 0.84, 0.9) };
                        let icon_cx = (l + r) / 2.0;
                        let icon_cy = (t + b) / 2.0;
                        mode.draw_icon(d2d, icon_cx, icon_cy, col, 1.6);
                    }
                }

                ToolbarItem::ColorButton => {
                    let is_open = self.open_popup == OpenPopup::ColorPicker;
                    if is_open || is_hovered {
                        d2d.fill_rounded_rectangle(l, t, r, b, 8.0, (1.0, 1.0, 1.0, 0.12));
                    }

                    // Draw swatch circle
                    let cr = (self.color.0 as f32) / 255.0;
                    let cg = (self.color.1 as f32) / 255.0;
                    let cb = (self.color.2 as f32) / 255.0;
                    let cy = (t + b) / 2.0;

                    d2d.fill_ellipse(l + 14.0, cy, 7.0, 7.0, (cr, cg, cb, self.opacity));
                    d2d.draw_ellipse(l + 14.0, cy, 7.0, 7.0, (1.0, 1.0, 1.0, 0.5), 1.0);

                    // Thickness label + arrow
                    let label = format!("{:.0}px ▾", self.thickness);
                    d2d.draw_text_simple(&label, l + 25.0, t, r - 4.0, b, 12.0, (0.85, 0.87, 0.90, 1.0), false, true);
                }

                ToolbarItem::Undo => {
                    let cx = (l + r) / 2.0;
                    let cy = (t + b) / 2.0;
                    let (bg, icon_col) = if self.can_undo {
                        if is_hovered {
                            (Some((1.0, 1.0, 1.0, 0.12)), (1.0, 1.0, 1.0, 1.0))
                        } else {
                            (None, (0.85, 0.88, 0.92, 0.95))
                        }
                    } else {
                        (None, (0.42, 0.45, 0.50, 0.45))
                    };

                    if let Some(bg_col) = bg {
                        d2d.fill_rounded_rectangle(l, t, r, b, 8.0, bg_col);
                    }
                    d2d.draw_undo_icon(cx, cy, icon_col, 1.8);
                }

                ToolbarItem::Redo => {
                    let cx = (l + r) / 2.0;
                    let cy = (t + b) / 2.0;
                    let (bg, icon_col) = if self.can_redo {
                        if is_hovered {
                            (Some((1.0, 1.0, 1.0, 0.12)), (1.0, 1.0, 1.0, 1.0))
                        } else {
                            (None, (0.85, 0.88, 0.92, 0.95))
                        }
                    } else {
                        (None, (0.42, 0.45, 0.50, 0.45))
                    };

                    if let Some(bg_col) = bg {
                        d2d.fill_rounded_rectangle(l, t, r, b, 8.0, bg_col);
                    }
                    d2d.draw_redo_icon(cx, cy, icon_col, 1.8);
                }

                ToolbarItem::Clear => {
                    let cx = (l + r) / 2.0;
                    let cy = (t + b) / 2.0;
                    if is_hovered {
                        d2d.fill_rounded_rectangle(l, t, r, b, 8.0, (1.0, 0.2, 0.2, 0.18));
                    }
                    let icon_col = if is_hovered { (1.0, 1.0, 1.0, 1.0) } else { (0.85, 0.88, 0.92, 0.95) };
                    d2d.draw_eraser_icon(cx, cy, icon_col, 1.8);
                }

                ToolbarItem::CanvasModeButton => {
                    let is_open = self.open_popup == OpenPopup::CanvasMode;
                    if is_open || is_hovered {
                        d2d.fill_rounded_rectangle(l, t, r, b, 8.0, (1.0, 1.0, 1.0, 0.12));
                    }
                    let col = if is_open || is_hovered { (1.0, 1.0, 1.0, 1.0) } else { (0.85, 0.87, 0.90, 1.0) };
                    let icon_cx = l + 14.0;
                    let icon_cy = (t + b) / 2.0;
                    d2d.draw_monitor_icon(icon_cx, icon_cy, col, 1.6);
                    d2d.draw_text_simple("Kijelző ▾", l + 25.0, t, r - 4.0, b, 12.0, col, false, true);
                }

                ToolbarItem::SnipButton => {
                    let is_active = self.active_tool == ToolMode::Snip;
                    let icon_cx = l + 16.0;
                    let icon_cy = (t + b) / 2.0;

                    if is_active {
                        d2d.fill_rounded_rectangle(l, t, r, b, 10.0, (0.04, 0.52, 1.0, 0.95));
                        d2d.draw_rounded_rectangle(l, t, r, b, 10.0, (1.0, 1.0, 1.0, 0.25), 1.0);
                        d2d.draw_crop_icon(icon_cx, icon_cy, (1.0, 1.0, 1.0, 1.0), 1.8);
                        d2d.draw_text_simple("Kivágás", l + 28.0, t, r - 6.0, b, 12.0, (1.0, 1.0, 1.0, 1.0), true, true);
                    } else {
                        if is_hovered {
                            d2d.fill_rounded_rectangle(l, t, r, b, 8.0, (1.0, 1.0, 1.0, 0.10));
                        }
                        let col = if is_hovered { (1.0, 1.0, 1.0, 1.0) } else { (0.85, 0.87, 0.90, 1.0) };
                        d2d.draw_crop_icon(icon_cx, icon_cy, col, 1.6);
                        d2d.draw_text_simple("Kivágás", l + 28.0, t, r - 6.0, b, 12.0, col, is_hovered, true);
                    }
                }

                ToolbarItem::SaveDisplayButton => {
                    let is_open = self.open_popup == OpenPopup::SaveDisplay;
                    if is_open || is_hovered {
                        d2d.fill_rounded_rectangle(l, t, r, b, 8.0, (1.0, 1.0, 1.0, 0.12));
                    }
                    let col = if is_open || is_hovered { (1.0, 1.0, 1.0, 1.0) } else { (0.85, 0.87, 0.90, 1.0) };
                    let icon_cx = l + 16.0;
                    let icon_cy = (t + b) / 2.0;
                    d2d.draw_save_icon(icon_cx, icon_cy, col, 1.6);
                    d2d.draw_text_simple("Kijelző mentése ▾", l + 28.0, t, r - 6.0, b, 12.0, col, false, true);
                }

                ToolbarItem::CloseButton => {
                    let cx = (l + r) / 2.0;
                    let cy = (t + b) / 2.0;
                    if is_hovered {
                        d2d.fill_rounded_rectangle(l, t, r, b, 8.0, (1.0, 0.23, 0.19, 0.85));
                        d2d.draw_close_icon(cx, cy, 4.5, (1.0, 1.0, 1.0, 1.0), 1.8);
                    } else {
                        d2d.draw_close_icon(cx, cy, 4.5, (0.75, 0.77, 0.80, 1.0), 1.6);
                    }
                }
            }
        }

        // 5. Render active popup card if open
        self.render_popup(d2d);
    }

    fn render_popup(&self, d2d: &D2DContext) {
        let (pop_rect, actions) = match self.layout_popup() {
            Some(p) => p,
            None => return,
        };

        let (pop_l, pop_t, pop_r, pop_b) = pop_rect;

        // Popover drop shadow
        d2d.fill_rounded_rectangle(pop_l - 2.0, pop_t + 2.0, pop_r + 2.0, pop_b + 6.0, 14.0, (0.0, 0.0, 0.0, 0.35));

        // Popover glassmorphic card background
        d2d.fill_rounded_rectangle(pop_l, pop_t, pop_r, pop_b, 14.0, (0.11, 0.13, 0.17, 0.96));
        d2d.draw_rounded_rectangle(pop_l, pop_t, pop_r, pop_b, 14.0, (1.0, 1.0, 1.0, 0.16), 1.0);

        match self.open_popup {
            OpenPopup::None => {}

            OpenPopup::LogoMenu => {
                for (act, (al, at, ar, ab)) in actions {
                    let is_hov = self.hovered_action == Some(act.clone());
                    let bg = if is_hov { (1.0, 1.0, 1.0, 0.12) } else { (1.0, 1.0, 1.0, 0.05) };
                    d2d.fill_rounded_rectangle(al, at, ar, ab, 8.0, bg);

                    let (label, is_danger) = match act {
                        PopupAction::Help => ("Súgó és billentyűk [F1]", false),
                        PopupAction::CheckUpdates => ("Frissítések keresése...", false),
                        PopupAction::About => ("Névjegy (DrawIt v0.1)", false),
                        PopupAction::ExitApp => ("Kilépés a rajzolóból", true),
                        _ => ("", false),
                    };
                    let col = if is_danger { (1.0, 0.35, 0.3, 1.0) } else { (0.9, 0.92, 0.95, 1.0) };
                    let icon_cx = al + 18.0;
                    let icon_cy = (at + ab) / 2.0;
                    match act {
                        PopupAction::Help => d2d.draw_help_book_icon(icon_cx, icon_cy, col, 1.6),
                        PopupAction::CheckUpdates => d2d.draw_refresh_icon(icon_cx, icon_cy, col, 1.6),
                        PopupAction::About => d2d.draw_info_icon(icon_cx, icon_cy, col, 1.6),
                        PopupAction::ExitApp => d2d.draw_exit_icon(icon_cx, icon_cy, col, 1.6),
                        _ => {}
                    }
                    d2d.draw_text_simple(label, al + 32.0, at, ar - 8.0, ab, 12.0, col, is_hov, false);
                }
            }

            OpenPopup::ColorPicker => {
                // Section Headers with aligned, generous spacing that never overlap buttons
                d2d.draw_text_simple("Színek és paletta", pop_l + 16.0, pop_t + 12.0, pop_r - 16.0, pop_t + 28.0, 12.0, (0.7, 0.75, 0.82, 1.0), true, false);

                d2d.draw_text_simple("Egyedi színskála és pipetta", pop_l + 16.0, pop_t + 106.0, pop_r - 16.0, pop_t + 122.0, 12.0, (0.7, 0.75, 0.82, 1.0), true, false);

                d2d.draw_text_simple("Átlátszatlanság", pop_l + 16.0, pop_t + 196.0, pop_r - 16.0, pop_t + 212.0, 12.0, (0.7, 0.75, 0.82, 1.0), true, false);

                d2d.draw_text_simple("Vonalvastagság", pop_l + 16.0, pop_t + 260.0, pop_r - 16.0, pop_t + 276.0, 12.0, (0.7, 0.75, 0.82, 1.0), true, false);

                for (act, (al, at, ar, ab)) in actions {
                    let is_hov = self.hovered_action == Some(act);

                    match act {
                        PopupAction::SelectColor(rgb) => {
                            let cr = (rgb.0 as f32) / 255.0;
                            let cg = (rgb.1 as f32) / 255.0;
                            let cb = (rgb.2 as f32) / 255.0;
                            let is_curr = self.color == rgb;

                            d2d.fill_rounded_rectangle(al, at, ar, ab, 6.0, (cr, cg, cb, 1.0));
                            if is_curr {
                                d2d.draw_rounded_rectangle(al - 2.0, at - 2.0, ar + 2.0, ab + 2.0, 8.0, (1.0, 1.0, 1.0, 0.95), 2.5);
                            } else if is_hov {
                                d2d.draw_rounded_rectangle(al - 1.0, at - 1.0, ar + 1.0, ab + 1.0, 7.0, (1.0, 1.0, 1.0, 0.6), 1.5);
                            } else {
                                d2d.draw_rounded_rectangle(al, at, ar, ab, 6.0, (1.0, 1.0, 1.0, 0.25), 1.0);
                            }
                        }

                        PopupAction::HueBar(_) => {
                            let bar_w = ar - al;
                            let steps = 36;
                            let step_w = bar_w / (steps as f32);
                            for i in 0..steps {
                                let frac = (i as f32) / (steps as f32);
                                let hue = frac * 360.0;
                                let (r, g, b) = crate::utils::hsv_to_rgb(hue, 1.0, 1.0);
                                let sl = al + (i as f32) * step_w;
                                let sr = if i == steps - 1 { ar } else { sl + step_w + 0.5 };
                                d2d.fill_rectangle(sl, at, sr, ab, ((r as f32)/255.0, (g as f32)/255.0, (b as f32)/255.0, 1.0));
                            }
                            d2d.draw_rounded_rectangle(al, at, ar, ab, 6.0, (1.0, 1.0, 1.0, 0.40), 1.2);

                            // Indicator knob
                            let knob_x = (al + (self.custom_hue / 360.0) * bar_w).clamp(al + 4.0, ar - 4.0);
                            let cy = (at + ab) / 2.0;
                            d2d.fill_ellipse(knob_x, cy, 6.0, 6.0, (1.0, 1.0, 1.0, 1.0));
                            d2d.draw_ellipse(knob_x, cy, 6.0, 6.0, (0.0, 0.0, 0.0, 0.6), 1.2);
                        }

                        PopupAction::LightnessBar(_) => {
                            let bar_w = ar - al;
                            let steps = 30;
                            let step_w = bar_w / (steps as f32);
                            for i in 0..steps {
                                let frac = (i as f32) / (steps as f32);
                                let (r, g, b) = if frac <= 0.5 {
                                    let v = frac * 2.0;
                                    crate::utils::hsv_to_rgb(self.custom_hue, 1.0, v)
                                } else {
                                    let s = (1.0 - frac) * 2.0;
                                    crate::utils::hsv_to_rgb(self.custom_hue, s, 1.0)
                                };
                                let sl = al + (i as f32) * step_w;
                                let sr = if i == steps - 1 { ar } else { sl + step_w + 0.5 };
                                d2d.fill_rectangle(sl, at, sr, ab, ((r as f32)/255.0, (g as f32)/255.0, (b as f32)/255.0, 1.0));
                            }
                            d2d.draw_rounded_rectangle(al, at, ar, ab, 6.0, (1.0, 1.0, 1.0, 0.40), 1.2);

                            // Indicator knob
                            let curr_frac = if self.custom_val < 0.99 {
                                self.custom_val * 0.5
                            } else {
                                0.5 + (1.0 - self.custom_sat) * 0.5
                            };
                            let knob_x = (al + curr_frac * bar_w).clamp(al + 4.0, ar - 4.0);
                            let cy = (at + ab) / 2.0;
                            d2d.fill_ellipse(knob_x, cy, 5.0, 5.0, (1.0, 1.0, 1.0, 1.0));
                            d2d.draw_ellipse(knob_x, cy, 5.0, 5.0, (0.0, 0.0, 0.0, 0.6), 1.2);

                            // Hex pill next to lightness bar
                            let hex_l = ar + 10.0;
                            let hex_w = 88.0;
                            let hex_r = hex_l + hex_w;
                            let hex_str = format!("#{:02X}{:02X}{:02X}", self.color.0, self.color.1, self.color.2);
                            let (cr, cg, cb) = ((self.color.0 as f32)/255.0, (self.color.1 as f32)/255.0, (self.color.2 as f32)/255.0);
                            d2d.fill_rounded_rectangle(hex_l, at, hex_r, ab, 5.0, (1.0, 1.0, 1.0, 0.08));
                            d2d.draw_rounded_rectangle(hex_l, at, hex_r, ab, 5.0, (1.0, 1.0, 1.0, 0.16), 1.0);
                            d2d.fill_ellipse(hex_l + 12.0, cy, 4.0, 4.0, (cr, cg, cb, 1.0));
                            d2d.draw_ellipse(hex_l + 12.0, cy, 4.0, 4.0, (1.0, 1.0, 1.0, 0.6), 0.8);
                            d2d.draw_text_simple(&hex_str, hex_l + 20.0, at, hex_r - 2.0, ab, 11.0, (0.95, 0.95, 0.95, 1.0), true, true);
                        }

                        PopupAction::Eyedropper => {
                            let (bg, border, col) = if is_hov {
                                ((0.04, 0.52, 1.0, 0.45), (0.20, 0.70, 1.0, 0.90), (1.0, 1.0, 1.0, 1.0))
                            } else {
                                ((1.0, 1.0, 1.0, 0.08), (1.0, 1.0, 1.0, 0.16), (0.90, 0.92, 0.96, 1.0))
                            };
                            d2d.fill_rounded_rectangle(al, at, ar, ab, 6.0, bg);
                            d2d.draw_rounded_rectangle(al, at, ar, ab, 6.0, border, 1.0);
                            d2d.draw_pipette_icon(al + 16.0, (at + ab) / 2.0, col, 1.5);
                            d2d.draw_text_simple("Pipetta", al + 28.0, at, ar - 4.0, ab, 11.5, col, is_hov, false);
                        }

                        PopupAction::SelectOpacity(op) => {
                            let is_curr = (self.opacity - op).abs() < 0.05;
                            let bg = if is_curr {
                                (0.04, 0.52, 1.0, 0.9)
                            } else if is_hov {
                                (1.0, 1.0, 1.0, 0.16)
                            } else {
                                (1.0, 1.0, 1.0, 0.08)
                            };
                            d2d.fill_rounded_rectangle(al, at, ar, ab, 6.0, bg);
                            let label = format!("{:.0}%", op * 100.0);
                            d2d.draw_text_simple(&label, al, at, ar, ab, 12.0, (1.0, 1.0, 1.0, 1.0), is_curr, true);
                        }

                        PopupAction::SelectThickness(th) => {
                            let is_curr = (self.thickness - th).abs() < 0.1;
                            let bg = if is_curr {
                                (0.04, 0.52, 1.0, 0.9)
                            } else if is_hov {
                                (1.0, 1.0, 1.0, 0.16)
                            } else {
                                (1.0, 1.0, 1.0, 0.08)
                            };
                            d2d.fill_rounded_rectangle(al, at, ar, ab, 6.0, bg);
                            let label = format!("{:.0}px", th);
                            d2d.draw_text_simple(&label, al, at, ar, ab, 12.0, (1.0, 1.0, 1.0, 1.0), is_curr, true);
                        }

                        PopupAction::AdjustThickness(delta) => {
                            let bg = if is_hov { (1.0, 1.0, 1.0, 0.22) } else { (1.0, 1.0, 1.0, 0.12) };
                            d2d.fill_rounded_rectangle(al, at, ar, ab, 6.0, bg);
                            let sign = if delta > 0.0 { "+" } else { "−" };
                            d2d.draw_text_simple(sign, al, at, ar, ab, 14.0, (1.0, 1.0, 1.0, 1.0), true, true);
                        }

                        _ => {}
                    }
                }
            }

            OpenPopup::CanvasMode => {
                for (act, (al, at, ar, ab)) in actions {
                    if let PopupAction::SelectCanvasMode(mode) = act {
                        let is_curr = self.canvas_mode == mode;
                        let is_hov = self.hovered_action == Some(PopupAction::SelectCanvasMode(mode));
                        let bg = if is_curr {
                            (0.04, 0.52, 1.0, 0.35)
                        } else if is_hov {
                            (1.0, 1.0, 1.0, 0.12)
                        } else {
                            (1.0, 1.0, 1.0, 0.05)
                        };
                        d2d.fill_rounded_rectangle(al, at, ar, ab, 8.0, bg);

                        let col = if is_curr { (0.2, 0.7, 1.0, 1.0) } else { (0.9, 0.92, 0.95, 1.0) };
                        let icon_cx = al + 18.0;
                        let icon_cy = (at + ab) / 2.0;
                        match mode {
                            CanvasMode::Transparent => d2d.draw_window_icon(icon_cx, icon_cy, col, 1.6),
                            CanvasMode::Blurred => d2d.draw_blur_tool_icon(icon_cx, icon_cy, col, 1.6),
                            CanvasMode::Whiteboard => d2d.draw_whiteboard_icon(icon_cx, icon_cy, col, 1.6),
                            CanvasMode::Blackboard => d2d.draw_blackboard_icon(icon_cx, icon_cy, col, 1.6),
                        }
                        d2d.draw_text_simple(mode.label_hu(), al + 32.0, at, ar - 8.0, ab, 12.0, col, is_curr, false);
                    }
                }
            }

            OpenPopup::SaveDisplay => {
                // Header
                d2d.draw_text_simple(
                    "Kijelző képének mentése",
                    pop_l + 16.0,
                    pop_t + 12.0,
                    pop_r - 16.0,
                    pop_t + 30.0,
                    13.0,
                    (0.95, 0.96, 0.98, 1.0),
                    true,
                    false,
                );

                // Subtitle hairline divider
                d2d.draw_line(
                    pop_l + 12.0,
                    pop_t + 38.0,
                    pop_r - 12.0,
                    pop_t + 38.0,
                    (1.0, 1.0, 1.0, 0.12),
                    1.0,
                );

                let mut row_indices = Vec::new();
                for (act, _) in &actions {
                    let idx = match act {
                        PopupAction::SaveMonitorPng(i) => *i,
                        PopupAction::SaveMonitorPdf(i) => *i,
                        PopupAction::CopyMonitorClipboard(i) => *i,
                        _ => continue,
                    };
                    if !row_indices.contains(&idx) {
                        row_indices.push(idx);
                    }
                }

                for idx in row_indices {
                    let name = if idx == usize::MAX {
                        "Teljes asztal (összes kijelző)".to_string()
                    } else if let Some(m) = self.monitors.iter().find(|m| m.index == idx) {
                        m.name.clone()
                    } else {
                        format!("{}. Kijelző", idx + 1)
                    };

                    let png_rect = actions.iter().find(|(a, _)| *a == PopupAction::SaveMonitorPng(idx)).map(|(_, r)| *r);
                    let pdf_rect = actions.iter().find(|(a, _)| *a == PopupAction::SaveMonitorPdf(idx)).map(|(_, r)| *r);
                    let clip_rect = actions.iter().find(|(a, _)| *a == PopupAction::CopyMonitorClipboard(idx)).map(|(_, r)| *r);

                    if let (Some(pr), Some(pdr), Some(cr)) = (png_rect, pdf_rect, clip_rect) {
                        let card_t = pr.1 - 32.0;
                        let card_b = pr.3 + 8.0;
                        let card_l = pop_l + 12.0;
                        let card_r = pop_r - 12.0;

                        // Card background & subtle border
                        d2d.fill_rounded_rectangle(card_l, card_t, card_r, card_b, 8.0, (1.0, 1.0, 1.0, 0.05));
                        d2d.draw_rounded_rectangle(card_l, card_t, card_r, card_b, 8.0, (1.0, 1.0, 1.0, 0.10), 1.0);

                        // Monitor vector icon + Title cleanly inside card
                        d2d.draw_monitor_icon(card_l + 18.0, card_t + 17.0, (0.75, 0.85, 1.0, 1.0), 1.6);
                        d2d.draw_text_simple(&name, card_l + 30.0, card_t + 8.0, card_r - 10.0, card_t + 26.0, 12.0, (0.90, 0.92, 0.96, 1.0), true, false);

                        // PNG Button
                        let is_png_hov = self.hovered_action == Some(PopupAction::SaveMonitorPng(idx));
                        let png_bg = if is_png_hov { (0.04, 0.52, 1.0, 0.45) } else { (0.04, 0.52, 1.0, 0.22) };
                        d2d.fill_rounded_rectangle(pr.0, pr.1, pr.2, pr.3, 6.0, png_bg);
                        d2d.draw_rounded_rectangle(pr.0, pr.1, pr.2, pr.3, 6.0, (0.04, 0.52, 1.0, 0.6), 1.0);
                        d2d.draw_save_icon(pr.0 + 16.0, (pr.1 + pr.3) / 2.0, (1.0, 1.0, 1.0, 1.0), 1.5);
                        d2d.draw_text_simple("PNG", pr.0 + 26.0, pr.1, pr.2 - 4.0, pr.3, 11.5, (1.0, 1.0, 1.0, 1.0), true, true);

                        // PDF Button
                        let is_pdf_hov = self.hovered_action == Some(PopupAction::SaveMonitorPdf(idx));
                        let pdf_bg = if is_pdf_hov { (1.0, 0.35, 0.2, 0.45) } else { (1.0, 0.35, 0.2, 0.22) };
                        d2d.fill_rounded_rectangle(pdr.0, pdr.1, pdr.2, pdr.3, 6.0, pdf_bg);
                        d2d.draw_rounded_rectangle(pdr.0, pdr.1, pdr.2, pdr.3, 6.0, (1.0, 0.35, 0.2, 0.6), 1.0);
                        d2d.draw_pdf_doc_icon(pdr.0 + 16.0, (pdr.1 + pdr.3) / 2.0, (1.0, 1.0, 1.0, 1.0), 1.5);
                        d2d.draw_text_simple("PDF", pdr.0 + 26.0, pdr.1, pdr.2 - 4.0, pdr.3, 11.5, (1.0, 1.0, 1.0, 1.0), true, true);

                        // Clipboard Button
                        let is_clip_hov = self.hovered_action == Some(PopupAction::CopyMonitorClipboard(idx));
                        let clip_bg = if is_clip_hov { (0.2, 0.8, 0.4, 0.45) } else { (0.2, 0.8, 0.4, 0.22) };
                        d2d.fill_rounded_rectangle(cr.0, cr.1, cr.2, cr.3, 6.0, clip_bg);
                        d2d.draw_rounded_rectangle(cr.0, cr.1, cr.2, cr.3, 6.0, (0.2, 0.8, 0.4, 0.6), 1.0);
                        d2d.draw_clipboard_icon(cr.0 + 16.0, (cr.1 + cr.3) / 2.0, (1.0, 1.0, 1.0, 1.0), 1.5);
                        d2d.draw_text_simple("Másolás", cr.0 + 26.0, cr.1, cr.2 - 4.0, cr.3, 11.5, (1.0, 1.0, 1.0, 1.0), true, true);
                    }
                }
            }
        }
    }
}
