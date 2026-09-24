# Native M002 — Target Route, Interface, and Egress-MTU Evidence

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

Primary work class: Capability

## Objective

Add target-scoped local path-selection evidence: kernel-observed source address where available, correlated interface metadata, route-table evidence, and egress interface MTU without overstating correlation as authoritative route selection.

## Readiness and dependencies

Hard dependency: Native M001 closure. Candidate interface/route dependencies must have been accepted by M001.

Implementation MUST re-inspect current `main` and dependency APIs before editing. The baseline above is planning provenance, not permission to overwrite newer work.

## Current implementation evidence

Current TCP evidence may include local/peer socket addresses, but Eggprobe has no interface identity/state/MTU, gateway, metric, table, protocol, or route-correlation evidence.

Research indicates `netdev` can provide cross-platform interface metadata and `netroute` read-only route-table entries. Table enumeration alone does not prove Linux policy-routing or source-specific selection.

## Invariants

- label evidence provenance;
- do not conflate kernel-observed source selection with route-table correlation;
- keep ambiguity/unavailable states explicit;
- introduce no route mutation authority;
- no shell command parsing;
- apply target policy before target-scoped observation.

## In scope

- resolve/select a target address under existing policy;
- obtain kernel-selected local/source address using a no-application-payload socket technique where portable;
- map source address to interface index/name/addresses/state/MTU;
- enumerate/correlate route-table candidates by family/prefix/interface;
- retain gateway/metric/table/protocol/scope only when observed;
- expose selection provenance such as observed source, correlated route, ambiguous, unavailable.

## Out of scope

Whole-host route/interface dump, route mutation, a policy-routing rules engine, VPN proprietary metadata, active PMTU, or claiming a candidate is selected when unproven.

## Required production changes

Add a target-scoped route/interface probe backed by `eggprobe-native`. Evidence should include the selected/resolved target address, observed local source when available, interface identity and MTU, bounded route candidates, and explicit provenance/confidence. Do not name a field `selected_route` unless the backend actually performs a kernel route lookup with authoritative semantics.

## Ordered work packages

1. Implement bounded interface enumeration adapter.
2. Reuse core target resolution/policy semantics for the target address.
3. Implement kernel source-address observation without sending application payload where supported.
4. Correlate source address with interface metadata.
5. Enumerate route entries and perform deterministic prefix/candidate correlation.
6. Normalize missing/ambiguous fields without fabricating defaults.
7. Add thin CLI plan constructor/renderer, for example `eggprobe route <target> --json`.
8. Add schemas/golden fixtures and platform documentation.

## Failure, cancellation, restart, and contention semantics

Interface/route reads are bounded local operations. Resolution/source selection stays within the plan deadline. Missing optional route metadata yields unavailable evidence rather than failing an otherwise valid observation. A backend failure preventing the requested diagnostic yields a typed failed probe. No persistent state is mutated.

## Compatibility and migration

Additive within schema 0.4. Existing DNS/TCP semantics remain unchanged.

## Required tests

- loopback source/interface correlation;
- IPv4 and IPv6 where available;
- route-prefix correlation including longest-prefix, equal-prefix ambiguity, gateway/no-gateway;
- bounded list behavior;
- strict-policy rejection before backend action;
- fake backend with missing ifindex/name/MTU/table;
- Linux/macOS/Windows native smoke;
- serialization proof that correlation-only evidence is not labeled authoritative.

## Verification

Run the M001 workspace verification family plus focused native route/interface tests and hosted platform CI. Linux aarch64 may remain build-qualified unless native SBC evidence is deliberately added.

## Documentation updates

Document observed versus correlated fields, interface-MTU semantics, policy-routing limitations, platform field availability, and direct-only route scope.

## Acceptance criteria

A target-scoped route probe produces stable source/interface/MTU evidence and truthful bounded route candidates across qualified systems, preserving ambiguity and unavailable facts.

## Stop conditions

Stop if accepted dependencies cannot expose enough identity to correlate interface/route evidence safely or a platform requires Eggprobe-authored unsafe code. Register a narrow backend corrective instead of parsing command output.

## Closure evidence

Create `plans/closure/native-path-host-diagnostics/002-status.md` with platform matrix, observed-vs-correlated examples, dependency versions, exact verification results, and known routing limitations.

## Handoff notes

The purpose is diagnostic evidence, not reimplementation of each OS routing-policy engine.
