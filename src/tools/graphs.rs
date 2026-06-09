//! 2D and 3D plotting tools, rendered to SVG. Geographic-oriented: most tools
//! take latitude/longitude (and optionally elevation). Strictly 2D and 3D —
//! there is intentionally no 4D plotting here.

use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::CallToolResult;
use rmcp::{tool, tool_router};
use schemars::JsonSchema;
use serde::Deserialize;

use crate::graphics::{
    Camera, HEIGHT, Svg, WIDTH, bounds, err_result, heat, legend_width, pad_range, padded_bounds,
    palette, scale, svg_result,
};
use crate::server::MathServer;

// ── Parameter types ───────────────────────────────────────────────────────────

#[derive(Deserialize, JsonSchema)]
pub struct GeoPoint {
    /// Latitude in degrees (−90..90)
    pub lat: f64,
    /// Longitude in degrees (−180..180)
    pub lon: f64,
    /// Optional label drawn next to the marker
    #[serde(default)]
    pub label: Option<String>,
}

#[derive(Deserialize, JsonSchema)]
pub struct GeoScatterParams {
    pub points: Vec<GeoPoint>,
    #[serde(default)]
    pub title: Option<String>,
}

#[derive(Deserialize, JsonSchema)]
pub struct LatLon {
    pub lat: f64,
    pub lon: f64,
}

#[derive(Deserialize, JsonSchema)]
pub struct GeoRouteParams {
    /// Waypoints connected in order
    pub points: Vec<LatLon>,
    #[serde(default)]
    pub title: Option<String>,
}

#[derive(Deserialize, JsonSchema)]
pub struct Point2 {
    pub x: f64,
    pub y: f64,
}

#[derive(Deserialize, JsonSchema)]
pub struct Scatter2dParams {
    pub points: Vec<Point2>,
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub x_label: Option<String>,
    #[serde(default)]
    pub y_label: Option<String>,
}

#[derive(Deserialize, JsonSchema)]
pub struct Line2dParams {
    /// Points of the series, plotted in the given order
    pub series: Vec<Point2>,
    #[serde(default)]
    pub title: Option<String>,
}

#[derive(Deserialize, JsonSchema)]
pub struct Bar {
    pub label: String,
    pub value: f64,
}

#[derive(Deserialize, JsonSchema)]
pub struct Bar2dParams {
    pub bars: Vec<Bar>,
    #[serde(default)]
    pub title: Option<String>,
}

#[derive(Deserialize, JsonSchema)]
pub struct Point3 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

#[derive(Deserialize, JsonSchema)]
pub struct Scatter3dParams {
    pub points: Vec<Point3>,
    /// Camera rotation about the vertical axis, in degrees (default 35)
    #[serde(default)]
    pub yaw: Option<f64>,
    /// Camera tilt in degrees (default 25)
    #[serde(default)]
    pub pitch: Option<f64>,
    #[serde(default)]
    pub title: Option<String>,
}

#[derive(Deserialize, JsonSchema)]
pub struct Surface3dParams {
    /// Row-major grid of heights z[row][col]
    pub z: Vec<Vec<f64>>,
    #[serde(default)]
    pub yaw: Option<f64>,
    #[serde(default)]
    pub pitch: Option<f64>,
    #[serde(default)]
    pub title: Option<String>,
}

#[derive(Deserialize, JsonSchema)]
pub struct GeoElev {
    pub lat: f64,
    pub lon: f64,
    pub elevation: f64,
}

#[derive(Deserialize, JsonSchema)]
pub struct GeoTerrain3dParams {
    pub points: Vec<GeoElev>,
    #[serde(default)]
    pub yaw: Option<f64>,
    #[serde(default)]
    pub pitch: Option<f64>,
    #[serde(default)]
    pub title: Option<String>,
}

// ── Overlay (superposition) parameter types ──────────────────────────────────

#[derive(Deserialize, JsonSchema)]
pub struct Series2d {
    pub name: String,
    /// "line" (default), "scatter", or "both"
    #[serde(default)]
    pub kind: Option<String>,
    pub points: Vec<Point2>,
}

#[derive(Deserialize, JsonSchema)]
pub struct Overlay2dParams {
    /// Two or more series drawn on shared, auto-scaled axes with a legend
    pub series: Vec<Series2d>,
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub x_label: Option<String>,
    #[serde(default)]
    pub y_label: Option<String>,
}

