use rmcp::{handler::server::wrapper::Parameters, tool, tool_router};
use schemars::JsonSchema;
use serde::Deserialize;

use crate::server::MathServer;

#[derive(Deserialize, JsonSchema)]
pub struct FloatList {
    pub values: Vec<f64>,
}
#[derive(Deserialize, JsonSchema)]
pub struct TwoLists {
    pub x: Vec<f64>,
    pub y: Vec<f64>,
}
#[derive(Deserialize, JsonSchema)]
pub struct ZScoreParams {
    pub value: f64,
    pub mean: f64,
    pub std_dev: f64,
}
#[derive(Deserialize, JsonSchema)]
pub struct PercentileParams {
    pub values: Vec<f64>,
    pub p: f64,
}
#[derive(Deserialize, JsonSchema)]
pub struct MovingAvgParams {
    pub values: Vec<f64>,
    pub window: usize,
}
#[derive(Deserialize, JsonSchema)]
pub struct EmaParams {
    pub values: Vec<f64>,
    pub alpha: f64,
}

fn mean_of(d: &[f64]) -> f64 {
    d.iter().sum::<f64>() / d.len() as f64
}

fn variance_of(d: &[f64], pop: bool) -> f64 {
    let m = mean_of(d);
    let denom = if pop {
        d.len() as f64
    } else {
        (d.len() - 1) as f64
    };
    d.iter().map(|v| (v - m).powi(2)).sum::<f64>() / denom
}

#[tool_router(router = statistics_router, vis = "pub")]
impl MathServer {
    #[tool(description = "Arithmetic mean")]
    fn mean(&self, p: Parameters<FloatList>) -> String {
        if p.0.values.is_empty() {
            return "error: empty list".into();
        }
        format!("{}", mean_of(&p.0.values))
    }

    #[tool(description = "Median (middle value after sorting)")]
    fn median(&self, p: Parameters<FloatList>) -> String {
        let mut v = p.0.values;
        if v.is_empty() {
            return "error: empty list".into();
        }
        v.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let n = v.len();
        if n.is_multiple_of(2) {
            format!("{}", (v[n / 2 - 1] + v[n / 2]) / 2.0)
        } else {
            format!("{}", v[n / 2])
        }
    }

    #[tool(description = "Mode(s): most frequent value(s) as JSON array")]
    fn mode(&self, p: Parameters<FloatList>) -> String {
        let values = p.0.values;
        if values.is_empty() {
            return "error: empty list".into();
        }
        use std::collections::HashMap;
        let mut counts: HashMap<String, usize> = HashMap::new();
        for v in &values {
            *counts.entry(format!("{v}")).or_insert(0) += 1;
        }
        let max = *counts.values().max().unwrap();
        let mut modes: Vec<f64> = counts
            .iter()
            .filter(|(_, c)| **c == max)
            .map(|(k, _)| k.parse::<f64>().unwrap())
            .collect();
        modes.sort_by(|a, b| a.partial_cmp(b).unwrap());
        format!("{modes:?}")
    }

    #[tool(description = "Population variance (divides by n)")]
    fn variance_population(&self, p: Parameters<FloatList>) -> String {
        if p.0.values.is_empty() {
            return "error: empty list".into();
        }
        format!("{}", variance_of(&p.0.values, true))
    }

    #[tool(description = "Sample variance (Bessel-corrected, divides by n-1)")]
    fn variance_sample(&self, p: Parameters<FloatList>) -> String {
        if p.0.values.len() < 2 {
            return "error: need ≥ 2 values".into();
        }
        format!("{}", variance_of(&p.0.values, false))
    }

    #[tool(description = "Population standard deviation")]
    fn std_dev_population(&self, p: Parameters<FloatList>) -> String {
        if p.0.values.is_empty() {
            return "error: empty list".into();
        }
        format!("{}", variance_of(&p.0.values, true).sqrt())
    }

