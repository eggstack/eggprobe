# Release Corrective C001 — Conditional Closure

Status: conditionally closed

Implementation plan:

- `plans/implementation/release-operational-qualification-corrective/001-packaging-workflow-correction-and-hosted-evidence.md`

## Delivered correction

- Manual dispatch requires an explicit semantic-version tag input and checks
  out that exact tag with full history.
- The workflow proves tag, checked-out commit, and Cargo package version agree.
- Linux aarch64 is cross-built and inspected with `file`; it is never executed
  on the x86_64 hosted runner. Host-native targets run `--version` and a
  schema-0.2 empty-plan NDJSON smoke with no public network.
- Archives contain only the binary and release metadata. Checksums are scoped
  to one archive and verified immediately; unpacked directories are excluded.
- Artifact upload paths are explicit and permissions remain `contents: read`.

## Local evidence

The Linux host-native packaging path was exercised locally: release build,
version smoke, schema-0.2 empty-plan NDJSON smoke, archive extraction, and
archive-only SHA-256 verification all passed. Workflow YAML parsing and
`git diff --check` also passed.

## Operational blocker

This repository has no existing Git tag. The corrected workflow intentionally
requires one, so hosted artifact evidence cannot be produced without a release
identity decision. Preliminary dispatch `35766690963` was cancelled because it
used the old remote workflow before these corrections were pushed. Corrected
workflow dispatch `35768825481` was also cancelled while queued because the
requested `v0.1.0` tag does not exist; it produced no artifacts and is not
qualification evidence.

The corrective is conditionally closed, not operationally qualified. Remaining
evidence is a supported dispatch against an existing tag with run IDs,
per-target artifact metadata/hashes, and host-native smoke results. Linux
aarch64 remains build-qualified until native SBC evidence exists.

## Downstream disposition

Release M003 remains blocked on the unpublished shared `eggup` interface.
Release M004 remains blocked on the remaining corrective qualification work.

Historical snapshot at C001 closure: the M003 statement above records the
pre-ADR-0002 disposition and is retained as evidence. ADR-0002 superseded that
mixed milestone with producer-only M003a (Eggpack) and optional consumer M003b
(Eggup). C002 has now closed the documentation cleanup; M004's remaining gate
is valid-tag hosted artifact evidence for C001/M002. Neither M003a nor M003b is
a first-release prerequisite.
