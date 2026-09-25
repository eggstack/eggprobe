# Native Corrective C005 Closure — Traceroute Privilege and Hosted-Platform Qualification

Status: closed

Source plan: `plans/implementation/native-path-host-diagnostics-corrective/005-traceroute-privilege-and-hosted-qualification.md`

Source roadmap: `plans/subsystems/native-path-host-diagnostics-roadmap.md`

Planning baseline: `1401548` (C005 plan registration; failure baseline `0ce9597`).

Implementation: `1401548..53226a3` across three commits:

- `067de09` — platform-aware privilege selection, permission normalization,
  deterministic selection tests, host-aware smokes, diagnostics/docs;
- `05c332f` — Windows fail-safe refusal before the backend abort + host-aware
  zero-deadline test;
- `53226a3` — Windows `WSAECONNRESET` unreachable mapping for deterministic
  closed-port UDP evidence.

Historical M005 closure retained unchanged at
`plans/closure/native-path-host-diagnostics/005-status.md`; this record is the
current corrective authority.

## Pre-fix failure classification

Two hosted runs on the pre-fix tree failed the same smoke on Ubuntu and
Windows while macOS, MSRV, and audit passed:

- run `36138451900` on `0ce9597`: Ubuntu job `108082118033` failed, Windows
  job `108082118099` failed; macOS `108082118186`, MSRV `108082117794`,
  audit `108082117983` passed;
- run `36150112254` on planning-only `1401548` repeated the identical
  Ubuntu/Windows-only signature, confirming the defect was in production
  code, not planning.

The pre-fix CLI smoke asserted only `output.status.success()`, hiding the
diagnostic. Source inspection root-caused defect 1 before any change:
`crates/eggprobe-native/src/lib.rs` forced
`trippy_core::PrivilegeMode::Unprivileged` on every platform, while Trippy
0.13 documents unprivileged mode as macOS-only (Linux always needs privilege
for the trace receive path; Windows always needs privilege). Target policy
accepts `127.0.0.1`, schema 0.4 was untouched, and CLI parsing was correct,
leaving backend execution mode as the only per-OS variable — matching the
observed Ubuntu/Windows-only failure exactly.

Hosted forensics during implementation exposed defect 2 (below): even with
the correct privileged mode, `trippy-core 0.13.0` cannot execute on elevated
Windows.

## Upstream privilege references and versions

- Trippy 0.13 privilege guide:
  `https://github.com/fujiapple852/trippy/blob/master/docs/src/content/docs/0.13.0/guides/privileges.md`
- Trippy issue #101: `https://github.com/fujiapple852/trippy/issues/101`
- `trippy-privilege 0.13.0` API:
  `https://docs.rs/trippy-privilege/0.13.0/trippy_privilege/`
- Windows stability context (prior fixed family, ours is a remaining
  instance): trippy 0.13.0 release notes cite #1527 (memory corruption),
  #1443, #1547, #1558.
- Pinned lines: `trippy-core =0.13.0`, direct `trippy-privilege =0.13.0`
  (same release line; `cargo tree` shows a single `trippy-privilege 0.13.0`
  shared by both paths — no duplication). Transitive privilege support:
  `caps 0.5.6`, `nix 0.29.0`. `trippy-core` declares rust-version 1.78;
  workspace MSRV remains 1.89.

## Exact privilege-mode policy

`select_privilege_mode(unprivileged_supported, privileged_supported, facts)`
(pure, injected facts) + `current_privilege_mode()` (runtime) +
`trace_capability()` (public oracle for host-aware tests):

- macOS: `PrivilegeMode::Unprivileged` (backend-documented; no discovery).
- Linux/other: read-only `Privilege::discover()`; already-effective
  privilege (`CAP_NET_RAW` effective, elevated token) selects
  `PrivilegeMode::Privileged`, otherwise `NativeErrorKind::PermissionDenied`.
  Discovery failure also denies; a mere deficit never becomes `Unsupported`.
- Windows: `NativeErrorKind::Unsupported` before any backend contact when
  privilege is present; non-elevated Windows still denies with
  `PermissionDenied` via discovery. Rationale below.
- Trippy privilege types never cross the crate boundary; the engine maps
  `PermissionDenied` to `PermissionDenied/HopProbe` with fixed
  `"UDP trace requires additional local privilege"` and `Unsupported` to
  `Unsupported/HopProbe` with fixed `"UDP trace is not supported on this
  platform"`. No dependency/OS text reaches reports.

## Windows abort forensics and fail-safe refusal

With correct privileged-mode selection, the elevated Windows hosted runner
(`TRACE_CAPABILITY=Executable` under the pre-refusal code, i.e. an elevated
token) entered the backend and the child process fail-fast aborted
(`0xC0000409`, empty stdout):

```text
thread 'tokio-rt-worker' panicked at library/alloc/src/raw_vec/mod.rs:
unsafe precondition(s) violated: Layout::from_size_align_unchecked ...
```

Full backtrace (captured via a throwaway debug branch, since deleted):
`Strategy::run` → drop of `TracerState` → drop of `ProbeComplete` →
`Extensions` → `Extension::Unknown` → `UnknownExtension` → `Vec<u8>`
deallocation with a corrupted header. Extension construction in
`trippy-core`/`trippy-packet` is safe code (`to_owned()`; `trippy-packet`
forbids `unsafe`), so the corruption is heap damage from unsafe code in the
dependency's Windows receive path, detected at drop — the same family as
upstream #1527, a remaining instance in 0.13.0. An abort cannot be normalized
into any typed error (no report is produced; the exit status is
uninterpretable; the whole test binary dies), so Eggprobe refuses the
unsupported backend before contact. This is the plan's designed fourth
helper outcome (`Unsupported` — backend genuinely cannot support the family
independent of privilege), not a weakening: per-OS smokes assert the exact
refusal shape. Privileged Windows live execution is tracked by blocked
follow-up C006, gated on a backend release that fixes the corruption.