    #[tool(description = "Sample standard deviation (Bessel-corrected)")]
    fn std_dev_sample(&self, p: Parameters<FloatList>) -> String {
        if p.0.values.len() < 2 {
            return "error: need ≥ 2 values".into();
        }
        format!("{}", variance_of(&p.0.values, false).sqrt())
    }

    #[tool(description = "Range: max - min")]
    fn range_stat(&self, p: Parameters<FloatList>) -> String {
        if p.0.values.is_empty() {
            return "error: empty list".into();
        }
        let min = p.0.values.iter().cloned().fold(f64::INFINITY, f64::min);
        let max = p.0.values.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        format!("{}", max - min)
    }

    #[tool(description = "Interquartile range: Q3 - Q1 (needs ≥ 4 values)")]
    fn iqr(&self, p: Parameters<FloatList>) -> String {
        let mut v = p.0.values;
        if v.len() < 4 {
            return "error: need ≥ 4 values".into();
        }
        v.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let n = v.len();
        format!("{}", v[3 * n / 4] - v[n / 4])
    }

    #[tool(description = "Pth percentile (p in [0,100]) via linear interpolation")]
    fn percentile(&self, p: Parameters<PercentileParams>) -> String {
        let PercentileParams { mut values, p } = p.0;
        if values.is_empty() {
            return "error: empty list".into();
        }
        if !(0.0..=100.0).contains(&p) {
            return "error: p must be in [0,100]".into();
        }
        values.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let idx = (p / 100.0) * (values.len() - 1) as f64;
        let lo = idx.floor() as usize;
        let hi = idx.ceil() as usize;
        let frac = idx - lo as f64;
        format!("{}", values[lo] * (1.0 - frac) + values[hi] * frac)
    }

    #[tool(description = "Z-score: (value - mean) / std_dev")]
    fn z_score(&self, p: Parameters<ZScoreParams>) -> String {
        let ZScoreParams {
            value,
            mean,
            std_dev,
        } = p.0;
        if std_dev == 0.0 {
            "error: std_dev must be non-zero".into()
        } else {
            format!("{}", (value - mean) / std_dev)
        }
    }

    #[tool(description = "Sum of all values")]
    fn sum(&self, p: Parameters<FloatList>) -> String {
        format!("{}", p.0.values.iter().sum::<f64>())
    }

    #[tool(description = "Product of all values")]
    fn product_stat(&self, p: Parameters<FloatList>) -> String {
        format!("{}", p.0.values.iter().product::<f64>())
    }

    #[tool(description = "Geometric mean of positive values")]
    fn geometric_mean(&self, p: Parameters<FloatList>) -> String {
        let values = p.0.values;
        if values.is_empty() {
            return "error: empty list".into();
        }
        if values.iter().any(|&v| v <= 0.0) {
            return "error: all values must be positive".into();
        }
        let log_sum: f64 = values.iter().map(|v| v.ln()).sum();
        format!("{}", (log_sum / values.len() as f64).exp())
    }

    #[tool(description = "Harmonic mean of positive values")]
    fn harmonic_mean(&self, p: Parameters<FloatList>) -> String {
        let values = p.0.values;
        if values.is_empty() {
            return "error: empty list".into();
        }
        if values.iter().any(|&v| v <= 0.0) {
            return "error: all values must be positive".into();
        }
        let recip: f64 = values.iter().map(|v| 1.0 / v).sum();
        format!("{}", values.len() as f64 / recip)
    }

    #[tool(description = "Root mean square (RMS)")]
    fn rms(&self, p: Parameters<FloatList>) -> String {
        let values = p.0.values;
        if values.is_empty() {
            return "error: empty list".into();
        }
        let sq: f64 = values.iter().map(|v| v * v).sum();
        format!("{}", (sq / values.len() as f64).sqrt())
    }

