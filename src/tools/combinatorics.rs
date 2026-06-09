use rmcp::{handler::server::wrapper::Parameters, tool, tool_router};
use schemars::JsonSchema;
use serde::Deserialize;

use crate::server::MathServer;

#[derive(Deserialize, JsonSchema)]
pub struct NkParams {
    pub n: u64,
    pub k: u64,
}
#[derive(Deserialize, JsonSchema)]
pub struct OneUInt {
    pub n: u64,
}
#[derive(Deserialize, JsonSchema)]
pub struct MultinomialParams {
    pub ks: Vec<u64>,
}

fn factorial_big(n: u64) -> Option<u128> {
    if n > 34 {
        return None;
    }
    Some((1..=n as u128).product())
}

fn binom(n: u64, k: u64) -> Option<u128> {
    if k > n {
        return Some(0);
    }
    let k = k.min(n - k);
    let mut r = 1u128;
    for i in 0..k {
        r = r
            .checked_mul(n as u128 - i as u128)?
            .checked_div(i as u128 + 1)?;
    }
    Some(r)
}

#[tool_router(router = combinatorics_router, vis = "pub")]
impl MathServer {
    #[tool(description = "Binomial coefficient C(n,k) = n! / (k!(n-k)!)")]
    fn combinations(&self, p: Parameters<NkParams>) -> String {
        match binom(p.0.n, p.0.k) {
            Some(v) => format!("{v}"),
            None => "error: overflow".into(),
        }
    }

    #[tool(description = "Permutations P(n,k) = n!/(n-k)!")]
    fn permutations(&self, p: Parameters<NkParams>) -> String {
        let NkParams { n, k } = p.0;
        if k > n {
            return "0".into();
        }
        let mut r = 1u128;
        for i in 0..k {
            r = match r.checked_mul(n as u128 - i as u128) {
                Some(v) => v,
                None => return "error: overflow".into(),
            };
        }
        format!("{r}")
    }

    #[tool(description = "Multiset coefficient (combinations with repetition): C(n+k-1, k)")]
    fn multiset_coefficient(&self, p: Parameters<NkParams>) -> String {
        match binom(p.0.n + p.0.k - 1, p.0.k) {
            Some(v) => format!("{v}"),
            None => "error: overflow".into(),
        }
    }

    #[tool(description = "Catalan number Cₙ = C(2n,n) / (n+1)")]
    fn catalan(&self, p: Parameters<OneUInt>) -> String {
        let n = p.0.n;
        match binom(2 * n, n) {
            Some(v) => format!("{}", v / (n as u128 + 1)),
            None => "error: overflow".into(),
        }
    }

    #[tool(
        description = "Stirling number of the second kind S(n,k): partitions of n into k non-empty subsets"
    )]
    fn stirling_second(&self, p: Parameters<NkParams>) -> String {
        let NkParams { n, k } = p.0;
        if k == 0 {
            return if n == 0 { "1".into() } else { "0".into() };
        }
        if n == 0 || k > n {
            return "0".into();
        }
        let mut sum = 0i128;
        for j in 0..=k {
            let sign: i128 = if (k - j) % 2 == 0 { 1 } else { -1 };
            let c = match binom(k, j) {
                Some(v) => v as i128,
                None => return "error: overflow".into(),
            };
            sum += sign * c * (j as i128).pow(n as u32);
        }
        let kf = match factorial_big(k) {
            Some(v) => v as i128,
            None => return "error: overflow".into(),
        };
        format!("{}", sum / kf)
    }

    #[tool(description = "Bell number Bₙ: total number of partitions of n elements (n ≤ 15)")]
    fn bell(&self, p: Parameters<OneUInt>) -> String {
        let n = p.0.n;
        if n > 15 {
            return "error: n must be ≤ 15".into();
        }
        let n = n as usize;
        let mut row = vec![1u128; n + 1];
        for _ in 0..n {
            let mut next = vec![0u128; n + 1];
            next[0] = row[n];
            for j in 1..=n {
                next[j] = next[j - 1] + row[j - 1];
            }
            row = next;
        }
        format!("{}", row[0])
    }

    #[tool(description = "Derangement count D(n): permutations with no fixed points")]
    fn derangements(&self, p: Parameters<OneUInt>) -> String {
        let n = p.0.n;
        if n == 0 {
            return "1".into();
        }
        if n == 1 {
            return "0".into();
        }
        let (mut a, mut b) = (0u128, 1u128);
        for i in 2..=n {
            let next = (i - 1) as u128 * (a + b);
            a = b;
            b = next;
        }
        format!("{b}")
    }

    #[tool(description = "Pascal's triangle row n (0-indexed), returns JSON array (n ≤ 30)")]
    fn pascals_row(&self, p: Parameters<OneUInt>) -> String {
        let n = p.0.n;
        if n > 30 {
            return "error: n must be ≤ 30".into();
        }
        let row: Vec<u128> = (0..=n).map(|k| binom(n, k).unwrap_or(0)).collect();
        format!("{row:?}")
    }

    #[tool(description = "Surjections (onto functions) from n to k")]
    fn surjections(&self, p: Parameters<NkParams>) -> String {
        let NkParams { n, k } = p.0;
        if k > n {
            return "0".into();
        }
        let mut sum = 0i128;
        for j in 0..=k {
            let sign: i128 = if j % 2 == 0 { 1 } else { -1 };
            let c = match binom(k, j) {
                Some(v) => v as i128,
                None => return "error: overflow".into(),
            };
            sum += sign * c * ((k - j) as i128).pow(n as u32);
        }
        format!("{sum}")
    }

    #[tool(description = "Multinomial coefficient n!/(k₁!·k₂!·…) where ks sums to n")]
    fn multinomial(&self, p: Parameters<MultinomialParams>) -> String {
        let ks = p.0.ks;
        let n: u64 = ks.iter().sum();
        let num = match factorial_big(n) {
            Some(v) => v,
            None => return "error: n too large".into(),
        };
        let mut den = 1u128;
        for &k in &ks {
            match factorial_big(k) {
                Some(kf) => match den.checked_mul(kf) {
                    Some(v) => den = v,
                    None => return "error: overflow".into(),
                },
                None => return "error: overflow".into(),
            }
        }
        format!("{}", num / den)
    }
}