#[derive(Deserialize, JsonSchema)]
pub struct GeoLayer {
    pub name: String,
    /// "points" (default) or "route"
    #[serde(default)]
    pub kind: Option<String>,
    pub points: Vec<LatLon>,
}

#[derive(Deserialize, JsonSchema)]
pub struct GeoOverlay2dParams {
    /// Two or more geographic layers drawn on one shared map with a legend
    pub layers: Vec<GeoLayer>,
    #[serde(default)]
    pub title: Option<String>,
}

#[derive(Deserialize, JsonSchema)]
pub struct Series3d {
    pub name: String,
    /// "scatter" or "surface"; inferred from which data field is present if omitted
    #[serde(default)]
    pub kind: Option<String>,
    /// Points for a scatter series
    #[serde(default)]
    pub points: Vec<Point3>,
    /// Row-major height grid for a surface series
    #[serde(default)]
    pub z: Vec<Vec<f64>>,
    /// Surface placement on the x axis as [min, max] (defaults to column indices)
    #[serde(default)]
    pub x_range: Option<[f64; 2]>,
    /// Surface placement on the y axis as [min, max] (defaults to row indices)
    #[serde(default)]
    pub y_range: Option<[f64; 2]>,
}

#[derive(Deserialize, JsonSchema)]
pub struct Overlay3dParams {
    /// Two or more 3D series sharing one camera and coordinate space, with a legend
    pub series: Vec<Series3d>,
    #[serde(default)]
    pub yaw: Option<f64>,
    #[serde(default)]
    pub pitch: Option<f64>,
    #[serde(default)]
    pub title: Option<String>,
}

// ── 2D framing helpers ──────────────────────────────────────────────────────────

const L: f64 = 60.0;
const R: f64 = WIDTH - 24.0;
const T: f64 = 44.0;
const B: f64 = HEIGHT - 52.0;

fn fmt_tick(v: f64) -> String {
    if v == 0.0 {
        "0".into()
    } else if v.abs() >= 1000.0 || v.abs() < 0.01 {
        format!("{v:.2e}")
    } else {
        let s = format!("{v:.3}");
        s.trim_end_matches('0').trim_end_matches('.').to_string()
    }
}

/// Draw the plot frame, gridlines, and numeric ticks for a Cartesian 2D area.
fn draw_axes(svg: &mut Svg, xr: (f64, f64), yr: (f64, f64), xlabel: &str, ylabel: &str) {
    svg.rect(L, T, R - L, B - T, "#ffffff");
    for i in 0..=5 {
        let f = i as f64 / 5.0;
        let gx = L + f * (R - L);
        let gy = B - f * (B - T);
        svg.line(gx, T, gx, B, "#ececf3", 1.0);
        svg.line(L, gy, R, gy, "#ececf3", 1.0);
        let xv = xr.0 + f * (xr.1 - xr.0);
        let yv = yr.0 + f * (yr.1 - yr.0);
        svg.text(gx, B + 16.0, &fmt_tick(xv), 11.0, "middle", "#6b6b80");
        svg.text(L - 6.0, gy + 4.0, &fmt_tick(yv), 11.0, "end", "#6b6b80");
    }
    svg.line(L, T, L, B, "#9a9ab0", 1.5);
    svg.line(L, B, R, B, "#9a9ab0", 1.5);
    if !xlabel.is_empty() {
        svg.text(
            (L + R) / 2.0,
            HEIGHT - 14.0,
            xlabel,
            13.0,
            "middle",
            "#3a3a50",
        );
    }
    if !ylabel.is_empty() {
        // Rotated y-axis label.
        svg.text(16.0, (T + B) / 2.0, ylabel, 13.0, "middle", "#3a3a50");
    }
}

/// Draw a faint reference cube `[-1,1]^3` to orient a 3D scene.
fn draw_cube(svg: &mut Svg, cam: &Camera) {
    let corners = [
        (-1.0, -1.0, -1.0),
        (1.0, -1.0, -1.0),
        (1.0, 1.0, -1.0),
        (-1.0, 1.0, -1.0),
        (-1.0, -1.0, 1.0),
        (1.0, -1.0, 1.0),
        (1.0, 1.0, 1.0),
        (-1.0, 1.0, 1.0),
    ];
    let p: Vec<(f64, f64)> = corners
        .iter()
        .map(|c| {
            let (sx, sy, _) = cam.project(c.0, c.1, c.2);
            (sx, sy)
        })
        .collect();
    let edges = [
        (0, 1),
        (1, 2),
        (2, 3),
        (3, 0), // bottom
        (4, 5),
        (5, 6),
        (6, 7),
        (7, 4), // top
        (0, 4),
        (1, 5),
        (2, 6),
        (3, 7), // verticals
    ];
    for (a, b) in edges {
        svg.line(p[a].0, p[a].1, p[b].0, p[b].1, "#d3d3e0", 1.0);
    }
}

