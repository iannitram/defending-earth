use macroquad::prelude::*;
use crate::data::{Look, Gender};

pub const BG: Color = Color { r: 0.020, g: 0.030, b: 0.045, a: 1.0 };
pub const PANEL: Color = Color { r: 0.035, g: 0.060, b: 0.080, a: 0.97 };
pub const PANEL_2: Color = Color { r: 0.055, g: 0.095, b: 0.120, a: 0.96 };
pub const CYAN: Color = Color { r: 0.38, g: 0.86, b: 0.96, a: 1.0 };
pub const CYAN_DIM: Color = Color { r: 0.20, g: 0.46, b: 0.54, a: 1.0 };
pub const AMBER: Color = Color { r: 1.00, g: 0.68, b: 0.30, a: 1.0 };
pub const GREEN: Color = Color { r: 0.44, g: 0.95, b: 0.60, a: 1.0 };
pub const RED: Color = Color { r: 1.00, g: 0.36, b: 0.34, a: 1.0 };
pub const TEXT: Color = Color { r: 0.88, g: 0.94, b: 0.97, a: 1.0 };
pub const DIM: Color = Color { r: 0.56, g: 0.68, b: 0.73, a: 1.0 };
pub const WHITE: Color = Color { r: 1.0, g: 1.0, b: 1.0, a: 1.0 };

pub struct Ui { pub font: Font, pub bold: Font }

impl Ui {
    pub fn load() -> Self {
        let font = load_ttf_font_from_bytes(include_bytes!("../assets/ui.ttf")).unwrap_or_default();
        let bold = load_ttf_font_from_bytes(include_bytes!("../assets/ui_bold.ttf")).unwrap_or_default();
        Self { font, bold }
    }
    pub fn text(&self, x: f32, y: f32, size: f32, color: Color, s: &str) {
        draw_text_ex(s, x, y, TextParams { font: Some(&self.font), font_size: 32, font_scale: size / 32.0, color, ..Default::default() });
    }
    pub fn bold(&self, x: f32, y: f32, size: f32, color: Color, s: &str) {
        draw_text_ex(s, x, y, TextParams { font: Some(&self.bold), font_size: 32, font_scale: size / 32.0, color, ..Default::default() });
    }
    pub fn width(&self, size: f32, s: &str) -> f32 {
        measure_text(s, Some(&self.font), 32, size / 32.0).width
    }
    pub fn bwidth(&self, size: f32, s: &str) -> f32 {
        measure_text(s, Some(&self.bold), 32, size / 32.0).width
    }
    pub fn center(&self, x: f32, y: f32, size: f32, color: Color, s: &str) {
        let w = self.width(size, s);
        self.text(x - w * 0.5, y, size, color, s);
    }
    pub fn bcenter(&self, x: f32, y: f32, size: f32, color: Color, s: &str) {
        let w = self.bwidth(size, s);
        self.bold(x - w * 0.5, y, size, color, s);
    }
    pub fn label(&self, x: f32, y: f32, size: f32, color: Color, s: &str) {
        let up = s.to_uppercase();
        self.bold(x, y, size, color, &up);
    }
    /// Shrink or clip a single line so it never spills into the next column.
    pub fn fit(&self, size: f32, s: &str, max_w: f32, bold: bool) -> String {
        let measure = |t: &str| if bold { self.bwidth(size, t) } else { self.width(size, t) };
        if measure(s) <= max_w { return s.to_string(); }
        let mut out = String::new();
        for ch in s.chars() {
            let test = format!("{}{}", out, ch);
            if measure(&test) > max_w - measure("...") { break; }
            out.push(ch);
        }
        out.push_str("...");
        out
    }
}

