# Native Path and Host Diagnostics Roadmap

Status: active planning; M001/M002 closed, M003 blocked, M004 ready, M005/M006 blocked

Long-term references:

- `plans/000-long-term-specification.md#4-architectural-principles`
- `plans/000-long-term-specification.md#7-route-model`
- `plans/000-long-term-specification.md#8-probe-families`
- `plans/000-long-term-specification.md#11-security-model`
- `plans/000-long-term-specification.md#12-timeout-repetition-and-concurrency-model`
- `plans/000-long-term-specification.md#14-platforms-and-packaging`
- `plans/000-long-term-specification.md#15-deferred-and-future-capabilities`
- `plans/002-long-term-roadmap.md#phase-8--native-path-and-host-diagnostics`

Related ADRs:

- `plans/adrs/ADR-0001-json-first-core-and-transport-ownership.md`
- `plans/adrs/ADR-0003-native-diagnostics-platform-and-subject-boundary.md`

## 1. Purpose and ownership boundary

This subsystem adds direct/native path and host diagnostics after the initial production release.

Eggprobe owns diagnostic orchestration, target policy, evidence normalization, the public plan/report schema, and probe-specific direct primitives. Platform mechanics live behind the internal native backend boundary accepted by ADR-0003.

It does not take ownership of Eggress proxy/datagram protocols, Eggfetch HTTP/QUIC semantics, packet capture, topology crawling, or general host inventory.

## 2. Work classification

### Invariants

- machine-readable evidence remains canonical;
- the CLI remains thin;
- Eggprobe-authored code keeps `unsafe_code = "forbid"`;
- active probes have finite deadlines and bounded attempts/payloads/hops;
- direct evidence is distinguished from route-table correlation/inference;
- privilege/platform limitations are typed;
- native probes never silently bypass an Eggress route;
- routed datagrams are not represented through the byte-stream route abstraction.

### Capabilities

- target-scoped route/interface/source/MTU evidence;
- ICMP echo;
- direct UDP service diagnostics;
- traceroute/path tracing;
- active PMTU discovery with exact/bounded/inconclusive semantics.

### Infrastructure

- schema 0.4 native probe/evidence vocabulary;
- `eggprobe-native` backend crate;
- safe platform/dependency adapters;
- deterministic fake backend;
- platform capability matrix;
- synthetic topology fixtures where practical.

### Polish

- richer hop classifications;
- latency/loss summaries;
- route-correlation confidence;
- improved PMTU bounds;
- later whole-host inventory after a separate subject-model decision.

## 3. Non-goals

- no shelling out to `ping`, `traceroute`, `route`, `ip`, or `netsh`;
- no packet capture/sniffing;
- no port scanner or arbitrary raw-packet scripting;
- no whole-host route/interface dump in this milestone sequence;
- no routed UDP/QUIC in Phase 8;
- no weakening of workspace unsafe policy merely to reach an OS API.

## 4. Current state and dependency evidence

Eggprobe 0.1.1 remains the product version. Schema `0.4` is current and exact-version validation is enforced. `ProbeEngine` owns the outer deadline and target policy; `eggprobe-native` owns platform adapters. Route inspection is implemented, while ICMP/UDP/trace/PMTU request families remain explicitly unsupported until their milestones qualify.

Research and implementation qualification at the current baseline identified
these dependencies and decisions:

- `netdev` 0.46.x for cross-platform interface metadata;
- `netroute` 0.4.x for read-only Linux/macOS/Windows route-table enumeration;
- `ping-async` 1.2.x repository line for asynchronous ICMP echo using native/safe adapters subject to host policy;
- `tracert` 0.12.x for focused async ICMP/UDP traceroute with Linux capability and Windows firewall caveats.

| Crate | Decision | Qualification evidence |
|---|---|---|
| `netdev` 0.46.3 | accepted for M002 | MIT; no declared `rust-version`; workspace MSRV check passed on 1.89.0; target checks passed for Linux x86_64/aarch64, macOS x86_64/arm64, and Windows x86_64; exposes interface index/name/addresses/state/MTU. Default features are disabled to avoid unrelated metadata enrichment. |
| `netroute` 0.4.0 | accepted for M002 | MIT; no declared `rust-version`; workspace MSRV check passed on 1.89.0; exposes structured read-only route entries on supported target builds. Enumeration is correlation evidence, not an authoritative policy-route lookup. |
| `ping-async` 1.0.2 | rejected for M003 | Current crates.io version differs from the plan's noted 1.2.x line. Its public request API does not accept payload bytes/length and its reply exposes destination/status/RTT without a separate responder address. |
| `ping-rs` 0.1.2 | rejected alternative for M003 | MIT, no declared MSRV. The async Windows path unwraps ICMP handle creation and can panic on permission/OS failure; its OS error includes display text. This does not meet no-panic, safe-normalization requirements. |
| `tracert` 0.12.0 | rejected for M005 | MIT, edition 2024; public results omit silent per-hop attempts, deduplicate responders, and reverse-resolve the destination unconditionally. It also requires a separate `netdev` 0.41.x alongside 0.46.3. |

The dependency audit found no new vulnerability; it reported one pre-existing
allowed unmaintained-crate advisory (`paste` via existing dependencies).
M003 remains blocked until a safe backend or accepted upstream interface meets
payload, structured-outcome, permission, responder, and cancellation needs.

