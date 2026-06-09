use rmcp::{handler::server::wrapper::Parameters, tool, tool_router};
use schemars::JsonSchema;
use serde::Deserialize;

use crate::server::MathServer;

#[derive(Deserialize, JsonSchema)]
pub struct TwoFloats {
    pub a: f64,
    pub b: f64,
}
#[derive(Deserialize, JsonSchema)]
pub struct OneFloat {
    pub x: f64,
}
#[derive(Deserialize, JsonSchema)]
pub struct TwoInts {
    pub a: i64,
    pub b: i64,
}
#[derive(Deserialize, JsonSchema)]
pub struct OneInt {
    pub n: i64,
}
#[derive(Deserialize, JsonSchema)]
pub struct OneUInt {
    pub n: u64,
}
#[derive(Deserialize, JsonSchema)]
pub struct PowerParams {
    pub base: f64,
    pub exponent: f64,
}
#[derive(Deserialize, JsonSchema)]
pub struct NthRootParams {
    pub x: f64,
    pub n: f64,
}
#[derive(Deserialize, JsonSchema)]
pub struct LogBaseParams {
    pub x: f64,
    pub base: f64,
}
#[derive(Deserialize, JsonSchema)]
pub struct ClampParams {
    pub x: f64,
    pub min: f64,
    pub max: f64,
}
#[derive(Deserialize, JsonSchema)]
pub struct ModuloParams {
    pub dividend: f64,
    pub divisor: f64,
}
#[derive(Deserialize, JsonSchema)]
pub struct BitwiseParams {
    pub a: i64,
    pub b: i64,
}
#[derive(Deserialize, JsonSchema)]
pub struct ShiftParams {
    pub value: i64,
    pub shift: u32,
}

fn fmt(v: f64) -> String {
    if v.fract() == 0.0 && v.abs() < 1e15 && v.is_finite() {
        format!("{}", v as i64)
    } else {
        format!("{v}")
    }
}

fn gcd_u64(mut a: u64, mut b: u64) -> u64 {
    while b != 0 {
        let t = b;
        b = a % b;
        a = t;
    }
    a
}

fn gcd_i64(a: i64, b: i64) -> i64 {
    gcd_u64(a.unsigned_abs(), b.unsigned_abs()) as i64
}

#[tool_router(router = arithmetic_router, vis = "pub")]
impl MathServer {
    #[tool(description = "Add: a + b")]
    fn add(&self, p: Parameters<TwoFloats>) -> String {
        fmt(p.0.a + p.0.b)
    }

    #[tool(description = "Subtract: a - b")]
    fn subtract(&self, p: Parameters<TwoFloats>) -> String {
        fmt(p.0.a - p.0.b)
    }

    #[tool(description = "Multiply: a * b")]
    fn multiply(&self, p: Parameters<TwoFloats>) -> String {
        fmt(p.0.a * p.0.b)
    }

    #[tool(description = "Divide: a / b (error if b = 0)")]
    fn divide(&self, p: Parameters<TwoFloats>) -> String {
        if p.0.b == 0.0 {
            "error: division by zero".into()
        } else {
            fmt(p.0.a / p.0.b)
        }
    }

    #[tool(description = "Modulo: dividend mod divisor")]
    fn modulo(&self, p: Parameters<ModuloParams>) -> String {
        if p.0.divisor == 0.0 {
            "error: modulo by zero".into()
        } else {
            fmt(p.0.dividend % p.0.divisor)
        }
    }

    #[tool(description = "Power: base ^ exponent")]
    fn power(&self, p: Parameters<PowerParams>) -> String {
        fmt(p.0.base.powf(p.0.exponent))
    }

    #[tool(description = "Absolute value: |x|")]
    fn abs(&self, p: Parameters<OneFloat>) -> String {
        fmt(p.0.x.abs())
    }

    #[tool(description = "Negate: -x")]
    fn negate(&self, p: Parameters<OneFloat>) -> String {
        fmt(-p.0.x)
    }

    #[tool(description = "Square root: √x (error if x < 0)")]
    fn sqrt(&self, p: Parameters<OneFloat>) -> String {
        if p.0.x < 0.0 {
            "error: sqrt of negative".into()
        } else {
            fmt(p.0.x.sqrt())
        }
    }

    #[tool(description = "Cube root: ∛x")]
    fn cbrt(&self, p: Parameters<OneFloat>) -> String {
        fmt(p.0.x.cbrt())
    }

    #[tool(description = "Nth root: x^(1/n)")]
    fn nth_root(&self, p: Parameters<NthRootParams>) -> String {
        if p.0.n == 0.0 {
            "error: zeroth root undefined".into()
        } else {
            fmt(p.0.x.powf(1.0 / p.0.n))
        }
    }

    #[tool(description = "Natural log: ln(x)")]
    fn ln(&self, p: Parameters<OneFloat>) -> String {
        if p.0.x <= 0.0 {
            "error: ln of non-positive".into()
        } else {
            fmt(p.0.x.ln())
        }
    }