    #[tool(description = "Skewness (Fisher's moment coefficient, needs ≥ 3 values)")]
    fn skewness(&self, p: Parameters<FloatList>) -> String {
        let values = p.0.values;
        if values.len() < 3 {
            return "error: need ≥ 3 values".into();
        }
        let m = mean_of(&values);
        let n = values.len() as f64;
        let s = variance_of(&values, false).sqrt();
        if s == 0.0 {
            return "0".into();
        }
        let sk =
            values.iter().map(|v| ((v - m) / s).powi(3)).sum::<f64>() * n / ((n - 1.0) * (n - 2.0));
        format!("{sk}")
    }

    #[tool(description = "Excess kurtosis (normal → 0, needs ≥ 4 values)")]
    fn kurtosis(&self, p: Parameters<FloatList>) -> String {
        let values = p.0.values;
        if values.len() < 4 {
            return "error: need ≥ 4 values".into();
        }
        let m = mean_of(&values);
        let n = values.len() as f64;
        let s = variance_of(&values, false).sqrt();
        if s == 0.0 {
            return "0".into();
        }
        let k = values.iter().map(|v| ((v - m) / s).powi(4)).sum::<f64>() * n * (n + 1.0)
            / ((n - 1.0) * (n - 2.0) * (n - 3.0))
            - 3.0 * (n - 1.0).powi(2) / ((n - 2.0) * (n - 3.0));
        format!("{k}")
    }

    #[tool(description = "Pearson correlation coefficient between two equal-length lists")]
    fn pearson_correlation(&self, p: Parameters<TwoLists>) -> String {
        let TwoLists { x, y } = p.0;
        if x.len() != y.len() {
            return "error: lists must be equal length".into();
        }
        if x.len() < 2 {
            return "error: need ≥ 2 values".into();
        }
        let mx = mean_of(&x);
        let my = mean_of(&y);
        let num: f64 = x
            .iter()
            .zip(y.iter())
            .map(|(xi, yi)| (xi - mx) * (yi - my))
            .sum();
        let dx: f64 = x.iter().map(|xi| (xi - mx).powi(2)).sum();
        let dy: f64 = y.iter().map(|yi| (yi - my).powi(2)).sum();
        let denom = (dx * dy).sqrt();
        if denom == 0.0 {
            "error: zero variance".into()
        } else {
            format!("{}", num / denom)
        }
    }

    #[tool(description = "Sample covariance between two equal-length lists")]
    fn covariance(&self, p: Parameters<TwoLists>) -> String {
        let TwoLists { x, y } = p.0;
        if x.len() != y.len() {
            return "error: lists must be equal length".into();
        }
        if x.len() < 2 {
            return "error: need ≥ 2 values".into();
        }
        let mx = mean_of(&x);
        let my = mean_of(&y);
        let cov: f64 = x
            .iter()
            .zip(y.iter())
            .map(|(xi, yi)| (xi - mx) * (yi - my))
            .sum::<f64>()
            / (x.len() - 1) as f64;
        format!("{cov}")
    }

