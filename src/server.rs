use rmcp::{
    ErrorData as McpError, RoleServer, ServerHandler,
    model::{
        Annotated, Implementation, ListResourcesResult, PaginatedRequestParams, RawResource,
        ReadResourceRequestParams, ReadResourceResult, ResourceContents, ServerCapabilities,
        ServerInfo,
    },
    service::RequestContext,
    tool_handler, tool_router,
};
use std::f64::consts;

pub struct MathServer;

impl MathServer {
    pub fn new() -> Self {
        Self
    }
}

// ── Ping ─────────────────────────────────────────────────────────────────────

#[tool_router(router = base_router, vis = "pub")]
impl MathServer {
    #[rmcp::tool(description = "Ping: returns 'pong'")]
    fn ping(&self) -> String {
        "pong".to_string()
    }
}

// ── Math constants as MCP resources ──────────────────────────────────────────

/// A constant resource: `(uri, human description, value provider)`.
type Constant = (&'static str, &'static str, fn() -> f64);

const CONSTANTS: &[Constant] = &[
    ("math://constants/pi", "π = 3.14159…", || consts::PI),
    (
        "math://constants/e",
        "e = 2.71828… (Euler's number)",
        || consts::E,
    ),
    ("math://constants/tau", "τ = 2π = 6.28318…", || {
        consts::TAU
    }),
    ("math://constants/sqrt2", "√2 = 1.41421…", || {
        consts::SQRT_2
    }),
    ("math://constants/sqrt3", "√3 = 1.73205…", || {
        3.0f64.sqrt()
    }),
    ("math://constants/sqrt5", "√5 = 2.23606…", || {
        5.0f64.sqrt()
    }),
    ("math://constants/ln2", "ln(2) = 0.69314…", || {
        consts::LN_2
    }),
    ("math://constants/ln10", "ln(10) = 2.30258…", || {
        consts::LN_10
    }),
    ("math://constants/log2_e", "log₂(e) = 1.44269…", || {
        consts::LOG2_E
    }),
    (
        "math://constants/log10_e",
        "log₁₀(e) = 0.43429…",
        || consts::LOG10_E,
    ),
    (
        "math://constants/frac_1_sqrt2",
        "1/√2 = 0.70710…",
        || consts::FRAC_1_SQRT_2,
    ),
    ("math://constants/frac_pi_2", "π/2 = 1.57079…", || {
        consts::FRAC_PI_2
    }),
    ("math://constants/frac_pi_3", "π/3 = 1.04719…", || {
        consts::FRAC_PI_3
    }),
    ("math://constants/frac_pi_4", "π/4 = 0.78539…", || {
        consts::FRAC_PI_4
    }),
    ("math://constants/frac_pi_6", "π/6 = 0.52359…", || {
        consts::FRAC_PI_6
    }),
    ("math://constants/frac_pi_8", "π/8 = 0.39269…", || {
        consts::FRAC_PI_8
    }),
    (
        "math://constants/golden_ratio",
        "φ = (1+√5)/2 = 1.61803…",
        || (1.0 + 5.0f64.sqrt()) / 2.0,
    ),
    (
        "math://constants/silver_ratio",
        "δ_S = 1+√2 = 2.41421…",
        || 1.0 + 2.0f64.sqrt(),
    ),
    (
        "math://constants/euler_mascheroni",
        "γ (Euler–Mascheroni) = 0.57721…",
        || 0.577_215_664_901_532_9,
    ),
    (
        "math://constants/apery",
        "ζ(3) Apéry = 1.20205…",
        || 1.202_056_903_159_594_2,
    ),
    (
        "math://constants/catalan",
        "G (Catalan) = 0.91596…",
        || 0.915_965_594_177_219,
    ),
    (
        "math://constants/glaisher_kinkelin",
        "A (Glaisher–Kinkelin) = 1.28242…",
        || 1.282_427_129_100_622_6,
    ),
    (
        "math://constants/khinchin",
        "K (Khinchin) = 2.68545…",
        || 2.685_452_001_065_306_4,
    ),
    ("math://constants/omega", "Ω = W(1) = 0.56714…", || {
        0.567_143_290_409_783_8
    }),
    (
        "math://constants/plastic",
        "ρ (plastic) = 1.32471…",
        || 1.324_717_957_244_746,
    ),
    (
        "math://constants/feigenbaum_delta",
        "δ (Feigenbaum δ) = 4.66920…",
        || 4.669_201_609_102_99,
    ),
    (
        "math://constants/feigenbaum_alpha",
        "α (Feigenbaum α) = 2.50290…",
        || 2.502_907_875_095_892_8,
    ),
    (
        "math://constants/twin_prime",
        "C₂ (twin prime) = 0.66016…",
        || 0.660_161_815_846_869_5,
    ),
    (
        "math://constants/meissel_mertens",
        "M (Meissel–Mertens) = 0.26149…",
        || 0.261_497_212_847_642_8,
    ),
    (
        "math://constants/f64_epsilon",
        "Machine epsilon for f64",
        || f64::EPSILON,
    ),
    ("math://constants/f64_max", "Maximum finite f64", || {
        f64::MAX
    }),
    (
        "math://constants/f64_min_positive",
        "Minimum positive f64",
        || f64::MIN_POSITIVE,
    ),
    ("math://constants/inf", "+∞ (positive infinity)", || {
        f64::INFINITY
    }),
];

fn make_resource(uri: &'static str, description: &'static str) -> Annotated<RawResource> {
    Annotated::new(
        RawResource::new(uri, description)
            .with_description(description)
            .with_mime_type("text/plain"),
        None,
    )
}

// ── ServerHandler ─────────────────────────────────────────────────────────────

#[tool_handler(
    router = (Self::base_router()
        + Self::arithmetic_router()
        + Self::trig_router()
        + Self::statistics_router()
        + Self::probability_router()
        + Self::combinatorics_router()
        + Self::graphs_router())
)]
impl ServerHandler for MathServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo::new(
            ServerCapabilities::builder()
                .enable_tools()
                .enable_resources()
                .build(),
        )
        .with_server_info(Implementation::new(
            env!("CARGO_PKG_NAME"),
            env!("CARGO_PKG_VERSION"),
        ))
        .with_instructions(
            "A comprehensive math server. Use list_resources to browse ~34 mathematical \
             constants (URI scheme math://constants/…). Tools cover arithmetic, \
             trigonometry, statistics, probability distributions, combinatorics, and \
             2D/3D geographic graphing (SVG output).",
        )
    }

    async fn list_resources(
        &self,
        _request: Option<PaginatedRequestParams>,
        _ctx: RequestContext<RoleServer>,
    ) -> Result<ListResourcesResult, McpError> {
        let resources = CONSTANTS
            .iter()
            .map(|(uri, desc, _)| make_resource(uri, desc))
            .collect();
        Ok(ListResourcesResult::with_all_items(resources))
    }

    async fn read_resource(
        &self,
        request: ReadResourceRequestParams,
        _ctx: RequestContext<RoleServer>,
    ) -> Result<ReadResourceResult, McpError> {
        let uri = request.uri.as_str();
        if let Some(&(_, desc, f)) = CONSTANTS.iter().find(|(u, _, _)| *u == uri) {
            return Ok(ReadResourceResult::new(vec![ResourceContents::text(
                format!("{desc}: {}", f()),
                uri,
            )]));
        }
        Err(McpError::resource_not_found(
            "resource_not_found",
            Some(serde_json::json!({ "uri": uri })),
        ))
    }
}
