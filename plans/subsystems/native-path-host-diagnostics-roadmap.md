# Native Path and Host Diagnostics Roadmap

Status: active planning; M001/M002/M004 closed, M005 conditionally closed via C005; C001/C004/C005/C007 closed, C002 corrective required, C003/M003 blocked, C006 blocked on #1881 fix publication, M006 blocked

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
| `tracert` 0.12.0 | rejected for M005 | MIT; edition 2024; public results omit silent per-hop attempts, deduplicate responders, and reverse-resolve the destination unconditionally. It also requires a separate `netdev` 0.41.x alongside 0.46.3. Still the latest published version at M005 execution; rejection retained. |
| `trippy-core` 0.13.0 | accepted for M005 data/path API; privilege qualification corrected by C005 | Apache-2.0; declared rust-version 1.78, workspace MSRV check passed on 1.89.0; structured `Round`/`ProbeStatus`/`ProbeComplete`/`CompletionReason`/`Error` API preserving silent `Awaited` attempts; no reverse-DNS crate and no second `netdev` in its tree. Post-closure hosted evidence proved the original privilege assumption was wrong: Trippy 0.13 documents unprivileged mode as macOS-only; Linux requires root/`CAP_NET_RAW` and Windows requires an elevated token. C005 owns platform-aware privilege selection/normalization and hosted requalification. Exact pin remains. Known bounds: `Classic` supports only `FixedSrc`/`FixedDest` (adapter uses per-trace unique `FixedSrc`); round completion follows backend loop wakeups (engine fixes 100 ms read granularity); `IcmpPacketCode`/`ProbeFailed` lack public exports. |
| Tokio `UdpSocket` + `socket2` 0.5/0.6 | insufficient for M006 | No `IP_MTU_DISCOVER`/`IP_DONTFRAGMENT`/`IP_RECVERR` surface in either crate at the M006 survey; connected-socket ICMP errors carry no MTU data, so only timeout heuristics would remain. |
| `nix` 0.29 | insufficient for M006 | Safe `Ipv4RecvErr`/`Ipv6RecvErr` plus errqueue parsing exist, but no `IpMtuDiscover` setter; DF control (required so probes are not silently fragmented) is missing. |
| macOS / Windows PMTU feedback | explicit-unsupported direction for M006 | Neither platform delivers packet-too-big feedback to unprivileged sockets (macOS folds it into the route cache; Windows does not deliver it); only timeout heuristics would remain, which the milestone forbids. |

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
       M005 traceroute/path [CONDITIONALLY CLOSED VIA C005]
               |
       +------+-------------------+
       |                          |
       v                          v