pub fn panel(r: Rect, fill: Color, edge: Color) {
    draw_rectangle(r.x, r.y, r.w, r.h, fill);
    draw_rectangle_lines(r.x, r.y, r.w, r.h, 1.0, edge);
    // cut corners: small dark triangles and neon ticks
    let c = 7.0;
    draw_triangle(vec2(r.x, r.y), vec2(r.x + c, r.y), vec2(r.x, r.y + c), BG);
    draw_triangle(vec2(r.x + r.w, r.y), vec2(r.x + r.w - c, r.y), vec2(r.x + r.w, r.y + c), BG);
    draw_triangle(vec2(r.x, r.y + r.h), vec2(r.x + c, r.y + r.h), vec2(r.x, r.y + r.h - c), BG);
    draw_triangle(vec2(r.x + r.w, r.y + r.h), vec2(r.x + r.w - c, r.y + r.h), vec2(r.x + r.w, r.y + r.h - c), BG);
    draw_line(r.x + c, r.y, r.x + c + 18.0, r.y, 2.0, CYAN);
    draw_line(r.x, r.y + c, r.x, r.y + c + 14.0, 2.0, CYAN);
}

pub fn hline(x: f32, y: f32, w: f32, color: Color) {
    draw_rectangle(x, y, w, 1.0, color);
}

pub fn bar(r: Rect, value: f32, max: f32, color: Color, bg: Color) {
    let p = if max > 0.0 { (value / max).clamp(0.0, 1.0) } else { 0.0 };
    draw_rectangle(r.x, r.y, r.w, r.h, bg);
    draw_rectangle(r.x, r.y, r.w * p, r.h, color);
    draw_rectangle(r.x, r.y, r.w, 1.0, Color { a: 0.25, ..WHITE });
    draw_rectangle_lines(r.x, r.y, r.w, r.h, 1.0, Color { a: 0.35, ..color });
}

#[derive(Clone, Copy, PartialEq)]
pub enum Btn { Primary, Normal, Danger, Ghost, Amber }

pub fn button(ui: &Ui, r: Rect, label: &str, kind: Btn, enabled: bool) -> bool {
    let m = mouse_position();
    let hover = r.contains(vec2(m.0, m.1)) && enabled;
    let (fill, edge, txt) = match kind {
        Btn::Primary => (if hover { Color { r: 0.10, g: 0.32, b: 0.38, a: 1.0 } } else { Color { r: 0.05, g: 0.18, b: 0.22, a: 1.0 } }, CYAN, if enabled { CYAN } else { DIM }),
        Btn::Normal => (if hover { Color { r: 0.09, g: 0.15, b: 0.19, a: 1.0 } } else { Color { r: 0.04, g: 0.07, b: 0.09, a: 0.95 } }, if hover { CYAN } else { CYAN_DIM }, if enabled { TEXT } else { DIM }),
        Btn::Danger => (if hover { Color { r: 0.25, g: 0.07, b: 0.07, a: 1.0 } } else { Color { r: 0.12, g: 0.04, b: 0.04, a: 1.0 } }, RED, if enabled { RED } else { DIM }),
        Btn::Amber => (if hover { Color { r: 0.26, g: 0.19, b: 0.07, a: 1.0 } } else { Color { r: 0.12, g: 0.09, b: 0.04, a: 1.0 } }, AMBER, if enabled { AMBER } else { DIM }),
        Btn::Ghost => (Color { r: 0.02, g: 0.04, b: 0.05, a: 0.0 }, if hover { CYAN } else { Color { a: 0.0, ..CYAN_DIM } }, if enabled { if hover { CYAN } else { DIM } } else { Color { a: 0.4, ..DIM } }),
    };
    if hover {
        draw_rectangle(r.x - 2.0, r.y - 2.0, r.w + 4.0, r.h + 4.0, Color { a: 0.10, ..edge });
    }
    draw_rectangle(r.x, r.y, r.w, r.h, fill);
    draw_rectangle_lines(r.x, r.y, r.w, r.h, 1.0, edge);
    let size = (r.h * 0.38).clamp(10.0, 18.0);
    ui.bcenter(r.x + r.w * 0.5, r.y + r.h * 0.5 + size * 0.36, size, txt, label);
    hover && is_mouse_button_released(MouseButton::Left)
}

