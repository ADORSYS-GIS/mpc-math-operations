use rmcp::{handler::server::wrapper::Parameters, tool, tool_router};
use schemars::JsonSchema;
use serde::Deserialize;
use statrs::distribution::Continuous;
use statrs::distribution::{
    Beta, Binomial, Cauchy, ChiSquared, ContinuousCDF, Discrete, DiscreteCDF, Empirical, Exp,
    FisherSnedecor, Gamma, Geometric, Hypergeometric, Laplace, LogNormal, NegativeBinomial, Normal,
    Poisson, StudentsT, Triangular, Uniform, Weibull,
};
use statrs::statistics::Distribution;

use crate::server::MathServer;

macro_rules! dist_or_err {
    ($expr:expr) => {
        match $expr {
            Ok(d) => d,
            Err(e) => return format!("error: {e}"),
        }
    };
}

#[derive(Deserialize, JsonSchema)]
pub struct NormalParams {
    pub mean: f64,
    pub std_dev: f64,
    pub x: f64,
}
#[derive(Deserialize, JsonSchema)]
pub struct NormalQuantileParams {
    pub mean: f64,
    pub std_dev: f64,
    pub p: f64,
}
#[derive(Deserialize, JsonSchema)]
pub struct StdNormalParam {
    pub x: f64,
}
#[derive(Deserialize, JsonSchema)]
pub struct BinomialParams {
    pub n: u64,
    pub p: f64,
    pub k: u64,
}
#[derive(Deserialize, JsonSchema)]
pub struct BinomialStatsParams {
    pub n: u64,
    pub p: f64,
}
#[derive(Deserialize, JsonSchema)]
pub struct PoissonParams {
    pub lambda: f64,
    pub k: u64,
}
#[derive(Deserialize, JsonSchema)]
pub struct ExpParams {
    /// Rate λ > 0
    pub lambda: f64,
    pub x: f64,
}
#[derive(Deserialize, JsonSchema)]
pub struct BetaParams {
    pub alpha: f64,
    pub beta: f64,
    pub x: f64,
}
#[derive(Deserialize, JsonSchema)]
pub struct GammaParams {
    pub alpha: f64,
    pub beta: f64,
    pub x: f64,
}
#[derive(Deserialize, JsonSchema)]
pub struct ChiSquaredParams {
    pub k: f64,
    pub x: f64,
}
#[derive(Deserialize, JsonSchema)]
pub struct StudentsTParams {
    pub nu: f64,
    pub x: f64,
}
#[derive(Deserialize, JsonSchema)]
pub struct FParams {
    pub d1: f64,
    pub d2: f64,
    pub x: f64,
}
#[derive(Deserialize, JsonSchema)]
pub struct GeometricParams {
    pub p: f64,
    pub k: u64,
}
#[derive(Deserialize, JsonSchema)]
pub struct HypergeometricParams {
    pub population: u64,
    pub successes: u64,
    pub draws: u64,
    pub k: u64,
}
#[derive(Deserialize, JsonSchema)]
pub struct NegBinomParams {
    pub r: f64,
    pub p: f64,
    pub k: u64,
}
#[derive(Deserialize, JsonSchema)]
pub struct WeibullParams {
    pub k: f64,
    pub lambda: f64,
    pub x: f64,
}
#[derive(Deserialize, JsonSchema)]
pub struct LogNormalParams {
    pub mu: f64,
    pub sigma: f64,
    pub x: f64,
}
#[derive(Deserialize, JsonSchema)]
pub struct CauchyParams {
    pub x0: f64,
    pub gamma: f64,
    pub x: f64,
}
#[derive(Deserialize, JsonSchema)]
pub struct LaplaceParams {
    pub mu: f64,
    pub b: f64,
    pub x: f64,
}
#[derive(Deserialize, JsonSchema)]
pub struct UniformParams {
    pub min: f64,
    pub max: f64,
    pub x: f64,
}
#[derive(Deserialize, JsonSchema)]
pub struct TriangularParams {
    pub min: f64,
    pub max: f64,
    pub mode: f64,
    pub x: f64,
}
#[derive(Deserialize, JsonSchema)]
pub struct EcdfParams {
    pub values: Vec<f64>,
    pub x: f64,
}

