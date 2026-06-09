# syntax=docker/dockerfile:1

# ── Stage 1: chef base ────────────────────────────────────────────────────────
# cargo-chef lets us cache the compiled dependency tree as its own layer, so a
# source-only change doesn't trigger a full rebuild of every crate. The base is
# Debian bookworm (glibc) to match the distroless/cc-debian12 runtime below.
FROM lukemathwalker/cargo-chef:latest-rust-1.95 AS chef
WORKDIR /build

# ── Stage 2: plan ─────────────────────────────────────────────────────────────
# Produce a dependency "recipe" from the manifests + sources. This layer's cache
# key is the recipe, so it only changes when the dependency graph changes.
FROM chef AS planner
COPY Cargo.toml Cargo.lock ./
COPY src ./src
RUN cargo chef prepare --recipe-path recipe.json

# ── Stage 3: build ────────────────────────────────────────────────────────────
FROM chef AS builder
# Cook (compile) just the dependencies — cached unless recipe.json changes.
COPY --from=planner /build/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json
# Now build the application against the pre-compiled dependency layer.
COPY Cargo.toml Cargo.lock ./
COPY src ./src
RUN cargo build --release --locked --bin mcp-math-operations

# ── Stage 4: distroless runtime ───────────────────────────────────────────────
# No shell, no package manager; just glibc/libgcc for the optimized binary.
FROM gcr.io/distroless/cc-debian13:nonroot
COPY --from=builder /build/target/release/mcp-math-operations /app/mcp-math-operations

ENV BIND_ADDR=0.0.0.0:3000 \
    RUST_LOG=info
EXPOSE 3000

ENTRYPOINT ["/app/mcp-math-operations"]
