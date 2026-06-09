# mcp-math-operations

An HTTP-streamable [Model Context Protocol](https://modelcontextprotocol.io)
server, written in Rust, exposing **165+ math tools** — arithmetic, trigonometry,
statistics, probability distributions, combinatorics — plus **2D/3D geographic
SVG plotting** (including superposition of multiple series), and ~34 mathematical
constants as MCP resources.

- **Transport:** Streamable HTTP (`rmcp`), mounted at `/mcp`.
- **Runtime:** single optimized binary on a distroless, non-root image.
- **Supply chain:** every release is signed (cosign), carries an SBOM and SLSA
  provenance, and is scanned with Trivy.

## Quick start (Docker Compose)

Runs the server alongside the MCP Inspector:

```bash
docker compose up --build
```

Open the Inspector at <http://localhost:6274>, choose **Streamable HTTP**, and
connect to `http://math:3000/mcp`.

## Run the image directly

```bash
docker run --rm -p 3000:3000 \
  -e ALLOWED_HOSTS=localhost \
  ghcr.io/adorsys-gis/mcp-math-operations:latest
```

### Configuration

| Env var         | Default        | Purpose                                                            |
| --------------- | -------------- | ------------------------------------------------------------------ |
| `BIND_ADDR`     | `0.0.0.0:3000` | Listen address.                                                    |
| `RUST_LOG`      | `info`         | Tracing filter.                                                    |
| `ALLOWED_HOSTS` | (loopback)     | Comma-separated extra `Host` values accepted by the DNS-rebinding guard. A bare host matches any port. |

## Deploy with Helm

The chart is published to GHCR as an OCI artifact:

```bash
helm install math oci://ghcr.io/adorsys-gis/charts/mcp-math-operations --version <X.Y.Z>
```

Reach it via port-forward (the chart already allows `localhost`):

```bash
kubectl port-forward svc/math-mcp-math-operations 3000:3000
```

Behind an ingress, add the hostname so the rebinding guard accepts it:

```bash
helm upgrade --install math oci://ghcr.io/adorsys-gis/charts/mcp-math-operations \
  --set ingress.enabled=true \
  --set 'ingress.hosts[0].host=mcp-math.example.com' \
  --set 'config.allowedHosts={mcp-math.example.com}'
```

The chart ships hardened defaults: non-root (uid 65532), read-only root
filesystem, all capabilities dropped, `RuntimeDefault` seccomp, and TCP probes.

## Verify a release before trusting it

```bash
# Image signature (keyless / GitHub OIDC)
cosign verify ghcr.io/adorsys-gis/mcp-math-operations:<X.Y.Z> \
  --certificate-identity-regexp 'https://github.com/.+/.github/workflows/.+' \
  --certificate-oidc-issuer https://token.actions.githubusercontent.com

# Provenance + SBOM attestations
cosign verify-attestation --type slsaprovenance ghcr.io/adorsys-gis/mcp-math-operations:<X.Y.Z> \
  --certificate-identity-regexp 'https://github.com/.+' \
  --certificate-oidc-issuer https://token.actions.githubusercontent.com
```

## Development

```bash
cargo run                       # serve on 0.0.0.0:3000
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
cargo test --all
cargo deny check                # supply-chain gate (cargo install cargo-deny)
```

Toolchain is pinned in `rust-toolchain.toml` (Rust 1.95, edition 2024).

## CI/CD

| Workflow      | Trigger              | Does                                                                       |
| ------------- | -------------------- | -------------------------------------------------------------------------- |
| `ci.yml`      | PRs & pushes to main | fmt · clippy (`-D warnings`) · test · `cargo-deny` · Helm lint/template · Docker build + Trivy |
| `release.yml` | `vX.Y.Z` tags        | Multi-arch image (amd64+arm64) → GHCR, cosign-signed, SBOM + provenance, Trivy; Helm chart → GHCR (signed); GitHub Release |

To cut a release, align the crate version and the chart `version`/`appVersion`,
then push a tag:

```bash
git tag v0.2.0 && git push origin v0.2.0
```

## Governance

This repository follows the
[adorsys-gis AI governance](https://adorsys-gis.github.io/ai-governance/):
humans own intent, verification, and consequences; AI-generated artifacts are
reviewed as untrusted; and every release ships with verifiable evidence
(signatures, SBOM, provenance, scans). See [`CONTRIBUTING.md`](CONTRIBUTING.md)
and the pull-request template.

> **AI usage disclosure.** Portions of this codebase were drafted with AI
> assistance and reviewed by a human owner who is accountable for the result.

## License

[Apache-2.0](LICENSE).
