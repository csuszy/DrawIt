use std::ops::Deref;
use windows::core::{w, PCWSTR, Result};
use windows::Win32::Foundation::HWND;
use windows::Win32::Graphics::Direct2D::Common::{
    D2D1_BEZIER_SEGMENT, D2D1_COLOR_F, D2D1_FIGURE_BEGIN_FILLED, D2D1_FIGURE_BEGIN_HOLLOW,
    D2D1_FIGURE_END_CLOSED, D2D1_FIGURE_END_OPEN, D2D1_PIXEL_FORMAT, D2D_POINT_2F, D2D_RECT_F,
    D2D_SIZE_U,
};
use windows::Win32::Graphics::Direct2D::{
    D2D1CreateFactory, ID2D1Bitmap, ID2D1Factory, ID2D1HwndRenderTarget, ID2D1RenderTarget,
    ID2D1SolidColorBrush, ID2D1StrokeStyle, D2D1_BITMAP_PROPERTIES, D2D1_CAP_STYLE_ROUND,
    D2D1_CAP_STYLE_TRIANGLE, D2D1_ELLIPSE, D2D1_FACTORY_TYPE_SINGLE_THREADED,
    D2D1_HWND_RENDER_TARGET_PROPERTIES, D2D1_LINE_JOIN_ROUND, D2D1_PRESENT_OPTIONS_NONE,
    D2D1_RENDER_TARGET_PROPERTIES, D2D1_ROUNDED_RECT, D2D1_STROKE_STYLE_PROPERTIES,
};
use windows::Win32::Graphics::DirectWrite::{
    DWriteCreateFactory, IDWriteFactory, IDWriteTextFormat, DWRITE_FACTORY_TYPE_SHARED,
    DWRITE_FONT_STRETCH_NORMAL, DWRITE_FONT_STYLE_NORMAL, DWRITE_FONT_WEIGHT,
    DWRITE_FONT_WEIGHT_BOLD, DWRITE_FONT_WEIGHT_REGULAR, DWRITE_PARAGRAPH_ALIGNMENT_CENTER,
    DWRITE_TEXT_ALIGNMENT_CENTER, DWRITE_TEXT_ALIGNMENT_LEADING,
};
use windows::Win32::Graphics::Dxgi::Common::DXGI_FORMAT_B8G8R8A8_UNORM;

pub struct D2DContext {
    pub factory: ID2D1Factory,
    pub dwrite_factory: IDWriteFactory,
    pub target: ID2D1HwndRenderTarget,
    pub brush: ID2D1SolidColorBrush,
    pub stroke_round: ID2D1StrokeStyle,
    pub _stroke_arrow: ID2D1StrokeStyle,
    pub width: u32,
    pub height: u32,
}

impl D2DContext {
    pub fn new(hwnd: HWND, width: u32, height: u32) -> Result<Self> {
        unsafe {
            let factory: ID2D1Factory =
                D2D1CreateFactory(D2D1_FACTORY_TYPE_SINGLE_THREADED, None)?;

            let dwrite_factory: IDWriteFactory =
                DWriteCreateFactory(DWRITE_FACTORY_TYPE_SHARED)?;

            let rt_props = D2D1_RENDER_TARGET_PROPERTIES::default();
            let hwnd_props = D2D1_HWND_RENDER_TARGET_PROPERTIES {
                hwnd,
                pixelSize: D2D_SIZE_U {
                    width,
                    height,
                },
                presentOptions: D2D1_PRESENT_OPTIONS_NONE,
            };

            let target = factory.CreateHwndRenderTarget(&rt_props, &hwnd_props)?;
            let render_target: &ID2D1RenderTarget = target.deref();

            let initial_color = D2D1_COLOR_F {
                r: 1.0,
                g: 1.0,
                b: 1.0,
                a: 1.0,
            };
            let brush = render_target.CreateSolidColorBrush(&initial_color, None)?;

            // Round stroke for smooth freehand drawing
            let round_props = D2D1_STROKE_STYLE_PROPERTIES {
                startCap: D2D1_CAP_STYLE_ROUND,
                endCap: D2D1_CAP_STYLE_ROUND,
                dashCap: D2D1_CAP_STYLE_ROUND,
                lineJoin: D2D1_LINE_JOIN_ROUND,
                ..Default::default()
            };
            let stroke_round = factory.CreateStrokeStyle(&round_props, None)?;

            // Arrow stroke
            let arrow_props = D2D1_STROKE_STYLE_PROPERTIES {
                startCap: D2D1_CAP_STYLE_ROUND,
                endCap: D2D1_CAP_STYLE_TRIANGLE,
                lineJoin: D2D1_LINE_JOIN_ROUND,
                ..Default::default()
            };
            let stroke_arrow = factory.CreateStrokeStyle(&arrow_props, None)?;

            Ok(Self {
                factory,
                dwrite_factory,
                target,
                brush,
                stroke_round,
                _stroke_arrow: stroke_arrow,
                width,
                height,
            })
        }
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.width = width;
        self.height = height;
        let size = D2D_SIZE_U { width, height };
        unsafe {
            let _ = self.target.Resize(&size);
        }
    }

    pub fn begin_draw(&self) {
        unsafe {
            self.target.BeginDraw();
        }
    }

    pub fn end_draw(&self) -> Result<()> {
        unsafe { self.target.EndDraw(None, None) }
    }

    pub fn clear(&self, r: f32, g: f32, b: f32, a: f32) {
        unsafe {
            let c = D2D1_COLOR_F { r, g, b, a };
            self.target.Clear(Some(&c));
        }
    }

    pub fn set_brush_color(&self, r: f32, g: f32, b: f32, a: f32) {
        unsafe {
            let c = D2D1_COLOR_F { r, g, b, a };
            self.brush.SetColor(&c);
        }
    }

    pub fn fill_rectangle(&self, left: f32, top: f32, right: f32, bottom: f32, color: (f32, f32, f32, f32)) {
        self.set_brush_color(color.0, color.1, color.2, color.3);
        let rect = D2D_RECT_F { left, top, right, bottom };
        unsafe {
            self.target.FillRectangle(&rect, &self.brush);
        }
    }

    pub fn draw_rectangle(
        &self,
        left: f32,
        top: f32,
        right: f32,
        bottom: f32,
        color: (f32, f32, f32, f32),
        stroke_width: f32,
    ) {
        self.set_brush_color(color.0, color.1, color.2, color.3);
        let rect = D2D_RECT_F { left, top, right, bottom };
        unsafe {
            self.target.DrawRectangle(&rect, &self.brush, stroke_width, None);
        }
    }