#[tool_router(router = probability_router, vis = "pub")]
impl MathServer {
    // ── Normal ────────────────────────────────────────────────────────────────

    #[tool(description = "Normal PDF: density at x for N(mean, std_dev²)")]
    fn normal_pdf(&self, p: Parameters<NormalParams>) -> String {
        let NormalParams { mean, std_dev, x } = p.0;
        format!("{}", dist_or_err!(Normal::new(mean, std_dev)).pdf(x))
    }

    #[tool(description = "Normal CDF: P(X ≤ x) for N(mean, std_dev²)")]
    fn normal_cdf(&self, p: Parameters<NormalParams>) -> String {
        let NormalParams { mean, std_dev, x } = p.0;
        format!("{}", dist_or_err!(Normal::new(mean, std_dev)).cdf(x))
    }

    #[tool(description = "Normal inverse CDF (quantile): x s.t. P(X ≤ x) = p")]
    fn normal_quantile(&self, p: Parameters<NormalQuantileParams>) -> String {
        let NormalQuantileParams { mean, std_dev, p } = p.0;
        format!(
            "{}",
            dist_or_err!(Normal::new(mean, std_dev)).inverse_cdf(p)
        )
    }

    #[tool(description = "Standard normal PDF φ(x) for N(0,1)")]
    fn standard_normal_pdf(&self, p: Parameters<StdNormalParam>) -> String {
        format!("{}", Normal::new(0.0, 1.0).unwrap().pdf(p.0.x))
    }

    #[tool(description = "Standard normal CDF Φ(x) for N(0,1)")]
    fn standard_normal_cdf(&self, p: Parameters<StdNormalParam>) -> String {
        format!("{}", Normal::new(0.0, 1.0).unwrap().cdf(p.0.x))
    }

    // ── Binomial ──────────────────────────────────────────────────────────────

    #[tool(description = "Binomial PMF: P(X = k) for Binomial(n, p)")]
    fn binomial_pmf(&self, p: Parameters<BinomialParams>) -> String {
        let BinomialParams { n, p, k } = p.0;
        format!("{}", dist_or_err!(Binomial::new(p, n)).pmf(k))
    }

    #[tool(description = "Binomial CDF: P(X ≤ k) for Binomial(n, p)")]
    fn binomial_cdf(&self, p: Parameters<BinomialParams>) -> String {
        let BinomialParams { n, p, k } = p.0;
        format!("{}", dist_or_err!(Binomial::new(p, n)).cdf(k))
    }

    #[tool(description = "Binomial mean and variance: 'mean=… variance=…'")]
    fn binomial_stats(&self, p: Parameters<BinomialStatsParams>) -> String {
        let BinomialStatsParams { n, p } = p.0;
        let d = dist_or_err!(Binomial::new(p, n));
        format!(
            "mean={} variance={}",
            d.mean().unwrap_or(f64::NAN),
            d.variance().unwrap_or(f64::NAN)
        )
    }

    // ── Poisson ───────────────────────────────────────────────────────────────

    #[tool(description = "Poisson PMF: P(X = k) for Poisson(λ)")]
    fn poisson_pmf(&self, p: Parameters<PoissonParams>) -> String {
        let PoissonParams { lambda, k } = p.0;
        format!("{}", dist_or_err!(Poisson::new(lambda)).pmf(k))
    }

    #[tool(description = "Poisson CDF: P(X ≤ k) for Poisson(λ)")]
    fn poisson_cdf(&self, p: Parameters<PoissonParams>) -> String {
        let PoissonParams { lambda, k } = p.0;
        format!("{}", dist_or_err!(Poisson::new(lambda)).cdf(k))
    }