    #[tool(description = "Base-2 log: log₂(x)")]
    fn log2(&self, p: Parameters<OneFloat>) -> String {
        if p.0.x <= 0.0 {
            "error: log2 of non-positive".into()
        } else {
            fmt(p.0.x.log2())
        }
    }

    #[tool(description = "Base-10 log: log₁₀(x)")]
    fn log10(&self, p: Parameters<OneFloat>) -> String {
        if p.0.x <= 0.0 {
            "error: log10 of non-positive".into()
        } else {
            fmt(p.0.x.log10())
        }
    }

    #[tool(description = "Logarithm with arbitrary base: log_base(x)")]
    fn log_base(&self, p: Parameters<LogBaseParams>) -> String {
        let LogBaseParams { x, base } = p.0;
        if x <= 0.0 {
            return "error: log of non-positive".into();
        }
        if base <= 0.0 || base == 1.0 {
            return "error: invalid base".into();
        }
        fmt(x.log(base))
    }

    #[tool(description = "Floor: largest integer ≤ x")]
    fn floor(&self, p: Parameters<OneFloat>) -> String {
        fmt(p.0.x.floor())
    }

    #[tool(description = "Ceiling: smallest integer ≥ x")]
    fn ceil(&self, p: Parameters<OneFloat>) -> String {
        fmt(p.0.x.ceil())
    }

    #[tool(description = "Round to nearest integer (half away from zero)")]
    fn round(&self, p: Parameters<OneFloat>) -> String {
        fmt(p.0.x.round())
    }

    #[tool(description = "Truncate toward zero")]
    fn trunc(&self, p: Parameters<OneFloat>) -> String {
        fmt(p.0.x.trunc())
    }

    #[tool(description = "Fractional part of x")]
    fn fract(&self, p: Parameters<OneFloat>) -> String {
        format!("{}", p.0.x.fract())
    }

    #[tool(description = "Clamp x to [min, max]")]
    fn clamp(&self, p: Parameters<ClampParams>) -> String {
        let ClampParams { x, min, max } = p.0;
        fmt(x.clamp(min, max))
    }

    #[tool(description = "Minimum of a and b")]
    fn min(&self, p: Parameters<TwoFloats>) -> String {
        fmt(p.0.a.min(p.0.b))
    }

    #[tool(description = "Maximum of a and b")]
    fn max(&self, p: Parameters<TwoFloats>) -> String {
        fmt(p.0.a.max(p.0.b))
    }

    #[tool(description = "Sign of x: -1, 0, or 1")]
    fn sign(&self, p: Parameters<OneFloat>) -> String {
        if p.0.x < 0.0 {
            "-1"
        } else if p.0.x > 0.0 {
            "1"
        } else {
            "0"
        }
        .into()
    }

    #[tool(description = "Reciprocal: 1/x")]
    fn reciprocal(&self, p: Parameters<OneFloat>) -> String {
        if p.0.x == 0.0 {
            "error: reciprocal of zero".into()
        } else {
            fmt(1.0 / p.0.x)
        }
    }

    #[tool(description = "Hypotenuse: √(a² + b²)")]
    fn hypot(&self, p: Parameters<TwoFloats>) -> String {
        fmt(p.0.a.hypot(p.0.b))
    }

    #[tool(description = "e^x")]
    fn exp(&self, p: Parameters<OneFloat>) -> String {
        fmt(p.0.x.exp())
    }

    #[tool(description = "2^x")]
    fn exp2(&self, p: Parameters<OneFloat>) -> String {
        fmt(p.0.x.exp2())
    }

    #[tool(description = "e^x - 1, stable near x = 0")]
    fn exp_m1(&self, p: Parameters<OneFloat>) -> String {
        fmt(p.0.x.exp_m1())
    }

    #[tool(description = "ln(1 + x), stable near x = 0 (requires x > -1)")]
    fn ln_1p(&self, p: Parameters<OneFloat>) -> String {
        if p.0.x <= -1.0 {
            "error: argument must be > -1".into()
        } else {
            fmt(p.0.x.ln_1p())
        }
    }

    // ── Integer / number theory ──────────────────────────────────────────────

    #[tool(description = "Greatest common divisor of two integers")]
    fn gcd(&self, p: Parameters<TwoInts>) -> String {
        format!("{}", gcd_i64(p.0.a, p.0.b))
    }

    #[tool(description = "Least common multiple of two integers")]
    fn lcm(&self, p: Parameters<TwoInts>) -> String {
        let (a, b) = (p.0.a, p.0.b);
        let g = gcd_i64(a, b);
        if g == 0 {
            return "0".into();
        }
        match (a / g).checked_mul(b) {
            Some(v) => format!("{}", v.abs()),
            None => "error: overflow".into(),
        }
    }

