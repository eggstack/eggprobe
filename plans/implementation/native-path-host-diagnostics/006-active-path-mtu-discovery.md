# Native M006 — Active Path-MTU Discovery

Status: blocked — ready after M001 and supporting Phase 8 seams close

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

Primary work class: Capability + infrastructure

## Objective

Add active PMTU diagnostics that report exact, bounded, or inconclusive evidence without interpreting packet silence as a size signal.

## Readiness and dependencies

Hard dependencies: M001 and closure-backed route/interface/UDP/path test seams from M002/M004/M005. M003 is a soft dependency where ICMP normalization can be reused.

Implementation MUST re-inspect current `main` and dependency APIs before editing. The baseline above is planning provenance, not permission to overwrite newer work.

## Current implementation evidence

M002 will expose egress interface MTU, which is not path MTU. Active PMTU requires platform-specific control over fragmentation/DF behavior and access to packet-too-big/EMSGSIZE/ICMP feedback when the OS exposes it.

Linux has explicit PMTU discovery socket behavior; Windows and macOS expose different APIs/semantics. No safe Rust backend is preselected because it must first satisfy Rust 1.89 and the workspace unsafe policy.

## Invariants

- interface MTU and path MTU are separate concepts and fields;
- timeout/silence alone never proves a packet is too large;
- search probes are bounded by size range, attempts, and deadline;
- results carry method/provenance and exact/bounded/inconclusive state;
- no Eggprobe-authored unsafe code;
- platform unsupported/deferred is acceptable and explicit.

## In scope

- qualify safe platform PMTU controls/backends;
- IPv4/IPv6 where truthful support exists;
- use M002 egress-interface MTU as context, not as an inferred PMTU;
- bounded active probing;
- exact/bounded/inconclusive result;
- observed lower/upper bounds and packet-too-big metadata when available;
- deterministic constrained-MTU topology qualification.

## Out of scope

Changing host/interface MTU, black-hole healing, TCP MSS rewriting, routed PMTU through Eggress, continuous path monitoring, or claiming complete PLPMTUD for protocols Eggprobe does not own.

## Required production changes

Add a PMTU native adapter and core evidence representation. Backend observations must distinguish successful size, explicit too-big/known-MTU feedback, timeout/no-evidence, unsupported, and permission/error states. Search logic belongs in Eggprobe-owned code over backend-neutral observations.

## Ordered work packages

1. Inspect current safe Rust socket APIs/crates for Linux/macOS/Windows PMTU controls under Rust 1.89.
2. Define capability matrix and select only backends exposing trustworthy too-big/MTU evidence.
3. Implement backend-neutral PMTU request/result types in `eggprobe-native`.
4. Implement a bounded search algorithm using only explicit success/too-big evidence.
5. Reuse target policy, route/interface context, and UDP/ICMP normalization where semantically valid.
6. Build deterministic Linux namespace/veth topology with a constrained link MTU and validate exact/bounded behavior.
7. Add platform tests for supported/unsupported/permission cases.
8. Add thin CLI, schemas, renderer, and docs.

## Failure, cancellation, restart, and contention semantics

Backend execution failure is distinct from inconclusive discovery. If small probes succeed and larger probes only disappear, return bounded/inconclusive evidence rather than fabricating an upper bound unless explicit too-big evidence exists. Cancellation does not synthesize a final MTU and must release sockets/tasks.

## Compatibility and migration

Additive schema 0.4. Egress interface MTU remains valid context even when active PMTU is unsupported/inconclusive.

## Required tests

- search algorithm exact boundary;
- lower-bound-only/inconclusive case;
- all-timeout case;
- IPv4/IPv6 normalization where supported;
- synthetic known-reduced-MTU topology;
- unsupported capability;
- permission denied;
- deadline/attempt/size bounds;
- proof no route/interface settings are mutated;
- schema/render fixtures.

## Verification

Run the workspace verification family, PMTU algorithm tests, safe-backend MSRV checks, and synthetic-topology qualification. Platform claims require host-native evidence where feasible.

## Documentation updates

Document interface MTU versus PMTU, discovery method, filtering/black-hole limitations, exact/bounded/inconclusive semantics, platform support, and configured probe bounds.

## Acceptance criteria

Eggprobe reports a PMTU only when supported by direct evidence; otherwise it returns bounds, inconclusive, or unsupported truthfully. A deterministic constrained-MTU topology validates the search behavior.

## Stop conditions

Stop if viable implementations require Eggprobe-authored unsafe code, expose only timeout heuristics, or cannot satisfy the current MSRV/platform contract. Record unsupported/deferred disposition rather than weakening invariants.

## Closure evidence

Create `plans/closure/native-path-host-diagnostics/006-status.md` with backend matrix, topology setup/results, algorithm cases, platform dispositions, and exact verification commands.

## Handoff notes

This milestone is intentionally last because it benefits from every preceding native seam and carries the highest platform-specific risk.