pub fn icon_button(ui: &Ui, r: Rect, glyph: &str, kind: Btn, enabled: bool) -> bool {
    button(ui, r, glyph, kind, enabled)
}

pub fn glow(cx: f32, cy: f32, radius: f32, color: Color, intensity: f32) {
    for i in 0..6 {
        let f = i as f32 / 6.0;
        let rr = radius * (1.0 - f * 0.82);
        draw_circle(cx, cy, rr, Color { a: intensity * (0.055 + f * 0.05), ..color });
    }
}

pub fn scanlines(alpha: f32) {
    let h = screen_height();
    let w = screen_width();
    let mut y = 0.0;
    while y < h {
        draw_rectangle(0.0, y, w, 1.0, Color { a: alpha, ..BLACK });
        y += 3.0;
    }
}

pub fn vignette() {
    let w = screen_width(); let h = screen_height();
    let steps = 28;
    for i in 0..steps {
        let f = i as f32 / steps as f32;
        let a = 0.018 * (1.0 - f);
        let sw = w * 0.16 * (1.0 - f);
        draw_rectangle(0.0, 0.0, sw, h, Color { a, ..BLACK });
        draw_rectangle(w - sw, 0.0, sw, h, Color { a, ..BLACK });
        let sh = h * 0.16 * (1.0 - f);
        draw_rectangle(0.0, 0.0, w, sh, Color { a, ..BLACK });
        draw_rectangle(0.0, h - sh, w, sh, Color { a, ..BLACK });
    }
}

pub fn grad_v(r: Rect, top: Color, bottom: Color) {
    let n = 24;
    for i in 0..n {
        let f = i as f32 / n as f32;
        let c = Color { r: top.r + (bottom.r - top.r) * f, g: top.g + (bottom.g - top.g) * f, b: top.b + (bottom.b - top.b) * f, a: top.a + (bottom.a - top.a) * f };
        draw_rectangle(r.x, r.y + r.h * f, r.w, r.h / n as f32 + 1.0, c);
    }
}

pub fn wrap(ui: &Ui, text: &str, size: f32, max_w: f32) -> Vec<String> {
    let mut out = Vec::new();
    let mut line = String::new();
    for word in text.split_whitespace() {
        let test = if line.is_empty() { word.to_string() } else { format!("{} {}", line, word) };
        if ui.width(size, &test) > max_w && !line.is_empty() {
            out.push(line.clone());
            line = word.to_string();
        } else {
            line = test;
        }
    }
    if !line.is_empty() { out.push(line); }
    out
}

const SKIN: [Color; 6] = [
    Color { r: 0.96, g: 0.82, b: 0.72, a: 1.0 },
    Color { r: 0.88, g: 0.70, b: 0.56, a: 1.0 },
    Color { r: 0.77, g: 0.57, b: 0.42, a: 1.0 },
    Color { r: 0.64, g: 0.45, b: 0.32, a: 1.0 },
    Color { r: 0.46, g: 0.31, b: 0.22, a: 1.0 },
    Color { r: 0.31, g: 0.20, b: 0.15, a: 1.0 },
];
const HAIR: [Color; 8] = [
    Color { r: 0.07, g: 0.06, b: 0.06, a: 1.0 },
    Color { r: 0.16, g: 0.11, b: 0.07, a: 1.0 },
    Color { r: 0.29, g: 0.19, b: 0.10, a: 1.0 },
    Color { r: 0.44, g: 0.27, b: 0.12, a: 1.0 },
    Color { r: 0.62, g: 0.46, b: 0.24, a: 1.0 },
    Color { r: 0.30, g: 0.30, b: 0.30, a: 1.0 },
    Color { r: 0.78, g: 0.78, b: 0.76, a: 1.0 },
    Color { r: 0.55, g: 0.16, b: 0.09, a: 1.0 },
];
const EYES: [Color; 5] = [
    Color { r: 0.28, g: 0.17, b: 0.09, a: 1.0 },
    Color { r: 0.40, g: 0.31, b: 0.13, a: 1.0 },
    Color { r: 0.22, g: 0.38, b: 0.22, a: 1.0 },
    Color { r: 0.20, g: 0.38, b: 0.55, a: 1.0 },
    Color { r: 0.40, g: 0.45, b: 0.47, a: 1.0 },
];

