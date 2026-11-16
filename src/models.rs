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

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct ToolKeybindings {
    pub freehand: char,
    pub line: char,
    pub arrow: char,
    pub rectangle: char,
    pub filled_rectangle: char,
    pub circle: char,
    pub badge: char,
    pub curve: char,
    pub text: char,
    pub blur: char,
    pub snip: char,
    pub pipette: char,
}

impl Default for ToolKeybindings {
    fn default() -> Self {
        Self {
            freehand: 'P',
            line: 'L',
            arrow: 'A',
            rectangle: 'R',
            filled_rectangle: 'M',
            circle: 'C',
            badge: 'N',
            curve: 'V',
            text: 'T',
            blur: 'X',
            snip: 'S',
            pipette: 'I',
        }
    }
}

impl ToolKeybindings {
    pub fn get_key(&self, tool: ToolMode) -> char {
        match tool {
            ToolMode::Freehand => self.freehand,
            ToolMode::Line => self.line,
            ToolMode::Arrow => self.arrow,
            ToolMode::Rectangle => self.rectangle,
            ToolMode::FilledRectangle => self.filled_rectangle,
            ToolMode::Circle => self.circle,
            ToolMode::NumberedBadge => self.badge,
            ToolMode::Curve => self.curve,
            ToolMode::Text => self.text,
            ToolMode::Blur => self.blur,
            ToolMode::Snip => self.snip,
            ToolMode::Pipette => self.pipette,
        }
    }

    pub fn set_key(&mut self, target_tool: ToolMode, mut new_key: char) {
        new_key = new_key.to_ascii_uppercase();
        let old_key = self.get_key(target_tool);
        if old_key == new_key {
            return;
        }

        let all_tools = [
            ToolMode::Freehand,
            ToolMode::Line,
            ToolMode::Arrow,
            ToolMode::Rectangle,
            ToolMode::FilledRectangle,
            ToolMode::Circle,
            ToolMode::NumberedBadge,
            ToolMode::Curve,
            ToolMode::Text,
            ToolMode::Blur,
            ToolMode::Snip,
            ToolMode::Pipette,
        ];

        // Swap with previous tool if there is a conflict
        for t in all_tools {
            if t != target_tool && self.get_key(t) == new_key {
                self.set_key_direct(t, old_key);
                break;
            }
        }
        self.set_key_direct(target_tool, new_key);
    }

    fn set_key_direct(&mut self, tool: ToolMode, key: char) {
        match tool {
            ToolMode::Freehand => self.freehand = key,
            ToolMode::Line => self.line = key,
            ToolMode::Arrow => self.arrow = key,
            ToolMode::Rectangle => self.rectangle = key,
            ToolMode::FilledRectangle => self.filled_rectangle = key,
            ToolMode::Circle => self.circle = key,
            ToolMode::NumberedBadge => self.badge = key,
            ToolMode::Curve => self.curve = key,
            ToolMode::Text => self.text = key,
            ToolMode::Blur => self.blur = key,
            ToolMode::Snip => self.snip = key,
            ToolMode::Pipette => self.pipette = key,
        }
    }

    pub fn get_tool(&self, key: char) -> Option<ToolMode> {
        let upper = key.to_ascii_uppercase();
        let all_tools = [
            ToolMode::Freehand,
            ToolMode::Line,
            ToolMode::Arrow,
            ToolMode::Rectangle,
            ToolMode::FilledRectangle,
            ToolMode::Circle,
            ToolMode::NumberedBadge,
            ToolMode::Curve,
            ToolMode::Text,
            ToolMode::Blur,
            ToolMode::Snip,
            ToolMode::Pipette,
        ];
        all_tools.into_iter().find(|&t| self.get_key(t) == upper)
    }

    #[allow(dead_code)]
    pub fn all_tool_keys(&self) -> [(ToolMode, char); 12] {
        [
            (ToolMode::Freehand, self.freehand),
            (ToolMode::Line, self.line),
            (ToolMode::Arrow, self.arrow),
            (ToolMode::Rectangle, self.rectangle),
            (ToolMode::FilledRectangle, self.filled_rectangle),
            (ToolMode::Circle, self.circle),
            (ToolMode::NumberedBadge, self.badge),
            (ToolMode::Curve, self.curve),
            (ToolMode::Text, self.text),
            (ToolMode::Blur, self.blur),
            (ToolMode::Snip, self.snip),
            (ToolMode::Pipette, self.pipette),
        ]
    }
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

    #[allow(dead_code)]
    pub fn label_hu(&self) -> &'static str {
        self.label(crate::i18n::Language::Hu)
    }

    pub fn label(&self, lang: crate::i18n::Language) -> &'static str {
        match lang {
            crate::i18n::Language::Hu => match self {
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
            },
            crate::i18n::Language::En => match self {
                ToolMode::Freehand => "Pen",
                ToolMode::Line => "Line",
                ToolMode::Rectangle => "Rectangle",
                ToolMode::FilledRectangle => "Mask",
                ToolMode::Circle => "Circle",
                ToolMode::Arrow => "Arrow",
                ToolMode::NumberedBadge => "Numbered",
                ToolMode::Curve => "Curve",
                ToolMode::Text => "Text",
                ToolMode::Blur => "Blur",
                ToolMode::Snip => "Snip",
                ToolMode::Pipette => "Picker",
            },
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
    #[allow(dead_code)]
    pub fn label_hu(&self) -> &'static str {
        self.label(crate::i18n::Language::Hu)
    }

    pub fn label(&self, lang: crate::i18n::Language) -> &'static str {
        match lang {
            crate::i18n::Language::Hu => match self {
                CanvasMode::Transparent => "Átlátszó (Képernyő)",
                CanvasMode::Blurred => "Homályosított képernyő",
                CanvasMode::Whiteboard => "Fehér tábla",
                CanvasMode::Blackboard => "Fekete tábla",
            },
            crate::i18n::Language::En => match self {
                CanvasMode::Transparent => "Transparent (Screen)",
                CanvasMode::Blurred => "Blurred Screen",
                CanvasMode::Whiteboard => "Whiteboard",
                CanvasMode::Blackboard => "Blackboard",
            },
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keybindings_defaults_and_swap() {
        let mut kb = ToolKeybindings::default();
        assert_eq!(kb.get_key(ToolMode::Freehand), 'P');
        assert_eq!(kb.get_key(ToolMode::Line), 'L');
        assert_eq!(kb.get_tool('P'), Some(ToolMode::Freehand));
        assert_eq!(kb.get_tool('p'), Some(ToolMode::Freehand)); // case-insensitive

        // Rebind Freehand to 'L' -> should swap Line to 'P'
        kb.set_key(ToolMode::Freehand, 'L');
        assert_eq!(kb.get_key(ToolMode::Freehand), 'L');
        assert_eq!(kb.get_key(ToolMode::Line), 'P');
        assert_eq!(kb.get_tool('L'), Some(ToolMode::Freehand));
        assert_eq!(kb.get_tool('P'), Some(ToolMode::Line));

        // Assign a brand new key 'B' to Freehand
        kb.set_key(ToolMode::Freehand, 'B');
        assert_eq!(kb.get_key(ToolMode::Freehand), 'B');
        assert_eq!(kb.get_tool('B'), Some(ToolMode::Freehand));
    }
}
