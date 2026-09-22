# Release and Operational Qualification Roadmap

Status: active; M001 ready for handoff

Long-term references:

- `plans/000-long-term-specification.md#14-platforms-and-packaging`
- `plans/002-long-term-roadmap.md#phase-7--release-packaging-and-operational-qualification`

Related ADRs:

- `plans/adrs/ADR-0001-json-first-core-and-transport-ownership.md`

## 1. Purpose and ownership boundary

This subsystem owns release artifacts, supported-target evidence, installation/update integration, release smoke tests, dependency/security qualification, and operator-facing release documentation.

It does not own diagnostic semantics.

## 2. Work classification

### Invariants

- release binaries correspond to tested source;
- checksums/provenance are published;
- MSRV claims are tested;
- unsupported targets are not advertised as supported;
- release smoke tests validate JSON as well as human CLI behavior;
- installer/updater logic should reuse shared Eggstack machinery when a stable interface exists.

### Capabilities

- downloadable standalone binary;
- installer;
- version command;
- shell completions;
- upgrade path.

### Infrastructure

- CI target matrix;
- release packaging scripts/workflows;
- checksum/provenance generation;
- smoke fixtures.

### Polish

- compact binary profile;
- package-manager distribution;
- man pages.

## 3. Non-goals

- no auto-update daemon;
- no telemetry requirement;
- no package-manager matrix before direct release artifacts are stable;
- no copied updater implementation if `eggup` provides the required shared interface.

## 4. Current state

No production code or release artifact existed at planning bootstrap.

The intended target class follows current Eggstack practice:

- Linux x86_64/aarch64 including SBCs;
- macOS x86_64/arm64;
- Windows x86_64;
- Rust 1.89 source-build floor unless dependencies force an explicit reviewed change.

The sibling `eggup` project is being designed as shared verified installer/updater/service-management machinery. Eggprobe SHOULD consume it when its public interface is available rather than duplicating release-update logic.

## 5. Target architecture

Release packaging should produce version-aligned archives containing the `eggprobe` binary, checksums, license/readme material as appropriate, and deterministic smoke metadata.

Installer/updater ownership should be injected/shared rather than baked into the diagnostics core.

## 6. Dependency graph

```text
foundation + executable CLI
          |
          v
M001 CI/MSRV/security/release skeleton
          |
          v
M002 cross-platform binary packaging
          |
          +--> M003 installer/update integration
          |
          `--> M004 release qualification + docs
```

M003 has an interface dependency on shared Eggstack updater machinery; it may remain deferred without blocking manual binary releases.

## 7. Milestones

### M001 — CI, MSRV, audit, and release skeleton

Class: infrastructure.

Exit conditions:

- format, strict Clippy, tests, audit;
- MSRV lane;
- primary platform compile/test matrix;
- release workflow skeleton does not publish unverified artifacts.

### M002 — Cross-platform binary packaging

Class: capability + infrastructure.

Exit conditions:

- target archives and checksums;
- local/release smoke test;
- `eggprobe --version`;
- JSON smoke output parseable;
- Linux aarch64/SBC target evidence.

### M003 — Shared installer/update integration

Class: capability.

Objective:

Consume `eggup` or equivalent shared Eggstack interface once stable.

Exit conditions:

- verified download/checksum path;
- pinned version/custom install dir;
- no duplicated updater core;
- safe failure/rollback semantics documented.

### M004 — Release qualification and operator documentation

Class: polish + invariant.

Exit conditions:

- installation docs;
- supported target table;
- release verification record;
- schema fixture compatibility against release binary;
- dependency/security evidence;
- no unresolved medium-or-higher release blocker.

## 8. Cross-cutting requirements

### Compatibility

Release version and schema version are separate.

### Security

No unsigned/unverified download path should be presented as the default installer behavior. Security audit exceptions require documented rationale and expiry/review.

### Performance

Binary-size/throughput claims require measurement. Small release profiles may be evaluated but must not compromise diagnostics or unwind safety.

### Operations

A source build remains available even when no prebuilt target exists.

## 9. Verification strategy

- fresh-install smoke;
- release archive extraction;
- `--version`;
- JSON parse smoke;
- one local no-network/core command and later loopback probe;
- checksum verification;
- MSRV compile;
- target-native tests where runners exist;
- artifact inspection.

## 10. Risks and decision points

- Windows networking semantics may require platform-specific qualification.
- Linux aarch64 cross-builds are not equivalent to target-class runtime evidence; at least one SBC/native smoke should be recorded before claiming operational support.
- Eggup interface timing may lag Eggprobe release readiness; do not block standalone artifacts unnecessarily.

## 11. Completion definition

The subsystem closes when users can obtain, verify, install, run, and update a supported binary through documented paths and release artifacts have machine-contract smoke evidence.

## 12. Milestone status

| Milestone | Status | Implementation plan | Closure record | Blockers |
|---|---|---|---|---|
| M001 CI/MSRV/audit/release skeleton | ready | `plans/implementation/release-operational-qualification/001-ci-msrv-audit-release-skeleton.md` | pending | executable workspace and CLI baseline exists |
| M002 cross-platform binary packaging | blocked | `plans/implementation/release-operational-qualification/002-cross-platform-binary-packaging.md` | pending | M001 + usable diagnostic CLI |
| M003 shared installer/update integration | blocked | `plans/implementation/release-operational-qualification/003-shared-installer-update-integration.md` | pending | M002 + eggup interface |
| M004 release qualification/operator docs | blocked | `plans/implementation/release-operational-qualification/004-release-qualification-and-operator-docs.md` | pending | M002; M003 if installer advertised |
