# Release and Operational Qualification M001 — CI, MSRV, Audit, and Release Skeleton

Status: ready for handoff

Planning baseline: `e55a21fe73915b7a38c2fa674807354539806c60`

Source roadmap:

- `plans/subsystems/release-operational-qualification-roadmap.md#m001--ci-msrv-audit-and-release-skeleton`

Primary class: infrastructure

## 1. Objective

Expand the foundation CI into a release-capable quality skeleton without publishing binaries prematurely.

## 2. Current state

Foundation M001 already provides Linux format/Clippy/test and Rust 1.89 checks. This milestone should extend, not duplicate, that workflow.

## 3. Required production changes

- Linux/macOS/Windows compile/test jobs at appropriate tiers;
- explicit Rust 1.89 MSRV lane;
- dependency/security policy including `cargo audit` or maintained equivalent;
- reproducible release-profile configuration;
- release workflow triggered manually/tagged but initially gated from upload until packaging M002;
- artifact naming/version helper shared with M002;
- `eggprobe --version` smoke;
- workflow permissions minimized.

Do not add cross-compilation frameworks before M002 needs them.

## 4. Work packages

A. Rationalize existing CI into reusable quality commands/jobs.
B. Add macOS/Windows host qualification.
C. Add audit/license/dependency policy appropriate to Eggstack.
D. Add non-publishing release workflow skeleton.
E. Document release prerequisites and failure handling.

## 5. Tests/verification

- local format/Clippy/test/MSRV/audit;
- workflow syntax validation;
- host-native test jobs;
- release job proves build/version smoke but does not create a public release;
- permissions review.

## 6. Acceptance criteria

CI is cross-platform enough to catch portable-core regressions, MSRV is enforced, security policy is explicit, and no unverified artifact can be published from the skeleton.

## 7. Stop conditions

Stop if a workflow requires broad write permissions, secrets for ordinary PR verification, or target claims not supported by real runners.

## 8. Closure evidence

Create `plans/closure/release-operational-qualification/001-status.md` with workflow runs, host matrix, MSRV/audit results, permission review, and proof that publishing remains gated.
