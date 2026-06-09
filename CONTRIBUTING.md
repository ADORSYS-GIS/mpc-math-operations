# Contributing

This project follows the
[adorsys-gis AI governance](https://adorsys-gis.github.io/ai-governance/).
The core rule: **AI may produce artifacts, but humans own intent, verification,
and consequences.** If you cannot explain a change, you are not ready to submit it.

## Workflow

1. Branch from `main`: `feat/<short-name>`, `fix/<short-name>`, `chore/<short-name>`.
2. Make focused commits using [Conventional Commits](https://www.conventionalcommits.org/):
   `feat:`, `fix:`, `docs:`, `chore:`, `refactor:`, `test:`, `ci:`, `build:`.
3. Open a PR and fill in the template — including the **verification evidence**
   and **AI usage disclosure** sections. Both are mandatory.

## Local checks (the same gate CI enforces)

```bash
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
cargo test --all
cargo deny check          # cargo install cargo-deny
```

Treat AI-generated code as untrusted: read it, test it, and confirm the tests
exercise meaningful behavior before relying on green checkmarks.

## Releasing

Releases are tag-driven. Pushing a `vX.Y.Z` tag triggers the release workflow,
which builds and **signs** the multi-arch container image and the Helm chart and
publishes both to GHCR (with SBOM, provenance, and a Trivy scan). Keep the chart
`version`/`appVersion` and the crate version aligned with the tag.
