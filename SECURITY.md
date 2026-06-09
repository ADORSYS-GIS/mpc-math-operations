# Security Policy

## Reporting a vulnerability

Please report suspected vulnerabilities **privately** — do not open a public issue.

- Use GitHub's [private vulnerability reporting](https://docs.github.com/en/code-security/security-advisories/guidance-on-reporting-and-writing-information-about-vulnerabilities/privately-reporting-a-security-vulnerability)
  ("Report a vulnerability" under the repository's **Security** tab), or
- Email the maintainers at **security@adorsys.com**.

Please include enough detail to reproduce: affected version/commit, impact, and
a proof of concept where possible. We aim to acknowledge within **3 business
days** and to provide a remediation timeline after triage.

## Supported versions

Security fixes target the latest released `0.x` line. Older tags are not
patched; please upgrade.

## Supply-chain assurances

Each released container image and Helm chart is:

- built reproducibly via a pinned toolchain (`rust-toolchain.toml`) and a
  multi-stage, distroless runtime (no shell, non-root);
- **signed** with [Sigstore cosign](https://www.sigstore.dev/) (keyless / OIDC);
- accompanied by an **SBOM** and **SLSA build provenance** attestation;
- scanned with [Trivy](https://trivy.dev/), with results uploaded to GitHub
  code scanning.

See the README for how to verify a published artifact before deploying it.