C007 #1793 fix qualification   M006 active PMTU
[CLOSED CASE C, #1881 FILED]   [BLOCKED INDEPENDENTLY]
       |
       v
C006 Windows live requalification
[BLOCKED ON #1881 FIX PUBLICATION]
```

M001 hard-depends on Phase 7 and ADR-0003. M002 hard-depends on M001. M003/M004 hard-depend only on M001; they soft-depend on M002 and are parallel branches after M001, not a sequence. M005 hard-depends on M001 plus backend qualification; M003/M004 are soft dependencies. M006 hard-depends on M001 and should follow M002/M004/M005 so route/datagram/path test seams exist. The Phase 9 routed-datagram boundary remains separate from this graph.

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

Current disposition: historical implementation/closure exists; C005 restored
truthful qualification without changing the schema — Linux/macOS live
dispositions qualified, Windows live execution blocked on the upstream
backend fix tracked by C006 (fail-safe `Unsupported` refusal qualified in
the meantime).

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

## 10.1 Post-closure corrective

### C001 — Planning, documentation, and execution realignment

Status: closed.

Plan: `plans/implementation/native-path-host-diagnostics-corrective/001-planning-documentation-and-execution-realignment.md`

Closure: `plans/closure/native-path-host-diagnostics-corrective/001-status.md`.

This corrective reconciled the registry and user-facing documentation after M001/M002 closure. It does not reopen their production behavior. It removed the accidental serialization of M004 behind blocked M003: after M001, ICMP research and direct UDP are independent branches.

### C002 — ping-async upstream ICMP contract enablement

Status: corrective required; historical conditional closure retained.

Plan: `plans/implementation/native-path-host-diagnostics-corrective/002-ping-async-upstream-icmp-contract-enablement.md`

Closure: `plans/closure/native-path-host-diagnostics-corrective/002-status.md`.

C002 owns the preferred M003 unblock: qualify current `ping-async` upstream and land an additive public interface that preserves exact payload-length semantics, truthful responder identity, and structured network outcomes distinct from local deadline expiry. Eggsec currently has no reusable echo primitive; Synvoid's ICMP work is filtering/platform reference material, not a suitable Eggprobe dependency.

The upstream-ready additive branch (`eggprobe/c002-additive-evidence` at `968cdec0654c2c7ef674750aba0513df071a6aba` from upstream `b3769ef1b1e8828cf3de14944cb2f80120b2e893`) adds `IcmpOutcome`, `IcmpEchoReply::responder`, `IcmpEchoReply::outcome`, `IcmpEchoRequestor::with_payload_len`, and the supporting `PING_MIN_REQUEST_DATA_LENGTH` / `PING_MAX_REQUEST_DATA_LENGTH` bounds. It compiles and passes the upstream test suite on every Eggprobe target (Linux x86_64/aarch64, macOS x86_64/arm64, Windows x86_64 MSVC/GNU); clippy with `-D warnings` is clean. PR description and patch file are prepared; upstream review/merge + crates.io publication remain the explicit C003 readiness gate.

Later review found that C002 over-closed the upstream implementation leg: short requested payloads still depended on an internal 8-byte timestamp, the responder accessor was not wired to platform receive/error sources, and the prototype commit/patch existed only in transient local paths. C004 is now the corrective authority for those gaps.

### C004 — Complete ICMP semantics and durable upstream handoff

Status: closed.

Plan: `plans/implementation/native-path-host-diagnostics-corrective/004-complete-icmp-semantics-and-durable-upstream-handoff.md`

Closure: `plans/closure/native-path-host-diagnostics-corrective/004-status.md`.

C004 completed exact 0..=1024 on-wire payload semantics without weakening reply or ICMP-error correlation, wired truthful responder identity on Unix/Linux/Windows where available, retained structured Time Exceeded versus local timeout semantics, and preserved the tested commit in durable fork branch `eggprobe/c004-icmp-semantics-and-durable-handoff` at `040771431b6e0b3f66bb17629636c8a3b4956b45` with upstream PR https://github.com/hankbao/ping-async/pull/9. C003 remains blocked on upstream acceptance/merge and publication of the qualified surface.


### C005 — Traceroute privilege and hosted-platform qualification

Status: closed.

Plan: `plans/implementation/native-path-host-diagnostics-corrective/005-traceroute-privilege-and-hosted-qualification.md`

Closure: `plans/closure/native-path-host-diagnostics-corrective/005-status.md`.

C005 replaced the invalid universal-unprivileged assumption with
platform-aware privilege selection (`trippy-privilege 0.13.0` direct pin,
read-only discovery, no acquisition, `drop_privileges(false)`), typed
`PermissionDenied/HopProbe` normalization, deterministic selection tests,
and host-aware smokes. Hosted forensics additionally proved `trippy-core`
0.13.0 privileged runs abort on elevated Windows (heap corruption at
`UnknownExtension` drop), so Windows refuses with typed
`Unsupported/HopProbe` before backend contact. Qualifying hosted run
`36156303047` on `53226a3` is fully green (Ubuntu/macOS/Windows/MSRV/audit).
M005 Linux/macOS dispositions are qualified; Windows live execution waits on
blocked C006. The closure also records the `WSAECONNRESET`-as-unreachable
hardening that keeps closed-port UDP evidence deterministic on Windows.

### C007 — Trippy #1793 Windows fix qualification and upstream handoff

Status: closed (Case C — distinct remaining defect).

Plan: `plans/implementation/native-path-host-diagnostics-corrective/007-trippy-1793-windows-fix-qualification-and-upstream-handoff.md`

Closure: `plans/closure/native-path-host-diagnostics-corrective/007-status.md`.

Post-C005 research found that the crash signature is already tracked upstream
as Trippy issue #1793. That issue records the same
`Layout::from_size_align_unchecked` panic and Windows
`0xC0000409/STATUS_STACK_BUFFER_OVERRUN` abort. Upstream identified a
`WSARecvFrom` overlapped-I/O lifetime defect: `lpFlags` pointed at the
temporary `&mut 0`; commit
`0b3c85bae2257915583d71392ea6d10c1cb4b75b` stores `recv_flags` in
`SocketImpl` so the pointer remains alive. The fix is assigned to milestone
0.14.0 and is present on current master `c0c758eb1069151eafa6bd8227bb446c2b4d1506`,
but 0.13.0 remains the latest published release.

C007 proved that this exact upstream fix does NOT resolve Eggprobe's C005
elevated Windows reproducer: the durable isolated harness
(`tools/repros/trippy-windows-1793/`, isolated `workflow_dispatch` evidence
job, runs `36166315873`/`36167102544` on elevated Windows Server 2025 with
Rust 1.89) shows registry 0.13.0 aborting 3/3 while the exact fix commit
`0b3c85b` aborts 10/10 and master `c0c758e` aborts 10/10 with the
byte-identical `UnknownExtension`-drop signature. C007 therefore filed the
distinct-defect upstream report `https://github.com/fujiapple852/trippy/issues/1881`
instead of binding C006 to #1793. Production pins and the C005 Windows
refusal are unchanged.

### C006 — Windows trace re-qualification on fixed backend

Status: blocked on a published immutable upstream
`trippy-core`/`trippy-privilege` release containing the fix for **upstream
issue #1881** (C007 proved a #1793-only release insufficient) or an
explicitly reconciled equivalent.

Plan: `plans/implementation/native-path-host-diagnostics-corrective/006-windows-trace-requalification-on-fixed-backend.md`

C005 proved elevated Windows cannot execute on the published
`trippy-core 0.13.0` and installed a version-pinned `Unsupported` refusal.
C007 qualified the already-landed #1793 fix against the Eggprobe reproducer
and found it insufficient, filing #1881 instead.
C006 re-enables and live-qualifies the privileged Windows path only after the
#1881 fix is published and consumed as an exact crates.io pin — never a production
Git SHA, fork, or vendored patch. M005's Windows disposition stays refusal
until C006 closes.

### C003 — Published ICMP backend adoption qualification

Status: blocked on C004 closure, upstream acceptance/merge, and a crates.io publication containing the C004-qualified interface.

Plan: `plans/implementation/native-path-host-diagnostics-corrective/003-published-icmp-backend-adoption-qualification.md`

C003 qualifies the immutable published artifact against Rust 1.89, all supported targets, dependency/security policy, and host-native semantics before M003 may add the production dependency. Git pins/forks are not an accepted production bridge.

## 11. Completion definition

Phase 8 closes when M001-M006 have closure evidence or an accepted unsupported/deferred disposition and every implemented primitive has a typed schema contract, bounded execution, deterministic qualification, and truthful platform documentation.

Routed UDP/QUIC and whole-host inventory are not Phase 8 closure requirements.

## 12. Milestone status

| Milestone | Status | Implementation plan | Closure | Blocker |
|---|---|---|---|---|
| M001 native contract/platform substrate | closed | `plans/implementation/native-path-host-diagnostics/001-native-contract-and-platform-substrate.md` | `plans/closure/native-path-host-diagnostics/001-status.md` | — |
| M002 target route/interface/MTU evidence | closed | `plans/implementation/native-path-host-diagnostics/002-target-route-interface-and-egress-mtu.md` | `plans/closure/native-path-host-diagnostics/002-status.md` | — |
| C002 ping-async upstream ICMP enablement | corrective required | `plans/implementation/native-path-host-diagnostics-corrective/002-ping-async-upstream-icmp-contract-enablement.md` | historical: `plans/closure/native-path-host-diagnostics-corrective/002-status.md` | Corrected by C004 |
| C004 complete ICMP semantics/durable handoff | closed | `plans/implementation/native-path-host-diagnostics-corrective/004-complete-icmp-semantics-and-durable-upstream-handoff.md` | `plans/closure/native-path-host-diagnostics-corrective/004-status.md` | PR open; C003 waits on upstream acceptance/merge and crates.io publication |
| C003 published ICMP backend qualification | blocked | `plans/implementation/native-path-host-diagnostics-corrective/003-published-icmp-backend-adoption-qualification.md` | pending | C004 closure + upstream acceptance/merge + published crates.io release |
| M003 ICMP echo diagnostics | blocked | `plans/implementation/native-path-host-diagnostics/003-icmp-echo-diagnostics.md` | pending | C004 + C003 closure |
| M004 direct UDP service diagnostics | closed | `plans/implementation/native-path-host-diagnostics/004-direct-udp-service-diagnostics.md` | `plans/closure/native-path-host-diagnostics/004-status.md` | — |
| M005 traceroute/path diagnostics | conditionally closed | `plans/implementation/native-path-host-diagnostics/005-traceroute-path-diagnostics.md` | historical: `plans/closure/native-path-host-diagnostics/005-status.md`; current: `plans/closure/native-path-host-diagnostics-corrective/005-status.md` | Linux/macOS live-qualified by C005; Windows live execution blocked on upstream backend fix (C006) |
| C005 traceroute privilege and hosted-platform qualification | closed | `plans/implementation/native-path-host-diagnostics-corrective/005-traceroute-privilege-and-hosted-qualification.md` | `plans/closure/native-path-host-diagnostics-corrective/005-status.md` | Qualifying hosted run `36156303047` on `53226a3` fully green |
| C007 Trippy #1793 Windows fix qualification/upstream handoff | closed (Case C) | `plans/implementation/native-path-host-diagnostics-corrective/007-trippy-1793-windows-fix-qualification-and-upstream-handoff.md` | `plans/closure/native-path-host-diagnostics-corrective/007-status.md` | #1793 fix proven insufficient (fix 10/10 + master 10/10 aborts); distinct defect filed as upstream #1881 |
| C006 Windows trace re-qualification on fixed backend | blocked | `plans/implementation/native-path-host-diagnostics-corrective/006-windows-trace-requalification-on-fixed-backend.md` | pending | C007 closed (Case C); requires a published immutable Trippy release containing the fix for upstream #1881 |
| M006 active path-MTU discovery | blocked | `plans/implementation/native-path-host-diagnostics/006-active-path-mtu-discovery.md` | pending | PMTU control survey (see §4) finds no qualifying DF/PTB controls and no Linux/netns validation environment in reach; C005 does not unblock that separate control gap |