    pub fn fill_rounded_rectangle(
        &self,
        left: f32,
        top: f32,
        right: f32,
        bottom: f32,
        radius: f32,
        color: (f32, f32, f32, f32),
    ) {
        self.set_brush_color(color.0, color.1, color.2, color.3);
        let rr = D2D1_ROUNDED_RECT {
            rect: D2D_RECT_F { left, top, right, bottom },
            radiusX: radius,
            radiusY: radius,
        };
        unsafe {
            self.target.FillRoundedRectangle(&rr, &self.brush);
        }
    }

    pub fn draw_rounded_rectangle(
        &self,
        left: f32,
        top: f32,
        right: f32,
        bottom: f32,
        radius: f32,
        color: (f32, f32, f32, f32),
        stroke_width: f32,
    ) {
        self.set_brush_color(color.0, color.1, color.2, color.3);
        let rr = D2D1_ROUNDED_RECT {
            rect: D2D_RECT_F { left, top, right, bottom },
            radiusX: radius,
            radiusY: radius,
        };
        unsafe {
            self.target.DrawRoundedRectangle(&rr, &self.brush, stroke_width, None);
        }
    }

    pub fn fill_ellipse(
        &self,
        cx: f32,
        cy: f32,
        rx: f32,
        ry: f32,
        color: (f32, f32, f32, f32),
    ) {
        self.set_brush_color(color.0, color.1, color.2, color.3);
        let ellipse = D2D1_ELLIPSE {
            point: D2D_POINT_2F { x: cx, y: cy },
            radiusX: rx,
            radiusY: ry,
        };
        unsafe {
            self.target.FillEllipse(&ellipse, &self.brush);
        }
    }

    pub fn draw_ellipse(
        &self,
        cx: f32,
        cy: f32,
        rx: f32,
        ry: f32,
        color: (f32, f32, f32, f32),
        stroke_width: f32,
    ) {
        self.set_brush_color(color.0, color.1, color.2, color.3);
        let ellipse = D2D1_ELLIPSE {
            point: D2D_POINT_2F { x: cx, y: cy },
            radiusX: rx,
            radiusY: ry,
        };
        unsafe {
            self.target.DrawEllipse(&ellipse, &self.brush, stroke_width, None);
        }
    }

    pub fn draw_line(
        &self,
        x0: f32,
        y0: f32,
        x1: f32,
        y1: f32,
        color: (f32, f32, f32, f32),
        stroke_width: f32,
    ) {
        self.set_brush_color(color.0, color.1, color.2, color.3);
        let p0 = D2D_POINT_2F { x: x0, y: y0 };
        let p1 = D2D_POINT_2F { x: x1, y: y1 };
        unsafe {
            self.target.DrawLine(p0, p1, &self.brush, stroke_width, Some(&self.stroke_round));
        }
    }

    pub fn draw_arrow(
        &self,
        x0: f32,
        y0: f32,
        x1: f32,
        y1: f32,
        color: (f32, f32, f32, f32),
        stroke_width: f32,
    ) {
        self.set_brush_color(color.0, color.1, color.2, color.3);

        let dx = x1 - x0;
        let dy = y1 - y0;
        let len = (dx * dx + dy * dy).sqrt();

        if len < 4.0 {
            self.draw_line(x0, y0, x1, y1, color, stroke_width);
            return;
        }

        // Arrow head size proportional to stroke width, clamped
        let head_len = (stroke_width * 3.5).clamp(16.0, 48.0);
        let head_width = head_len * 0.65;

        let u_x = dx / len;
        let u_y = dy / len;
        let p_x = -u_y;
        let p_y = u_x;

        // Base of the arrow head
        let base_x = x1 - u_x * head_len;
        let base_y = y1 - u_y * head_len;

        // Draw main shaft ending slightly inside head
        self.draw_line(x0, y0, base_x + u_x * 4.0, base_y + u_y * 4.0, color, stroke_width);

        // Arrow head vertices
        let left_x = base_x + p_x * head_width;
        let left_y = base_y + p_y * head_width;
        let right_x = base_x - p_x * head_width;
        let right_y = base_y - p_y * head_width;

        // Draw filled arrow head geometry
        unsafe {
            if let Ok(geom) = self.factory.CreatePathGeometry() {
                if let Ok(sink) = geom.Open() {
                    sink.BeginFigure(D2D_POINT_2F { x: x1, y: y1 }, D2D1_FIGURE_BEGIN_FILLED);
                    sink.AddLine(D2D_POINT_2F { x: left_x, y: left_y });
                    sink.AddLine(D2D_POINT_2F { x: right_x, y: right_y });
                    sink.EndFigure(D2D1_FIGURE_END_CLOSED);
                    let _ = sink.Close();
                    self.target.FillGeometry(&geom, &self.brush, None);
                }
            }
        }
    }

    pub fn draw_quadratic_curve(
        &self,
        x0: f32,
        y0: f32,
        cx: f32,
        cy: f32,
        x1: f32,
        y1: f32,
        color: (f32, f32, f32, f32),
        stroke_width: f32,
    ) {
        self.set_brush_color(color.0, color.1, color.2, color.3);
        unsafe {
            if let Ok(geom) = self.factory.CreatePathGeometry() {
                if let Ok(sink) = geom.Open() {
                    sink.BeginFigure(D2D_POINT_2F { x: x0, y: y0 }, D2D1_FIGURE_BEGIN_HOLLOW);
                    sink.AddQuadraticBezier(&windows::Win32::Graphics::Direct2D::D2D1_QUADRATIC_BEZIER_SEGMENT {
                        point1: D2D_POINT_2F { x: cx, y: cy },
                        point2: D2D_POINT_2F { x: x1, y: y1 },
                    });
                    sink.EndFigure(D2D1_FIGURE_END_OPEN);
                    let _ = sink.Close();
                    self.target.DrawGeometry(&geom, &self.brush, stroke_width, Some(&self.stroke_round));
                }
            }
        }
    }