    // ── Exponential ───────────────────────────────────────────────────────────

    #[tool(description = "Exponential PDF: λ·e^(-λx) for x ≥ 0")]
    fn exponential_pdf(&self, p: Parameters<ExpParams>) -> String {
        let ExpParams { lambda, x } = p.0;
        format!("{}", dist_or_err!(Exp::new(lambda)).pdf(x))
    }

    #[tool(description = "Exponential CDF: 1 - e^(-λx)")]
    fn exponential_cdf(&self, p: Parameters<ExpParams>) -> String {
        let ExpParams { lambda, x } = p.0;
        format!("{}", dist_or_err!(Exp::new(lambda)).cdf(x))
    }

    // ── Beta ──────────────────────────────────────────────────────────────────

    #[tool(description = "Beta PDF: density at x in [0,1] for Beta(α, β)")]
    fn beta_pdf(&self, p: Parameters<BetaParams>) -> String {
        let BetaParams { alpha, beta, x } = p.0;
        format!("{}", dist_or_err!(Beta::new(alpha, beta)).pdf(x))
    }

    #[tool(description = "Beta CDF: P(X ≤ x) for Beta(α, β)")]
    fn beta_cdf(&self, p: Parameters<BetaParams>) -> String {
        let BetaParams { alpha, beta, x } = p.0;
        format!("{}", dist_or_err!(Beta::new(alpha, beta)).cdf(x))
    }

    // ── Gamma ─────────────────────────────────────────────────────────────────

    #[tool(description = "Gamma PDF: density at x for Gamma(α, β) where β is the rate")]
    fn gamma_pdf(&self, p: Parameters<GammaParams>) -> String {
        let GammaParams { alpha, beta, x } = p.0;
        format!("{}", dist_or_err!(Gamma::new(alpha, 1.0 / beta)).pdf(x))
    }

    #[tool(description = "Gamma CDF: P(X ≤ x) for Gamma(α, β)")]
    fn gamma_cdf(&self, p: Parameters<GammaParams>) -> String {
        let GammaParams { alpha, beta, x } = p.0;
        format!("{}", dist_or_err!(Gamma::new(alpha, 1.0 / beta)).cdf(x))
    }

    // ── Chi-squared ───────────────────────────────────────────────────────────

    #[tool(description = "Chi-squared PDF: density at x for χ²(k)")]
    fn chi_squared_pdf(&self, p: Parameters<ChiSquaredParams>) -> String {
        let ChiSquaredParams { k, x } = p.0;
        format!("{}", dist_or_err!(ChiSquared::new(k)).pdf(x))
    }

    #[tool(description = "Chi-squared CDF: P(X ≤ x) for χ²(k)")]
    fn chi_squared_cdf(&self, p: Parameters<ChiSquaredParams>) -> String {
        let ChiSquaredParams { k, x } = p.0;
        format!("{}", dist_or_err!(ChiSquared::new(k)).cdf(x))
    }

    // ── Student's t ───────────────────────────────────────────────────────────

    #[tool(description = "Student's t PDF: density at x for t(ν)")]
    fn students_t_pdf(&self, p: Parameters<StudentsTParams>) -> String {
        let StudentsTParams { nu, x } = p.0;
        format!("{}", dist_or_err!(StudentsT::new(0.0, 1.0, nu)).pdf(x))
    }

    #[tool(description = "Student's t CDF: P(X ≤ x) for t(ν)")]
    fn students_t_cdf(&self, p: Parameters<StudentsTParams>) -> String {
        let StudentsTParams { nu, x } = p.0;
        format!("{}", dist_or_err!(StudentsT::new(0.0, 1.0, nu)).cdf(x))
    }

    // ── F-distribution ────────────────────────────────────────────────────────

