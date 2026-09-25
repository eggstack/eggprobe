# Native M005 Closure — Traceroute and Path Diagnostics

Status: closed

Source plan: `plans/implementation/native-path-host-diagnostics/005-traceroute-path-diagnostics.md`

Source roadmap: `plans/subsystems/native-path-host-diagnostics-roadmap.md`

Reviewed repository baseline: `6e89470` (`main` after the M004 closure, before implementation). The plan's `8055730…` provenance baseline was stale; current `main`, architecture ownership, and dependency APIs were inspected before changes. The plan's registered blocker (rejected `tracert` 0.12.0) was re-qualified at this baseline and replaced (see below).

Implementation: `6e89470..2afc2e9` (unpushed at closure drafting; push follows with the closure commit).

## Finding

The direct-only `trace` probe and `eggprobe trace` CLI command are implemented. Ordered per-hop/per-attempt evidence is preserved, silent hops are `TimedOut` observations rather than engine failures, termination is explicit, privilege denial is typed, and no terminal tool is scraped. The `tracert` 0.12.0 rejection stands (still the latest published version); the unblock came from newly published `trippy-core` 0.13.0, qualified below.

## Backend qualification (work package 1)

| Criterion | Evidence |
|---|---|
| Rust 1.89 build | `trippy-core =0.13.0` (exact pin) declares `rust-version 1.78`; workspace `cargo +1.89.0 check` passes. |
| Structured public hop/error API | `Round{probes, largest_ttl, reason}` with pub fields and `new()`; `ProbeStatus::{NotSent, Skipped, Failed, Awaited, Complete}` preserves silent attempts as `Awaited`; `ProbeComplete{ttl, host, sent, received, icmp_packet_type, …}` with pub fields; `CompletionReason::{TargetFound, RoundTimeLimitExceeded}`; `Error::{PrivilegeError, …}`. |
| Silent-attempt truth | Spike traces keep unanswered TTLs as `Awaited` probes with send timestamps; the adapter maps them to `TimedOut` attempts. |
| No reverse DNS | `host` is a raw `IpAddr`; the dependency tree (`cargo tree -p eggprobe-native`) contains no resolver/DNS crate — `trippy-dns` is a separate crate that is not depended on. Reports were asserted to contain no resolved names. |
| No `netdev` duplication | `trippy-core` depends on `socket2 0.5` (already in the lockfile at 0.5.10), `parking_lot`, `thiserror`, `tracing`, `trippy-packet`, `trippy-privilege` — no second `netdev`. |
| Cancellation/boundedness | Blocking backend runs on `spawn_blocking`; `max_rounds`/`max_round_duration`/`read_timeout` bound it; the engine divides the remaining outer budget across rounds with a 100 ms per-round margin and wraps the join in the remaining timeout, retaining partial rounds as `Deadline` termination. Overrun after engine timeout is bounded by the round budget, recorded as a known limitation. |
| Linux capability requirements | Unprivileged UDP mode needs no capabilities; verified live on this host plus loopback on all CI families. Privileged raw-socket mode is never selected. |
| Footprint/license | Apache-2.0; `cargo audit` shows no new advisories (one pre-existing allowed `paste` note). |
| `tracert` status | Still 0.12.0 on crates.io with the same defects; rejection retained. |

Two backend realities found during qualification are handled in the adapter, not hidden:

- `Classic` + `FixedBoth`/`None` port directions hit an upstream `unimplemented!()`; the adapter uses `Classic` + `FixedSrc` with a per-trace unique source port (atomic `43534..44534` window), which also isolates concurrent batch traces that previously collided on the default bind.
- Round completion is evaluated on backend loop wakeups, so `read_timeout` sets completion latency, not reply cutoffs; the engine fixes it at 100 ms (upstream default scale) instead of scaling it with the budget. Slow replies are still recorded whenever they arrive before the round budget ends.

## Requirements and evidence

