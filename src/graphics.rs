//! Pure-Rust SVG generation for 2D and 3D plots.
//!
//! No font or raster dependencies: text is emitted as SVG `<text>` and rendered
//! by the viewer, which keeps the runtime image trivially distroless.

use base64::Engine;
use rmcp::model::{CallToolResult, Content};

pub const WIDTH: f64 = 720.0;
pub const HEIGHT: f64 = 540.0;
pub const MARGIN: f64 = 56.0;

/// Incremental SVG document builder.
pub struct Svg {
    body: String,
    width: f64,
    height: f64,
}

impl Svg {
    pub fn new(width: f64, height: f64) -> Self {
        Self {
            body: String::new(),
            width,
            height,
        }
    }

    pub fn rect(&mut self, x: f64, y: f64, w: f64, h: f64, fill: &str) {
        self.body.push_str(&format!(
            r#"<rect x="{x:.2}" y="{y:.2}" width="{w:.2}" height="{h:.2}" fill="{fill}"/>"#
        ));
    }

    pub fn line(&mut self, x1: f64, y1: f64, x2: f64, y2: f64, stroke: &str, w: f64) {
        self.body.push_str(&format!(
            r#"<line x1="{x1:.2}" y1="{y1:.2}" x2="{x2:.2}" y2="{y2:.2}" stroke="{stroke}" stroke-width="{w}"/>"#
        ));
    }

    pub fn circle(&mut self, cx: f64, cy: f64, r: f64, fill: &str) {
        self.body.push_str(&format!(
            r#"<circle cx="{cx:.2}" cy="{cy:.2}" r="{r:.2}" fill="{fill}"/>"#
        ));
    }

    pub fn polyline(&mut self, pts: &[(f64, f64)], stroke: &str, w: f64) {
        if pts.is_empty() {
            return;
        }
        self.body.push_str(&format!(
            r#"<polyline points="{}" fill="none" stroke="{stroke}" stroke-width="{w}" stroke-linejoin="round"/>"#,
            points_attr(pts)
        ));
    }

    pub fn polygon(&mut self, pts: &[(f64, f64)], fill: &str, stroke: &str, w: f64) {
        if pts.is_empty() {
            return;
        }
        self.body.push_str(&format!(
            r#"<polygon points="{}" fill="{fill}" stroke="{stroke}" stroke-width="{w}"/>"#,
            points_attr(pts)
        ));
    }

    /// Anchored text. `anchor` is one of "start", "middle", "end".
    pub fn text(&mut self, x: f64, y: f64, s: &str, size: f64, anchor: &str, fill: &str) {
        self.body.push_str(&format!(
            r#"<text x="{x:.2}" y="{y:.2}" font-family="system-ui,sans-serif" font-size="{size}" text-anchor="{anchor}" fill="{fill}">{}</text>"#,
            escape(s)
        ));
    }

    /// Draw a legend box of `(label, color)` swatches anchored at top-left `(x, y)`.
    pub fn legend(&mut self, items: &[(String, String)], x: f64, y: f64) {
        if items.is_empty() {
            return;
        }
        let w = legend_width(items);
        let h = items.len() as f64 * 18.0 + 10.0;
        // Panel: white fill + faint border.
        self.body.push_str(&format!(
            r##"<rect x="{x:.2}" y="{y:.2}" width="{w:.2}" height="{h:.2}" rx="4" fill="#ffffff" fill-opacity="0.88" stroke="#c8c8d6" stroke-width="1"/>"##
        ));
        for (i, (name, color)) in items.iter().enumerate() {
            let ly = y + 9.0 + i as f64 * 18.0;
            self.rect(x + 8.0, ly, 12.0, 12.0, color);
            self.text(x + 26.0, ly + 10.0, name, 12.0, "start", "#222233");
        }
    }

    pub fn finish(self, title: Option<&str>) -> String {
        let title_el = title
            .map(|t| {
                format!(
                    r##"<text x="{:.2}" y="28" font-family="system-ui,sans-serif" font-size="18" font-weight="600" text-anchor="middle" fill="#1a1a2e">{}</text>"##,
                    self.width / 2.0,
                    escape(t)
                )
            })
            .unwrap_or_default();
        format!(
            r##"<svg xmlns="http://www.w3.org/2000/svg" width="{w:.0}" height="{h:.0}" viewBox="0 0 {w:.0} {h:.0}"><rect width="{w:.0}" height="{h:.0}" fill="#fafafe"/>{title}{body}</svg>"##,
            w = self.width,
            h = self.height,
            title = title_el,
            body = self.body,
        )
    }
}

fn escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

/// Format a point list as a space-separated `x,y` string for `points="…"`.
fn points_attr(pts: &[(f64, f64)]) -> String {
    let mut s = String::with_capacity(pts.len() * 14);
    for (x, y) in pts {
        s.push_str(&format!("{x:.2},{y:.2} "));
    }
    s.truncate(s.trim_end().len());
    s
}

/// Width of the legend box for `items` — the single source of truth shared by
/// `Svg::legend` and callers that right-align it.
pub fn legend_width(items: &[(String, String)]) -> f64 {
    let longest = items
        .iter()
        .map(|(n, _)| n.chars().count())
        .max()
        .unwrap_or(0);
    36.0 + longest as f64 * 7.2
}