    #[tool(
        description = "Simple linear regression: y = slope*x + intercept; returns 'slope=… intercept=… r2=…'"
    )]
    fn linear_regression(&self, p: Parameters<TwoLists>) -> String {
        let TwoLists { x, y } = p.0;
        if x.len() != y.len() {
            return "error: lists must be equal length".into();
        }
        if x.len() < 2 {
            return "error: need ≥ 2 values".into();
        }
        let mx = mean_of(&x);
        let my = mean_of(&y);
        let num: f64 = x
            .iter()
            .zip(y.iter())
            .map(|(xi, yi)| (xi - mx) * (yi - my))
            .sum();
        let den: f64 = x.iter().map(|xi| (xi - mx).powi(2)).sum();
        if den == 0.0 {
            return "error: zero variance in x".into();
        }
        let slope = num / den;
        let intercept = my - slope * mx;
        let ss_res: f64 = x
            .iter()
            .zip(y.iter())
            .map(|(xi, yi)| (yi - (slope * xi + intercept)).powi(2))
            .sum();
        let ss_tot: f64 = y.iter().map(|yi| (yi - my).powi(2)).sum();
        let r2 = if ss_tot == 0.0 {
            1.0
        } else {
            1.0 - ss_res / ss_tot
        };
        format!("slope={slope} intercept={intercept} r2={r2}")
    }

    #[tool(description = "Shannon entropy H = -∑ p·ln(p) of a probability distribution")]
    fn entropy(&self, p: Parameters<FloatList>) -> String {
        let values = p.0.values;
        if values.is_empty() {
            return "error: empty list".into();
        }
        if values.iter().any(|&v| v < 0.0) {
            return "error: probabilities must be non-negative".into();
        }
        let h: f64 = values
            .iter()
            .filter(|&&v| v > 0.0)
            .map(|&v| -v * v.ln())
            .sum();
        format!("{h}")
    }

    #[tool(
        description = "KL divergence D_KL(P‖Q) = ∑ p·ln(p/q); lists x=P and y=Q must be same length"
    )]
    fn kl_divergence(&self, p: Parameters<TwoLists>) -> String {
        let TwoLists {
            x: prob_p,
            y: prob_q,
        } = p.0;
        if prob_p.len() != prob_q.len() {
            return "error: distributions must match in length".into();
        }
        let kl: f64 = prob_p
            .iter()
            .zip(prob_q.iter())
            .filter(|(pi, qi)| **pi > 0.0 && **qi > 0.0)
            .map(|(pi, qi)| *pi * (*pi / *qi).ln())
            .sum();
        format!("{kl}")
    }

    #[tool(description = "Simple moving average with given window size; returns JSON array")]
    fn moving_average(&self, p: Parameters<MovingAvgParams>) -> String {
        let MovingAvgParams { values, window } = p.0;
        if window == 0 {
            return "error: window must be > 0".into();
        }
        if window > values.len() {
            return "error: window larger than data".into();
        }
        let result: Vec<f64> = values
            .windows(window)
            .map(|w| w.iter().sum::<f64>() / window as f64)
            .collect();
        format!("{result:?}")
    }

    #[tool(description = "Exponential moving average with smoothing factor α; returns JSON array")]
    fn exponential_moving_average(&self, p: Parameters<EmaParams>) -> String {
        let EmaParams { values, alpha } = p.0;
        if values.is_empty() {
            return "error: empty list".into();
        }
        if !(0.0..=1.0).contains(&alpha) || alpha == 0.0 {
            return "error: alpha must be in (0,1]".into();
        }
        let mut ema = vec![values[0]];
        for &v in values.iter().skip(1) {
            let prev = *ema.last().unwrap();
            ema.push(alpha * v + (1.0 - alpha) * prev);
        }
        format!("{ema:?}")
    }

    #[tool(description = "Five-number summary: {min, q1, median, q3, max} as JSON object")]
    fn five_number_summary(&self, p: Parameters<FloatList>) -> String {
        let mut v = p.0.values;
        if v.len() < 4 {
            return "error: need ≥ 4 values".into();
        }
        v.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let n = v.len();
        let median = if n.is_multiple_of(2) {
            (v[n / 2 - 1] + v[n / 2]) / 2.0
        } else {
            v[n / 2]
        };
        format!(
            r#"{{"min":{},"q1":{},"median":{},"q3":{},"max":{}}}"#,
            v[0],
            v[n / 4],
            median,
            v[3 * n / 4],
            v[n - 1]
        )
    }

    #[tool(description = "Mean absolute deviation (MAD) from the mean")]
    fn mean_absolute_deviation(&self, p: Parameters<FloatList>) -> String {
        let values = p.0.values;
        if values.is_empty() {
            return "error: empty list".into();
        }
        let m = mean_of(&values);
        let mad: f64 = values.iter().map(|v| (v - m).abs()).sum::<f64>() / values.len() as f64;
        format!("{mad}")
    }
}