| Requirement | Evidence |
|---|---|
| Direct ICMP and/or UDP trace modes | UDP mode implemented; ICMP echo mode stays reserved for the M003 backend (documented, typed unsupported). |
| IPv4/IPv6 | Loopback echo traces pass for `127.0.0.1` and `::1` in native, engine, and CLI tests. |
| Bounded first/max hop and attempts | Plan validation enforces `max_hops 1–64`, `attempts_per_hop 1–5` (`InvalidNativeBounds`); hops capped at `max_hops`, attempts at the round count; max-bound output serializes. |
| Per-attempt responder, RTT, state | `TraceAttempt{responder, rtt_micros, outcome}`; RTT from backend send/receive timestamps with clock-skew tolerance (`None` on negative deltas). |
| Explicit termination | `DestinationReached` (any target reply), `Unreachable` (router-reported), `MaxHops` (completed without target), `Deadline` (bounded wait expired, partial rounds kept), `PermissionDenied`/`Unsupported` (backend errors). |
| No terminal parsing | Pure `trippy-core` API consumption; no subprocess. |
| Direct-only route handling | Eggress plans yield typed `Unsupported` with no fallback (tested). |
| Privilege normalization | `PrivilegeError` → `PermissionDenied/HopProbe` with a fixed message; dependency `Display` (addresses, interfaces) is never forwarded. |
| Thin CLI, schema fixtures, renderer | `eggprobe trace <target> [--max-hops] [--attempts] [--json]` (no port flag exists — ports are backend-internal and invisible in reports); CLI tests cover reached evidence, stderr cleanliness, and usage errors. No domain type changed, so schema 0.4 artifacts regenerate byte-identical. |

## Platform qualification

| Target | Evidence and disposition |
|---|---|
| macOS hosted runner (local) | Full workspace verification family passed (see below). Loopback v4/v6 reached traces, scripted normalization tests, and a live two-hop observation (gateway `TimeExceeded` + silent second hop, `MaxHops`) all pass. |
| Linux x86_64 / Windows x86_64 | Hosted CI matrix covers format/clippy/tests; unprivileged UDP fixtures need no capabilities. Windows firewall behavior is recorded per-attempt (silent hops stay `TimedOut` evidence) rather than skipped. |
| Linux aarch64 / macOS aarch64 | Build-qualified via the existing target-check posture; no separate native trace smoke claim is made. |

No Internet-dependent test was committed. Deterministic coverage comes from scripted `Round` fixtures (ordering, silent hops, unreachable, max-hop, deadline, TTL filtering, unsent-probe exclusion) plus loopback integration. A Linux namespace/veth synthetic topology was not built: the dev host has no netns capability and shared CI jobs run unprivileged; this is recorded as deferred under the plan's "where CI permits" clause, not as a silent cut.

## Verification

The exact repository verification sequence passed locally at `2afc2e9`:

```text
cargo fmt --all -- --check                         PASS
cargo check --workspace --all-targets --all-features --locked PASS
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings PASS
cargo test --workspace --all-features --locked     PASS (93 tests across 14 suites)
cargo +1.89.0 check --workspace --all-targets --locked PASS
cargo tree --locked                                PASS
cargo audit                                        PASS (one pre-existing allowed unmaintained `paste` advisory)
```

Schema regeneration produces no diff; the `expression` leak assertion passed unchanged. New dependency footprint: `trippy-core`, `trippy-packet`, `trippy-privilege` (+ already-present transitive crates), all locked.

## Review and disposition

- Compatibility/schema: additive behavior only; no domain field added or altered; historical schemas untouched.
- Security/privacy: policy checks run before backend use; fixed error messages only; responder addresses are observed network facts (no reverse lookup); no credentials or dependency strings reach reports. Trace never touches an Eggress route; trace TTL indexes never mix with Eggress hop indexes.
- Failure/cancellation/resources: one bounded backend run per probe; no background tracing beyond a round-budget-bounded overrun after engine timeout (documented above); per-trace source ports prevent concurrent-trace contention.
- Documentation/operations: operator guide documents protocols, privilege posture, firewall caveats, bounds, termination semantics, and why missing hops do not prove loss at a router; `architecture/engine-transport.md` records the new §2c adapter section.
- Known limitations: (1) `trippy-core` 0.13.0 does not re-export `IcmpPacketCode`/`ProbeFailed`/`Probe` failure constructors, so the `TimeExceeded`/`Reply`/individual-`Failed` extraction arms are pinned by live-gateway evidence (this host) and review rather than literal unit construction; an upstream export request is follow-up, not a blocker. (2) ICMP trace mode awaits M003. (3) Namespace-topology fixtures deferred as above.
- Unresolved findings: none against M005 acceptance. M006 still requires its own PMTU-control qualification (assessed next).

Roadmap disposition: M005 closed. M006 remains blocked pending trustworthy platform PMTU control qualification; its route/UDP/path test seams now exist.
