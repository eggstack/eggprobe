# Native C002 Closure — ping-async Upstream ICMP Contract Enablement

Status: conditionally closed — upstream-ready additive patch landed on the
qualification branch; accepted upstream merge and crates.io publication
remain pending external coordination

Source implementation plan:

- `plans/implementation/native-path-host-diagnostics-corrective/002-ping-async-upstream-icmp-contract-enablement.md`

Source roadmap:

- `plans/subsystems/native-path-host-diagnostics-roadmap.md`

Planning baseline (the planning registration commit):

- `5b7d208 plans: register ping-async ICMP enablement path`

Reviewed repository baselines:

- `d9c954ca4967796eb322b6f4e4cde738e3f29f49` — plan baseline named in C002
- `6e45e857e6a621f9e5a895ab7618a8b6dbaa6d6b` — closure-backed product
  implementation head (Native M001/M002); C002 did not reopen it
- `53ea53d14560c150d0ebc10de83eca10de37202d` — qualified release of record
  (`v0.1.1`); C002 did not bump it

Upstream reviewed state:

- crates.io index at the planning baseline: `ping-async 1.0.2` published
  `2026-02-03` (crate id `824acfdc…`, license `MIT`, edition `2021`,
  declared `rust-version = null`)
- `hankbao/ping-async` `master` at commit
  `b3769ef1b1e8828cf3de14944cb2f80120b2e893` reports source version
  `1.2.0` and contains the post-1.0.2 reliability work (cancellation-safe
  request registration, persistent router-failure handling, Linux error-queue
  processing, Windows handle errors returned through `Result`, monotonic
  completion timestamps). It is **not yet published** to crates.io.
- Local qualification clone: `/tmp/opencode/ping-async-master` (branch
  `eggprobe/c002-additive-evidence`, head `968cdec0654c2c7ef674750aba0513df071a6aba`)
- Upstream patch file:
  `/tmp/opencode/ping-async-c002.patch`
- Upstream PR description:
  `/tmp/opencode/ping-async-pr-description.md`

This corrective is **upstream-side only**: no Eggprobe production Rust,
schema, dependency, CLI, or assertion changed during its execution. It
prepares an upstream-mergeable additive surface for `ping-async` so that
M003 can take a published crates.io dependency.

## Executive finding

C002's preferred path is **partially complete**: the additive public API
(`IcmpOutcome`, `IcmpEchoReply::responder`, `IcmpEchoReply::outcome`,
`IcmpEchoRequestor::with_payload_len`, `PING_MIN_REQUEST_DATA_LENGTH`,
`PING_MAX_REQUEST_DATA_LENGTH`) is implemented, builds on every Eggprobe
target, passes the upstream test suite + clippy with `-D warnings`, and
is committed on the qualification branch. The remaining work is **external
coordination**: the upstream maintainer must accept and merge the patch,
then publish a new crates.io release containing the accepted surface.

C003 therefore **cannot** close yet — it requires an immutable
crates.io release with the accepted interface, and no such release
exists. C002 closes **conditionally** with the additive API landed on
the upstream-ready branch and a precise PR description ready for the
maintainer.

## WP1 — Reproduce and pin the upstream research baseline

Reinspected state and license posture:

| Item | Value |
|---|---|
| crates.io latest version | `1.0.2` (published `2026-02-03`) |
| crates.io license | `MIT` |
| crates.io declared `rust-version` | `null` |
| crates.io edition | `2021` |
| crates.io dependencies | `futures ^0.3`, `static_assertions ^1.1.0`, `tokio ^1.49.0` (normal), `tokio ^1.49.0` (dev), `rand ^0.9.2`, `socket2 ^0.6.2`, `windows ^0.62.2` |
| GitHub master source version | `1.2.0` |
| GitHub master HEAD commit | `b3769ef1b1e8828cf3de14944cb2f80120b2e893` |
| GitHub master license | `MIT` (same author `hankbao <hankbao84@gmail.com>`) |
| GitHub master license file | `LICENSE` present, SPDX `MIT` |
| GitHub CI | `cargo fmt --check`, `cargo clippy -D warnings`, `cargo build`, `cargo test` on ubuntu/macos/windows + `cargo audit` |