/// A projected, depth-tagged quadrilateral ready to paint. Surfaces collect these
/// and draw them far-to-near (painter's algorithm).
struct Quad {
    depth: f64,
    corners: [(f64, f64); 4],
    color: String,
}

/// Aspect-correct equirectangular projection fitting geographic points into the
/// 2D plot frame. Longitude degrees are compressed by cos(mean latitude), and the
/// result is centered within the frame preserving aspect ratio.
struct GeoProjection {
    lon0: f64,
    lat1: f64,
    k: f64,
    fit: f64,
    ox: f64,
    oy: f64,
    pw: f64,
    ph: f64,
}

impl GeoProjection {
    /// Fit raw longitude/latitude bounds into the plot frame, padded by `pad`.
    fn fit_to(lon: (f64, f64), lat: (f64, f64), pad: f64) -> Self {
        let (lon0, lon1) = pad_range(lon.0, lon.1, pad);
        let (lat0, lat1) = pad_range(lat.0, lat.1, pad);
        let k = ((lat0 + lat1) / 2.0).to_radians().cos().max(0.05);
        let lon_span = (lon1 - lon0) * k;
        let lat_span = lat1 - lat0;
        let fit = ((R - L) / lon_span.max(1e-9)).min((B - T) / lat_span.max(1e-9));
        let pw = lon_span * fit;
        let ph = lat_span * fit;
        Self {
            lon0,
            lat1,
            k,
            fit,
            ox: L + (R - L - pw) / 2.0,
            oy: T + (B - T - ph) / 2.0,
            pw,
            ph,
        }
    }

    fn px(&self, lon: f64) -> f64 {
        self.ox + (lon - self.lon0) * self.k * self.fit
    }

    fn py(&self, lat: f64) -> f64 {
        self.oy + (self.lat1 - lat) * self.fit
    }

    /// Draw the map background panel.
    fn draw_base(&self, svg: &mut Svg) {
        svg.rect(self.ox, self.oy, self.pw, self.ph, "#eef3fb");
    }
}

// ── Tools ───────────────────────────────────────────────────────────────────────

#[tool_router(router = graphs_router, vis = "pub")]
impl MathServer {
    #[tool(description = "Plot geographic points (lat/lon) on an aspect-correct \
        equirectangular map; returns an SVG image. Optional per-point labels.")]
    fn geo_scatter_2d(&self, p: Parameters<GeoScatterParams>) -> CallToolResult {
        let GeoScatterParams { points, title } = p.0;
        if points.is_empty() {
            return err_result("error: no points");
        }
        let (Some(lon), Some(lat)) = (
            bounds(points.iter().map(|q| q.lon)),
            bounds(points.iter().map(|q| q.lat)),
        ) else {
            return err_result("error: invalid coordinates");
        };
        let proj = GeoProjection::fit_to(lon, lat, 0.08);
        let mut svg = Svg::new(WIDTH, HEIGHT);
        proj.draw_base(&mut svg);
        for q in &points {
            let (cx, cy) = (proj.px(q.lon), proj.py(q.lat));
            svg.circle(cx, cy, 5.0, "#2a6df4");
            svg.circle(cx, cy, 2.0, "#ffffff");
            if let Some(lbl) = &q.label {
                svg.text(cx + 8.0, cy + 4.0, lbl, 11.0, "start", "#1a1a2e");
            }
        }
        let n = points.len();
        svg_result(
            svg.finish(title.as_deref()),
            format!("Geographic scatter of {n} point(s)."),
        )
    }

