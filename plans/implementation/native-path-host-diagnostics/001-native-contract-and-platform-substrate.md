# Native M001 — Native Contract and Platform Substrate

Status: closing

Repository baseline: `8055730df76f187d94ff83b810d93cb935bb8c37`

Source roadmap:

- `plans/subsystems/native-path-host-diagnostics-roadmap.md`

Long-term requirements:

- `plans/000-long-term-specification.md#4-architectural-principles`
- `plans/000-long-term-specification.md#7-route-model`
- `plans/000-long-term-specification.md#11-security-model`
- `plans/000-long-term-specification.md#12-timeout-repetition-and-concurrency-model`
- `plans/000-long-term-specification.md#14-platforms-and-packaging`
- `plans/000-long-term-specification.md#15-deferred-and-future-capabilities`
- `plans/002-long-term-roadmap.md#phase-8--native-path-and-host-diagnostics`

Applicable ADRs:

- `plans/adrs/ADR-0001-json-first-core-and-transport-ownership.md`
- `plans/adrs/ADR-0003-native-diagnostics-platform-and-subject-boundary.md`

Primary work class: Invariant + infrastructure

## Objective

Create the Phase 8 contract/platform foundation without yet claiming route inspection, ICMP, UDP, traceroute, or PMTU capability.

## Readiness and dependencies

Hard dependencies are closed: Phase 7/Release M004 is closure-backed and ADR-0003 is accepted. No sibling-project interface is required.

Implementation MUST re-inspect current `main` and dependency APIs before editing. The baseline above is planning provenance, not permission to overwrite newer work.

## Current implementation evidence

- schema `0.3` is current and plans are exact-version validated;
- `ProbeSpec`, `ProbeKind`, and `ProbeEvidence` cover DNS/TCP/TLS/HTTP only;
- `DiagnosticErrorKind` has no explicit permission-denied category;
- `ProbeEngine` owns the outer deadline and target policy;
- workspace members are `eggprobe-core` and `eggprobe-cli`;
- workspace lints forbid unsafe code;
- candidate native dependencies are researched but not repository-qualified.

## Invariants

- CLI remains network-free;
- native backend types never become public schema;
- no Eggprobe-authored unsafe code;
- schema change is explicit and fixture-backed;
- direct-only native route semantics never fall back silently from Eggress;
- existing DNS/TCP/TLS/HTTP behavior remains unchanged.

## In scope

- advance `SchemaVersion::CURRENT` from 0.3 to 0.4;
- add internal `eggprobe-native` crate and backend-neutral request/observation/error seam;
- reserve typed native probe/evidence vocabulary required by M002-M006;
- add permission/capability-aware error and stage vocabulary;
- add fake backend for deterministic core normalization tests;
- define platform capability and unsupported semantics;
- qualify candidate dependencies against Rust 1.89, dependency policy, structured API sufficiency, and footprint;
- generate 0.4 schemas/golden fixtures and retain historical 0.3 evidence.

## Out of scope

No real route/interface enumeration, ICMP send, UDP service probe, traceroute, PMTU, whole-host inventory, routed datagrams, or QUIC.

## Required production changes

The 0.4 contract must use concrete typed variants rather than generic JSON maps. It must be able to represent:

- observed source/interface facts separately from correlated route entries;
- ICMP destination/family/sequence/payload/RTT/outcome;
- UDP local transmit, response, unreachable, and timeout evidence;
- ordered trace hops and attempts plus termination reason;
- PMTU exact/bounded/inconclusive evidence.

Add a stable `PermissionDenied` category (or semantically equivalent type). Add native stages narrow enough for route/interface inspection, packet send/reply, hop probing, and PMTU discovery without importing dependency-specific enums.

## Ordered work packages

1. Re-audit version/fixture behavior and document 0.3 -> 0.4 migration semantics.
2. Add `eggprobe-native` to the workspace; keep public schema ownership in core.
3. Define backend-neutral native types and a deterministic fake backend.
4. Extend core plan/probe/error/evidence types for Phase 8 families.
5. Wire dispatch so not-yet-implemented native operations return typed unsupported, never internal errors.
6. Make Eggress-route/native-probe combinations explicitly unsupported.
7. Qualify `netdev`, `netroute`, `ping-async`, and `tracert` candidates for MSRV, platform/API needs, licenses/security, and dependency duplication.
8. Generate `schemas/plan-0.4.json` and `schemas/report-0.4.json`; update fixtures.
9. Update schema/operator/architecture documentation.

## Failure, cancellation, restart, and contention semantics

No real long-running native I/O lands here. Fake backend paths still exercise outer-deadline and error/status normalization. Unsupported and permission failures are probe results, not panics. Do not introduce a global mutable native runtime or background worker.

## Compatibility and migration

Schema 0.4 becomes current. Historical 0.3 fixtures remain evidence. Document whether the binary continues exact-current-only plan acceptance or gains an explicit narrow migration path; do not change acceptance implicitly. Tool version and schema version remain distinct.

## Required tests

- 0.4 plan/report round trips and deterministic schemas;
- retained historical 0.3 fixtures;
- every native `ProbeSpec` maps to its `ProbeKind`;
- fake backend success/failure/unsupported/permission normalization;
- Eggress route + native probe cannot execute direct;
- target policy runs before active-native dispatch where applicable;
- current contract/engine/CLI tests remain green.

## Verification

```text
cargo fmt --all -- --check
cargo check --workspace --all-targets --all-features --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test --workspace --all-features --locked
cargo +1.89.0 check --workspace --all-targets --locked
cargo tree --locked
cargo audit
```

Record separate Rust 1.89/API/footprint results for each dependency candidate accepted or rejected.

## Documentation updates

README contract version; `docs/operator.md`; schema documentation; architecture documentation for `eggprobe-native`; subsystem roadmap/registry status; dependency rationale.

## Acceptance criteria

Schema 0.4 is canonical and fixture-backed; `eggprobe-native` compiles on supported target CI without Eggprobe-authored unsafe code; fake backend proves normalization; downstream milestones have explicit dependency decisions; no native command falsely claims implementation.

## Stop conditions

Stop and register corrective/decision work if a required contract cannot fit schema 0.4, a required backend forces Eggprobe-authored unsafe code, Rust 1.89 cannot be met without a reviewed alternative, or target-scoped semantics prove insufficient.

## Closure evidence

Create `plans/closure/native-path-host-diagnostics/001-status.md` with commit range, schema diff, dependency qualification matrix, exact verification outcomes, and M002-M006 readiness changes.

## Handoff notes

Build the smallest backend seam needed by the known Phase 8 primitives. Do not create a plugin system or generic packet engine.
