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
| Foundation and diagnostic contract | active | `plans/subsystems/foundation-diagnostic-contract-roadmap.md` | historical M001 closed; M002 blocked | M002 hard-blocked on post-closure corrective C001. |
| Foundation post-closure corrective | active | `plans/subsystems/foundation-diagnostic-contract-corrective-addendum.md` | C001 ready | Security/contract corrective for route redaction, validated deserialization, and target authority. |
| Transport and probe engine | blocked | `plans/subsystems/transport-probe-engine-roadmap.md` | M001 blocked | Hard: foundation corrective C001. |
| CLI and automation | blocked | `plans/subsystems/cli-automation-roadmap.md` | M001 blocked | Requires real primitive probe capabilities. |
| Release and operational qualification | active | `plans/subsystems/release-operational-qualification-roadmap.md` | M001 ready | Independent release/CI skeleton can proceed from current executable workspace. |

## Dependency-ready implementation plans

| Subsystem | Milestone | Status | Implementation plan | Handoff note |
|---|---|---|---|---|
| Foundation post-closure corrective | C001 route redaction and contract integrity | ready | `plans/implementation/foundation-diagnostic-contract-corrective/001-route-redaction-and-contract-integrity.md` | Immediate contract/security gate. Keep raw Eggress route opaque until Eggress owns parsing. |
| Release and operational qualification | M001 CI/MSRV/audit/release skeleton | ready | `plans/implementation/release-operational-qualification/001-ci-msrv-audit-release-skeleton.md` | Independent of route semantics; extends existing CI without publishing artifacts. |

## Registered blocked implementation plans

| Subsystem | Milestone | Plan | Blocker |
|---|---|---|---|
| Foundation and diagnostic contract | M002 published schema and compatibility fixture gate | `plans/implementation/foundation-diagnostic-contract/002-published-schema-and-compatibility-fixture-gate.md` | corrective C001 |
| Transport and probe engine | M001 direct route, DNS, and TCP primitives | `plans/implementation/transport-probe-engine/001-direct-route-dns-and-tcp-primitives.md` | corrective C001 |
| Transport and probe engine | M002 standalone TLS diagnostic probe | `plans/implementation/transport-probe-engine/002-standalone-tls-diagnostic-probe.md` | transport M001 |
| Transport and probe engine | M003 Eggfetch-backed HTTP probe | `plans/implementation/transport-probe-engine/003-eggfetch-backed-http-probe.md` | transport M001 |
| Transport and probe engine | M004 Eggress listener-free route core | `plans/implementation/transport-probe-engine/004-eggress-listener-free-route-core.md` | transport M001 |
| Transport and probe engine | M005 Eggfetch over Eggress for routed HTTP(S) | `plans/implementation/transport-probe-engine/005-eggfetch-over-egress-routed-http.md` | transport M003 + M004 |
| Transport and probe engine | M006 upstream diagnostic observability refinement | `plans/implementation/transport-probe-engine/006-upstream-diagnostic-observability-refinement.md` | demonstrated gaps from M003-M005 |
| CLI and automation | M001 primitive command surface and renderers | `plans/implementation/cli-automation/001-primitive-command-surface-and-renderers.md` | required primitive probes |
| CLI and automation | M002 composite check/assertions/exit codes | `plans/implementation/cli-automation/002-composite-check-assertions-and-exit-codes.md` | CLI M001 |
| CLI and automation | M003 plan files/schema/NDJSON/batch | `plans/implementation/cli-automation/003-plan-files-schema-ndjson-and-batch.md` | foundation M002 + CLI M002 |
| CLI and automation | M004 route comparison/repetition statistics | `plans/implementation/cli-automation/004-route-comparison-and-repetition-statistics.md` | CLI M002 + transport M005 |
| Release and operational qualification | M002 cross-platform binary packaging | `plans/implementation/release-operational-qualification/002-cross-platform-binary-packaging.md` | release M001 + usable diagnostic CLI |
| Release and operational qualification | M003 shared installer/update integration | `plans/implementation/release-operational-qualification/003-shared-installer-update-integration.md` | release M002 + stable eggup interface |
| Release and operational qualification | M004 release qualification/operator docs | `plans/implementation/release-operational-qualification/004-release-qualification-and-operator-docs.md` | release M002; M003 only if installer advertised |

## Current execution order and dependency gates

1. **Execute corrective C001 first for contract-bearing work.**
   - It repairs the M001 route-redaction invariant and validation/target-authority gaps.
   - Historical M001 closure remains immutable evidence; C001 supersedes its broad redaction conclusion.
2. **Release M001 may execute in parallel.**
   - It changes CI/release scaffolding, not route/report semantics.
3. **After C001 closure:**
   - Foundation M002 becomes ready.
   - Transport M001 becomes ready.
4. **After Transport M001:**
   - Transport M002 TLS, M003 Eggfetch HTTP, and M004 Eggress route work become dependency-ready according to their plans.
5. **After Transport M003 + M004:**
   - Transport M005 routed H1/H2 becomes ready.
6. **Transport M006 is evidence-driven.**
   - Execute only for observer gaps proven by working M003-M005 paths.
7. **CLI milestones follow implemented core capabilities.**
8. **Release M002 follows release M001 plus a meaningful diagnostic CLI.**
9. **Release M003 remains blocked on the shared Eggstack updater interface; do not copy updater logic.**
10. **Release M004 is the final release-candidate evidence/doc reconciliation.**

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
| Foundation corrective C001 | ready | `plans/subsystems/foundation-diagnostic-contract-corrective-addendum.md`; `plans/implementation/foundation-diagnostic-contract-corrective/001-route-redaction-and-contract-integrity.md` | Must close before schema freeze or transport execution. |

## Long-range roadmap disposition

Phases 8–9 of the master roadmap (native ICMP/path diagnostics and QUIC/routed HTTP/3 expansion) remain intentionally at roadmap level. They depend on platform/datagram interface research not yet stable enough for truthful implementation-agent handoffs. Per `plans/003-planning-process.md`, do not convert them into speculative implementation plans until their prerequisite interfaces and ownership decisions exist.

## Planning hygiene

- Register a plan here before handoff.
- Keep canonical documents free of transient implementation details.
- Preserve closure records as evidence.
- Do not mark capability closed because infrastructure compiled.
- Do not promote blocked plans until their named dependencies actually close.