    pub fn draw_undo_icon(&self, cx: f32, cy: f32, color: (f32, f32, f32, f32), stroke_width: f32) {
        self.set_brush_color(color.0, color.1, color.2, color.3);

        // Arrowhead lines with rounded caps meeting at the tip
        self.draw_line(cx - 2.5, cy - 6.5, cx - 6.5, cy - 2.5, color, stroke_width);
        self.draw_line(cx - 2.5, cy + 1.5, cx - 6.5, cy - 2.5, color, stroke_width);

        // Smooth curved stem path
        unsafe {
            if let Ok(geom) = self.factory.CreatePathGeometry() {
                if let Ok(sink) = geom.Open() {
                    sink.BeginFigure(D2D_POINT_2F { x: cx - 5.5, y: cy - 2.5 }, D2D1_FIGURE_BEGIN_HOLLOW);
                    sink.AddLine(D2D_POINT_2F { x: cx + 1.5, y: cy - 2.5 });
                    sink.AddQuadraticBezier(&windows::Win32::Graphics::Direct2D::D2D1_QUADRATIC_BEZIER_SEGMENT {
                        point1: D2D_POINT_2F { x: cx + 6.5, y: cy - 2.5 },
                        point2: D2D_POINT_2F { x: cx + 6.5, y: cy + 3.0 },
                    });
                    sink.AddLine(D2D_POINT_2F { x: cx + 4.5, y: cy + 6.0 });
                    sink.EndFigure(D2D1_FIGURE_END_OPEN);
                    let _ = sink.Close();
                    self.target.DrawGeometry(&geom, &self.brush, stroke_width, Some(&self.stroke_round));
                }
            }
        }
    }

    pub fn draw_redo_icon(&self, cx: f32, cy: f32, color: (f32, f32, f32, f32), stroke_width: f32) {
        self.set_brush_color(color.0, color.1, color.2, color.3);

        // Arrowhead lines with rounded caps meeting at the tip
        self.draw_line(cx + 2.5, cy - 6.5, cx + 6.5, cy - 2.5, color, stroke_width);
        self.draw_line(cx + 2.5, cy + 1.5, cx + 6.5, cy - 2.5, color, stroke_width);

        // Smooth curved stem path (horizontally mirrored)
        unsafe {
            if let Ok(geom) = self.factory.CreatePathGeometry() {
                if let Ok(sink) = geom.Open() {
                    sink.BeginFigure(D2D_POINT_2F { x: cx + 5.5, y: cy - 2.5 }, D2D1_FIGURE_BEGIN_HOLLOW);
                    sink.AddLine(D2D_POINT_2F { x: cx - 1.5, y: cy - 2.5 });
                    sink.AddQuadraticBezier(&windows::Win32::Graphics::Direct2D::D2D1_QUADRATIC_BEZIER_SEGMENT {
                        point1: D2D_POINT_2F { x: cx - 6.5, y: cy - 2.5 },
                        point2: D2D_POINT_2F { x: cx - 6.5, y: cy + 3.0 },
                    });
                    sink.AddLine(D2D_POINT_2F { x: cx - 4.5, y: cy + 6.0 });
                    sink.EndFigure(D2D1_FIGURE_END_OPEN);
                    let _ = sink.Close();
                    self.target.DrawGeometry(&geom, &self.brush, stroke_width, Some(&self.stroke_round));
                }
            }
        }
    }

    pub fn draw_eraser_icon(&self, cx: f32, cy: f32, color: (f32, f32, f32, f32), stroke_width: f32) {
        self.set_brush_color(color.0, color.1, color.2, color.3);

        // Angled eraser body (45-degree tilted rectangle)
        unsafe {
            if let Ok(geom) = self.factory.CreatePathGeometry() {
                if let Ok(sink) = geom.Open() {
                    sink.BeginFigure(D2D_POINT_2F { x: cx + 2.5, y: cy - 6.0 }, D2D1_FIGURE_BEGIN_HOLLOW);
                    sink.AddLine(D2D_POINT_2F { x: cx + 6.0, y: cy - 2.5 });
                    sink.AddLine(D2D_POINT_2F { x: cx - 2.0, y: cy + 5.5 });
                    sink.AddLine(D2D_POINT_2F { x: cx - 5.5, y: cy + 2.0 });
                    sink.EndFigure(D2D1_FIGURE_END_CLOSED);
                    let _ = sink.Close();
                    self.target.DrawGeometry(&geom, &self.brush, stroke_width, Some(&self.stroke_round));
                }
            }
        }

        // Sleeve band dividing line
        self.draw_line(cx - 2.0, cy - 1.5, cx + 1.5, cy + 2.0, color, stroke_width);

        // Canvas surface baseline
        self.draw_line(cx - 6.5, cy + 6.5, cx + 1.5, cy + 6.5, color, stroke_width);
    }

    pub fn draw_pen_icon(&self, cx: f32, cy: f32, color: (f32, f32, f32, f32), stroke_width: f32) {
        self.set_brush_color(color.0, color.1, color.2, color.3);
        // Pencil shaft
        self.draw_line(cx - 3.0, cy + 4.5, cx + 4.0, cy - 2.5, color, stroke_width);
        self.draw_line(cx - 4.5, cy + 3.0, cx + 2.5, cy - 4.0, color, stroke_width);
        // Top cap
        self.draw_line(cx + 2.5, cy - 4.0, cx + 4.0, cy - 2.5, color, stroke_width);
        // Nib collar
        self.draw_line(cx - 4.0, cy + 2.5, cx - 2.5, cy + 4.0, color, stroke_width * 0.85);
        // Nib cone to sharp tip
        self.draw_line(cx - 4.5, cy + 3.0, cx - 6.0, cy + 6.0, color, stroke_width);
        self.draw_line(cx - 3.0, cy + 4.5, cx - 6.0, cy + 6.0, color, stroke_width);
        // Lead tip dot
        self.fill_ellipse(cx - 5.5, cy + 5.5, 1.2, 1.2, color);
    }

    pub fn draw_line_tool_icon(&self, cx: f32, cy: f32, color: (f32, f32, f32, f32), stroke_width: f32) {
        self.draw_line(cx - 6.0, cy + 5.0, cx + 6.0, cy - 5.0, color, stroke_width);
        self.fill_ellipse(cx - 6.0, cy + 5.0, 1.6, 1.6, color);
        self.fill_ellipse(cx + 6.0, cy - 5.0, 1.6, 1.6, color);
    }

    pub fn draw_rect_tool_icon(&self, cx: f32, cy: f32, color: (f32, f32, f32, f32), stroke_width: f32) {
        self.draw_rounded_rectangle(cx - 6.0, cy - 5.0, cx + 6.0, cy + 5.0, 2.0, color, stroke_width);
    }

