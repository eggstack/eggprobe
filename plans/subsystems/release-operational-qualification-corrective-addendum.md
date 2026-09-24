# Release and Operational Qualification — Post-Closure Corrective Addendum

Status: active; C001 conditionally closed; C002 ready

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

This plan was dependency-ready and executed independently of Foundation C002.
It is conditionally closed pending a hosted dispatch against an existing
release tag; the repository currently has no tags.

The product-correctness correctives are now closed. C001 remains operationally
conditional on hosted artifact evidence against a valid tag.

## 3. C002 — Release boundary and documentation cleanup

A later cross-repo planning review established the durable producer/consumer
boundary recorded in ADR-0002:

- Eggpack owns producer release construction/evidence/bootstrap/CI;
- Eggup owns optional consumer deployment/update transactions;
- Eggprobe owns product policy.

The historical Release M003 mixed these responsibilities and incorrectly
appeared as a first-release Eggup blocker.

Implementation:

- `plans/implementation/release-operational-qualification-corrective/002-release-boundary-and-documentation-cleanup.md`

Closure target:

- `plans/closure/release-operational-qualification-corrective/002-status.md`

Status: ready for handoff.

C002 is documentation/planning-adjacent cleanup only. It does not create a tag,
publish a release, or implement Eggpack/Eggup integration.

## 4. Downstream disposition

- Historical M003 is superseded by M003a/M003b under ADR-0002.
- M003a Eggpack producer integration is deferred and not a first-release gate.
- M003b Eggup runtime self-update is optional/deferred and not a first-release gate.
- Release M004 is blocked only on C001/M002 hosted artifact qualification plus
  C002 cleanup for the first standalone release.
