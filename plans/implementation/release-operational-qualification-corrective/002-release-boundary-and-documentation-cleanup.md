# Release Corrective C002 — Release Boundary and Documentation Cleanup

Status: closed

Planning baseline: `a96911854b73e055ab67ea2f7c1592796f031d58` (release roadmap/ADR realignment); refresh if implementation starts from a later code baseline.

Source roadmap:

- `plans/subsystems/release-operational-qualification-roadmap.md`

Architecture:

- `plans/adrs/ADR-0002-release-producer-consumer-ownership.md`

Historical records to preserve:

- `plans/implementation/release-operational-qualification/003-shared-installer-update-integration.md`
- `plans/closure/release-operational-qualification/003-status.md`
- `plans/closure/release-operational-qualification-corrective/001-status.md`

Primary class: corrective + documentation/polish

## 1. Objective

Remove stale user/operator and planning-adjacent language that still implies the
first Eggprobe release is blocked on an unpublished Eggup "shared installer"
interface.

This corrective does not implement Eggpack or Eggup integration. It makes the
current standalone release path and future ownership boundaries truthful.

## 2. Required documentation corrections

Update current user/operator documentation so it states:

- ordinary Eggprobe releases currently use qualified archives/checksums;
- producer-side shared release construction/bootstrap/CI belongs to Eggpack;
- Eggpack adoption is future/deferred until its required producer interfaces
  are stable and qualified;
- runtime in-place self-update is optional and, if implemented, belongs to an
  Eggup consumer integration;
- lack of an Eggup self-update command is not a blocker for archive release;
- no unverified download/update script should be recommended.

At minimum inspect:

- `README.md`;
- `docs/operator.md`;
- release workflow comments/help text;
- release-related examples;
- any generated/help text that claims an updater is expected.

## 3. Planning-history hygiene

Do not rewrite the historical M003 blocked disposition as if it never
happened.

Required:

- keep `plans/closure/release-operational-qualification/003-status.md` as
  historical evidence;
- ensure the superseded M003 implementation plan points to ADR-0002 and the
  M003a/M003b replacement;
- ensure current roadmaps/registry use M003a/M003b rather than the old mixed
  milestone;
- ensure M004 is gated by release artifact/qualification evidence and C002,
  with M003b only conditional if self-update is advertised.

## 4. Scope exclusions

Do not:

- implement Eggpack producer integration;
- implement Eggup self-update;
- create a release tag;
- publish artifacts;
- copy installer/update logic locally;
- change diagnostic schemas or transport behavior.

## 5. Verification

- grep/review release-facing docs for stale "blocked on Eggup" or "shared
  updater required" claims;
- verify roadmap/registry links resolve;
- `git diff --check`;
- run documentation/help tests if any touched generated CLI/help surface exists.

## 6. Acceptance criteria

- current docs clearly distinguish standalone release, Eggpack producer
  integration, and Eggup runtime self-update;
- old M003 history is preserved but no longer appears as an active release
  blocker;
- M004 dependency wording is consistent everywhere;
- no duplicated producer/consumer ownership is introduced.

## 7. Closure evidence

Create
`plans/closure/release-operational-qualification-corrective/002-status.md`
with files reviewed/changed, stale-language search results, final dependency
graph, and verification commands.