    pub fn draw_filled_rect_tool_icon(&self, cx: f32, cy: f32, color: (f32, f32, f32, f32)) {
        self.fill_rounded_rectangle(cx - 6.0, cy - 5.0, cx + 6.0, cy + 5.0, 2.0, color);
        let border_col = (color.0, color.1, color.2, (color.3 + 0.35).min(1.0));
        self.draw_rounded_rectangle(cx - 6.0, cy - 5.0, cx + 6.0, cy + 5.0, 2.0, border_col, 1.0);
    }

    pub fn draw_circle_tool_icon(&self, cx: f32, cy: f32, color: (f32, f32, f32, f32), stroke_width: f32) {
        self.draw_ellipse(cx, cy, 6.0, 6.0, color, stroke_width);
    }

    pub fn draw_arrow_tool_icon(&self, cx: f32, cy: f32, color: (f32, f32, f32, f32), stroke_width: f32) {
        // Arrow pointing up-right
        self.draw_line(cx - 5.5, cy + 5.5, cx + 4.5, cy - 4.5, color, stroke_width);
        self.draw_line(cx + 4.5, cy - 4.5, cx + 0.0, cy - 4.5, color, stroke_width);
        self.draw_line(cx + 4.5, cy - 4.5, cx + 4.5, cy - 0.0, color, stroke_width);
    }

    pub fn draw_badge_tool_icon(&self, cx: f32, cy: f32, color: (f32, f32, f32, f32), stroke_width: f32) {
        self.draw_ellipse(cx, cy, 6.2, 6.2, color, stroke_width);
        // Digit 1
        self.draw_line(cx + 0.2, cy - 3.2, cx + 0.2, cy + 3.2, color, stroke_width * 0.9);
        self.draw_line(cx - 1.6, cy - 1.6, cx + 0.2, cy - 3.2, color, stroke_width * 0.9);
        self.draw_line(cx - 1.6, cy + 3.2, cx + 2.0, cy + 3.2, color, stroke_width * 0.9);
    }

    pub fn draw_curve_tool_icon(&self, cx: f32, cy: f32, color: (f32, f32, f32, f32), stroke_width: f32) {
        self.draw_quadratic_curve(cx - 6.0, cy + 3.0, cx, cy - 6.0, cx + 6.0, cy + 3.0, color, stroke_width);
        self.fill_ellipse(cx - 6.0, cy + 3.0, 1.5, 1.5, color);
        self.fill_ellipse(cx + 6.0, cy + 3.0, 1.5, 1.5, color);
    }

    pub fn draw_text_tool_icon(&self, cx: f32, cy: f32, color: (f32, f32, f32, f32), stroke_width: f32) {
        self.draw_line(cx - 5.5, cy - 5.0, cx + 5.5, cy - 5.0, color, stroke_width);
        self.draw_line(cx, cy - 5.0, cx, cy + 5.5, color, stroke_width);
        self.draw_line(cx - 2.5, cy + 5.5, cx + 2.5, cy + 5.5, color, stroke_width);
    }

    pub fn draw_blur_tool_icon(&self, cx: f32, cy: f32, color: (f32, f32, f32, f32), stroke_width: f32) {
        self.set_brush_color(color.0, color.1, color.2, color.3);
        unsafe {
            if let Ok(geom) = self.factory.CreatePathGeometry() {
                if let Ok(sink) = geom.Open() {
                    sink.BeginFigure(D2D_POINT_2F { x: cx, y: cy - 6.0 }, D2D1_FIGURE_BEGIN_HOLLOW);
                    sink.AddQuadraticBezier(&windows::Win32::Graphics::Direct2D::D2D1_QUADRATIC_BEZIER_SEGMENT {
                        point1: D2D_POINT_2F { x: cx + 5.5, y: cy - 0.5 },
                        point2: D2D_POINT_2F { x: cx + 4.5, y: cy + 3.0 },
                    });
                    sink.AddQuadraticBezier(&windows::Win32::Graphics::Direct2D::D2D1_QUADRATIC_BEZIER_SEGMENT {
                        point1: D2D_POINT_2F { x: cx, y: cy + 7.0 },
                        point2: D2D_POINT_2F { x: cx - 4.5, y: cy + 3.0 },
                    });
                    sink.AddQuadraticBezier(&windows::Win32::Graphics::Direct2D::D2D1_QUADRATIC_BEZIER_SEGMENT {
                        point1: D2D_POINT_2F { x: cx - 5.5, y: cy - 0.5 },
                        point2: D2D_POINT_2F { x: cx, y: cy - 6.0 },
                    });
                    sink.EndFigure(D2D1_FIGURE_END_CLOSED);
                    let _ = sink.Close();
                    self.target.DrawGeometry(&geom, &self.brush, stroke_width, Some(&self.stroke_round));
                }
            }
        }
        // Inner highlight
        self.draw_quadratic_curve(cx - 2.0, cy - 1.5, cx - 2.8, cy + 2.0, cx - 1.0, cy + 3.8, color, stroke_width * 0.7);
    }

    pub fn draw_crop_icon(&self, cx: f32, cy: f32, color: (f32, f32, f32, f32), stroke_width: f32) {
        // Top-left crop corner
        self.draw_line(cx - 6.0, cy - 2.5, cx + 3.5, cy - 2.5, color, stroke_width);
        self.draw_line(cx - 2.5, cy - 6.0, cx - 2.5, cy + 3.5, color, stroke_width);
        // Bottom-right crop corner
        self.draw_line(cx - 3.5, cy + 2.5, cx + 6.0, cy + 2.5, color, stroke_width);
        self.draw_line(cx + 2.5, cy - 3.5, cx + 2.5, cy + 6.0, color, stroke_width);
    }

    pub fn draw_save_icon(&self, cx: f32, cy: f32, color: (f32, f32, f32, f32), stroke_width: f32) {
        // Disk body
        self.draw_rounded_rectangle(cx - 5.5, cy - 5.5, cx + 5.5, cy + 5.5, 1.5, color, stroke_width);
        // Top shutter
        self.draw_line(cx - 3.0, cy - 5.5, cx - 3.0, cy - 2.0, color, stroke_width);
        self.draw_line(cx + 2.0, cy - 5.5, cx + 2.0, cy - 2.0, color, stroke_width);
        self.draw_line(cx - 3.0, cy - 2.0, cx + 2.0, cy - 2.0, color, stroke_width);
        // Bottom label
        self.draw_line(cx - 3.5, cy + 1.5, cx + 3.5, cy + 1.5, color, stroke_width);
        self.draw_line(cx - 3.5, cy + 4.0, cx + 3.5, cy + 4.0, color, stroke_width);
    }