    #[tool(description = "F-distribution PDF: density at x for F(d1, d2)")]
    fn f_pdf(&self, p: Parameters<FParams>) -> String {
        let FParams { d1, d2, x } = p.0;
        format!("{}", dist_or_err!(FisherSnedecor::new(d1, d2)).pdf(x))
    }

    #[tool(description = "F-distribution CDF: P(X ≤ x) for F(d1, d2)")]
    fn f_cdf(&self, p: Parameters<FParams>) -> String {
        let FParams { d1, d2, x } = p.0;
        format!("{}", dist_or_err!(FisherSnedecor::new(d1, d2)).cdf(x))
    }

    // ── Geometric ─────────────────────────────────────────────────────────────

    #[tool(description = "Geometric PMF: P(X = k), k trials until first success")]
    fn geometric_pmf(&self, p: Parameters<GeometricParams>) -> String {
        let GeometricParams { p, k } = p.0;
        format!("{}", dist_or_err!(Geometric::new(p)).pmf(k))
    }

    #[tool(description = "Geometric CDF: P(X ≤ k)")]
    fn geometric_cdf(&self, p: Parameters<GeometricParams>) -> String {
        let GeometricParams { p, k } = p.0;
        format!("{}", dist_or_err!(Geometric::new(p)).cdf(k))
    }

    // ── Hypergeometric ────────────────────────────────────────────────────────

    #[tool(description = "Hypergeometric PMF: P(X = k) drawing without replacement")]
    fn hypergeometric_pmf(&self, p: Parameters<HypergeometricParams>) -> String {
        let HypergeometricParams {
            population,
            successes,
            draws,
            k,
        } = p.0;
        format!(
            "{}",
            dist_or_err!(Hypergeometric::new(population, successes, draws)).pmf(k)
        )
    }

    #[tool(description = "Hypergeometric CDF: P(X ≤ k)")]
    fn hypergeometric_cdf(&self, p: Parameters<HypergeometricParams>) -> String {
        let HypergeometricParams {
            population,
            successes,
            draws,
            k,
        } = p.0;
        format!(
            "{}",
            dist_or_err!(Hypergeometric::new(population, successes, draws)).cdf(k)
        )
    }

    // ── Negative Binomial ─────────────────────────────────────────────────────

    #[tool(description = "Negative Binomial PMF: P(X = k failures before r successes)")]
    fn negative_binomial_pmf(&self, p: Parameters<NegBinomParams>) -> String {
        let NegBinomParams { r, p, k } = p.0;
        format!("{}", dist_or_err!(NegativeBinomial::new(r, p)).pmf(k))
    }

    #[tool(description = "Negative Binomial CDF: P(X ≤ k)")]
    fn negative_binomial_cdf(&self, p: Parameters<NegBinomParams>) -> String {
        let NegBinomParams { r, p, k } = p.0;
        format!("{}", dist_or_err!(NegativeBinomial::new(r, p)).cdf(k))
    }

    // ── Weibull ───────────────────────────────────────────────────────────────

    #[tool(description = "Weibull PDF: density at x for Weibull(k, λ)")]
    fn weibull_pdf(&self, p: Parameters<WeibullParams>) -> String {
        let WeibullParams { k, lambda, x } = p.0;
        format!("{}", dist_or_err!(Weibull::new(k, lambda)).pdf(x))
    }

    #[tool(description = "Weibull CDF: P(X ≤ x) for Weibull(k, λ)")]
    fn weibull_cdf(&self, p: Parameters<WeibullParams>) -> String {
        let WeibullParams { k, lambda, x } = p.0;
        format!("{}", dist_or_err!(Weibull::new(k, lambda)).cdf(x))
    }

    // ── Log-normal ────────────────────────────────────────────────────────────

    #[tool(description = "Log-normal PDF: density at x > 0 for LogNormal(μ, σ)")]
    fn lognormal_pdf(&self, p: Parameters<LogNormalParams>) -> String {
        let LogNormalParams { mu, sigma, x } = p.0;
        format!("{}", dist_or_err!(LogNormal::new(mu, sigma)).pdf(x))
    }

