use rmcp::{handler::server::wrapper::Parameters, tool, tool_router};
use schemars::JsonSchema;
use serde::Deserialize;
use std::f64::consts::PI;

use crate::server::MathServer;

#[derive(Deserialize, JsonSchema)]
pub struct Radians {
    pub radians: f64,
}
#[derive(Deserialize, JsonSchema)]
pub struct Degrees {
    pub degrees: f64,
}
#[derive(Deserialize, JsonSchema)]
pub struct OneFloat {
    pub x: f64,
}
#[derive(Deserialize, JsonSchema)]
pub struct Atan2Params {
    pub y: f64,
    pub x: f64,
}
#[derive(Deserialize, JsonSchema)]
pub struct HaversineParams {
    /// Latitude of point 1 in degrees
    pub lat1: f64,
    /// Longitude of point 1 in degrees
    pub lon1: f64,
    /// Latitude of point 2 in degrees
    pub lat2: f64,
    /// Longitude of point 2 in degrees
    pub lon2: f64,
}

#[tool_router(router = trig_router, vis = "pub")]
impl MathServer {
    #[tool(description = "sin(radians)")]
    fn sin(&self, p: Parameters<Radians>) -> String {
        format!("{}", p.0.radians.sin())
    }

    #[tool(description = "cos(radians)")]
    fn cos(&self, p: Parameters<Radians>) -> String {
        format!("{}", p.0.radians.cos())
    }

    #[tool(description = "tan(radians)")]
    fn tan(&self, p: Parameters<Radians>) -> String {
        format!("{}", p.0.radians.tan())
    }

    #[tool(description = "asin(x): arc-sine in radians, x in [-1, 1]")]
    fn asin(&self, p: Parameters<OneFloat>) -> String {
        if !(-1.0..=1.0).contains(&p.0.x) {
            "error: x must be in [-1,1]".into()
        } else {
            format!("{}", p.0.x.asin())
        }
    }

    #[tool(description = "acos(x): arc-cosine in radians, x in [-1, 1]")]
    fn acos(&self, p: Parameters<OneFloat>) -> String {
        if !(-1.0..=1.0).contains(&p.0.x) {
            "error: x must be in [-1,1]".into()
        } else {
            format!("{}", p.0.x.acos())
        }
    }

    #[tool(description = "atan(x): arc-tangent in radians")]
    fn atan(&self, p: Parameters<OneFloat>) -> String {
        format!("{}", p.0.x.atan())
    }

    #[tool(description = "atan2(y, x): angle of vector (x,y) in (-π, π]")]
    fn atan2(&self, p: Parameters<Atan2Params>) -> String {
        format!("{}", p.0.y.atan2(p.0.x))
    }

    #[tool(description = "sin(degrees)")]
    fn sin_deg(&self, p: Parameters<Degrees>) -> String {
        format!("{}", (p.0.degrees * PI / 180.0).sin())
    }

    #[tool(description = "cos(degrees)")]
    fn cos_deg(&self, p: Parameters<Degrees>) -> String {
        format!("{}", (p.0.degrees * PI / 180.0).cos())
    }

    #[tool(description = "tan(degrees)")]
    fn tan_deg(&self, p: Parameters<Degrees>) -> String {
        format!("{}", (p.0.degrees * PI / 180.0).tan())
    }

    #[tool(description = "asin in degrees for x in [-1, 1]")]
    fn asin_deg(&self, p: Parameters<OneFloat>) -> String {
        if !(-1.0..=1.0).contains(&p.0.x) {
            "error: x must be in [-1,1]".into()
        } else {
            format!("{}", p.0.x.asin() * 180.0 / PI)
        }
    }

    #[tool(description = "acos in degrees for x in [-1, 1]")]
    fn acos_deg(&self, p: Parameters<OneFloat>) -> String {
        if !(-1.0..=1.0).contains(&p.0.x) {
            "error: x must be in [-1,1]".into()
        } else {
            format!("{}", p.0.x.acos() * 180.0 / PI)
        }
    }