    #[tool(description = "Plot a geographic route: connect lat/lon waypoints in \
        order as a polyline with start (green) and end (red) markers. Returns SVG.")]
    fn geo_route_2d(&self, p: Parameters<GeoRouteParams>) -> CallToolResult {
        let GeoRouteParams { points, title } = p.0;
        if points.len() < 2 {
            return err_result("error: need ≥ 2 waypoints");
        }
        let (Some(lon), Some(lat)) = (
            bounds(points.iter().map(|q| q.lon)),
            bounds(points.iter().map(|q| q.lat)),
        ) else {
            return err_result("error: invalid coordinates");
        };
        let proj = GeoProjection::fit_to(lon, lat, 0.1);
        let mut svg = Svg::new(WIDTH, HEIGHT);
        proj.draw_base(&mut svg);
        let pts: Vec<(f64, f64)> = points
            .iter()
            .map(|q| (proj.px(q.lon), proj.py(q.lat)))
            .collect();
        svg.polyline(&pts, "#2a6df4", 2.5);
        let last = pts.len() - 1;
        for (i, (cx, cy)) in pts.iter().enumerate() {
            let color = match i {
                0 => "#1faa59",
                _ if i == last => "#e63946",
                _ => "#2a6df4",
            };
            svg.circle(*cx, *cy, 4.0, color);
        }
        let n = points.len();
        svg_result(
            svg.finish(title.as_deref()),
            format!("Geographic route through {n} waypoint(s)."),
        )
    }

