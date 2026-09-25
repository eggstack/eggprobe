# Native M005 — Traceroute and Path Diagnostics

Status: historical closure retained; corrective C005 required after hosted Linux/Windows failure

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

Add structured traceroute/path evidence without scraping terminal tools, preserving ordered hop/attempt outcomes and explicit termination/privilege semantics.

## Readiness and dependencies

Hard dependency: Native M001 closure. The traceroute backend must pass M001 MSRV/API/footprint qualification. M003/M004 are soft dependencies for normalized ICMP/UDP semantics and shared fixtures.

Implementation MUST re-inspect current `main` and dependency APIs before editing. The baseline above is planning provenance, not permission to overwrite newer work.

## Current implementation evidence

`tracert` 0.12.0 exposes structured node results, but omits silent hop attempts from results, deduplicates responders, reverse-resolves destination nodes unconditionally, and depends on `netdev` 0.41.x alongside Eggprobe's accepted 0.46.3 line. It does not qualify. The plan remains blocked pending a backend that preserves bounded per-attempt evidence and obeys the no-reverse-DNS default.

## Invariants

- no subprocess `traceroute`/`tracert`;
- retain ordered per-hop/per-attempt evidence;
- silent hops are evidence, not engine failure;
- path completion and diagnostic execution success are separate;
- hop count, attempts per hop, packet size, and deadline are bounded;
- privilege denial is explicit;
- reverse DNS, if later added, is opt-in/bounded and never changes hop identity.

## In scope

- direct ICMP and/or UDP trace modes supported by the qualified backend;
- IPv4/IPv6 where supported;
- bounded first/max hop and attempts per hop;
- per-attempt responder, RTT, timeout/unreachable/target-reached state;
- explicit termination reason: destination reached, max hops, deadline, unreachable, permission denied, unsupported;
- no terminal parsing.

## Out of scope

Continuous MTR-style monitoring, Paris traceroute/ECMP characterization unless essentially free from the chosen backend, ASN/geolocation enrichment, reverse-DNS-by-default, routed Eggress traces, or packet capture.

## Required production changes

Add traceroute request/evidence types from M001 to the native adapter and core dispatch. Hop evidence must be Eggprobe-owned and independent of dependency display types. Keep trace hop indexes distinct from Eggress proxy-hop indexes.

## Ordered work packages

1. Re-run backend qualification at the current baseline:
   - Rust 1.89 build;
   - structured public hop/error API;
   - cancellation behavior;
   - Linux capability requirements;
   - Windows/macOS behavior;
   - transitive dependency and duplicate-`netdev` footprint.
2. If acceptable, add a narrow adapter in `eggprobe-native`; otherwise stop and register a backend corrective/alternative plan.
3. Normalize hop and attempt evidence into Eggprobe-owned types.
4. Enforce strict max-hop/attempt/packet/deadline bounds.
5. Add direct-only route handling and privilege normalization.
6. Build deterministic Linux synthetic topology fixtures using namespaces/veth/router hops where CI permits; retain fake-backend tests everywhere.
7. Add host-platform smoke evidence without public-Internet correctness dependence.
8. Add thin CLI, schema fixtures, renderer, and capability documentation.

## Failure, cancellation, restart, and contention semantics

A trace that executes to max hops with silent hops is a completed trace outcome. Socket creation/permission/backend failure is a failed probe. The outer deadline cancels pending hop attempts and releases sockets. Completed hop evidence may be retained on bounded termination if the 0.4 contract supports partial evidence. No background tracing survives request completion.

## Compatibility and migration

Additive schema 0.4. Do not overload `TcpEvidence` or Eggress route-hop fields.

## Required tests

- fake backend ordered hops;
- silent intermediate hop;
- destination reached;
- destination unreachable;
- max-hop termination;
- global deadline termination;
- permission denied;
- IPv4/IPv6;
- deterministic multi-hop Linux topology where capability permits;
- serialization/output bounds at configured maximums;
- no reverse-DNS side effects by default.

## Verification

Run the full workspace verification family, backend-specific tests, and synthetic-topology tests. Hosted OS evidence must record capability/firewall conditions instead of converting them to generic silent skips.

## Documentation updates

Document trace protocols, privilege requirements, firewall/ICMP filtering caveats, bounds, termination semantics, and why missing hops do not prove packet loss at a particular router.

## Acceptance criteria

A trace report is consumable without terminal parsing and preserves enough ordered evidence to distinguish reached target, silent hops, unreachable, deadline, and inability to execute.

## Stop conditions

Stop if the candidate backend exposes only formatted strings, cannot be cancelled/bounded, fails Rust 1.89, or introduces unacceptable dependency duplication/footprint. Do not silently build a broad traceroute stack in core.

## Closure evidence

Create `plans/closure/native-path-host-diagnostics/005-status.md` with backend choice rationale, privilege matrix, synthetic topology evidence, platform smoke, and exact verification results.

## Handoff notes

A richer backend can be reconsidered later only if Eggprobe explicitly adopts ECMP/Paris/MTR-class goals.


## Post-closure corrective disposition

Hosted CI after the M005 implementation invalidated one qualification
assumption from the closure record. Run `36138451900` fails
`trace_command_reaches_loopback_with_structured_hops` on ordinary Ubuntu and
Windows runners while macOS passes.

The implementation forces `PrivilegeMode::Unprivileged` on all platforms.
Trippy 0.13 documents that mode as macOS-only; Linux requires root or
`CAP_NET_RAW`, and Windows requires an elevated token. The structured trace
model remains useful, but current Linux/Windows live execution and platform
qualification are not closure-backed.

Current corrective authority:

- `plans/implementation/native-path-host-diagnostics-corrective/005-traceroute-privilege-and-hosted-qualification.md`

Do not rewrite the historical M005 closure record. C005 must supply the
regression evidence and current qualification.
