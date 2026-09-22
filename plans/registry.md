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
| Foundation and diagnostic contract | active | `plans/subsystems/foundation-diagnostic-contract-roadmap.md` | M001 ready | No hard dependency. Establish workspace + canonical JSON/report contract first. |
| Transport and probe engine | blocked | `plans/subsystems/transport-probe-engine-roadmap.md` | M001 blocked | Hard: foundation M001 must close so probes target a real canonical report contract. |
| CLI and automation | blocked | `plans/subsystems/cli-automation-roadmap.md` | M001 blocked | Requires implemented primitive probe contracts; CLI must not invent placeholder network semantics. |
| Release and operational qualification | blocked | `plans/subsystems/release-operational-qualification-roadmap.md` | M001 blocked | Requires executable workspace/product baseline; packaging follows usable CLI/core behavior. |

## Dependency-ready implementation plans

| Subsystem | Milestone | Status | Implementation plan | Dependencies / handoff note |
|---|---|---|---|---|
| Foundation and diagnostic contract | M001 Rust workspace and canonical diagnostic contract | ready | `plans/implementation/foundation-diagnostic-contract/001-rust-workspace-and-canonical-diagnostic-contract.md` | Fresh planning-only baseline. Create core/CLI crates, typed plan/report contract, redaction boundary, deterministic JSON fixtures, MSRV/CI quality gates. No networking in this milestone. |

## Current execution order and dependency gates

1. **Execute foundation M001.**
   - This is the only dependency-ready production milestone.
   - Do not mix real DNS/TCP/TLS/HTTP work into it.
2. **Close foundation M001 with evidence.**
   - Required closure path: `plans/closure/foundation-diagnostic-contract/001-status.md`.
3. **After M001 closure, prepare two next handoffs from the landed code rather than the planning-only baseline:**
   - foundation M002 — generated schema + compatibility fixture gate;
   - transport M001 — direct route + DNS/TCP primitives.
4. **Transport sequencing after direct primitives:**
   - standalone TLS and Eggfetch HTTP may proceed with the dependencies described in the transport roadmap;
   - Eggress route core can proceed after the direct route abstraction is stable;
   - routed H1/H2 integration requires both Eggfetch HTTP and Eggress route work.
5. **CLI capability work follows real probes.**
   - A minimal `--help`/`--version` binary in foundation M001 is not CLI subsystem closure.
6. **Release work follows executable product capability.**
   - Shared installer/updater integration is an interface dependency on Eggstack `eggup` when that library becomes available; it is not a reason to copy updater logic.

## Blocked work

| Work | Status | Blocker | Unblock condition |
|---|---|---|---|
| Foundation M002 schema/compatibility gate | blocked | M001 not closed | M001 closure accepted and canonical type shape reviewed |
| Transport M001 direct route/DNS/TCP | blocked | M001 not closed | M001 closure accepted |
| Transport M002 standalone TLS | blocked | transport M001 | Direct route/dial contract closed |
| Transport M003 Eggfetch HTTP | blocked | transport M001 | Route/error/deadline foundation closed |
| Transport M004 Eggress route core | blocked | transport M001 | Route abstraction closed; Eggress public API still qualified |
| Transport M005 Eggfetch-over-Eggress H1/H2 | blocked | transport M003 + M004 | Both consumer paths closed |
| Transport M006 observer refinement | blocked | working consumer paths absent | M003-M005 provide concrete evidence gaps |
| CLI M001 primitive commands | blocked | primitive probes absent | Required probe APIs closed |
| CLI M002 check/assertions | blocked | CLI M001 | Primitive command/report path closed |
| CLI M003 plan/NDJSON/batch | blocked | foundation M002 + CLI M002 | Schema + composite semantics stable |
| CLI M004 compare/repetition | blocked | CLI M002 + route support | Assertions and direct/routed reports closed |
| Release M001 | blocked | no executable product baseline | Foundation/probe work creates meaningful CI/release target |
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

None. The repository is in planning bootstrap state.

## Planning hygiene

- Register a plan here before handoff.
- Keep canonical documents free of transient implementation details.
- Preserve closure records as evidence.
- Do not mark a subsystem closed because infrastructure compiled.
- Do not promote blocked future plans to ready until their dependency is actually closed.