Eggress upstream also contains listener-free UDP association work, but Phase 8 does not consume it.

## 5. Target architecture

```text
ProbePlan schema 0.4
        |
        v
eggprobe-core
 contract / policy / normalization / deadlines
        |
        v
eggprobe-native
 +-- route/interface
 +-- ICMP echo
 +-- direct UDP
 +-- traceroute
 `-- PMTU
        |
        +--> safe OS/library backends
```

Backend observations remain internal.

## 6. Dependency graph

```text
Phase 7 / 0.1.1 qualification [CLOSED]
              |
              v
M001 contract + platform substrate [CLOSED]
              |
              v
M002 route/interface/egress-MTU [CLOSED]
       +------+------+
       |             |
       v             v
M003 ICMP echo   M004 direct UDP
       \             /
        \           /
         v         v
       M005 traceroute/path
              |
              v
       M006 active PMTU
```

M001 hard-depends on Phase 7 and ADR-0003. M002 hard-depends on M001. M003/M004 hard-depend on M001 and soft-depend on M002. M005 hard-depends on M001 plus backend qualification; M003/M004 are soft dependencies. M006 hard-depends on M001 and should follow M002/M004/M005 so route/datagram/path test seams exist.

## 7. Milestones

### M001 — Native contract and platform substrate

Advance to schema 0.4, introduce `eggprobe-native`, add native error/stage/capability vocabulary, define direct-only route behavior, add a fake backend, and qualify candidate dependencies/MSRV.

### M002 — Target route, interface, source, and egress-MTU evidence

Expose target-scoped local path-selection evidence while distinguishing kernel-observed source/interface facts from correlated route-table candidates.

### M003 — ICMP echo diagnostics

Add bounded IPv4/IPv6 ICMP echo with structured RTT/status evidence and explicit privilege/platform handling.

### M004 — Direct UDP service diagnostics

Add direct UDP send/receive diagnostics with bounded payload/capture and honest transmit/response/unreachable/timeout semantics.

### M005 — Traceroute/path diagnostics

Add structured ordered hop/attempt evidence without terminal scraping, with bounded hops/attempts and explicit privilege/termination states.

### M006 — Active path-MTU discovery

Add active PMTU evidence only where a safe backend can distinguish packet-too-big information from silence; report exact, bounded, or inconclusive results.

## 8. Cross-cutting requirements

Schema 0.4 is the first native contract generation. Units remain explicit. Apply target policy before active sends. Reject accidental broadcast/multicast/unspecified targets. IPv6 link-local requires explicit scope/interface semantics. Payloads and result collections remain bounded.

Every active native future must be cancellation-safe. Do not infer route selection, PMTU, reply source, or packet disposition when unavailable.

Each capability needs a Linux x86_64, Linux aarch64, macOS x86_64/arm64, and Windows x86_64 disposition: native-qualified, build-qualified, privilege-conditional, or unsupported.

## 9. Verification strategy

Routine correctness must not require the public Internet. Use fake backend contract tests, loopback route/interface tests, local UDP fixtures, ICMP loopback plus permission normalization, Linux network namespaces/veth/router chains for deterministic traceroute/PMTU where available, platform-native hosted runs, and schema/golden fixtures.

Live public traces are exploratory only.

## 10. Risks and decision points

- route-table enumeration may not prove policy-routing selection;
- ICMP availability differs by host policy;
- traceroute dependencies may duplicate `netdev` versions or add excessive footprint;
- PMTU APIs are platform-specific;
- safe dependencies may not expose enough detail;
- future routed UDP requires a datagram-specific composition contract.

## 11. Completion definition

Phase 8 closes when M001-M006 have closure evidence or an accepted unsupported/deferred disposition and every implemented primitive has a typed schema contract, bounded execution, deterministic qualification, and truthful platform documentation.

Routed UDP/QUIC and whole-host inventory are not Phase 8 closure requirements.

## 12. Milestone status

| Milestone | Status | Implementation plan | Closure | Blocker |
|---|---|---|---|---|
| M001 native contract/platform substrate | closed | `plans/implementation/native-path-host-diagnostics/001-native-contract-and-platform-substrate.md` | `plans/closure/native-path-host-diagnostics/001-status.md` | — |
| M002 target route/interface/MTU evidence | closed | `plans/implementation/native-path-host-diagnostics/002-target-route-interface-and-egress-mtu.md` | `plans/closure/native-path-host-diagnostics/002-status.md` | — |
| M003 ICMP echo diagnostics | blocked | `plans/implementation/native-path-host-diagnostics/003-icmp-echo-diagnostics.md` | pending | Current safe candidates fail payload/reply/error semantics; qualify an alternative first |
| M004 direct UDP service diagnostics | ready | `plans/implementation/native-path-host-diagnostics/004-direct-udp-service-diagnostics.md` | pending | M001 closed; M002 source/interface seam closed |
| M005 traceroute/path diagnostics | blocked | `plans/implementation/native-path-host-diagnostics/005-traceroute-path-diagnostics.md` | pending | `tracert` 0.12.0 loses silent attempts, deduplicates hops, reverse-resolves by default, and duplicates `netdev` |
| M006 active path-MTU discovery | blocked | `plans/implementation/native-path-host-diagnostics/006-active-path-mtu-discovery.md` | pending | M004 and M005 seams remain open; safe platform controls still require qualification |
