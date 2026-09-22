# Eggprobe Active Planning Registry

This file is the compact control surface for active interim planning. Detailed requirements remain in canonical documents, subsystem roadmaps, implementation plans, closure records, and Git history.

Canonical direction:

- `plans/000-long-term-specification.md`
- `plans/001-terminology-and-domain-model.md`
- `plans/002-long-term-roadmap.md`
- `plans/003-planning-process.md`
- `plans/adrs/ADR-0001-json-first-core-and-transport-ownership.md`

## Status vocabulary

- **proposed** — roadmap or plan exists but is not approved for execution.
- **ready** — dependencies/interfaces are satisfied; plan may be handed off.
- **active** — implementation or closure work is in progress.
- **blocked** — a named dependency/evidence requirement prevents progress.
- **closing** — implementation landed and closure evidence is being gathered.
- **closed** — closure record accepted.
- **conditionally closed** — implementation is substantially complete but named evidence remains.
- **superseded** — replaced by another document.
- **archived** — no longer active and retained for traceability.

## Active subsystem roadmaps

| Subsystem | Status | Roadmap | Current milestone | Dependencies or blockers |
|---|---|---|---|---|
| Foundation and diagnostic contract | closed through M002 | `plans/subsystems/foundation-diagnostic-contract-roadmap.md` | M002 closed | Corrective C001 and schema gate closed. |
| Foundation post-closure corrective | closed | `plans/subsystems/foundation-diagnostic-contract-corrective-addendum.md` | C001 closed | Route redaction, validated wrappers, and target authority repaired. |
| Transport and probe engine | closed through M006 | `plans/subsystems/transport-probe-engine-roadmap.md` | M006 closed | H3/datagram and richer upstream observers remain deferred. |
| CLI and automation | closed through M004 | `plans/subsystems/cli-automation-roadmap.md` | M004 closed | Core-owned execution and automation surfaces are available. |
| Release and operational qualification | blocked at M003 | `plans/subsystems/release-operational-qualification-roadmap.md` | M003 blocked | No published shared eggup interface; manual archives remain. |

## Dependency-ready implementation plans

| Subsystem | Milestone | Status | Implementation plan | Handoff note |
|---|---|---|---|---|
| Foundation post-closure corrective | C001 route redaction and contract integrity | closed | `plans/implementation/foundation-diagnostic-contract-corrective/001-route-redaction-and-contract-integrity.md` | `plans/closure/foundation-diagnostic-contract-corrective/001-status.md` |
| Foundation and diagnostic contract | M002 published schema and compatibility fixture gate | closed | `plans/implementation/foundation-diagnostic-contract/002-published-schema-and-compatibility-fixture-gate.md` | `plans/closure/foundation-diagnostic-contract/002-status.md` |
| Release and operational qualification | M001 CI/MSRV/audit/release skeleton | closed | `plans/implementation/release-operational-qualification/001-ci-msrv-audit-release-skeleton.md` | `plans/closure/release-operational-qualification/001-status.md` |
| Release and operational qualification | M002 cross-platform binary packaging | conditionally closed | `plans/implementation/release-operational-qualification/002-cross-platform-binary-packaging.md` | `plans/closure/release-operational-qualification/002-status.md` |
| Release and operational qualification | M004 release qualification/operator docs | ready | `plans/implementation/release-operational-qualification/004-release-qualification-and-operator-docs.md` | Conditional M003 dependency is not advertised; execution intentionally paused at the M003 gate. |

## Registered blocked implementation plans

| Subsystem | Milestone | Plan | Blocker |
|---|---|---|---|
| Release and operational qualification | M003 shared installer/update integration | `plans/implementation/release-operational-qualification/003-shared-installer-update-integration.md` | release M002 + stable eggup interface |

## Current execution order and dependency gates

1. **C001, Foundation M002, Transport M001–M006, and CLI M001–M004 are closed.**
2. **Release M001 is closed and M002 is conditionally closed pending external artifact evidence.**
3. **Release M003 is blocked because no published shared Eggstack updater interface exists; updater logic was not copied.**
4. **Release M004 is marked ready in its roadmap because its updater dependency is conditional, but sequential execution stopped at the M003 blocker per the handoff instruction.**

## External interface baselines reviewed

These are research baselines, not permanent dependency pins:

| Project | Reviewed baseline | Relevant interface |
|---|---|---|
| Eggfetch | `8959ca890ee34f4cf456aed648315322f1e83ef7` / 0.2.0 line | HTTP engine, custom `Dialer`, connection/TLS metadata, timeout/error surfaces |
| Eggress | `cd18c19c391e72b93d994acf60298e872fce5f0c` / 1.0.7 line | listener-free `OutboundConnector`, typed connect errors, chain metadata |
| CodeGG planning convention | `239c51d19a63e8cf369a952b78d1e55092f4653b` | canonical docs -> ADR -> subsystem -> implementation -> closure -> registry lifecycle |

Implementation agents MUST re-check published dependency surfaces at their actual implementation baseline.

## Historical and corrective evidence

| Work | Status | Evidence | Note |
|---|---|---|---|
| Foundation M001 | historically closed | `plans/closure/foundation-diagnostic-contract/001-status.md`; implementation `f802e30` | Post-closure review found a route-redaction gap; do not rewrite this historical record. |
| Foundation corrective C001 | closed | `plans/closure/foundation-diagnostic-contract-corrective/001-status.md` | Corrective gate satisfied before schema/transport execution. |

## Long-range roadmap disposition

Phases 8–9 of the master roadmap (native ICMP/path diagnostics and QUIC/routed HTTP/3 expansion) remain intentionally at roadmap level. They depend on platform/datagram interface research not yet stable enough for truthful implementation-agent handoffs. Per `plans/003-planning-process.md`, do not convert them into speculative implementation plans until their prerequisite interfaces and ownership decisions exist.

## Planning hygiene

- Register a plan here before handoff.
- Keep canonical documents free of transient implementation details.
- Preserve closure records as evidence.
- Do not mark capability closed because infrastructure compiled.
- Do not promote blocked plans until their named dependencies actually close.