    #[tool(description = "Factorial n! (n ≤ 20)")]
    fn factorial(&self, p: Parameters<OneUInt>) -> String {
        let n = p.0.n;
        if n > 20 {
            return "error: n must be ≤ 20".into();
        }
        format!("{}", (1u64..=n).product::<u64>())
    }

    #[tool(description = "Nth Fibonacci number (n ≤ 93)")]
    fn fibonacci(&self, p: Parameters<OneUInt>) -> String {
        let n = p.0.n;
        if n > 93 {
            return "error: n must be ≤ 93".into();
        }
        if n == 0 {
            return "0".into();
        }
        let (mut a, mut b) = (0u64, 1u64);
        for _ in 1..n {
            (a, b) = (b, a + b);
        }
        format!("{b}")
    }

    #[tool(description = "Primality test: true if n is prime")]
    fn is_prime(&self, p: Parameters<OneUInt>) -> String {
        let n = p.0.n;
        if n < 2 {
            return "false".into();
        }
        if n == 2 {
            return "true".into();
        }
        if n.is_multiple_of(2) {
            return "false".into();
        }
        let limit = (n as f64).sqrt() as u64 + 1;
        for i in (3..limit).step_by(2) {
            if n.is_multiple_of(i) {
                return "false".into();
            }
        }
        "true".into()
    }

    #[tool(description = "Prime factorization of n (JSON array of prime factors)")]
    fn prime_factors(&self, p: Parameters<OneUInt>) -> String {
        let mut n = p.0.n;
        if n < 2 {
            return "[]".into();
        }
        let mut factors = Vec::new();
        let mut d = 2u64;
        while d * d <= n {
            while n.is_multiple_of(d) {
                factors.push(d);
                n /= d;
            }
            d += 1;
        }
        if n > 1 {
            factors.push(n);
        }
        format!("{factors:?}")
    }

    #[tool(description = "All divisors of n in ascending order (JSON array)")]
    fn divisors(&self, p: Parameters<OneUInt>) -> String {
        let n = p.0.n;
        if n == 0 {
            return "[]".into();
        }
        let mut divs: Vec<u64> = (1..=n).filter(|&i| n.is_multiple_of(i)).collect();
        divs.sort_unstable();
        format!("{divs:?}")
    }

    #[tool(description = "Integer sqrt: largest k with k² ≤ n")]
    fn isqrt(&self, p: Parameters<OneUInt>) -> String {
        format!("{}", (p.0.n as f64).sqrt() as u64)
    }

    #[tool(description = "Integer division: returns 'quotient remainder'")]
    fn div_rem(&self, p: Parameters<TwoInts>) -> String {
        let (a, b) = (p.0.a, p.0.b);
        if b == 0 {
            "error: division by zero".into()
        } else {
            format!("{} {}", a / b, a % b)
        }
    }

    // ── Bitwise ──────────────────────────────────────────────────────────────

    #[tool(description = "Bitwise AND: a & b")]
    fn bitwise_and(&self, p: Parameters<BitwiseParams>) -> String {
        format!("{}", p.0.a & p.0.b)
    }

    #[tool(description = "Bitwise OR: a | b")]
    fn bitwise_or(&self, p: Parameters<BitwiseParams>) -> String {
        format!("{}", p.0.a | p.0.b)
    }

    #[tool(description = "Bitwise XOR: a ^ b")]
    fn bitwise_xor(&self, p: Parameters<BitwiseParams>) -> String {
        format!("{}", p.0.a ^ p.0.b)
    }

    #[tool(description = "Bitwise NOT: ~n")]
    fn bitwise_not(&self, p: Parameters<OneInt>) -> String {
        format!("{}", !p.0.n)
    }

    #[tool(description = "Left bit shift: value << shift (shift < 64)")]
    fn shl(&self, p: Parameters<ShiftParams>) -> String {
        if p.0.shift >= 64 {
            "error: shift ≥ 64".into()
        } else {
            format!("{}", p.0.value << p.0.shift)
        }
    }

    #[tool(description = "Right bit shift: value >> shift (shift < 64)")]
    fn shr(&self, p: Parameters<ShiftParams>) -> String {
        if p.0.shift >= 64 {
            "error: shift ≥ 64".into()
        } else {
            format!("{}", p.0.value >> p.0.shift)
        }
    }

    #[tool(description = "Population count: number of set bits")]
    fn popcount(&self, p: Parameters<OneInt>) -> String {
        format!("{}", (p.0.n as u64).count_ones())
    }

    #[tool(description = "Leading zeros in the 64-bit representation")]
    fn leading_zeros(&self, p: Parameters<OneInt>) -> String {
        format!("{}", (p.0.n as u64).leading_zeros())
    }

    #[tool(description = "Trailing zeros in the 64-bit representation")]
    fn trailing_zeros(&self, p: Parameters<OneInt>) -> String {
        format!("{}", (p.0.n as u64).trailing_zeros())
    }
}
