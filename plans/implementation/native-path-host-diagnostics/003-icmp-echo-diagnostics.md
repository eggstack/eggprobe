# Native M003 — ICMP Echo Diagnostics

Status: blocked — safe ICMP backend qualification

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

Primary work class: Capability

## Objective

Add bounded IPv4/IPv6 ICMP echo diagnostics with structured RTT/reply evidence, explicit privilege handling, and no subprocess backend.

## Readiness and dependencies

Hard dependency: Native M001 closure. Soft dependency: M002 for richer interface/source evidence. The ICMP backend selected by M001 must have acceptable Rust 1.89 and platform behavior.

Implementation MUST re-inspect current `main` and dependency APIs before editing. The baseline above is planning provenance, not permission to overwrite newer work.

## Current implementation evidence

Eggprobe has no ICMP path. Current review found `ping-async` 1.0.2 does not accept a caller-selected payload or report a separate responder address. The alternative `ping-rs` 0.1.2 async Windows path unwraps handle creation, so permission/OS failures may panic; it also exposes OS error display text. Neither candidate qualifies. M001 and M002 are closed, but implementation remains blocked until a safe backend or accepted upstream API supplies bounded payload, structured reply/error evidence, cancellation, and permission normalization.

## Invariants

- no shell `ping`;
- finite count, interval, payload size, and deadline;
- request payload content is not emitted in reports;
- timeout, unreachable, permission denial, and unsupported remain distinct;
- target policy applies before send;
- ICMP failure never implies TCP/HTTP failure.

## In scope

- direct-route ICMP echo for IPv4 and IPv6 where supported;
- bounded count and payload length;
- per-attempt sequence/outcome/RTT;
- aggregate sent/received/lost and latency summary only if raw attempts remain available;
- reply TTL/hop-limit only when directly observable;
- explicit capability/permission diagnostics.

## Out of scope

Traceroute, arbitrary ICMP messages, flood mode, packet capture, and routed ICMP through Eggress.

## Required production changes

Add an ICMP probe through `eggprobe-native` and normalize backend results into schema 0.4. The core must own status/error semantics; dependency-specific statuses must not leak into JSON.

## Ordered work packages

1. Requalify the selected ICMP crate/API at execution baseline.
2. Implement the native ICMP adapter without exposing dependency types.
3. Map timeout/unreachable/permission/platform outcomes to Eggprobe taxonomy.
4. Enforce count/payload/interval/deadline bounds before socket creation.
5. Preserve attempt evidence and derive summaries without hiding attempts.
6. Add direct-only engine dispatch and typed unsupported for Eggress routes.
7. Add thin CLI command and human renderer.
8. Add schema fixtures and platform capability documentation.

## Failure, cancellation, restart, and contention semantics

Each attempt is bounded by the outer deadline and any narrower per-attempt budget. Cancellation drops/closes the backend operation. Permission denial is a failed operation with `PermissionDenied`, not `Unsupported`; unsupported means the platform/backend cannot provide the operation. Partial completed attempts may be retained if the 0.4 contract supports partial evidence.

## Compatibility and migration

Additive schema 0.4 evidence. Existing reachability probes retain their current meaning.

## Required tests

- loopback IPv4 success;
- loopback IPv6 where enabled;
- timeout normalization with fake backend;
- destination/network/host-unreachable normalization where the backend distinguishes them;
- permission-denied normalization;
- invalid/excessive count and payload bounds;
- outer deadline/cancellation;
- target-policy rejection;
- JSON and renderer parity;
- hosted platform smoke with capability/permission disposition recorded.

## Verification

Run workspace verification from M001 plus focused `eggprobe-native` ICMP tests and hosted Linux/macOS/Windows evidence. Record the Linux ping-group/raw-socket conditions of the qualifying runner.

## Documentation updates

Document privilege requirements, platform backend differences, ICMP filtering caveats, configured bounds, and the fact that ICMP silence is not general service failure.

## Acceptance criteria

ICMP echo produces bounded typed attempt evidence on each qualified platform and turns privilege/platform limitations into stable machine-readable states.

## Stop conditions

Stop if the selected dependency exposes only display strings, cannot distinguish the required outcomes safely, violates deadline/cancellation invariants, or requires weakening Eggprobe's unsafe policy.

## Closure evidence

Create `plans/closure/native-path-host-diagnostics/003-status.md` with backend version, platform privilege matrix, loopback evidence, limits, and exact verification outcomes.

## Handoff notes

Keep ping narrow. ICMP Time Exceeded handling needed for path tracing belongs to M005 unless the selected backend exposes it as structured reusable evidence.
