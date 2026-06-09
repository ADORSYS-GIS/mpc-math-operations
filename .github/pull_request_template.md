<!--
This template encodes the project's Definition of Done and AI working agreement.
The submitter is accountable for the result. If you cannot explain the change,
you are not ready to submit it.
-->

## Summary

<!-- What does this change do, and why? Link the issue/ticket. -->

Closes #

## Definition of Done

- [ ] Acceptance criteria are satisfied
- [ ] Code review is complete (AI-generated code reviewed as **untrusted**)
- [ ] Required tests pass locally and in CI (`cargo test`, `clippy`, `fmt`)
- [ ] Documentation updated where needed (README / chart values / comments)
- [ ] Security impact reviewed (see checklist below) — no unresolved critical issues
- [ ] Performance impact reviewed, if relevant

## Verification evidence

<!-- Required. Without evidence, the work is not Done. -->
<!-- Paste command output, logs, screenshots of rendered SVGs, `helm template`
     diffs, or links to a CI run. State known limitations. -->

```
# e.g. cargo clippy --all-targets, a tools/call response, helm lint output
```

## Security & supply chain

- [ ] No secrets, tokens, or credentials added to the repo or image
- [ ] Dependency changes reviewed (`cargo deny check` passes)
- [ ] Container/chart changes keep the non-root, read-only, drop-all-caps posture

## AI usage disclosure

<!-- Concealing AI usage that affects quality, risk, or understanding is prohibited. -->

- [ ] AI was used in this change
- **What for:** <!-- e.g. drafting code, tests, docs -->
- **What I manually verified:** <!-- be specific -->
- **Remaining uncertainties:** <!-- anything you are not fully confident about -->