    #[tool(description = "Log-normal CDF: P(X ≤ x) for LogNormal(μ, σ)")]
    fn lognormal_cdf(&self, p: Parameters<LogNormalParams>) -> String {
        let LogNormalParams { mu, sigma, x } = p.0;
        format!("{}", dist_or_err!(LogNormal::new(mu, sigma)).cdf(x))
    }

    // ── Cauchy ────────────────────────────────────────────────────────────────

    #[tool(description = "Cauchy PDF: density at x for Cauchy(x₀, γ)")]
    fn cauchy_pdf(&self, p: Parameters<CauchyParams>) -> String {
        let CauchyParams { x0, gamma, x } = p.0;
        format!("{}", dist_or_err!(Cauchy::new(x0, gamma)).pdf(x))
    }

    #[tool(description = "Cauchy CDF: P(X ≤ x) for Cauchy(x₀, γ)")]
    fn cauchy_cdf(&self, p: Parameters<CauchyParams>) -> String {
        let CauchyParams { x0, gamma, x } = p.0;
        format!("{}", dist_or_err!(Cauchy::new(x0, gamma)).cdf(x))
    }

    // ── Laplace ───────────────────────────────────────────────────────────────

    #[tool(description = "Laplace PDF: density at x for Laplace(μ, b)")]
    fn laplace_pdf(&self, p: Parameters<LaplaceParams>) -> String {
        let LaplaceParams { mu, b, x } = p.0;
        format!("{}", dist_or_err!(Laplace::new(mu, b)).pdf(x))
    }

    #[tool(description = "Laplace CDF: P(X ≤ x) for Laplace(μ, b)")]
    fn laplace_cdf(&self, p: Parameters<LaplaceParams>) -> String {
        let LaplaceParams { mu, b, x } = p.0;
        format!("{}", dist_or_err!(Laplace::new(mu, b)).cdf(x))
    }

    // ── Uniform ───────────────────────────────────────────────────────────────

    #[tool(description = "Uniform PDF: density at x for Uniform(min, max)")]
    fn uniform_pdf(&self, p: Parameters<UniformParams>) -> String {
        let UniformParams { min, max, x } = p.0;
        format!("{}", dist_or_err!(Uniform::new(min, max)).pdf(x))
    }

    #[tool(description = "Uniform CDF: P(X ≤ x) for Uniform(min, max)")]
    fn uniform_cdf(&self, p: Parameters<UniformParams>) -> String {
        let UniformParams { min, max, x } = p.0;
        format!("{}", dist_or_err!(Uniform::new(min, max)).cdf(x))
    }

    // ── Triangular ────────────────────────────────────────────────────────────

    #[tool(description = "Triangular PDF: density at x for Triangular(min, max, mode)")]
    fn triangular_pdf(&self, p: Parameters<TriangularParams>) -> String {
        let TriangularParams { min, max, mode, x } = p.0;
        format!("{}", dist_or_err!(Triangular::new(min, max, mode)).pdf(x))
    }

    #[tool(description = "Triangular CDF: P(X ≤ x) for Triangular(min, max, mode)")]
    fn triangular_cdf(&self, p: Parameters<TriangularParams>) -> String {
        let TriangularParams { min, max, mode, x } = p.0;
        format!("{}", dist_or_err!(Triangular::new(min, max, mode)).cdf(x))
    }

    // ── Empirical ─────────────────────────────────────────────────────────────

    #[tool(description = "Empirical CDF: fraction of sample values ≤ x")]
    fn empirical_cdf(&self, p: Parameters<EcdfParams>) -> String {
        let EcdfParams { values, x } = p.0;
        if values.is_empty() {
            return "error: empty sample".into();
        }
        let emp: Empirical = values.into_iter().collect();
        format!("{}", emp.cdf(x))
    }
}
