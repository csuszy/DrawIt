#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ToolMode {
    Freehand,
    Line,
    Rectangle,
    FilledRectangle,
    Circle,
    Arrow,
    NumberedBadge,
    Curve,
    Text,
    Blur,
    Snip,
    Pipette,
}

impl ToolMode {
    pub fn draw_icon(&self, d2d: &crate::d2d::D2DContext, cx: f32, cy: f32, color: (f32, f32, f32, f32), stroke_width: f32) {
        match self {
            ToolMode::Freehand => d2d.draw_pen_icon(cx, cy, color, stroke_width),
            ToolMode::Line => d2d.draw_line_tool_icon(cx, cy, color, stroke_width),
            ToolMode::Rectangle => d2d.draw_rect_tool_icon(cx, cy, color, stroke_width),
            ToolMode::FilledRectangle => d2d.draw_filled_rect_tool_icon(cx, cy, color),
            ToolMode::Circle => d2d.draw_circle_tool_icon(cx, cy, color, stroke_width),
            ToolMode::Arrow => d2d.draw_arrow_tool_icon(cx, cy, color, stroke_width),
            ToolMode::NumberedBadge => d2d.draw_badge_tool_icon(cx, cy, color, stroke_width),
            ToolMode::Curve => d2d.draw_curve_tool_icon(cx, cy, color, stroke_width),
            ToolMode::Text => d2d.draw_text_tool_icon(cx, cy, color, stroke_width),
            ToolMode::Blur => d2d.draw_blur_tool_icon(cx, cy, color, stroke_width),
            ToolMode::Snip => d2d.draw_crop_icon(cx, cy, color, stroke_width),
            ToolMode::Pipette => d2d.draw_pipette_icon(cx, cy, color, stroke_width),
        }
    }

    pub fn label_hu(&self) -> &'static str {
        match self {
            ToolMode::Freehand => "Rajz",
            ToolMode::Line => "Vonal",
            ToolMode::Rectangle => "Téglalap",
            ToolMode::FilledRectangle => "Takarás",
            ToolMode::Circle => "Kör",
            ToolMode::Arrow => "Nyíl",
            ToolMode::NumberedBadge => "Számozás",
            ToolMode::Curve => "Ív",
            ToolMode::Text => "Szöveg",
            ToolMode::Blur => "Elmosás",
            ToolMode::Snip => "Kivágás",
            ToolMode::Pipette => "Pipetta",
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum CanvasMode {
    Transparent,
    Blurred,
    Whiteboard,
    Blackboard,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ActiveModal {
    None,
    Help,
    Settings,
    About,
}

impl CanvasMode {
    pub fn label_hu(&self) -> &'static str {
        match self {
            CanvasMode::Transparent => "Átlátszó",
            CanvasMode::Blurred => "Elmosott",
            CanvasMode::Whiteboard => "Fehér tábla",
            CanvasMode::Blackboard => "Fekete tábla",
        }
    }
}

pub const COLOR_PRESETS: &[(u8, u8, u8, &str)] = &[
    (255, 59, 48, "Piros"),
    (255, 149, 0, "Narancs"),
    (255, 204, 0, "Sárga"),
    (52, 199, 89, "Zöld"),
    (50, 173, 230, "Cián"),
    (0, 122, 255, "Kék"),
    (175, 82, 222, "Lila"),
    (255, 255, 255, "Fehér"),
    (28, 28, 30, "Fekete"),
];

pub const COLOR_EXTENDED_PRESETS: &[(u8, u8, u8, &str)] = &[
    (225, 29, 72, "Karmazsin"),
    (251, 146, 60, "Korall"),
    (245, 158, 11, "Borostyán"),
    (132, 204, 22, "Lime"),
    (20, 184, 166, "Türkiz"),
    (99, 102, 241, "Indigó"),
    (236, 72, 153, "Rózsaszín"),
    (148, 163, 184, "Világosszürke"),
    (71, 85, 105, "Palaszürke"),
];

pub const THICKNESS_PRESETS: &[f32] = &[2.0, 4.0, 8.0, 12.0];
pub const OPACITY_PRESETS: &[f32] = &[0.25, 0.50, 0.75, 1.00];

#[derive(Clone, Debug)]
pub enum DrawAction {
    Freehand {
        points: Vec<(f32, f32)>,
        color: (u8, u8, u8),
        thickness: f32,
        opacity: f32,
    },
    Line {
        start: (f32, f32),
        end: (f32, f32),
        color: (u8, u8, u8),
        thickness: f32,
        opacity: f32,
    },
    Rectangle {
        start: (f32, f32),
        end: (f32, f32),
        color: (u8, u8, u8),
        thickness: f32,
        opacity: f32,
        fill: bool,
    },
    Circle {
        start: (f32, f32),
        end: (f32, f32),
        color: (u8, u8, u8),
        thickness: f32,
        opacity: f32,
    },
    Arrow {
        start: (f32, f32),
        end: (f32, f32),
        color: (u8, u8, u8),
        thickness: f32,
        opacity: f32,
    },
    NumberedArrow {
        start: (f32, f32),
        end: (f32, f32),
        number: usize,
        color: (u8, u8, u8),
        thickness: f32,
        opacity: f32,
    },
    Curve {
        start: (f32, f32),
        ctrl: (f32, f32),
        end: (f32, f32),
        color: (u8, u8, u8),
        thickness: f32,
        opacity: f32,
    },
    Text {
        pos: (f32, f32),
        text: String,
        color: (u8, u8, u8),
        font_size: f32,
    },
    Blur {
        start: (f32, f32),
        end: (f32, f32),
    },
}

#[derive(Clone, Debug)]
pub struct DrawingHistory {
    pub actions: Vec<DrawAction>,
    pub redo_stack: Vec<DrawAction>,
    pub next_number: usize,
}

impl DrawingHistory {
    pub fn new() -> Self {
        Self {
            actions: Vec::new(),
            redo_stack: Vec::new(),
            next_number: 1,
        }
    }

    pub fn push(&mut self, action: DrawAction) {
        if let DrawAction::NumberedArrow { .. } = &action {
            self.next_number += 1;
        }
        self.actions.push(action);
        self.redo_stack.clear();
    }

    pub fn undo(&mut self) -> bool {
        if let Some(action) = self.actions.pop() {
            if let DrawAction::NumberedArrow { .. } = &action {
                if self.next_number > 1 {
                    self.next_number -= 1;
                }
            }
            self.redo_stack.push(action);
            true
        } else {
            false
        }
    }

    pub fn redo(&mut self) -> bool {
        if let Some(action) = self.redo_stack.pop() {
            if let DrawAction::NumberedArrow { .. } = &action {
                self.next_number += 1;
            }
            self.actions.push(action);
            true
        } else {
            false
        }
    }

    pub fn clear(&mut self) {
        self.actions.clear();
        self.redo_stack.clear();
        self.next_number = 1;
    }

    pub fn reset_counter(&mut self) {
        self.next_number = 1;
    }
}
