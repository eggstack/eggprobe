# ADR-0003: Native Diagnostics Platform and Subject Boundary

Status: accepted

Date: 2026-09-24

Decision owners: project maintainers

Related specification sections:

- `plans/000-long-term-specification.md#4-architectural-principles`
- `plans/000-long-term-specification.md#7-route-model`
- `plans/000-long-term-specification.md#8-probe-families`
- `plans/000-long-term-specification.md#11-security-model`
- `plans/000-long-term-specification.md#14-platforms-and-packaging`
- `plans/000-long-term-specification.md#15-deferred-and-future-capabilities`
- `plans/002-long-term-roadmap.md#phase-8--native-path-and-host-diagnostics`

Affected subsystem roadmap:

- `plans/subsystems/native-path-host-diagnostics-roadmap.md`

## Context

Eggprobe 0.1.1 closes the initial production program through Phase 7 with a JSON-first `ProbePlan -> ProbeReport` contract, a thin CLI, direct DNS/TCP/TLS, Eggfetch-owned HTTP, and Eggress-owned byte-stream routing.

Phase 8 adds diagnostics that interact more directly with operating-system networking surfaces: route/interface inspection, ICMP echo, UDP, traceroute, and path MTU discovery. Three architectural questions must be settled before implementation: where OS/raw-socket mechanics live, whether Phase 8 broadens the target model into arbitrary host inventory, and whether datagrams may reuse the current Eggress byte-stream route abstraction.

The workspace currently sets `unsafe_code = "forbid"`. The canonical specification also says UDP/QUIC routing is distinct and MUST NOT be modeled as if a byte-stream dialer were sufficient.

## Decision drivers

- preserve the JSON-first core and thin CLI;
- keep Eggprobe-authored unsafe code out of the workspace unless a later ADR explicitly changes that policy;
- isolate platform variability from the canonical report model;
- avoid meaningless placeholder targets for whole-host inventory;
- distinguish kernel-observed facts from user-space route-table correlation;
- keep direct native datagrams separate from Eggress byte-stream routes;
- represent privilege/platform limitations explicitly.

## Decision

### 1. Add an internal native backend crate

Phase 8 SHALL introduce an internal workspace crate named `eggprobe-native` (or an equivalently narrow name if repository inspection requires a mechanically different name).

`eggprobe-native` owns platform/dependency adapters and backend-neutral native observations. It SHALL NOT own public Eggprobe schema, report serialization, CLI parsing, assertions, or product policy.

`eggprobe-core` remains authoritative for `ProbePlan`/`ProbeReport`, target policy, probe/status/error semantics, normalization, deadlines, and orchestration.

The workspace-wide `unsafe_code = "forbid"` policy remains in force for Eggprobe-authored code. Reviewed dependencies may contain necessary FFI/raw-descriptor unsafe code internally.

### 2. Keep initial Phase 8 target-scoped

Phase 8 SHALL initially answer target-oriented path/host questions: local source/interface selected for a target, relevant route-table evidence, ICMP reachability, UDP behavior, observed hop path, and PMTU evidence.

A full unscoped `interfaces` or `routes` inventory command is deferred. Adding a distinct host diagnostic subject that permits plans without a target requires a later schema/architecture review.

Route evidence SHALL distinguish directly observed kernel/socket facts, Eggprobe-correlated route-table entries, and ambiguous/unavailable selection evidence. Eggprobe SHALL NOT call a correlated route entry the actual selected kernel route unless the backend can prove that selection.

### 3. Evolve the machine contract to schema 0.4

The first Phase 8 milestone SHALL advance the current schema from `0.3` to `0.4` and add typed probe/evidence/error vocabulary for native diagnostics.

Historical 0.3 fixtures remain evidence. Migration behavior for existing 0.3 plan files must be documented explicitly; current exact-version validation MUST NOT be silently changed.

### 4. Initial native probes are direct-route capabilities

Initial route/interface, ICMP, UDP, traceroute, and PMTU probes are direct/native diagnostics.

If an Eggress route is selected for a native probe whose transport cannot be represented by the current byte-stream route contract, execution SHALL return a typed unsupported result rather than silently running direct or forcing the operation through a TCP adapter.

### 5. Routed datagrams stay outside Phase 8

Eggress upstream contains listener-free UDP work, but Eggprobe's qualified integration is a byte-stream `OutboundConnector` path. Routed UDP/QUIC adoption requires a separately reviewed datagram route contract and a published/qualified sibling interface.

That work belongs to the future Phase 9 datagram/QUIC composition line.

## Consequences

Positive consequences are platform isolation, preservation of the safe core, centralized policy/normalization, and the ability to ship native diagnostics independently of routed datagrams.

Costs are a third workspace crate, explicit platform capability states, a schema 0.4 migration event, and deferred full host inventory.

Exact third-party crates, active PMTU backends, and routed datagram APIs remain implementation/dependency decisions subject to qualification.

## Security and reliability implications

- private/local target policy remains authoritative;
- multicast, broadcast, unspecified, and IPv6 link-local targets require explicit handling;
- packet/payload capture remains bounded and Phase 8 does not become a packet-capture facility;
- privilege denial is a typed diagnostic fact;
- dependency adapters must not leak unbounded OS error text;
- all active probes remain deadline-bound and cancellation-safe.

## Verification

The decision is satisfied when:

- CLI contains no independent networking;
- no Eggprobe-authored unsafe code lands;
- native backend types do not enter the public schema;
- schema 0.4 fixtures cover native variants;
- Eggress-route/native-probe mismatch is typed unsupported;
- deterministic local/synthetic tests exist for implemented primitives;
- platform limitations appear in operator docs and machine evidence.

## Supersession

None.