    #[tool(description = "atan in degrees")]
    fn atan_deg(&self, p: Parameters<OneFloat>) -> String {
        format!("{}", p.0.x.atan() * 180.0 / PI)
    }

    #[tool(description = "sinh(x): hyperbolic sine")]
    fn sinh(&self, p: Parameters<OneFloat>) -> String {
        format!("{}", p.0.x.sinh())
    }

    #[tool(description = "cosh(x): hyperbolic cosine")]
    fn cosh(&self, p: Parameters<OneFloat>) -> String {
        format!("{}", p.0.x.cosh())
    }

    #[tool(description = "tanh(x): hyperbolic tangent")]
    fn tanh(&self, p: Parameters<OneFloat>) -> String {
        format!("{}", p.0.x.tanh())
    }

    #[tool(description = "asinh(x): inverse hyperbolic sine")]
    fn asinh(&self, p: Parameters<OneFloat>) -> String {
        format!("{}", p.0.x.asinh())
    }

    #[tool(description = "acosh(x): inverse hyperbolic cosine (requires x ≥ 1)")]
    fn acosh(&self, p: Parameters<OneFloat>) -> String {
        if p.0.x < 1.0 {
            "error: x must be ≥ 1".into()
        } else {
            format!("{}", p.0.x.acosh())
        }
    }

    #[tool(description = "atanh(x): inverse hyperbolic tangent (requires |x| < 1)")]
    fn atanh(&self, p: Parameters<OneFloat>) -> String {
        if p.0.x.abs() >= 1.0 {
            "error: |x| must be < 1".into()
        } else {
            format!("{}", p.0.x.atanh())
        }
    }

    #[tool(description = "Convert degrees to radians")]
    fn degrees_to_radians(&self, p: Parameters<Degrees>) -> String {
        format!("{}", p.0.degrees * PI / 180.0)
    }

    #[tool(description = "Convert radians to degrees")]
    fn radians_to_degrees(&self, p: Parameters<Radians>) -> String {
        format!("{}", p.0.radians * 180.0 / PI)
    }

    #[tool(description = "sec(radians) = 1/cos")]
    fn sec(&self, p: Parameters<Radians>) -> String {
        let c = p.0.radians.cos();
        if c == 0.0 {
            "error: undefined (cos=0)".into()
        } else {
            format!("{}", 1.0 / c)
        }
    }

    #[tool(description = "csc(radians) = 1/sin")]
    fn csc(&self, p: Parameters<Radians>) -> String {
        let s = p.0.radians.sin();
        if s == 0.0 {
            "error: undefined (sin=0)".into()
        } else {
            format!("{}", 1.0 / s)
        }
    }

    #[tool(description = "cot(radians) = cos/sin")]
    fn cot(&self, p: Parameters<Radians>) -> String {
        let s = p.0.radians.sin();
        if s == 0.0 {
            "error: undefined (sin=0)".into()
        } else {
            format!("{}", p.0.radians.cos() / s)
        }
    }

    #[tool(description = "versine: 1 - cos(x)")]
    fn versin(&self, p: Parameters<Radians>) -> String {
        format!("{}", 1.0 - p.0.radians.cos())
    }

    #[tool(description = "haversine: (1 - cos(x)) / 2")]
    fn haversin(&self, p: Parameters<Radians>) -> String {
        format!("{}", (1.0 - p.0.radians.cos()) / 2.0)
    }

    #[tool(description = "Great-circle distance in km (lat/lon in degrees) via haversine formula")]
    fn haversine_distance(&self, p: Parameters<HaversineParams>) -> String {
        let HaversineParams {
            lat1,
            lon1,
            lat2,
            lon2,
        } = p.0;
        let r = 6371.0f64;
        let dlat = (lat2 - lat1).to_radians();
        let dlon = (lon2 - lon1).to_radians();
        let a = (dlat / 2.0).sin().powi(2)
            + lat1.to_radians().cos() * lat2.to_radians().cos() * (dlon / 2.0).sin().powi(2);
        format!("{}", r * 2.0 * a.sqrt().asin())
    }
}
