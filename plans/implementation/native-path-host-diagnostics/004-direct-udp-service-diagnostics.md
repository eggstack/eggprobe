# Native M004 — Direct UDP Service Diagnostics

Status: ready

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

Add direct UDP diagnostics with honest local-send/response/unreachable/timeout semantics and bounded payload handling.

## Readiness and dependencies

Hard dependency: Native M001 closure (closed). Soft dependency: M002 for richer source/interface evidence (closed). No Eggress interface is required.

Implementation MUST re-inspect current `main` and dependency APIs before editing. The baseline above is planning provenance, not permission to overwrite newer work.

## Current implementation evidence

Tokio is already a workspace dependency with networking enabled. Eggprobe has no datagram probe. The current Eggress route integration used by Eggprobe is byte-stream oriented and must not be reused for this milestone.

## Invariants

- successful local `send` does not prove a remote service is healthy;
- no implicit retry;
- request and response sizes are bounded;
- request payload bytes are input-only by default and not reproduced in reports;
- broadcast, multicast, unspecified, and ambiguous link-local destinations are rejected unless a later explicit policy allows them;
- Eggress route selection yields typed unsupported.

## In scope

- connected direct UDP sockets;
- IPv4/IPv6;
- optional bounded request payload;
- optional bounded receive expectation/window;
- local/source socket evidence;
- transmitted byte count;
- observed response source/byte count and bounded sample only when explicitly requested;
- ICMP/OS unreachable evidence when surfaced by connected socket semantics;
- timeout distinct from local send failure.

## Out of scope

UDP scanning, multicast/broadcast discovery, source spoofing, UDP proxying, QUIC, or a DNS protocol implementation.

## Required production changes

Add a `Udp` native probe adapter and schema 0.4 evidence carrying local transmit state separately from receive/unreachable state. Preserve the distinction between operation execution and any user assertion.

## Ordered work packages

1. Finalize UDP request/evidence fields reserved by M001.
2. Implement safe direct UDP adapter with family-correct bind behavior.
3. Apply target policy and destination-class validation before socket creation/send.
4. Add bounded send and optional receive state machine under the outer deadline.
5. Normalize OS socket errors without unbounded strings.
6. Preserve local transmit success even if response observation times out.
7. Add deterministic local echo, discard/timeout, and closed-port fixtures.
8. Add thin CLI plan construction/rendering and schema fixtures.

## Failure, cancellation, restart, and contention semantics

Bind/connect/send failures fail the probe. A transmitted datagram followed by response timeout is a diagnostic outcome distinct from local execution failure. The M001 contract determines whether that outcome is represented as negative status plus retained evidence or a completed result with an outcome field; implementation must use one rule consistently. Cancellation closes the socket and leaves no background receive task.

## Compatibility and migration

Additive schema 0.4. No routed behavior is added to `RouteSpec::Eggress`.

## Required tests

- IPv4/IPv6 loopback echo;
- no-response timeout;
- closed-port/unreachable behavior without assuming identical OS timing;
- request/response size bounds;
- port validation;
- target policy;
- multicast/broadcast/unspecified rejection;
- cancellation/no leaked socket/task;
- exact JSON fixtures and stdout cleanliness.

## Verification

Run the workspace verification family plus UDP-focused Linux/macOS/Windows tests. Platform-specific ICMP Port Unreachable delivery must be represented by the capability contract rather than forced into one brittle test expectation.

## Documentation updates

Document transmitted versus response-observed versus unreachable versus timeout semantics, size/capture limits, destination restrictions, and direct-only scope.

## Acceptance criteria

The UDP probe deterministically proves local send behavior and local-fixture response behavior without overstating remote service health, while preserving bounded machine output.

## Stop conditions

Stop if implementation requires a generic packet engine or routed-datagram abstraction. Those belong outside this milestone.

## Closure evidence

Create `plans/closure/native-path-host-diagnostics/004-status.md` with platform fixture outcomes, exact bounds, and error-normalization evidence.

## Handoff notes

Prefer existing Tokio primitives. Add a socket helper dependency only for a narrowly demonstrated portable option gap.
