# Eggprobe Active Planning Registry

This file is the compact control surface for active interim planning. Detailed
requirements remain in the canonical documents, subsystem roadmaps,
implementation plans, closure records, corrective addenda, and Git history.

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
- **corrective required** — historical closure evidence exists, but a later finding must close before that area is treated as currently qualified.
- **superseded** — replaced by another document.
- **archived** — no longer active and retained for traceability.

## Current repository baseline

Planning/audit baseline for this corrective round:

`24965aa0ba2696b201e9c74531920877529821d5`

The large implementation commit under review is:

`792ad65524a869d2ec22ba88dd9daea427c74cab`

Historical closure records from that implementation remain immutable evidence.
They are not sufficient to override the corrective gates registered below.

## Active workstreams

| Workstream | Status | Current work | Authority |
|---|---|---|---|
| Foundation diagnostic contract | closed corrective | C002 closed | `plans/subsystems/foundation-diagnostic-contract-corrective-addendum.md` |
| Transport/probe engine | active corrective | C001 closed; C002 in implementation against Eggress 1.0.8 | `plans/subsystems/transport-probe-engine-corrective-addendum.md` |
| CLI/automation | closed corrective | C001 closed | `plans/subsystems/cli-automation-corrective-addendum.md` |
| Release/packaging | active corrective | C001 conditionally closed; hosted tag evidence pending | `plans/subsystems/release-operational-qualification-corrective-addendum.md` |
| Shared updater integration | blocked | Release M003 | no published stable `eggup` interface |
| Long-range ICMP/path + QUIC/H3 | roadmap-level | no implementation handoff | prerequisite platform/datagram ownership unresolved |

## Dependency-ready implementation plans

| Workstream | Milestone | Status | Implementation plan | Handoff note |
|---|---|---|---|---|
| Release corrective | C001 packaging workflow correction + hosted evidence | conditionally closed | `plans/implementation/release-operational-qualification-corrective/001-packaging-workflow-correction-and-hosted-evidence.md` | Hosted dispatch awaits an existing release tag; local package evidence is recorded. |
| Transport corrective | C002 routed/TLS/transport qualification | active | `plans/implementation/transport-probe-engine-corrective/002-routed-tls-and-transport-qualification.md` | Eggress 1.0.8 provides the typed outbound diagnostics API. |

## Registered blocked corrective plans

| Workstream | Milestone | Status | Implementation plan | Blocker |
|---|---|---|---|---|
| Release M003 | shared installer/update integration | blocked | `plans/implementation/release-operational-qualification/003-shared-installer-update-integration.md` | stable published `eggup` interface |
| Release M004 | release qualification/operator docs | blocked | `plans/implementation/release-operational-qualification/004-release-qualification-and-operator-docs.md` | release corrective C001 hosted tag evidence and Release M002 operational qualification; M003 only if updater advertised |

## Corrective findings being addressed

### Foundation C002

- historical `RouteSummary::Eggress { expression: String }` was not structurally secret-safe;
- historical report schema permitted arbitrary route expression text;
- HTTP authority parsing is hand-written and mishandles scheme-default ports/edge cases;
- corrected machine contract must move to schema 0.2 while preserving historical 0.1 artifacts.

### Transport C001/C002

- strict target policy can be bypassed by direct Eggfetch HTTP because the actual dial path does not use the policy-aware resolver;
- nonzero retry configuration is accepted but ignored;
- execution/deadline/cancellation evidence is incomplete;
- direct TCP omits observable local address;
- Eggress failures lose detailed kind/stage/hop/protocol provenance;
- required standalone TLS, SOCKS5, HTTP CONNECT, multi-hop, and routed HTTP fixture evidence is absent;
- routed H3 semantics are not yet represented as an explicit structured unsupported request.

### Transport C002 interface blocker (resolved)

Eggress 1.0.7 collapsed `OutboundConnector::connect_tcp` failures to a string,
so Transport C002 was blocked. Published Eggress 1.0.8 adds detailed connect
methods and typed kind/stage/hop/protocol accessors. C002 is now active and
uses that public API without parsing display strings or copying Eggress code.

### CLI C001

- `check` adds an HTTP assertion even with no HTTP probe;
- `--concurrency` is validated but batch execution is sequential;
- `compare` merges both sides into one statistic set and always exits 0;
- direct/routed role comparability is not validated;
- internal/interruption exit semantics are not demonstrated.

### Release C001

- Linux aarch64 cross-build attempts to execute an aarch64 binary on x86_64;
- checksum glob includes unpacked directories;
- required workflow version input is ignored;
- source/tag/Cargo version authority is not proven;
- hosted packaging/artifact/hash evidence is absent.

## Current execution order

1. **Foundation C002** and **Release corrective C001** may execute in parallel.
2. After Foundation C002 closes, execute **Transport corrective C001**.
3. After Transport C001 closes, CLI C001 may execute independently and is now
   closed. Transport C002 is active now that Eggress 1.0.8 publishes the
   required diagnostic seam.
4. After Foundation/Transport/CLI/Release correctives close, execute **Release M004** without updater claims unless M003 has independently unblocked. Release C001 still needs hosted tag evidence, so M004 remains blocked.
5. **Release M003** remains blocked until a stable shared `eggup` interface exists.
6. Do not generate Phase 8/9 implementation plans until the platform/datagram ownership prerequisites exist.

## Historical evidence

| Work | Historical status | Evidence | Current qualification note |
|---|---|---|---|
| Foundation M001 | closed | `plans/closure/foundation-diagnostic-contract/001-status.md` | preserved |
| Foundation corrective C001 | closed | `plans/closure/foundation-diagnostic-contract-corrective/001-status.md` | superseded only by narrower C002 findings |
| Foundation M002 | closed | `plans/closure/foundation-diagnostic-contract/002-status.md` | schema contract requires C002 correction |
| Transport M001-M006 | closed records | `plans/closure/transport-probe-engine/` | current release qualification gated by Transport C001/C002 |
| CLI M001-M004 | closed records | `plans/closure/cli-automation/` | current automation qualification gated by CLI C001 |
| Release M001 | closed | `plans/closure/release-operational-qualification/001-status.md` | retained |
| Release M002 | conditionally closed record | `plans/closure/release-operational-qualification/002-status.md` | corrective required before operational qualification |
| Release M003 | blocked | `plans/closure/release-operational-qualification/003-status.md` | external interface blocker remains |

## External interface baselines

These are research baselines, not permanent dependency pins:

| Project | Reviewed baseline | Relevant interface |
|---|---|---|
| Eggfetch | 0.2.0 line | HTTP engine, custom `Dialer`, origin TLS/metadata |
| Eggress | 1.0.8 line | listener-free outbound connector, typed detailed route errors, chain metadata |
| CodeGG planning convention | `239c51d19a63e8cf369a952b78d1e55092f4653b` | canonical -> ADR -> roadmap -> implementation -> closure -> corrective lifecycle |

Implementation agents MUST re-check current published dependency surfaces at
their execution baseline.

## Planning hygiene

- Historical closure records are never silently rewritten to erase later findings.
- Corrective addenda are current authority where they conflict with historical closure state.
- Register a plan here before handoff.
- Do not promote a blocked corrective until its named dependency actually closes.
- Release qualification must use demonstrated fixture/workflow evidence, not code inspection alone.
- Do not copy Eggfetch, Eggress, or eggup functionality into Eggprobe to bypass an interface blocker.