    pub fn draw_monitor_icon(&self, cx: f32, cy: f32, color: (f32, f32, f32, f32), stroke_width: f32) {
        self.draw_rounded_rectangle(cx - 6.5, cy - 5.5, cx + 6.5, cy + 2.5, 1.5, color, stroke_width);
        self.draw_line(cx, cy + 2.5, cx, cy + 5.5, color, stroke_width);
        self.draw_line(cx - 4.0, cy + 5.5, cx + 4.0, cy + 5.5, color, stroke_width);
    }

    pub fn draw_close_icon(&self, cx: f32, cy: f32, size: f32, color: (f32, f32, f32, f32), stroke_width: f32) {
        self.draw_line(cx - size, cy - size, cx + size, cy + size, color, stroke_width);
        self.draw_line(cx + size, cy - size, cx - size, cy + size, color, stroke_width);
    }

    pub fn draw_pipette_icon(&self, cx: f32, cy: f32, color: (f32, f32, f32, f32), stroke_width: f32) {
        // Shaft
        self.draw_line(cx - 2.0, cy + 4.0, cx + 3.0, cy - 1.0, color, stroke_width);
        self.draw_line(cx - 4.0, cy + 2.0, cx + 1.0, cy - 3.0, color, stroke_width);
        // Tip nozzle
        self.draw_line(cx - 4.0, cy + 2.0, cx - 5.5, cy + 5.5, color, stroke_width);
        self.draw_line(cx - 2.0, cy + 4.0, cx - 5.5, cy + 5.5, color, stroke_width);
        // Bulb
        self.fill_ellipse(cx + 3.5, cy - 3.5, 2.2, 2.2, color);
    }

    pub fn draw_clipboard_icon(&self, cx: f32, cy: f32, color: (f32, f32, f32, f32), stroke_width: f32) {
        // Board
        self.draw_rounded_rectangle(cx - 4.5, cy - 3.5, cx + 4.5, cy + 6.0, 1.5, color, stroke_width);
        // Clip
        self.draw_rounded_rectangle(cx - 2.2, cy - 5.5, cx + 2.2, cy - 3.0, 1.0, color, stroke_width);
        // Lines
        self.draw_line(cx - 2.5, cy - 0.5, cx + 2.5, cy - 0.5, color, stroke_width * 0.9);
        self.draw_line(cx - 2.5, cy + 2.2, cx + 2.5, cy + 2.2, color, stroke_width * 0.9);
    }

    pub fn draw_pdf_doc_icon(&self, cx: f32, cy: f32, color: (f32, f32, f32, f32), stroke_width: f32) {
        // Page outline with dog-ear at top-right
        self.draw_line(cx - 4.5, cy - 5.5, cx + 1.5, cy - 5.5, color, stroke_width);
        self.draw_line(cx + 4.5, cy - 2.5, cx + 4.5, cy + 6.0, color, stroke_width);
        self.draw_line(cx - 4.5, cy + 6.0, cx + 4.5, cy + 6.0, color, stroke_width);
        self.draw_line(cx - 4.5, cy - 5.5, cx - 4.5, cy + 6.0, color, stroke_width);
        // Dog-ear
        self.draw_line(cx + 1.5, cy - 5.5, cx + 4.5, cy - 2.5, color, stroke_width);
        self.draw_line(cx + 1.5, cy - 5.5, cx + 1.5, cy - 2.5, color, stroke_width);
        self.draw_line(cx + 1.5, cy - 2.5, cx + 4.5, cy - 2.5, color, stroke_width);
        // Text lines
        self.draw_line(cx - 2.5, cy + 1.0, cx + 2.5, cy + 1.0, color, stroke_width * 0.9);
        self.draw_line(cx - 2.5, cy + 3.5, cx + 2.5, cy + 3.5, color, stroke_width * 0.9);
    }

    pub fn draw_help_book_icon(&self, cx: f32, cy: f32, color: (f32, f32, f32, f32), stroke_width: f32) {
        // Circle with question mark
        self.draw_ellipse(cx, cy, 6.0, 6.0, color, stroke_width);
        // Question hook
        self.draw_quadratic_curve(cx - 1.8, cy - 2.5, cx, cy - 4.2, cx + 1.5, cy - 2.5, color, stroke_width * 0.9);
        self.draw_quadratic_curve(cx + 1.5, cy - 2.5, cx + 1.5, cy - 1.0, cx, cy - 0.2, color, stroke_width * 0.9);
        self.draw_line(cx, cy - 0.2, cx, cy + 1.2, color, stroke_width * 0.9);
        self.fill_ellipse(cx, cy + 2.8, 1.0, 1.0, color);
    }

    pub fn draw_refresh_icon(&self, cx: f32, cy: f32, color: (f32, f32, f32, f32), stroke_width: f32) {
        // 3/4 circular arrow
        self.draw_quadratic_curve(cx - 4.5, cy - 2.0, cx - 5.5, cy + 5.0, cx + 1.0, cy + 5.0, color, stroke_width);
        self.draw_quadratic_curve(cx + 1.0, cy + 5.0, cx + 5.5, cy + 4.5, cx + 4.5, cy - 1.0, color, stroke_width);
        self.draw_quadratic_curve(cx + 4.5, cy - 1.0, cx + 3.0, cy - 5.0, cx - 3.0, cy - 4.0, color, stroke_width);
        // Arrow tip at top
        self.draw_line(cx - 3.0, cy - 4.0, cx - 0.5, cy - 5.5, color, stroke_width);
        self.draw_line(cx - 3.0, cy - 4.0, cx - 1.5, cy - 1.5, color, stroke_width);
    }

    pub fn draw_info_icon(&self, cx: f32, cy: f32, color: (f32, f32, f32, f32), stroke_width: f32) {
        self.draw_ellipse(cx, cy, 6.0, 6.0, color, stroke_width);
        self.fill_ellipse(cx, cy - 2.8, 1.1, 1.1, color);
        self.draw_line(cx, cy - 1.0, cx, cy + 2.8, color, stroke_width * 0.9);
        self.draw_line(cx - 1.2, cy - 1.0, cx, cy - 1.0, color, stroke_width * 0.9);
        self.draw_line(cx - 1.2, cy + 2.8, cx + 1.2, cy + 2.8, color, stroke_width * 0.9);
    }

