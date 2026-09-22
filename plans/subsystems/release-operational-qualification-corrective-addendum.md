# Release and Operational Qualification — Post-Closure Corrective Addendum

Status: active; C001 ready for handoff

Planning baseline: `24965aa0ba2696b201e9c74531920877529821d5`

Historical evidence:

- `plans/closure/release-operational-qualification/001-status.md`
- `plans/closure/release-operational-qualification/002-status.md`
- `plans/subsystems/release-operational-qualification-roadmap.md`

## 1. Why this addendum exists

Release M002 was conditionally closed before the packaging workflow was actually exercised. Static review found correctness defects that must be fixed before hosted artifact evidence is meaningful:

- the Linux aarch64 cross-build job attempts to execute the aarch64 binary on an x86_64 runner;
- checksum generation operates over `dist/*` while unpacked directories are present;
- the required `workflow_dispatch.version` input is ignored in favor of `GITHUB_REF_NAME`;
- tag/version/source checkout consistency is not proven;
- there are no hosted workflow/artifact hashes yet.

The historical conditional closure record remains evidence of what was implemented, but M002 is not considered operationally qualified until this corrective closes.

## 2. C001 — Packaging workflow correction and hosted evidence

Implementation:

- `plans/implementation/release-operational-qualification-corrective/001-packaging-workflow-correction-and-hosted-evidence.md`

Closure:

- `plans/closure/release-operational-qualification-corrective/001-status.md`

This plan is dependency-ready and may execute in parallel with Foundation C002.

Release M004 remains blocked on this corrective plus the active product-correctness correctives. M003 remains independently blocked on the shared `eggup` interface and is not advertised.