fn shade(c: Color, f: f32) -> Color { Color { r: (c.r * f).clamp(0.0, 1.0), g: (c.g * f).clamp(0.0, 1.0), b: (c.b * f).clamp(0.0, 1.0), a: c.a } }
fn tint(c: Color, t: Color, f: f32) -> Color {
    Color { r: c.r + (t.r - c.r) * f, g: c.g + (t.g - c.g) * f, b: c.b + (t.b - c.b) * f, a: c.a + (t.a - c.a) * f }
}

/// A procedural, lit portrait painted from the soldier's own parameters.
pub fn portrait(ui: &Ui, r: Rect, look: Look, time: f32, bg: bool) {
    // Keep the shoulders inside the frame: the torso is drawn at +0.72s.
    let s = (r.h * 0.60).min(r.w * 0.92);
    let cx = r.x + r.w * 0.5;
    let cy = r.y + r.h * 0.48;
    #[allow(non_snake_case)]
    let X = |v: f32| cx + v * s;
    #[allow(non_snake_case)]
    let Y = |v: f32| cy + v * s;
    if bg {
        grad_v(r, Color { r: 0.020, g: 0.055, b: 0.070, a: 1.0 }, Color { r: 0.010, g: 0.018, b: 0.026, a: 1.0 });
        // rings and ID marks
        for i in 0..3 {
            let rr = s * (0.34 + i as f32 * 0.08);
            draw_circle_lines(cx, cy - s * 0.02, rr, 1.0, Color { a: 0.10 - i as f32 * 0.02, ..CYAN });
        }
        draw_line(r.x + 12.0, r.y + r.h - 18.0, r.x + r.w - 12.0, r.y + r.h - 18.0, 1.0, Color { a: 0.2, ..CYAN });
        ui.label(r.x + 10.0, r.y + 18.0, 7.5, Color { a: 0.65, ..CYAN }, "ID FILE");
        let code = format!("{:02X}{:02X}-{:02X}", look.face * 31 + look.skin, look.nose * 17 + look.jaw, look.hair * 13 + look.brow);
        ui.text(r.x + r.w - 58.0, r.y + 18.0, 7.5, Color { a: 0.45, ..CYAN }, &code);
        // scan glow
        let sy = r.y + ((time * 0.10) % 1.0) * r.h;
        draw_rectangle(r.x, sy, r.w, 2.0, Color { a: 0.035, ..CYAN });
    }
    let skin = SKIN[(look.skin as usize).min(5)];
    let skin_d = shade(skin, 0.72);
    let skin_l = tint(skin, WHITE, 0.18);
    let hair_c = HAIR[(look.hair_color as usize).min(7)];
    let fem = look.gender == Gender::Female;
    let face_w = 0.255 + look.face as f32 * 0.016;
    let jaw_w = 0.205 + look.jaw as f32 * 0.020 - if fem { 0.012 } else { 0.0 };
    let chin = 0.305 + look.jaw as f32 * 0.012;

    // torso
    let collar = if fem { 0.34 } else { 0.37 };
    draw_triangle(vec2(X(-0.49), Y(0.72)), vec2(X(-0.30), Y(collar)), vec2(X(-0.12), Y(0.38)), Color { r: 0.09, g: 0.13, b: 0.12, a: 1.0 });
    draw_triangle(vec2(X(0.49), Y(0.72)), vec2(X(0.30), Y(collar)), vec2(X(0.12), Y(0.38)), Color { r: 0.09, g: 0.13, b: 0.12, a: 1.0 });
    draw_rectangle(X(-0.34), Y(collar), s * 0.68, s * 0.36, Color { r: 0.10, g: 0.14, b: 0.13, a: 1.0 });
    draw_triangle(vec2(X(-0.31), Y(collar)), vec2(X(0.31), Y(collar)), vec2(X(0.0), Y(0.58)), Color { r: 0.15, g: 0.20, b: 0.18, a: 1.0 });
    draw_line(X(-0.12), Y(collar + 0.01), X(0.0), Y(0.57), 1.5, Color { a: 0.5, ..CYAN });
    draw_line(X(0.12), Y(collar + 0.01), X(0.0), Y(0.57), 1.5, Color { a: 0.5, ..CYAN });
    // rank patch / insignia (abstract)
    draw_rectangle(X(-0.31), Y(0.51), s * 0.055, s * 0.035, Color { a: 0.7, ..AMBER });
    draw_rectangle(X(-0.31), Y(0.56), s * 0.055, s * 0.012, Color { a: 0.35, ..AMBER });

    // neck
    draw_rectangle(X(-0.105), Y(0.14), s * 0.21, s * 0.22, shade(skin, 0.82));
    draw_ellipse(X(0.0), Y(0.17), s * 0.11, s * 0.05, 0.0, Color { a: 0.18, ..BLACK });

    // long hair behind head
    if look.hair == 2 || look.hair == 4 {
        draw_ellipse(X(0.0), Y(0.08), s * (face_w + 0.06), s * 0.42, 0.0, hair_c);
        draw_rectangle(X(-(face_w + 0.04)), Y(0.02), s * (face_w + 0.04) * 2.0, s * 0.42, hair_c);
    }

    // ears
    draw_ellipse(X(-face_w - 0.012), Y(0.02), s * 0.045, s * 0.075, -0.1, skin_d);
    draw_ellipse(X(face_w + 0.012), Y(0.02), s * 0.045, s * 0.075, 0.1, skin_d);

    // head base
    draw_ellipse(X(0.0), Y(-0.035), s * face_w, s * 0.34, 0.0, skin);
    // jaw / chin
    draw_ellipse(X(0.0), Y(0.155), s * jaw_w, s * (chin - 0.14), 0.0, skin);
    draw_ellipse(X(0.0), Y(0.225), s * (jaw_w * 0.65), s * 0.075, 0.0, skin);
    // cheek shading
    draw_ellipse(X(-face_w * 0.62), Y(0.075), s * 0.09, s * 0.17, -0.2, Color { a: 0.10, ..skin_d });
    draw_ellipse(X(face_w * 0.66), Y(0.075), s * 0.10, s * 0.18, 0.2, Color { a: 0.16, ..skin_d });
    // forehead highlight
    draw_ellipse(X(-face_w * 0.18), Y(-0.19), s * 0.16, s * 0.14, -0.2, Color { a: 0.18, ..skin_l });
    // under-eye soft shadow
    draw_ellipse(X(-0.11), Y(0.005), s * 0.075, s * 0.03, 0.0, Color { a: 0.12, ..skin_d });
    draw_ellipse(X(0.11), Y(0.005), s * 0.075, s * 0.03, 0.0, Color { a: 0.12, ..skin_d });

    // hair top
    let cap_y = -0.15;
    match look.hair {
        0 => {
            draw_ellipse(X(0.0), Y(cap_y - 0.10), s * (face_w + 0.012), s * 0.20, 0.0, shade(hair_c, 0.8));
            draw_rectangle(X(-face_w), Y(cap_y - 0.18), s * face_w * 2.0, s * 0.16, Color { a: 0.30, ..hair_c });
        }
        1 => {
            draw_ellipse(X(-0.02), Y(cap_y - 0.11), s * (face_w + 0.03), s * 0.23, -0.06, hair_c);
            draw_triangle(vec2(X(-face_w - 0.02), Y(cap_y - 0.02)), vec2(X(face_w + 0.02), Y(cap_y - 0.10)), vec2(X(face_w * 0.2), Y(cap_y + 0.05)), hair_c);
        }
        2 => {
            draw_ellipse(X(0.0), Y(cap_y - 0.08), s * (face_w + 0.045), s * 0.24, 0.0, hair_c);
            draw_rectangle(X(-face_w - 0.05), Y(-0.12), s * 0.10, s * 0.36, hair_c);
            draw_rectangle(X(face_w - 0.05), Y(-0.12), s * 0.10, s * 0.36, hair_c);
        }
        3 => {
            draw_ellipse(X(0.0), Y(cap_y - 0.10), s * (face_w + 0.02), s * 0.22, 0.0, hair_c);
            draw_circle(X(0.02), Y(cap_y - 0.28), s * 0.10, hair_c);
            draw_circle(X(0.02), Y(cap_y - 0.28), s * 0.065, shade(hair_c, 0.85));
        }
        4 => {
            draw_ellipse(X(0.0), Y(cap_y - 0.10), s * (face_w + 0.03), s * 0.23, 0.0, hair_c);
            for i in 0..5 {
                let yy = 0.02 + i as f32 * 0.075;
                draw_circle(X(-face_w - 0.05), Y(yy), s * 0.045, hair_c);
                draw_circle(X(face_w + 0.05), Y(yy), s * 0.045, hair_c);
            }
        }
        _ => {
            for i in 0..9 {
                let a = i as f32 * 0.7;
                let xx = (a.cos() * 0.16) + if i % 2 == 0 { -0.05 } else { 0.05 };
                let yy = cap_y - 0.10 + a.sin() * 0.06;
                draw_circle(X(xx), Y(yy), s * 0.095, hair_c);
            }
        }
    }
    // hairline shadow
    draw_ellipse(X(0.0), Y(cap_y - 0.02), s * face_w * 0.95, s * 0.05, 0.0, Color { a: 0.16, ..BLACK });

    // brows
    let brow_y = -0.085 + look.brow as f32 * 0.012;
    let brow_t = if fem { 0.012 } else { 0.018 };
    draw_line(X(-0.185), Y(brow_y + 0.008), X(-0.055), Y(brow_y - 0.006), s * brow_t, shade(hair_c, 0.65));
    draw_line(X(0.055), Y(brow_y - 0.006), X(0.185), Y(brow_y + 0.008), s * brow_t, shade(hair_c, 0.65));

    // eyes
    let ey = -0.025 + look.eyes as f32 * 0.006;
    for side in [-1.0f32, 1.0] {
        let ex = side * 0.108;
        draw_ellipse(X(ex), Y(ey), s * 0.060, s * 0.031, 0.0, Color { r: 0.93, g: 0.92, b: 0.88, a: 1.0 });
        let iris = EYES[(look.eye_color as usize).min(4)];
        draw_circle(X(ex), Y(ey), s * 0.026, iris);
        draw_circle(X(ex), Y(ey), s * 0.012, Color { r: 0.03, g: 0.03, b: 0.03, a: 1.0 });
        draw_circle(X(ex - 0.008), Y(ey - 0.009), s * 0.006, Color { a: 0.9, ..WHITE });
        draw_line(X(ex - 0.060), Y(ey - 0.034), X(ex + 0.060), Y(ey - 0.030), s * 0.011, skin_d);
        if fem {
            draw_line(X(ex + side * 0.058), Y(ey - 0.030), X(ex + side * 0.075), Y(ey - 0.046), 1.2, shade(hair_c, 0.7));
        }
    }

    // nose
    let nx = look.nose as f32 * 0.010;
    draw_triangle(vec2(X(-0.035 - nx * 0.3), Y(0.055)), vec2(X(0.035 + nx * 0.3), Y(0.055)), vec2(X(0.0), Y(0.115)), Color { a: 0.13, ..skin_d });
    draw_line(X(0.012), Y(0.02), X(0.030 + nx), Y(0.095), 1.2, Color { a: 0.30, ..skin_l });
    draw_ellipse(X(0.0), Y(0.105), s * (0.035 + nx), s * 0.016, 0.0, Color { a: 0.28, ..skin_d });
    draw_circle(X(-0.034 - nx), Y(0.105), s * 0.012, Color { a: 0.35, ..skin_d });
    draw_circle(X(0.034 + nx), Y(0.105), s * 0.012, Color { a: 0.35, ..skin_d });

    // mouth
    let my = 0.195 + look.jaw as f32 * 0.006;
    let lip = tint(skin, Color { r: 0.55, g: 0.20, b: 0.18, a: 1.0 }, if fem { 0.35 } else { 0.18 });
    draw_ellipse(X(0.0), Y(my), s * (0.070 + if fem { 0.008 } else { 0.0 }), s * 0.020, 0.0, lip);
    draw_ellipse(X(0.0), Y(my + 0.012), s * 0.060, s * 0.014, 0.0, shade(lip, 0.85));
    draw_line(X(-0.062), Y(my + 0.001), X(0.062), Y(my + 0.001), 1.1, Color { a: 0.55, ..shade(lip, 0.55) });
    draw_ellipse(X(0.0), Y(my + 0.030), s * 0.045, s * 0.010, 0.0, Color { a: 0.16, ..skin_d });

    // facial hair
    if look.facial_hair > 0 && !fem {
        let beard = shade(hair_c, 0.9);
        if look.facial_hair == 2 {
            draw_ellipse(X(0.0), Y(0.18), s * (jaw_w + 0.02), s * 0.16, 0.0, Color { a: 0.55, ..beard });
            draw_rectangle(X(-jaw_w), Y(0.10), s * jaw_w * 2.0, s * 0.18, Color { a: 0.25, ..beard });
            draw_ellipse(X(0.0), Y(my), s * 0.095, s * 0.040, 0.0, Color { a: 0.80, ..beard });
        } else if look.facial_hair == 1 {
            draw_ellipse(X(0.0), Y(0.20), s * jaw_w, s * 0.13, 0.0, Color { a: 0.20, ..beard });
            draw_ellipse(X(0.0), Y(my), s * 0.085, s * 0.028, 0.0, Color { a: 0.35, ..beard });
        } else {
            draw_ellipse(X(0.0), Y(my - 0.018), s * 0.085, s * 0.022, 0.0, Color { a: 0.65, ..beard });
        }
    }

    // key light + cool rim
    draw_ellipse(X(-face_w * 0.42), Y(-0.13), s * 0.13, s * 0.19, -0.25, Color { a: 0.055, ..WHITE });
    draw_line(X(face_w * 0.90), Y(-0.22), X(face_w * 0.82), Y(0.20), 2.2, Color { a: 0.22, ..CYAN });
    draw_line(X(face_w * 0.86), Y(-0.20), X(face_w * 0.78), Y(0.17), 0.8, Color { a: 0.55, ..CYAN });
    // subtle grain
    let mut seed = (look.face as u32 * 9176 + look.skin as u32 * 313 + look.nose as u32 * 71) | 1;
    for _ in 0..70 {
        seed = seed.wrapping_mul(1664525).wrapping_add(1013904223);
        let gx = (seed >> 8 & 0xffff) as f32 / 65535.0;
        seed = seed.wrapping_mul(1664525).wrapping_add(1013904223);
        let gy = (seed >> 8 & 0xffff) as f32 / 65535.0;
        let px = X(-face_w + gx * face_w * 2.0);
        let py = Y(-0.30 + gy * 0.52);
        draw_circle(px, py, 0.55, Color { a: 0.035, ..WHITE });
    }
}