    pub fn draw_exit_icon(&self, cx: f32, cy: f32, color: (f32, f32, f32, f32), stroke_width: f32) {
        // Door frame
        self.draw_line(cx + 1.0, cy - 5.5, cx - 4.5, cy - 5.5, color, stroke_width);
        self.draw_line(cx - 4.5, cy - 5.5, cx - 4.5, cy + 5.5, color, stroke_width);
        self.draw_line(cx - 4.5, cy + 5.5, cx + 1.0, cy + 5.5, color, stroke_width);
        // Arrow out right
        self.draw_line(cx - 2.0, cy, cx + 5.0, cy, color, stroke_width);
        self.draw_line(cx + 5.0, cy, cx + 2.5, cy - 2.5, color, stroke_width);
        self.draw_line(cx + 5.0, cy, cx + 2.5, cy + 2.5, color, stroke_width);
    }

    pub fn draw_window_icon(&self, cx: f32, cy: f32, color: (f32, f32, f32, f32), stroke_width: f32) {
        self.draw_rounded_rectangle(cx - 6.0, cy - 5.0, cx + 6.0, cy + 5.0, 1.5, color, stroke_width);
        self.draw_line(cx - 6.0, cy - 1.5, cx + 6.0, cy - 1.5, color, stroke_width);
        self.draw_line(cx, cy - 1.5, cx, cy + 5.0, color, stroke_width);
    }

    pub fn draw_whiteboard_icon(&self, cx: f32, cy: f32, color: (f32, f32, f32, f32), stroke_width: f32) {
        self.fill_rounded_rectangle(cx - 6.0, cy - 5.0, cx + 6.0, cy + 5.0, 1.5, (0.95, 0.95, 0.95, 0.9));
        self.draw_rounded_rectangle(cx - 6.0, cy - 5.0, cx + 6.0, cy + 5.0, 1.5, color, stroke_width);
    }

    pub fn draw_blackboard_icon(&self, cx: f32, cy: f32, color: (f32, f32, f32, f32), stroke_width: f32) {
        self.fill_rounded_rectangle(cx - 6.0, cy - 5.0, cx + 6.0, cy + 5.0, 1.5, (0.15, 0.16, 0.20, 0.9));
        self.draw_rounded_rectangle(cx - 6.0, cy - 5.0, cx + 6.0, cy + 5.0, 1.5, color, stroke_width);
    }

    pub fn draw_polyline(
        &self,
        points: &[(f32, f32)],
        color: (f32, f32, f32, f32),
        stroke_width: f32,
    ) {
        if points.len() < 2 {
            if let Some(p) = points.first() {
                self.fill_ellipse(p.0, p.1, stroke_width * 0.5, stroke_width * 0.5, color);
            }
            return;
        }

        self.set_brush_color(color.0, color.1, color.2, color.3);
        unsafe {
            if let Ok(geom) = self.factory.CreatePathGeometry() {
                if let Ok(sink) = geom.Open() {
                    sink.BeginFigure(
                        D2D_POINT_2F {
                            x: points[0].0,
                            y: points[0].1,
                        },
                        D2D1_FIGURE_BEGIN_HOLLOW,
                    );
                    for p in &points[1..] {
                        sink.AddLine(D2D_POINT_2F { x: p.0, y: p.1 });
                    }
                    sink.EndFigure(D2D1_FIGURE_END_OPEN);
                    let _ = sink.Close();
                    self.target.DrawGeometry(&geom, &self.brush, stroke_width, Some(&self.stroke_round));
                }
            }
        }
    }

    pub fn create_text_format(
        &self,
        font_family: PCWSTR,
        size: f32,
        weight: DWRITE_FONT_WEIGHT,
        center: bool,
    ) -> Result<IDWriteTextFormat> {
        unsafe {
            let format = self.dwrite_factory.CreateTextFormat(
                font_family,
                None,
                weight,
                DWRITE_FONT_STYLE_NORMAL,
                DWRITE_FONT_STRETCH_NORMAL,
                size,
                w!("hu-HU"),
            )?;

            // Always center text vertically within its box / button height
            let _ = format.SetParagraphAlignment(DWRITE_PARAGRAPH_ALIGNMENT_CENTER);

            if center {
                let _ = format.SetTextAlignment(DWRITE_TEXT_ALIGNMENT_CENTER);
            } else {
                let _ = format.SetTextAlignment(DWRITE_TEXT_ALIGNMENT_LEADING);
            }

            Ok(format)
        }
    }

    pub fn draw_text_simple(
        &self,
        text: &str,
        left: f32,
        top: f32,
        right: f32,
        bottom: f32,
        size: f32,
        color: (f32, f32, f32, f32),
        bold: bool,
        center: bool,
    ) {
        self.set_brush_color(color.0, color.1, color.2, color.3);

        let weight = if bold {
            DWRITE_FONT_WEIGHT_BOLD
        } else {
            DWRITE_FONT_WEIGHT_REGULAR
        };

        if let Ok(format) = self.create_text_format(w!("Segoe UI"), size, weight, center) {
            let utf16: Vec<u16> = text.encode_utf16().collect();
            let rect = D2D_RECT_F { left, top, right, bottom };
            unsafe {
                self.target.DrawText(
                    &utf16,
                    &format,
                    &rect,
                    &self.brush,
                    windows::Win32::Graphics::Direct2D::D2D1_DRAW_TEXT_OPTIONS_ENABLE_COLOR_FONT,
                    windows::Win32::Graphics::DirectWrite::DWRITE_MEASURING_MODE_NATURAL,
                );
            }
        }
    }

    pub fn create_bitmap_from_bgra(&self, width: u32, height: u32, data: &[u8]) -> Result<ID2D1Bitmap> {
        unsafe {
            let props = D2D1_BITMAP_PROPERTIES {
                pixelFormat: D2D1_PIXEL_FORMAT {
                    format: DXGI_FORMAT_B8G8R8A8_UNORM,
                    alphaMode: windows::Win32::Graphics::Direct2D::Common::D2D1_ALPHA_MODE_PREMULTIPLIED,
                },
                dpiX: 96.0,
                dpiY: 96.0,
            };
            let size = D2D_SIZE_U { width, height };
            let render_target: &ID2D1RenderTarget = self.target.deref();
            render_target.CreateBitmap(size, Some(data.as_ptr() as *const _), width * 4, &props)
        }
    }

