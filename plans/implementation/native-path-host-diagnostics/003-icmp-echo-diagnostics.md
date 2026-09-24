# Native M003 — ICMP Echo Diagnostics

Status: blocked — corrective C002/C003 upstream enablement and published-backend qualification

Repository baseline: `d9c954ca4967796eb322b6f4e4cde738e3f29f49`

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

Hard dependencies: Native M001 closure plus Native corrective C002 and C003 closure. C002 owns the upstream `ping-async` evidence-contract enablement; C003 owns qualification of the immutable published crates.io release. Soft dependency: M002 for richer interface/source evidence.

Implementation MUST re-inspect current `main` and dependency APIs before editing. The baseline above is planning provenance, not permission to overwrite newer work.

## Current implementation evidence

Eggprobe has no implemented ICMP path; schema 0.4 reserves the request/evidence family and currently dispatches it as typed unsupported. The original review rejected published `ping-async` 1.0.2 because it did not expose the payload/responder/error semantics required by Eggprobe. A later upstream review found `ping-async` master source version 1.2.0 at `b3769ef1b1e8828cf3de14944cb2f80120b2e893` with materially improved cancellation, router-failure, Linux error-queue, Windows error, and completion-timestamp behavior, but the public interface still discards facts Eggprobe needs. Corrective C002/C003 now own that unblock. Do not substitute `ping-rs`, Synvoid filtering code, or an unpublished Git dependency.

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

1. Confirm C002/C003 closure and adopt only the exact published `ping-async` version qualified there.
2. Add the dependency behind `eggprobe-native`; do not expose dependency types above that crate.
3. Implement a backend-neutral ICMP observation carrying sequence, truthful optional responder, RTT, and exact network/deadline outcome.
4. Synthesize only bounded payload content needed to satisfy the requested `payload_bytes`; report the configured length, never payload contents.
5. Map upstream outcomes without inference:
   - echo reply -> `NativeAttemptOutcome::Reply`;
   - local request deadline -> `TimedOut`;
   - network-generated Time Exceeded -> `TimeExceeded`;
   - unreachable subtype -> the narrowest available destination/network/host variant;
   - local permission failure -> `DiagnosticErrorKind::PermissionDenied`;
   - unsupported platform/backend -> typed unsupported.
6. Enforce count/payload/deadline bounds before native execution and preserve one-based Eggprobe attempt ordering independently of upstream sequence allocation.
7. Preserve completed attempts when the outer deadline ends later work; cancellation must leave no waiter/background request owned by Eggprobe.
8. Add direct-only engine dispatch and typed unsupported for Eggress routes.
9. Add thin CLI command/human renderer without moving networking into the CLI.
10. Add schema/golden fixtures and platform capability documentation; schema remains 0.4 unless implementation proves the reserved contract insufficient.

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

Stop if C002/C003 evidence is absent, the published dependency differs from the qualified interface, responder identity would have to be fabricated, Time Exceeded cannot be distinguished from a local deadline, expected OS errors may panic, cancellation leaks request state, or integration requires weakening Eggprobe's unsafe policy.

## Closure evidence

Create `plans/closure/native-path-host-diagnostics/003-status.md` with backend version, platform privilege matrix, loopback evidence, limits, and exact verification outcomes.

## Handoff notes

Keep ping narrow. C002 deliberately asks the upstream backend not to discard Time Exceeded/responder facts because they may later unblock M005, but M003 must not expand into traceroute or PMTU implementation.