    #[tool(
        description = "2D scatter plot of (x, y) points with axes, gridlines, \
        and numeric ticks. Returns an SVG image."
    )]
    fn scatter_2d(&self, p: Parameters<Scatter2dParams>) -> CallToolResult {
        let Scatter2dParams {
            points,
            title,
            x_label,
            y_label,
        } = p.0;
        if points.is_empty() {
            return err_result("error: no points");
        }
        let (Some(xr), Some(yr)) = (
            padded_bounds(points.iter().map(|q| q.x), 0.05),
            padded_bounds(points.iter().map(|q| q.y), 0.05),
        ) else {
            return err_result("error: no finite coordinates");
        };
        let mut svg = Svg::new(WIDTH, HEIGHT);
        draw_axes(
            &mut svg,
            xr,
            yr,
            x_label.as_deref().unwrap_or(""),
            y_label.as_deref().unwrap_or(""),
        );
        for q in &points {
            let cx = scale(q.x, xr.0, xr.1, L, R);
            let cy = scale(q.y, yr.0, yr.1, B, T);
            svg.circle(cx, cy, 3.5, "#7048e8");
        }
        let n = points.len();
        svg_result(
            svg.finish(title.as_deref()),
            format!("Scatter of {n} point(s)."),
        )
    }

    #[tool(
        description = "2D line chart through a series of (x, y) points in order, \
        with axes and gridlines. Returns an SVG image."
    )]
    fn line_2d(&self, p: Parameters<Line2dParams>) -> CallToolResult {
        let Line2dParams { series, title } = p.0;
        if series.len() < 2 {
            return err_result("error: need ≥ 2 points");
        }
        let (Some(xr), Some(yr)) = (
            padded_bounds(series.iter().map(|q| q.x), 0.02),
            padded_bounds(series.iter().map(|q| q.y), 0.05),
        ) else {
            return err_result("error: no finite coordinates");
        };
        let mut svg = Svg::new(WIDTH, HEIGHT);
        draw_axes(&mut svg, xr, yr, "", "");
        let pts: Vec<(f64, f64)> = series
            .iter()
            .map(|q| (scale(q.x, xr.0, xr.1, L, R), scale(q.y, yr.0, yr.1, B, T)))
            .collect();
        svg.polyline(&pts, "#2a6df4", 2.0);
        let n = series.len();
        svg_result(
            svg.finish(title.as_deref()),
            format!("Line chart of {n} point(s)."),
        )
    }

    #[tool(description = "2D vertical bar chart from labeled values. Returns an SVG image.")]
    fn bar_2d(&self, p: Parameters<Bar2dParams>) -> CallToolResult {
        let Bar2dParams { bars, title } = p.0;
        if bars.is_empty() {
            return err_result("error: no bars");
        }
        let hi = bars.iter().map(|b| b.value).fold(0.0_f64, f64::max);
        let lo = bars.iter().map(|b| b.value).fold(0.0_f64, f64::min);
        let yr = (lo.min(0.0), hi.max(0.0));
        let mut svg = Svg::new(WIDTH, HEIGHT);
        draw_axes(&mut svg, (0.0, bars.len() as f64), yr, "", "");
        let zero_y = scale(0.0, yr.0, yr.1, B, T);
        let slot = (R - L) / bars.len() as f64;
        let bw = slot * 0.62;
        for (i, b) in bars.iter().enumerate() {
            let cx = L + (i as f64 + 0.5) * slot;
            let vy = scale(b.value, yr.0, yr.1, B, T);
            let (top, h) = if b.value >= 0.0 {
                (vy, zero_y - vy)
            } else {
                (zero_y, vy - zero_y)
            };
            svg.rect(cx - bw / 2.0, top, bw, h.max(0.5), "#2a6df4");
            svg.text(cx, B + 16.0, &b.label, 11.0, "middle", "#3a3a50");
        }
        let n = bars.len();
        svg_result(
            svg.finish(title.as_deref()),
            format!("Bar chart with {n} bar(s)."),
        )
    }

    #[tool(description = "3D scatter plot of (x, y, z) points via orthographic \
        projection with adjustable yaw/pitch; points colored by depth. Returns SVG.")]
    fn scatter_3d(&self, p: Parameters<Scatter3dParams>) -> CallToolResult {
        let Scatter3dParams {
            points,
            yaw,
            pitch,
            title,
        } = p.0;
        if points.is_empty() {
            return err_result("error: no points");
        }
        let xr = bounds(points.iter().map(|q| q.x)).unwrap();
        let yr = bounds(points.iter().map(|q| q.y)).unwrap();
        let zr = bounds(points.iter().map(|q| q.z)).unwrap();
        let cam = Camera::new(yaw.unwrap_or(35.0), pitch.unwrap_or(25.0));
        let mut svg = Svg::new(WIDTH, HEIGHT);
        draw_cube(&mut svg, &cam);
        // Project all points, then paint far-to-near.
        let mut proj: Vec<(f64, f64, f64, f64)> = points
            .iter()
            .map(|q| {
                let nx = scale(q.x, xr.0, xr.1, -1.0, 1.0);
                let ny = scale(q.y, yr.0, yr.1, -1.0, 1.0);
                let nz = scale(q.z, zr.0, zr.1, -1.0, 1.0);
                let (sx, sy, d) = cam.project(nx, ny, nz);
                let t = scale(q.z, zr.0, zr.1, 0.0, 1.0);
                (sx, sy, d, t)
            })
            .collect();
        proj.sort_by(|a, b| b.2.total_cmp(&a.2));
        for (sx, sy, _, t) in &proj {
            svg.circle(*sx, *sy, 4.5, &heat(*t));
        }
        let n = points.len();
        svg_result(
            svg.finish(title.as_deref()),
            format!("3D scatter of {n} point(s)."),
        )
    }

    #[tool(
        description = "3D surface plot from a row-major grid of heights z[row][col], \
        rendered as depth-sorted colored quads (height-mapped). Returns an SVG image."
    )]
    fn surface_3d(&self, p: Parameters<Surface3dParams>) -> CallToolResult {
        let Surface3dParams {
            z,
            yaw,
            pitch,
            title,
        } = p.0;
        let rows = z.len();
        if rows < 2 || z.iter().any(|r| r.len() < 2) {
            return err_result("error: need a grid of at least 2×2");
        }
        let cols = z[0].len();
        if z.iter().any(|r| r.len() != cols) {
            return err_result("error: all rows must have equal length");
        }
        let (zlo, zhi) = match bounds(z.iter().flat_map(|r| r.iter().copied())) {
            Some(b) => b,
            None => return err_result("error: no finite heights"),
        };
        let cam = Camera::new(yaw.unwrap_or(35.0), pitch.unwrap_or(30.0));
        let mut svg = Svg::new(WIDTH, HEIGHT);
        let nx = |c: usize| scale(c as f64, 0.0, (cols - 1) as f64, -1.0, 1.0);
        let ny = |r: usize| scale(r as f64, 0.0, (rows - 1) as f64, -1.0, 1.0);
        let nz = |v: f64| scale(v, zlo, zhi, -0.8, 0.8);
        // Build one quad per grid cell, then paint far-to-near (painter's algorithm).
        let mut quads: Vec<Quad> = Vec::new();
        for r in 0..rows - 1 {
            for c in 0..cols - 1 {
                let cell = [(r, c), (r, c + 1), (r + 1, c + 1), (r + 1, c)];
                let mut corners = [(0.0, 0.0); 4];
                let mut depth = 0.0;
                let mut hsum = 0.0;
                for (k, (rr, cc)) in cell.iter().enumerate() {
                    let (sx, sy, d) = cam.project(nx(*cc), ny(*rr), nz(z[*rr][*cc]));
                    corners[k] = (sx, sy);
                    depth += d;
                    hsum += z[*rr][*cc];
                }
                let t = scale(hsum / 4.0, zlo, zhi, 0.0, 1.0);
                quads.push(Quad {
                    depth: depth / 4.0,
                    corners,
                    color: heat(t),
                });
            }
        }
        quads.sort_by(|a, b| b.depth.total_cmp(&a.depth));
        for q in &quads {
            svg.polygon(&q.corners, &q.color, "#33333322", 0.5);
        }
        svg_result(
            svg.finish(title.as_deref()),
            format!("3D surface over a {rows}×{cols} grid."),
        )
    }

    #[tool(
        description = "3D geographic terrain: plot lat/lon points with elevation as \
        height, colored by elevation, with vertical stems to the base plane. Returns SVG."
    )]
    fn geo_terrain_3d(&self, p: Parameters<GeoTerrain3dParams>) -> CallToolResult {
        let GeoTerrain3dParams {
            points,
            yaw,
            pitch,
            title,
        } = p.0;
        if points.is_empty() {
            return err_result("error: no points");
        }
        let (Some(lonr), Some(latr), Some(elevr)) = (
            bounds(points.iter().map(|q| q.lon)),
            bounds(points.iter().map(|q| q.lat)),
            bounds(points.iter().map(|q| q.elevation)),
        ) else {
            return err_result("error: no finite coordinates");
        };
        let cam = Camera::new(yaw.unwrap_or(40.0), pitch.unwrap_or(28.0));
        let mut svg = Svg::new(WIDTH, HEIGHT);
        draw_cube(&mut svg, &cam);
        // An elevation marker plus a stem dropped to the base plane, depth-sorted.
        struct Stem {
            top: (f64, f64),
            base: (f64, f64),
            depth: f64,
            t: f64,
        }
        let mut stems: Vec<Stem> = points
            .iter()
            .map(|q| {
                let nx = scale(q.lon, lonr.0, lonr.1, -1.0, 1.0);
                let ny = scale(q.lat, latr.0, latr.1, -1.0, 1.0);
                let nz = scale(q.elevation, elevr.0, elevr.1, -1.0, 1.0);
                let (sx, sy, depth) = cam.project(nx, ny, nz);
                let (bx, by, _) = cam.project(nx, ny, -1.0);
                let t = scale(q.elevation, elevr.0, elevr.1, 0.0, 1.0);
                Stem {
                    top: (sx, sy),
                    base: (bx, by),
                    depth,
                    t,
                }
            })
            .collect();
        stems.sort_by(|a, b| b.depth.total_cmp(&a.depth));
        for s in &stems {
            svg.line(s.base.0, s.base.1, s.top.0, s.top.1, "#c0c0d0", 1.0);
            svg.circle(s.top.0, s.top.1, 4.5, &heat(s.t));
        }
        let n = points.len();
        svg_result(
            svg.finish(title.as_deref()),
            format!("3D terrain of {n} geographic point(s)."),
        )
    }

    // ── Overlays (superposition) ──────────────────────────────────────────────

    #[tool(
        description = "Superpose multiple 2D series (line/scatter/both) on shared \
        auto-scaled axes, each in a distinct color with a legend. Input order is honored \
        1:1: series[i] keeps color i and legend slot i, and is drawn in order so later \
        series sit on top of earlier ones. Returns an SVG image."
    )]
    fn overlay_2d(&self, p: Parameters<Overlay2dParams>) -> CallToolResult {
        let Overlay2dParams {
            series,
            title,
            x_label,
            y_label,
        } = p.0;
        if series.len() < 2 {
            return err_result("error: need ≥ 2 series to superpose");
        }
        let (Some(xr), Some(yr)) = (
            padded_bounds(
                series.iter().flat_map(|s| s.points.iter().map(|q| q.x)),
                0.04,
            ),
            padded_bounds(
                series.iter().flat_map(|s| s.points.iter().map(|q| q.y)),
                0.05,
            ),
        ) else {
            return err_result("error: no finite coordinates across series");
        };
        let mut svg = Svg::new(WIDTH, HEIGHT);
        draw_axes(
            &mut svg,
            xr,
            yr,
            x_label.as_deref().unwrap_or(""),
            y_label.as_deref().unwrap_or(""),
        );
        for (i, s) in series.iter().enumerate() {
            let color = palette(i);
            let kind = s.kind.as_deref().unwrap_or("line");
            let pts: Vec<(f64, f64)> = s
                .points
                .iter()
                .map(|q| (scale(q.x, xr.0, xr.1, L, R), scale(q.y, yr.0, yr.1, B, T)))
                .collect();
            if kind == "line" || kind == "both" {
                svg.polyline(&pts, color, 2.0);
            }
            if kind == "scatter" || kind == "both" {
                for (cx, cy) in &pts {
                    svg.circle(*cx, *cy, 3.2, color);
                }
            }
        }
        let items: Vec<(String, String)> = series
            .iter()
            .enumerate()
            .map(|(i, s)| (s.name.clone(), palette(i).to_string()))
            .collect();
        svg.legend(&items, R - legend_width(&items) - 8.0, T + 8.0);
        let n = series.len();
        svg_result(
            svg.finish(title.as_deref()),
            format!("Superposed {n} 2D series."),
        )
    }

    #[tool(
        description = "Superpose multiple geographic layers (point sets and/or routes) \
        on one shared equirectangular map, each in a distinct color with a legend. Input \
        order is honored 1:1: layer[i] keeps color i and legend slot i, and is drawn in \
        order so later layers sit on top of earlier ones. Returns SVG."
    )]
    fn geo_overlay_2d(&self, p: Parameters<GeoOverlay2dParams>) -> CallToolResult {
        let GeoOverlay2dParams { layers, title } = p.0;
        if layers.len() < 2 {
            return err_result("error: need ≥ 2 layers to superpose");
        }
        let (Some(lon), Some(lat)) = (
            bounds(layers.iter().flat_map(|l| l.points.iter().map(|q| q.lon))),
            bounds(layers.iter().flat_map(|l| l.points.iter().map(|q| q.lat))),
        ) else {
            return err_result("error: no finite coordinates across layers");
        };
        let proj = GeoProjection::fit_to(lon, lat, 0.1);
        let mut svg = Svg::new(WIDTH, HEIGHT);
        proj.draw_base(&mut svg);
        for (i, layer) in layers.iter().enumerate() {
            let color = palette(i);
            let pts: Vec<(f64, f64)> = layer
                .points
                .iter()
                .map(|q| (proj.px(q.lon), proj.py(q.lat)))
                .collect();
            if layer.kind.as_deref() == Some("route") {
                svg.polyline(&pts, color, 2.5);
            }
            for (cx, cy) in &pts {
                svg.circle(*cx, *cy, 4.0, color);
            }
        }
        let items: Vec<(String, String)> = layers
            .iter()
            .enumerate()
            .map(|(i, l)| (l.name.clone(), palette(i).to_string()))
            .collect();
        svg.legend(&items, L + 8.0, T + 8.0);
        let n = layers.len();
        svg_result(
            svg.finish(title.as_deref()),
            format!("Superposed {n} geographic layer(s)."),
        )
    }

    #[tool(
        description = "Superpose multiple 3D series (scatter point-clouds and/or surfaces) \
        in one shared scene with a single camera and a legend. All series share one \
        coordinate space (combined bounds), but input order is honored 1:1: series[i] keeps \
        color i and legend slot i, and series are painted in order so later series sit on \
        top of earlier ones (each surface is still internally depth-sorted). \
        Adjustable yaw/pitch. Returns an SVG image."
    )]
    fn overlay_3d(&self, p: Parameters<Overlay3dParams>) -> CallToolResult {
        let Overlay3dParams {
            series,
            yaw,
            pitch,
            title,
        } = p.0;
        if series.len() < 2 {
            return err_result("error: need ≥ 2 series to superpose");
        }
        // Resolve each series' kind and its x/y/z extents in shared data space.
        struct Resolved<'a> {
            name: &'a str,
            color: &'static str,
            kind: &'a str,
            s: &'a Series3d,
            xext: (f64, f64),
            yext: (f64, f64),
        }
        let mut resolved = Vec::new();
        let mut gx = (f64::INFINITY, f64::NEG_INFINITY);
        let mut gy = (f64::INFINITY, f64::NEG_INFINITY);
        let mut gz = (f64::INFINITY, f64::NEG_INFINITY);
        let grow = |b: &mut (f64, f64), lo: f64, hi: f64| {
            b.0 = b.0.min(lo);
            b.1 = b.1.max(hi);
        };
        for (i, s) in series.iter().enumerate() {
            let kind = match s.kind.as_deref() {
                Some(k) => k,
                None if !s.points.is_empty() => "scatter",
                None if !s.z.is_empty() => "surface",
                None => return err_result(format!("error: series '{}' has no data", s.name)),
            };
            match kind {
                "scatter" => {
                    if s.points.is_empty() {
                        return err_result(format!(
                            "error: scatter series '{}' has no points",
                            s.name
                        ));
                    }
                    let xb = bounds(s.points.iter().map(|q| q.x)).unwrap();
                    let yb = bounds(s.points.iter().map(|q| q.y)).unwrap();
                    let zb = bounds(s.points.iter().map(|q| q.z)).unwrap();
                    grow(&mut gx, xb.0, xb.1);
                    grow(&mut gy, yb.0, yb.1);
                    grow(&mut gz, zb.0, zb.1);
                    resolved.push(Resolved {
                        name: &s.name,
                        color: palette(i),
                        kind: "scatter",
                        s,
                        xext: xb,
                        yext: yb,
                    });
                }
                "surface" => {
                    let rows = s.z.len();
                    if rows < 2 || s.z.iter().any(|r| r.len() < 2) {
                        return err_result(format!(
                            "error: surface '{}' needs a grid ≥ 2×2",
                            s.name
                        ));
                    }
                    let cols = s.z[0].len();
                    if s.z.iter().any(|r| r.len() != cols) {
                        return err_result(format!(
                            "error: surface '{}' rows must be equal length",
                            s.name
                        ));
                    }
                    let xext = s
                        .x_range
                        .map(|r| (r[0], r[1]))
                        .unwrap_or((0.0, (cols - 1) as f64));
                    let yext = s
                        .y_range
                        .map(|r| (r[0], r[1]))
                        .unwrap_or((0.0, (rows - 1) as f64));
                    let zb = bounds(s.z.iter().flat_map(|r| r.iter().copied())).unwrap();
                    grow(&mut gx, xext.0, xext.1);
                    grow(&mut gy, yext.0, yext.1);
                    grow(&mut gz, zb.0, zb.1);
                    resolved.push(Resolved {
                        name: &s.name,
                        color: palette(i),
                        kind: "surface",
                        s,
                        xext,
                        yext,
                    });
                }
                other => {
                    return err_result(format!(
                        "error: unknown kind '{other}' for series '{}'",
                        s.name
                    ));
                }
            }
        }
        let cam = Camera::new(yaw.unwrap_or(35.0), pitch.unwrap_or(28.0));
        let nx = |v: f64| scale(v, gx.0, gx.1, -1.0, 1.0);
        let ny = |v: f64| scale(v, gy.0, gy.1, -1.0, 1.0);
        let nz = |v: f64| scale(v, gz.0, gz.1, -1.0, 1.0);

        // Paint series in input order — series[0] first (back), the last on top.
        // A surface's own quads are still depth-sorted so each renders correctly.
        let mut svg = Svg::new(WIDTH, HEIGHT);
        draw_cube(&mut svg, &cam);
        for r in &resolved {
            match r.kind {
                "scatter" => {
                    for q in &r.s.points {
                        let (sx, sy, _) = cam.project(nx(q.x), ny(q.y), nz(q.z));
                        svg.circle(sx, sy, 4.0, r.color);
                    }
                }
                "surface" => {
                    let rows = r.s.z.len();
                    let cols = r.s.z[0].len();
                    let cx = |c: usize| scale(c as f64, 0.0, (cols - 1) as f64, r.xext.0, r.xext.1);
                    let cy =
                        |rr: usize| scale(rr as f64, 0.0, (rows - 1) as f64, r.yext.0, r.yext.1);
                    let mut quads: Vec<Quad> = Vec::new();
                    for rr in 0..rows - 1 {
                        for c in 0..cols - 1 {
                            let cell = [(rr, c), (rr, c + 1), (rr + 1, c + 1), (rr + 1, c)];
                            let mut corners = [(0.0, 0.0); 4];
                            let mut depth = 0.0;
                            for (k, (yi, xi)) in cell.iter().enumerate() {
                                let (sx, sy, d) =
                                    cam.project(nx(cx(*xi)), ny(cy(*yi)), nz(r.s.z[*yi][*xi]));
                                corners[k] = (sx, sy);
                                depth += d;
                            }
                            quads.push(Quad {
                                depth: depth / 4.0,
                                corners,
                                color: r.color.to_string(),
                            });
                        }
                    }
                    quads.sort_by(|a, b| b.depth.total_cmp(&a.depth));
                    for q in &quads {
                        svg.polygon(&q.corners, &q.color, "#ffffff", 0.4);
                    }
                }
                _ => {}
            }
        }
        let items: Vec<(String, String)> = resolved
            .iter()
            .map(|r| (r.name.to_string(), r.color.to_string()))
            .collect();
        svg.legend(&items, 12.0, T + 8.0);
        let n = resolved.len();
        svg_result(
            svg.finish(title.as_deref()),
            format!("Superposed {n} 3D series."),
        )
    }
}