/// Distinct categorical colors (Tableau-10), cycled by index. For superposing
/// multiple series, each gets `palette(i)`.
pub fn palette(i: usize) -> &'static str {
    const COLORS: [&str; 10] = [
        "#4e79a7", "#f28e2b", "#e15759", "#76b7b2", "#59a14f", "#edc948", "#b07aa1", "#ff9da7",
        "#9c755f", "#bab0ac",
    ];
    COLORS[i % COLORS.len()]
}

/// Map a value from `[lo, hi]` into `[a, b]`. Degenerate ranges map to the midpoint.
pub fn scale(v: f64, lo: f64, hi: f64, a: f64, b: f64) -> f64 {
    if (hi - lo).abs() < f64::EPSILON {
        (a + b) / 2.0
    } else {
        a + (v - lo) / (hi - lo) * (b - a)
    }
}

/// Finite min/max over an iterator, or `None` if it yields no finite values.
pub fn bounds(vals: impl Iterator<Item = f64>) -> Option<(f64, f64)> {
    let mut lo = f64::INFINITY;
    let mut hi = f64::NEG_INFINITY;
    let mut any = false;
    for v in vals {
        if v.is_finite() {
            lo = lo.min(v);
            hi = hi.max(v);
            any = true;
        }
    }
    if any { Some((lo, hi)) } else { None }
}

/// Pad a `[lo, hi]` range by `frac` on each side so points don't sit on the frame.
pub fn pad_range(lo: f64, hi: f64, frac: f64) -> (f64, f64) {
    let span = (hi - lo).abs().max(1e-9);
    (lo - span * frac, hi + span * frac)
}

/// [`bounds`] padded by `frac` on each side — one call instead of two.
pub fn padded_bounds(vals: impl Iterator<Item = f64>, frac: f64) -> Option<(f64, f64)> {
    bounds(vals).map(|(lo, hi)| pad_range(lo, hi, frac))
}

/// Viridis-like color ramp for `t` in `[0, 1]`, returned as `#rrggbb`.
pub fn heat(t: f64) -> String {
    let t = t.clamp(0.0, 1.0);
    // Five-stop gradient: deep blue → teal → green → yellow → red.
    const STOPS: [(f64, f64, f64); 5] = [
        (68.0, 1.0, 84.0),
        (59.0, 82.0, 139.0),
        (33.0, 145.0, 140.0),
        (94.0, 201.0, 98.0),
        (253.0, 231.0, 37.0),
    ];
    let seg = t * 4.0;
    let i = (seg.floor() as usize).min(3);
    let f = seg - i as f64;
    let (r1, g1, b1) = STOPS[i];
    let (r2, g2, b2) = STOPS[i + 1];
    let r = (r1 + (r2 - r1) * f) as u8;
    let g = (g1 + (g2 - g1) * f) as u8;
    let b = (b1 + (b2 - b1) * f) as u8;
    format!("#{r:02x}{g:02x}{b:02x}")
}

/// Orthographic 3D→2D projection with yaw (around vertical) and pitch (tilt).
///
/// Inputs are expected pre-normalized to roughly `[-1, 1]`. Returns
/// `(screen_x, screen_y, depth)`; larger `depth` is farther from the camera, so
/// sort ascending and draw far-to-near for a painter's-algorithm fill.
pub struct Camera {
    sin_yaw: f64,
    cos_yaw: f64,
    sin_pitch: f64,
    cos_pitch: f64,
    cx: f64,
    cy: f64,
    s: f64,
}

impl Camera {
    pub fn new(yaw_deg: f64, pitch_deg: f64) -> Self {
        let yaw = yaw_deg.to_radians();
        let pitch = pitch_deg.to_radians();
        Self {
            sin_yaw: yaw.sin(),
            cos_yaw: yaw.cos(),
            sin_pitch: pitch.sin(),
            cos_pitch: pitch.cos(),
            cx: WIDTH / 2.0,
            cy: HEIGHT / 2.0 + 16.0,
            s: (WIDTH.min(HEIGHT) - 2.0 * MARGIN) / 2.6,
        }
    }

    /// Project a normalized `(x, y, z)` point. `z` is the vertical (up) axis.
    pub fn project(&self, x: f64, y: f64, z: f64) -> (f64, f64, f64) {
        // Yaw about the vertical (z) axis.
        let xr = x * self.cos_yaw - y * self.sin_yaw;
        let yr = x * self.sin_yaw + y * self.cos_yaw;
        // Pitch tilts the yawed ground plane toward the camera.
        let depth = yr * self.cos_pitch - z * self.sin_pitch;
        let up = yr * self.sin_pitch + z * self.cos_pitch;
        let sx = self.cx + xr * self.s;
        let sy = self.cy - up * self.s;
        (sx, sy, depth)
    }
}

/// Wrap an SVG string into a `CallToolResult`: an `image/svg+xml` content item
/// (base64) for visual clients plus a short text caption.
pub fn svg_result(svg: String, caption: impl Into<String>) -> CallToolResult {
    let b64 = base64::engine::general_purpose::STANDARD.encode(svg.as_bytes());
    CallToolResult::success(vec![
        Content::image(b64, "image/svg+xml"),
        Content::text(caption.into()),
    ])
}

/// Convenience error result.
pub fn err_result(msg: impl Into<String>) -> CallToolResult {
    CallToolResult::error(vec![Content::text(msg.into())])
}