    pub fn draw_bitmap(
        &self,
        bitmap: &ID2D1Bitmap,
        dst_left: f32,
        dst_top: f32,
        dst_right: f32,
        dst_bottom: f32,
        opacity: f32,
    ) {
        unsafe {
            let dst_rect = D2D_RECT_F {
                left: dst_left,
                top: dst_top,
                right: dst_right,
                bottom: dst_bottom,
            };
            self.target.DrawBitmap(
                bitmap,
                Some(&dst_rect),
                opacity,
                windows::Win32::Graphics::Direct2D::D2D1_BITMAP_INTERPOLATION_MODE_LINEAR,
                None,
            );
        }
    }

    pub fn draw_bitmap_crop(
        &self,
        bitmap: &ID2D1Bitmap,
        src_left: f32,
        src_top: f32,
        src_right: f32,
        src_bottom: f32,
        dst_left: f32,
        dst_top: f32,
        dst_right: f32,
        dst_bottom: f32,
        opacity: f32,
    ) {
        unsafe {
            let src_rect = D2D_RECT_F {
                left: src_left,
                top: src_top,
                right: src_right,
                bottom: src_bottom,
            };
            let dst_rect = D2D_RECT_F {
                left: dst_left,
                top: dst_top,
                right: dst_right,
                bottom: dst_bottom,
            };
            self.target.DrawBitmap(
                bitmap,
                Some(&dst_rect),
                opacity,
                windows::Win32::Graphics::Direct2D::D2D1_BITMAP_INTERPOLATION_MODE_LINEAR,
                Some(&src_rect),
            );
        }
    }