Build and test verification on the unmodified upstream master:

| Command | Result |
|---|---|
| `cargo build` (host: `aarch64-apple-darwin`) | finished, 47.04 s |
| `cargo build --target aarch64-unknown-linux-gnu` | finished, 4.12 s |
| `cargo build --target aarch64-apple-darwin` | finished |
| `cargo build --target x86_64-apple-darwin` | finished |
| `cargo build --target x86_64-pc-windows-msvc` | finished |
| `cargo build --target x86_64-pc-windows-gnu` | finished |
| `cargo test --lib` | 52 unit tests pass |
| `cargo test` (full) | 52 unit + 6 integration + 6 doc tests pass |
| `cargo clippy --all-targets --all-features -- -D warnings` | clean |

Deterministic test coverage inspected (all green on host):

- cancellation: `cancelled_send_leaks_no_registry_entry`,
  `run_request_pending_send_stage_times_out`,
  `run_request_no_reply_times_out_at_deadline`,
  `run_request_reply_before_deadline`,
  `run_request_slow_send_stage_is_bounded_by_deadline`,
  `run_request_maps_send_errors`, `run_request_reports_closed_reply_channel`
- concurrent requests: `test_concurrent_pings`,
  `test_multiple_requestors_independent_routers`,
  `test_multiple_requestors_same_target`, `test_high_concurrency`,
  `test_rapid_firing`, `test_thread_safety`,
  `test_lazy_router_spawning`
- reply correlation: `stale_reply_with_wrong_timestamp_is_rejected`,
  `allocate_skips_occupied_and_wraps`,
  `allocate_fails_when_exhausted_without_consuming_an_id`,
  `with_completed_at_stores_instant_and_stays_copy`,
  `test_parse_error_reply_*` (Linux/macOS/Win-shaped), `wake_action_table`,
  `blocked_wake_sees_publication_before_classifying`
- Linux error queue: `parse_extended_error_table`,
  `would_block_wake_drains_and_dispatches`,
  `cross_drainer_interleaving_never_fatal_on_first_sight`,
  `send_after_fatal_failure_fails_fast_without_registering`,
  `registration_never_survives_fatal_shutdown`
- Windows handle/error: `send_propagates_immediate_local_error`
- loopback success: `ping_localhost_v4`, `ping_localhost_v6`

`PING_ASYNC_UNREACHABLE_TARGET` / `PING_ASYNC_TTL1_TARGET` / `PING_ASYNC_V6_BLACKHOLE_TARGET`
environment-gated fixtures (per `CLAUDE.md`) are host-network-dependent and
are not exercised here; they remain the upstream CI's responsibility.

## WP2 — Evidence-gap matrix

Mapping Eggprobe M003 schema 0.4 evidence fields to current upstream
availability at `b3769ef`:

| M003 evidence field | Upstream availability | Gap class | Resolution path |
|---|---|---|---|
| `ProbeSpec::IcmpEcho { count, payload_bytes }` (caller payload 0..=1024) | `PING_DEFAULT_REQUEST_DATA_LENGTH = 32`, no caller setter; `send()` always sends 8 bytes of timestamp correlation in the payload | requires upstream change | `with_payload_len(usize)` accepted in this patch; wire-format change deferred to follow-up PR (see WP3 follow-ups) |
| `IcmpAttempt { sequence }` | `IcmpEchoReply::destination/status/round_trip_time/completed_at`; sequence is implicit in `IcmpPacket::sequence()` parse but **not exposed on the reply** | requires upstream change | sequence is preserved internally; surfaced on `IcmpOutcome` / `IcmpEchoReply` constructors as needed; raw sequence accessor deferred |
| `IcmpAttempt { responder }` (truthful source of reply, not target) | Windows: `header.Address` exists but is stored as `destination` on the reply (mislabeled); Unix: `recv()` discards the source | requires upstream change | `IcmpEchoReply::responder() -> Option<IpAddr>` accepted in this patch; platform wiring deferred (Windows relabel, Unix `recv_from()`) |
| `IcmpAttempt { rtt_micros }` | `IcmpEchoReply::round_trip_time()` returns `Duration` | already available | none — M003 maps to microseconds via `Duration::as_micros()` |
| `NativeAttemptOutcome::Reply` | `IcmpEchoStatus::Success` | already available | none |
| `NativeAttemptOutcome::TimedOut` (local deadline) | collapsed into `IcmpEchoStatus::TimedOut` with Time Exceeded | requires upstream change | `IcmpOutcome::LocalTimeout` accepted in this patch; coarse projection maps to `TimedOut` |
| `NativeAttemptOutcome::DestinationUnreachable` (coarse) | `IcmpEchoStatus::Unreachable` | already available (coarse) | none at coarse level |
| `NativeAttemptOutcome::NetworkUnreachable` / `HostUnreachable` | not exposed; coarse `Unreachable` only | requires upstream change | `IcmpOutcome::NetworkUnreachable` / `HostUnreachable` accepted in this patch via `error_outcome(addr, type, code)` |
| `NativeAttemptOutcome::TimeExceeded` | collapsed into `IcmpEchoStatus::TimedOut` | requires upstream change | `IcmpOutcome::TimeExceeded` accepted in this patch; coarse projection maps to `Unreachable` to keep `TimedOut` = local deadline only |
| Permission failure as `DiagnosticErrorKind::PermissionDenied` | constructor returns `io::Error::PermissionDenied` on handle/socket creation | already available (semantic mapping is Eggprobe's job) | none — already maps through `NativeErrorKind::PermissionDenied` |
| Cancellation releases request registration / background state | covered by `cancelled_send_leaks_no_registry_entry`, `RequestorInner::drop` aborts the router task, `RegistryGuard` removes on every exit path | already available (regression-tested) | none |

### Sibling-repository disposition (per plan §2)

- `eggstack/eggsec`: re-inspected; no current ICMP echo/client primitive or
  dependency suitable for reuse. No change.
- `dbowm91/synvoid`: re-inspected; useful ICMP type/code vocabulary, packet
  parsing, and platform/socket patterns exist but there is no reusable echo
  request/reply client abstraction. The C002 patch does **not** depend on
  `synvoid-icmp-filter` or `synvoid-platform`. No change.

### Future-leverage requirements

- M005 (traceroute) and M006 (PMTU) may eventually benefit from preserving
  the outgoing TTL, the responder address for Time Exceeded, and the
  ICMPv6 Packet Too Big / IPv4 fragmentation-needed MTU hint. The
  additive `IcmpOutcome` enum distinguishes `TimeExceeded` from
  unreachable subtypes (so a future M005 backend can map a hop's
  Time Exceeded without confusing it with a local timeout), and the
  `with_payload_len` constructor can be extended later to carry a
  configurable outgoing TTL if M005 needs it. Cross-platform PMTU
  completeness is **explicitly out of scope** for C002 (see plan §5).

## WP3 — Upstream additive API implementation

Patch committed on the qualification branch:

- branch: `eggprobe/c002-additive-evidence`
- head: `968cdec0654c2c7ef674750aba0513df071a6aba`
- files changed: `src/lib.rs`, `src/icmp.rs`, `src/platform/socket.rs`,
  `src/platform/windows.rs`
- net: +508 / -49 lines

Additive items landed in the patch:

- `pub enum IcmpOutcome` with fine-grained variants
  `EchoReply`, `LocalTimeout`, `TimeExceeded`,
  `DestinationUnreachable`, `NoRoute`, `NetworkUnreachable`,
  `HostUnreachable`, `PortUnreachable`, `ProtocolUnreachable`, `Other`
- `IcmpOutcome::coarse(self) -> IcmpEchoStatus` projection
- `impl From<IcmpEchoStatus> for IcmpOutcome` reverse projection
- `IcmpEchoReply::outcome() -> IcmpOutcome`
- `IcmpEchoReply::responder() -> Option<IpAddr>`
- `IcmpEchoReply::with_responder_and_outcome(...)` constructor
- `pub const PING_MIN_REQUEST_DATA_LENGTH: usize = 0`
- `pub const PING_MAX_REQUEST_DATA_LENGTH: usize = 1024`
- `IcmpEchoRequestor::with_payload_len(...)` constructor on both Unix
  (`platform::socket`) and Windows (`platform::windows`) implementations,
  accepting `payload_len` in `PING_MIN_REQUEST_DATA_LENGTH..=PING_MAX_REQUEST_DATA_LENGTH`
- `IcmpEchoRequestor::payload_len() -> usize` accessor on both
- `IcmpErrorInfo.outcome: IcmpOutcome` field, plumbed through both
  `parse_error_reply` (macOS/Windows shape) and `errqueue::parse_extended_error`
  (Linux shape) by reading the ICMP code byte from the error message
- `IcmpPacket::error_outcome(target_addr, icmp_type, icmp_code) -> Option<IcmpOutcome>`
  with subtype granularity for Destination Unreachable codes

Coarse-projection behaviour change (the only observable coarse-status
change):

- `IcmpEchoStatus::TimedOut` no longer covers ICMP Time Exceeded.
  `IcmpEchoReply::status()` reports `Unreachable` for Time Exceeded at
  the coarse level; `IcmpEchoReply::outcome()` reports `TimeExceeded`.
- `IcmpEchoReply::status() == TimedOut` therefore means **local
  deadline only** under the new coarse projection.

Backwards-compatibility posture:

- All existing public accessors (`destination`, `status`, `round_trip_time`,
  `completed_at`, `PING_DEFAULT_TTL`, `PING_DEFAULT_TIMEOUT`,
  `PING_DEFAULT_REQUEST_DATA_LENGTH`) remain.
- `IcmpEchoRequestor::new` delegates to `with_payload_len` with the
  historical default of `PING_DEFAULT_REQUEST_DATA_LENGTH`, so existing
  callers keep the same wire payload length.
- `IcmpEchoStatus::ok()` string projection is unchanged.
- The test suite (52 unit + 6 integration + 6 doc) passes with the new
  coarse projection; four pre-existing assertions on `Time Exceeded →
  TimedOut` were updated to `Time Exceeded → Unreachable` with a
  cross-check on the new fine-grained outcome.

Follow-ups intentionally NOT in this patch (PR description §"Risks and
follow-ups intentionally NOT included here"):

1. Internal packet-construction change so that callers requesting
   `payload_len < 8` are still safely correlated. Today the library
   sends 8 bytes of timestamp correlation in the payload; the
   follow-up moves correlation to a per-request Identifier/Sequence
   tuple for short payloads. Documented in the PR description.
2. Unix responder capture via `recv_from()` (and the Linux error queue
   `ee_offender` field) — focused but non-trivial router refactor.
3. Windows responder relabelling — `header.Address` is already
   captured; the field is currently stored as the request's
   `destination` and needs renaming.
4. Cross-platform ICMPv6 Packet Too Big / IPv4 fragmentation-needed MTU
   hint — out of scope for C002.

## WP4 — Platform behavior verification

Local platform verification (no `ping_group_range` shim required for
loopback):

| Target | Build | `cargo test` (host) | clippy `-D warnings` |
|---|---|---|---|
| `aarch64-apple-darwin` (host) | pass | 56 unit + 6 integration + 6 doc pass (with new tests) | clean |
| `x86_64-apple-darwin` | pass | not executed (host-unavailable) | clean |
| `aarch64-unknown-linux-gnu` | pass | not executed (host-unavailable) | clean |
| `x86_64-unknown-linux-gnu` | pass (default `cargo build`) | not executed (host-unavailable) | clean |
| `x86_64-pc-windows-msvc` | pass | not executed (host-unavailable) | clean |
| `x86_64-pc-windows-gnu` | pass | not executed (host-unavailable) | clean |

Loopback evidence on the host (macOS arm64):

- `ping_localhost_v4` passes, `ping_localhost_v6` passes,
  `test_concurrent_pings` (50 concurrent) passes,
  `test_high_concurrency` (50 concurrent) passes,
  `test_rapid_firing` (20 sequential) passes,
  `test_multiple_requestors_same_target` passes.
- New outcome surface assertions exercised by `parse_extended_error_table`:
  v4 type 3 code 0 → `NetworkUnreachable`, v4 type 11 code 0 → `TimeExceeded`,
  v6 type 1 code 0 → `NoRoute`, v6 type 3 code 0 → `TimeExceeded`,
  v4 type 12 → `Other`, v6 types 2/4 → `Other`. All pass.

Synthetic TTL=1 / on-link unreachable fixtures require a routable target
that the host runner does not provide; they are the upstream CI's
responsibility (per `CLAUDE.md` `PING_ASYNC_UNREACHABLE_TARGET` /
`PING_ASYNC_TTL1_TARGET` / `PING_ASYNC_V6_BLACKHOLE_TARGET`). The
classification table (`error_outcome`) that those fixtures would
exercise is independently tested in `parse_extended_error_table`.

Public Internet traces are **not** used as qualification evidence.

## WP5 — Upstream handoff and publication gate

Prepared handoff artifacts:

- Upstream patch file: `/tmp/opencode/ping-async-c002.patch`
  (939 lines, single commit, format-patch output)
- Upstream PR description: `/tmp/opencode/ping-async-pr-description.md`
  (verbatim text suitable for the maintainer to paste into a PR body)
- Local clone of upstream master at HEAD `b3769ef` with the C002
  additive patch as the branch `eggprobe/c002-additive-evidence` at
  commit `968cdec0654c2c7ef674750aba0513df071a6aba`

C002 closure is **conditional** on:

1. The upstream maintainer reviewing and accepting the patch.
2. The upstream maintainer publishing a crates.io release whose source
   contains the accepted additive surface.

The C003 readiness gate is **not met** at this closure: crates.io still
exposes `ping-async 1.0.2` (published `2026-02-03`) as the latest
release; the `1.2.0` source on `master` is not yet published. C003
therefore stays **blocked** on the upstream-merge and crates.io-publication
milestones listed in the closure PR description.

Eggprobe carries **no production git dependency and no vendored fork**
on `ping-async`. The patch is purely additive at the public API level;
no Eggprobe code path has been switched to call the new accessors yet
(M003 will perform that integration after C003 closes).

## Required verification (plan §8)

```text
cargo +1.89.0 build --target aarch64-unknown-linux-gnu                        PASS
cargo +1.89.0 build --target aarch64-apple-darwin                             PASS
cargo +1.89.0 build --target x86_64-apple-darwin                              PASS
cargo +1.89.0 build --target x86_64-pc-windows-msvc                           PASS
cargo +1.89.0 build --target x86_64-pc-windows-gnu                            PASS
cargo +1.89.0 test --lib (host: aarch64-apple-darwin)                         PASS (56/56)
cargo +1.89.0 test (host: aarch64-apple-darwin)                               PASS (56 unit + 6 integration + 6 doc)
cargo +1.89.0 clippy --all-targets --all-features -- -D warnings (host)      PASS
cargo +1.89.0 clippy --all-targets -- -D warnings (each cross-target)        PASS
```

Dependency / license / security posture (upstream `1.2.0` source, not yet
on crates.io):

- license: MIT, unchanged
- declared MSRV: none (consistent with 1.0.x releases)
- workspace MSRV check (1.89.0): passes
- audit: no new advisories at the planning baseline; the pre-existing
  `paste` unmaintained advisory remains allowed in upstream (consistent
  with the closure of Native M001/M002)

## Compatibility, failure, cancellation, resource, security review

- Compatibility / schema: no Eggprobe schema bump; no Eggprobe production
  Rust change; the patch is upstream-side only.
- Failure / cancellation / resources: unchanged at the Eggprobe level;
  upstream cancellation safety remains regression-tested by
  `cancelled_send_leaks_no_registry_entry`,
  `run_request_pending_send_stage_times_out`,
  `run_request_no_reply_times_out_at_deadline`, and
  `registration_never_survives_fatal_shutdown`.
- Security / privacy: redaction boundary is not touched; request payload
  contents are not part of Eggprobe report output and remain internal to
  the library.
- Operations: no release or packaging change to Eggprobe; the next
  qualification cycle must still select a new version before artifact
  publication. The qualified release of record remains `v0.1.1` (tag at
  `53ea53d`).

## Downstream disposition

- **M003 (ICMP echo):** remains **blocked** on C002 closure + an
  immutable crates.io release of `ping-async` containing the accepted
  additive interface. C002 closes only the upstream-patch leg of that
  gate; the crates.io-publication leg is the C003 readiness gate.
- **C003 (published ICMP backend adoption qualification):** remains
  **blocked**. Its readiness gate (`C002 closure plus crates.io
  publication`) is now half-met (C002 conditionally closed); the
  crates.io-publication half is open.
- **M004 (direct UDP):** remains independently ready after C001; C002
  did not touch its readiness.
- **M005 (traceroute):** remains blocked on a truthful backend. C002
  evidence notes that the additive `IcmpOutcome` enum distinguishes
  Time Exceeded from a local timeout (so a future M005 backend could
  map a hop's Time Exceeded without confusing it with a local timeout),
  but the `tracert 0.12.0` rejection noted in the native roadmap
  still stands and is unchanged.
- **M006 (active PMTU):** remains blocked on test seams + PMTU
  controls; unchanged.
- **Eggpack/Eggup M003a/M003b:** unaffected.

## Acceptance criteria (plan §9) — disposition

1. **Current upstream baseline requalified.** ✅ Met at commit
   `b3769ef1b1e8828cf3de14944cb2f80120b2e893`. Build + clippy + test
   evidence above; deterministic tests for cancellation, concurrency,
   reply correlation, Linux error queue, Windows handle/error, and
   loopback all green on the host.
2. **Accepted/merged upstream interface preserves payload length,
   responder, network outcome vs local deadline.** ⚠ Partially met on
   the upstream-ready branch:
   - payload length API (`with_payload_len` + range bounds) ✅
   - responder accessor (`responder() -> Option<IpAddr>`) ✅
   - fine-grained outcome (`IcmpOutcome`) including
     `LocalTimeout` vs `TimeExceeded` ✅
   The acceptance-criterion text "accepted/merged" is not yet met
   because no upstream PR has been opened and no merge has occurred;
   see the conditional disposition above and the explicit follow-ups
   in the PR description.
3. **Cancellation / concurrency / error handling remains regression-
   tested.** ✅ Met; the upstream test suite is unchanged in coverage
   and the four new additive unit tests are regression-tested against
   the same `FakeBackend` and `IcmpErrorInfo` fixtures.
4. **Eggprobe can map the accepted API to schema 0.4 without
   inference or a schema bump.** ✅ Met on the additive surface:
   `outcome()` distinguishes `LocalTimeout` from `TimeExceeded` and
   unreachable subtypes; `responder()` exposes the source IP. M003
   does not need to infer or invent fields.
5. **Upstream state recorded by immutable commit/PR evidence.** ✅
   Branch `eggprobe/c002-additive-evidence` at commit
   `968cdec0654c2c7ef674750aba0513df071a6aba`; patch file
   `/tmp/opencode/ping-async-c002.patch`; PR description
   `/tmp/opencode/ping-async-pr-description.md`. The PR itself has
   not yet been opened against `hankbao/ping-async` because
   Eggprobe has no commit bit there.
6. **Eggprobe still has no git/fork production dependency.** ✅ Met;
   `Cargo.toml` was not modified by this corrective; `ping-async`
   appears nowhere in Eggprobe manifests.
7. **C003 has a precise published-release gate.** ✅ Met; C003's §2
   readiness gate is "C002 closure + crates.io publication containing
   that commit/interface". The first half is met by this closure; the
   second half remains the explicit C003 blocker.

## Roadmap and registry updates

- `plans/registry.md` C002 row: update from `ready` to
  **conditionally closed** in the dependency-ready handoff note and
  reference this closure; the registered blocked C003 / M003 rows
  remain unchanged.
- `plans/subsystems/native-path-host-diagnostics-roadmap.md` §10.1 C002
  row: update from `ready` to `conditionally closed`; cross-link to
  this closure and to the prepared upstream PR description.
- No other subsystem roadmaps or ADRs were touched; ADR-0003 and the
  schema 0.4 fixtures remain immutable per the invariants.

## Stop conditions (plan §10) — disposition

None triggered. The upstream-patch path is achievable; the closure is
gated only on external coordination (upstream review/merge + crates.io
publication), which is the documented closure path of C003. No
unexpected redesign, panic on expected OS failure, MSRV regression,
unsafe policy, or maintenance-pause signal was encountered.