## Acquisition/drop security review

- Discovery only: `Privilege::discover()` is read-only. Eggprobe never calls
  `acquire_privileges`, never raises/clears capability sets, never invokes
  `sudo`, never mutates file capabilities, never prompts for elevation —
  exercising the plan's explicit fallback over acquisition because Linux
  capabilities are thread-scoped and the Tokio blocking pool reuses threads,
  so acquire-then-drop would need manual drop discipline on every early
  return (trippy-core itself drops only on its success path).
- Consequently `drop_privileges(false)` is set deliberately: the tracer must
  not clear an effective set it did not raise.
- Concurrent traces cannot race on privilege state (nothing is mutated);
  per-trace source ports still isolate concurrent backend runs.
- Target policy (engine `trace()`) still precedes any backend contact;
  max-hops/attempts/deadline bounds unchanged; `RouteSpec` redaction and the
  `expression` leak assertion unchanged and green.

## Deterministic mode-selection tests

- macOS-style unprivileged selection (facts ignored);
- Linux privilege available → privileged; unavailable → denied;
- Windows non-elevated → denied; Windows elevated → unsupported refusal;
- discovery failure → bounded denial, never `Unsupported`;
- capability oracle agrees with the runtime decision;
- backend `Other` error → `Io` (`PrivilegeError` has no public constructor;
  the arm stays review-covered, as in M005).

## Live-smoke disposition (qualifying run)

Qualifying hosted run `36156303047` on `53226a3` is fully green on one head:

- Ubuntu quality `108141559598` — success (typed permission-denied path;
  ordinary runners lack `CAP_NET_RAW`);
- macOS quality `108141559449` — success (unprivileged loopback success);
- Windows quality `108141559394` — success (typed unsupported refusal; no
  backend contact, no abort);
- Rust 1.89 MSRV `108141559512` — success;
- dependency audit `108141559181` — success.

No smoke was skipped: every live assertion executes one arm dictated by the
same capability decision as production. Locally, all three dispositions were
additionally proven end-to-end by forced-selection simulation (denied and
refused stacks assert exact JSON kind/stage/message, exit code, and clean
stderr); the simulation code was reverted before commit.

## Incidental matrix enabler (M004 hardening)

The green Windows job additionally required mapping `ConnectionReset` to
`UdpExchangeOutcome::Unreachable` in `udp_exchange`: Windows surfaces ICMP
errors on a connected UDP socket as `WSAECONNRESET`, the same unreachable
feedback Linux surfaces as `ConnectionRefused`. Without this, closed-port
receive probes fail deterministically where the ICMP is delivered and pass
where the firewall eats it. The mapping preserves the M004 contract (OS
unreachable signal → completed `Unreachable` observation, never a local
failure) and changes nothing on Linux/macOS. Recorded here because C005
needed the green matrix; M004's closure record is retained as history.

## Verification (qualifying head)

```text
cargo fmt --all -- --check                         PASS
cargo check --workspace --all-targets --all-features --locked PASS
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings PASS
cargo test --workspace --all-features --locked     PASS (98 tests)
cargo +1.89.0 check --workspace --all-targets --locked PASS
cargo tree --locked                                PASS (single trippy-privilege 0.13.0)
cargo audit                                        PASS (one pre-existing allowed unmaintained `paste` advisory)
schema regeneration                                PASS (no diff; `expression` assertion green)
```

Intermediate hosted evidence (same-head sequence): `36151642395` proved the
selection fix (Ubuntu/macOS green) and exposed the zero-deadline race
(worker preflight denial beats a zero timeout) plus the Windows abort;
debug-branch runs `36152587509`/`36153274781` captured capability and the
full abort backtrace (branches deleted after use).

## Compatibility and remaining limitations

- No schema change (0.4 artifacts regenerate byte-identical); additive
  constants only (`TRACE_PERMISSION_MESSAGE`, `TRACE_UNSUPPORTED_MESSAGE`,
  `TraceCapability` re-exported through `eggprobe-core` for host-aware
  tests; CLI binary gains no dependency).
- macOS live unprivileged UDP tracing: qualified.
- Linux with effective `CAP_NET_RAW`: privileged path implemented and
  unit-selected, but unexercised in hosted CI (no privileged runner; optional
  per plan §6.4 — a future privileged smoke may still prove it).
- Linux without privilege: typed denial, qualified on hosted Ubuntu.
- Windows: fail-safe refusal qualified on hosted Windows; live execution
  requires an upstream backend fix (C006, blocked).
- `TimeExceeded`/`Unreachable`-code extraction arms remain review-pinned
  (`IcmpPacketCode` has no public export) — unchanged from M005; summary
  termination coverage for all four states is literal.
- Exit codes unchanged (`Failed`/`Unsupported` reports → exit 1).

## Downstream disposition

- M005: Linux/macOS live dispositions qualified by this closure; Windows
  live execution tracked by blocked C006. Historical M005 closure retained.
- M006: still blocked on its independent PMTU DF/PTB-control gap; untouched
  by C005.
- M003/C003: still blocked on upstream ICMP acceptance/merge/publication;
  untouched by C005.
- Ready-to-file upstream report (trippy `fujiapple852/trippy` issues):
  privileged UDP loopback trace on elevated Windows with `trippy-core`
  0.13.0 fail-fast aborts while dropping a parsed `UnknownExtension`
  (`Strategy::run`, `TracerState` teardown); reporter can supply the full
  backtrace and the `main`-branch refusal as the embedding workaround.
