# Eggprobe Active Planning Registry

This file is the compact control surface for active interim planning. Detailed requirements remain in the canonical documents, subsystem roadmaps, implementation plans, closure records, and Git history.

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
- **conditionally closed** — implementation is substantially complete but a named operational evidence condition remains.
- **superseded** — replaced by another document.
- **archived** — no longer active and retained for traceability.

## Active subsystem roadmaps

| Subsystem | Status | Roadmap | Current milestone | Dependencies or blockers |
|---|---|---|---|---|
| Foundation and diagnostic contract | active | `plans/subsystems/foundation-diagnostic-contract-roadmap.md` | M002 ready | M001 closed with canonical JSON/report contract. |
| Transport and probe engine | active | `plans/subsystems/transport-probe-engine-roadmap.md` | M001 ready | Foundation M001 closed; direct probes can target the landed report contract. |
| CLI and automation | blocked | `plans/subsystems/cli-automation-roadmap.md` | M001 blocked | Requires implemented primitive probe contracts; CLI must not invent placeholder network semantics. |
| Release and operational qualification | active | `plans/subsystems/release-operational-qualification-roadmap.md` | M001 ready | Executable workspace and versioned CLI baseline now exists; release skeleton can be planned. |

## Dependency-ready implementation plans

| Subsystem | Milestone | Status | Implementation plan | Dependencies / handoff note |
|---|---|---|---|---|
| Foundation and diagnostic contract | M001 Rust workspace and canonical diagnostic contract | closed | `plans/implementation/foundation-diagnostic-contract/001-rust-workspace-and-canonical-diagnostic-contract.md` | Closed by `plans/closure/foundation-diagnostic-contract/001-status.md`; implementation commit `f802e30`. |

## Next handoffs unblocked for planning

| Subsystem | Milestone | Status | Implementation plan | Handoff note |
|---|---|---|---|---|
| Foundation and diagnostic contract | M002 published schema and compatibility fixture gate | ready | not yet written | M001 is closed and the canonical type shape is reviewed. |
| Transport and probe engine | M001 direct route, DNS, and TCP primitives | ready | not yet written | M001 is closed; direct probes can consume real `ProbePlan`/`ProbeReport` types. |
| Release and operational qualification | M001 CI/MSRV/audit/release skeleton | ready | not yet written | The executable workspace and CLI version surface now provide the required product baseline. |

## Current execution order and dependency gates

1. **Foundation M001 is closed with evidence.**
   - Closure path: `plans/closure/foundation-diagnostic-contract/001-status.md`.
   - No real DNS/TCP/TLS/HTTP work was mixed into the foundation milestone.
2. **Prepare the next handoffs from the landed code rather than the planning-only baseline:**
   - foundation M002 — generated schema + compatibility fixture gate;
   - transport M001 — direct route + DNS/TCP primitives.
   - release M001 — CI/MSRV/audit/release skeleton.
3. **Transport sequencing after direct primitives:**
   - standalone TLS and Eggfetch HTTP may proceed with the dependencies described in the transport roadmap;
   - Eggress route core can proceed after the direct route abstraction is stable;
   - routed H1/H2 integration requires both Eggfetch HTTP and Eggress route work.
4. **CLI capability work follows real probes.**
   - A minimal `--help`/`--version` binary in foundation M001 is not CLI subsystem closure.
5. **Release work follows executable product capability.**
   - Shared installer/updater integration is an interface dependency on Eggstack `eggup` when that library becomes available; it is not a reason to copy updater logic.

## Blocked work

| Work | Status | Blocker | Unblock condition |
|---|---|---|---|
| Transport M002 standalone TLS | blocked | transport M001 | Direct route/dial contract closed |
| Transport M003 Eggfetch HTTP | blocked | transport M001 | Route/error/deadline foundation closed |
| Transport M004 Eggress route core | blocked | transport M001 | Route abstraction closed; Eggress public API still qualified |
| Transport M005 Eggfetch-over-Eggress H1/H2 | blocked | transport M003 + M004 | Both consumer paths closed |
| Transport M006 observer refinement | blocked | working consumer paths absent | M003-M005 provide concrete evidence gaps |
| CLI M001 primitive commands | blocked | primitive probes absent | Required probe APIs closed |
| CLI M002 check/assertions | blocked | CLI M001 | Primitive command/report path closed |
| CLI M003 plan/NDJSON/batch | blocked | foundation M002 + CLI M002 | Schema + composite semantics stable |
| CLI M004 compare/repetition | blocked | CLI M002 + route support | Assertions and direct/routed reports closed |
| Release M002 | blocked | release M001 + usable CLI | CI/release skeleton closed |
| Release M003 | blocked | release M002 + shared updater interface | `eggup` or equivalent public interface available |
| Release M004 | blocked | release M002; M003 only if installer advertised | Release artifacts ready for qualification |

## External interface baselines reviewed

These are research baselines, not permanent dependency pins:

| Project | Reviewed baseline | Relevant interface |
|---|---|---|
| Eggfetch | `8959ca890ee34f4cf456aed648315322f1e83ef7` / 0.2.0 line | HTTP engine, custom `Dialer`, connection/TLS metadata, timeout/error surfaces |
| Eggress | `cd18c19c391e72b93d994acf60298e872fce5f0c` / 1.0.7 line | listener-free `OutboundConnector`, typed connect errors, chain metadata |
| CodeGG planning convention | `239c51d19a63e8cf369a952b78d1e55092f4653b` | canonical docs -> ADR -> subsystem -> implementation -> closure -> registry lifecycle |

Implementation agents MUST re-check current published dependency surfaces at the milestone baseline. Do not treat these SHAs as required Git dependencies.

## Recently closed work

| Work | Status | Closure record | Summary |
|---|---|---|---|
| Foundation and diagnostic contract M001 | closed | `plans/closure/foundation-diagnostic-contract/001-status.md` | Typed workspace/report contract, fixtures, redaction boundary, CLI smoke surface, and baseline quality gates landed in `f802e30`. |

## Planning hygiene

- Register a plan here before handoff.
- Keep canonical documents free of transient implementation details.
- Preserve closure records as evidence.
- Do not mark a subsystem closed because infrastructure compiled.
- Do not promote blocked future plans to ready until their dependency is actually closed.