    pub fn draw_svg_logo(&self, l: f32, t: f32, r: f32, b: f32) {
        let w = (r - l).max(1.0);
        let h = (b - t).max(1.0);
        let sx = w / 1083.33;
        let sy = h / 1084.62;

        let pt = |x: f32, y: f32| -> D2D_POINT_2F {
            D2D_POINT_2F {
                x: l + x * sx,
                y: t + y * sy,
            }
        };

        // 1. Dark navy rounded squircle card background (#101a2c)
        let card_l = l + 55.0 * sx;
        let card_t = t + 55.0 * sy;
        let card_r = l + 1028.0 * sx;
        let card_b = t + 1029.0 * sy;
        let radius = 125.0 * sx;
        self.fill_rounded_rectangle(
            card_l,
            card_t,
            card_r,
            card_b,
            radius,
            (16.0 / 255.0, 26.0 / 255.0, 44.0 / 255.0, 1.0),
        );

        let blue_col = (23.0 / 255.0, 107.0 / 255.0, 255.0 / 255.0, 1.0);
        let stroke_w = (35.0 * sx).max(1.5);

        // 2. Outer blue border (cls-5)
        self.set_brush_color(blue_col.0, blue_col.1, blue_col.2, blue_col.3);
        unsafe {
            if let Ok(geom) = self.factory.CreatePathGeometry() {
                if let Ok(sink) = geom.Open() {
                    sink.BeginFigure(pt(1028.02, 344.94), D2D1_FIGURE_BEGIN_HOLLOW);
                    sink.AddBezier(&D2D1_BEZIER_SEGMENT {
                        point1: pt(1027.02, 588.94),
                        point2: pt(1027.36, 672.27),
                        point3: pt(1024.36, 916.27),
                    });
                    sink.AddBezier(&D2D1_BEZIER_SEGMENT {
                        point1: pt(1020.36, 937.27),
                        point2: pt(1012.36, 955.27),
                        point3: pt(1000.36, 973.27),
                    });
                    sink.AddBezier(&D2D1_BEZIER_SEGMENT {
                        point1: pt(994.36, 981.27),
                        point2: pt(988.36, 987.27),
                        point3: pt(981.36, 994.27),
                    });
                    sink.AddBezier(&D2D1_BEZIER_SEGMENT {
                        point1: pt(961.36, 1011.27),
                        point2: pt(937.36, 1022.27),
                        point3: pt(912.36, 1027.27),
                    });
                    sink.AddBezier(&D2D1_BEZIER_SEGMENT {
                        point1: pt(665.36, 1029.27),
                        point2: pt(419.36, 1029.27),
                        point3: pt(172.36, 1027.27),
                    });
                    sink.AddBezier(&D2D1_BEZIER_SEGMENT {
                        point1: pt(116.36, 1018.27),
                        point2: pt(69.36, 971.27),
                        point3: pt(60.36, 915.27),
                    });
                    sink.AddBezier(&D2D1_BEZIER_SEGMENT {
                        point1: pt(58.36, 670.27),
                        point2: pt(58.36, 424.27),
                        point3: pt(60.36, 179.27),
                    });
                    sink.AddBezier(&D2D1_BEZIER_SEGMENT {
                        point1: pt(61.36, 160.27),
                        point2: pt(69.36, 143.27),
                        point3: pt(78.36, 126.27),
                    });
                    sink.AddLine(pt(81.36, 123.27));
                    sink.AddBezier(&D2D1_BEZIER_SEGMENT {
                        point1: pt(91.36, 108.27),
                        point2: pt(102.36, 95.27),
                        point3: pt(117.36, 86.27),
                    });
                    sink.AddLine(pt(120.36, 83.27));
                    sink.AddBezier(&D2D1_BEZIER_SEGMENT {
                        point1: pt(139.36, 71.27),
                        point2: pt(159.36, 65.27),
                        point3: pt(180.36, 62.27),
                    });
                    sink.AddBezier(&D2D1_BEZIER_SEGMENT {
                        point1: pt(424.36, 63.27),
                        point2: pt(699.37, 58.90),
                        point3: pt(943.37, 61.90),
                    });
                    sink.EndFigure(D2D1_FIGURE_END_OPEN);
                    let _ = sink.Close();
                    self.target
                        .DrawGeometry(&geom, &self.brush, stroke_w, Some(&self.stroke_round));
                }
            }

            // 3. Drawing wave stroke (cls-4) exiting top-right
            if let Ok(geom) = self.factory.CreatePathGeometry() {
                if let Ok(sink) = geom.Open() {
                    sink.BeginFigure(pt(706.55, 374.59), D2D1_FIGURE_BEGIN_HOLLOW);
                    sink.AddBezier(&D2D1_BEZIER_SEGMENT {
                        point1: pt(730.57, 230.59),
                        point2: pt(897.67, 277.18),
                        point3: pt(897.67, 277.18),
                    });
                    sink.AddBezier(&D2D1_BEZIER_SEGMENT {
                        point1: pt(1004.41, 123.89),
                        point2: pt(1004.41, 123.89),
                        point3: pt(1004.41, 123.89),
                    });
                    sink.EndFigure(D2D1_FIGURE_END_OPEN);
                    let _ = sink.Close();
                    self.target
                        .DrawGeometry(&geom, &self.brush, stroke_w, Some(&self.stroke_round));
                }
            }

            // 4. Stylus / Pointer Body (polygon cls-2) - white
            self.set_brush_color(1.0, 1.0, 1.0, 1.0);
            if let Ok(geom) = self.factory.CreatePathGeometry() {
                if let Ok(sink) = geom.Open() {
                    sink.BeginFigure(pt(229.83, 607.02), D2D1_FIGURE_BEGIN_FILLED);
                    sink.AddLine(pt(656.89, 418.55));
                    sink.AddLine(pt(667.48, 433.22));
                    sink.AddLine(pt(487.48, 864.67));
                    sink.AddLine(pt(468.42, 844.90));
                    sink.AddLine(pt(438.38, 730.55));
                    sink.AddLine(pt(365.36, 788.91));
                    sink.AddLine(pt(301.12, 724.20));
                    sink.AddLine(pt(363.24, 652.90));
                    sink.EndFigure(D2D1_FIGURE_END_CLOSED);
                    let _ = sink.Close();
                    self.target.FillGeometry(&geom, &self.brush, None);
                }
            }

            // 5. Stylus Inner Detail (path cls-6) - light tinted
            self.set_brush_color(241.0 / 255.0, 247.0 / 255.0, 250.0 / 255.0, 1.0);
            if let Ok(geom) = self.factory.CreatePathGeometry() {
                if let Ok(sink) = geom.Open() {
                    sink.BeginFigure(pt(418.10, 749.05), D2D1_FIGURE_BEGIN_FILLED);
                    sink.AddLine(pt(370.91, 796.24));
                    sink.AddLine(pt(302.17, 727.50));
                    sink.AddLine(pt(349.35, 680.32));
                    sink.AddLine(pt(212.26, 643.97));
                    sink.AddLine(pt(691.66, 442.81));
                    sink.AddLine(pt(490.76, 922.38));
                    sink.AddLine(pt(454.39, 785.25));
                    sink.EndFigure(D2D1_FIGURE_END_CLOSED);

                    sink.BeginFigure(pt(251.59, 614.06), D2D1_FIGURE_BEGIN_FILLED);
                    sink.AddLine(pt(368.06, 644.94));
                    sink.AddLine(pt(311.59, 701.41));
                    sink.AddLine(pt(355.14, 744.96));
                    sink.AddLine(pt(411.61, 688.49));
                    sink.AddLine(pt(442.51, 805.00));
                    sink.AddLine(pt(606.68, 413.96));
                    sink.AddLine(pt(215.67, 578.16));
                    sink.EndFigure(D2D1_FIGURE_END_CLOSED);

                    let _ = sink.Close();
                    self.target.FillGeometry(&geom, &self.brush, None);
                }
            }

            // 6. Pencil Body / Accents (cls-1) - blue
            self.set_brush_color(blue_col.0, blue_col.1, blue_col.2, blue_col.3);
            if let Ok(geom) = self.factory.CreatePathGeometry() {
                if let Ok(sink) = geom.Open() {
                    // Left grip
                    sink.BeginFigure(pt(93.00, 849.45), D2D1_FIGURE_BEGIN_FILLED);
                    sink.AddLine(pt(92.81, 718.76));
                    sink.AddLine(pt(166.06, 646.27));
                    sink.AddLine(pt(270.44, 675.53));
                    sink.EndFigure(D2D1_FIGURE_END_CLOSED);

                    // Right grip
                    sink.BeginFigure(pt(228.81, 993.63), D2D1_FIGURE_BEGIN_FILLED);
                    sink.AddLine(pt(361.82, 993.41));
                    sink.AddLine(pt(437.37, 918.97));
                    sink.AddLine(pt(410.57, 812.87));
                    sink.EndFigure(D2D1_FIGURE_END_CLOSED);

                    // Pencil shaft (rotated rect 114.08 x 240.08 at 45 deg)
                    let cos45 = std::f32::consts::FRAC_1_SQRT_2;
                    let sin45 = cos45;
                    let rx1 = 160.98;
                    let ry1 = 748.41;
                    let rw = 114.08;
                    let rh = 240.08;
                    let rpts = [
                        (rx1, ry1),
                        (rx1 + rw, ry1),
                        (rx1 + rw, ry1 + rh),
                        (rx1, ry1 + rh),
                    ];
                    let rot_pt = |(px, py): (f32, f32)| -> D2D_POINT_2F {
                        let nx = px * cos45 - py * sin45 + 677.94;
                        let ny = px * sin45 + py * cos45 + 100.20;
                        pt(nx, ny)
                    };
                    sink.BeginFigure(rot_pt(rpts[0]), D2D1_FIGURE_BEGIN_FILLED);
                    sink.AddLine(rot_pt(rpts[1]));
                    sink.AddLine(rot_pt(rpts[2]));
                    sink.AddLine(rot_pt(rpts[3]));
                    sink.EndFigure(D2D1_FIGURE_END_CLOSED);

                    // Tip lead
                    sink.BeginFigure(pt(686.23, 410.58), D2D1_FIGURE_BEGIN_FILLED);
                    sink.AddLine(pt(635.80, 530.88));
                    sink.AddLine(pt(564.65, 459.91));
                    sink.AddLine(pt(684.79, 409.50));
                    sink.EndFigure(D2D1_FIGURE_END_CLOSED);

                    let _ = sink.Close();
                    self.target.FillGeometry(&geom, &self.brush, None);
                }
            }

            // 7. Pencil Tip Cone (path cls-2) - white
            self.set_brush_color(1.0, 1.0, 1.0, 1.0);
            if let Ok(geom) = self.factory.CreatePathGeometry() {
                if let Ok(sink) = geom.Open() {
                    sink.BeginFigure(pt(613.92, 565.50), D2D1_FIGURE_BEGIN_FILLED);
                    sink.AddLine(pt(520.87, 472.45));
                    sink.AddLine(pt(467.07, 494.95));
                    sink.AddLine(pt(602.32, 639.45));
                    sink.AddLine(pt(627.57, 579.28));
                    sink.EndFigure(D2D1_FIGURE_END_CLOSED);
                    let _ = sink.Close();
                    self.target.FillGeometry(&geom, &self.brush, None);
                }
            }
        }
    }
}
