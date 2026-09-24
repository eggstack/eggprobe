# Release Corrective C004 Closure

Status: closed

Source implementation plan:

- `plans/implementation/release-operational-qualification-corrective/004-windows-http-fixture-and-hosted-ci-requalification.md`

Source roadmap/addendum:

- `plans/subsystems/release-operational-qualification-roadmap.md`
- `plans/subsystems/release-operational-qualification-corrective-addendum.md`

Planning baseline: `23da04ec32d3981aa0cb899d4d5b1d120e66f75e`

Implementation commits (all test-only, no production code changed):

- `8ed17de` — portable 503 HTTP fixture (bounded request drain, flush, orderly shutdown) + strengthened probe/assertion separation assertions
- `406670f` — fresh-port retry for the refused-hop probe (later superseded in budget terms by `04c84d3`)
- `1fca5ea` — temporary Windows dial diagnostic (reverted before closure; no diagnostic output remains)
- `04c84d3` — closure head: diagnostic removed, 6 s per-attempt dial budget for slow Windows RST

## Executive finding

C004 is complete. Hosted run `36015806725` on closure head `04c84d3`
is fully green across Ubuntu quality, macOS quality, Windows quality,
Rust 1.89 MSRV, and dependency audit. Two Windows-only test-fixture
defects were root-caused and corrected without changing any production,
schema, CLI, or release behavior:

1. The `http_status_is_observed_even_when_assertion_fails` failure was a
   non-portable write-and-drop HTTP fixture: closing a Windows socket
   with unread request bytes sends RST and discards the queued 503
   response, while Unix delivers it with FIN. The fixture now drains the
   request (bounded), flushes, and shuts down orderly.
2. Once the engine binary went green, the previously masked
   `routed_hop_connect_error_keeps_typed_stage_and_never_dials_target_directly`
   failure was exposed. Hosted diagnostics proved it is environmental,
   not a race: the Windows runner delays RST for closed loopback ports
   by ~2 s (raw control dial shows no refusal within 2 s; the same dial
   refuses in 0 ms on macOS/Linux), so any sub-second dial budget can
   never observe the genuine refusal there. The test now allows a 6 s
   per-attempt dial budget and still accepts only a genuine
   `ConnectionRefused`/`HopConnect` outcome.

Probe-versus-assertion semantics are preserved and now more strongly
asserted: a valid HTTP 503 remains successful probe evidence while the
2xx status-range assertion fails independently. C003 remains historical
evidence for the CRLF/audit defects it fixed; C004 owns these two
later findings.

## Root-cause analysis

### Finding 1 — 503 fixture RST on Windows (proven fixture defect)

The failing test created a local `TcpListener`, accepted one socket,
wrote `HTTP/1.1 503 Service Unavailable\r\nContent-Length: 0\r\n\r\n`,
and dropped the stream without reading the client's GET request,
flushing, or shutting down.

Differential proof that the fixture (not Eggprobe/Eggfetch production
code) is the defect:

- The identical Eggfetch code path serves every other hosted Windows
  HTTP test green, including the routed `serve_one_http_response`
  fixture, which already drains request headers before responding and
  passes on Windows.
- The only Windows-failing test was the only HTTP fixture that never
  reads the request — matching the documented Winsock behavior that
  `closesocket` with unread receive data sends RST, discarding queued
  response bytes, while Unix delivers them with FIN.
- Response bytes were intentionally kept byte-identical; only the
  lifecycle changed, and the test went green on Windows.

### Finding 2 — refused-hop Timeout on Windows (proven environmental)

With finding 1 fixed, the Windows job progressed to the
`routed_qualification` binary (cargo stops later targets after an
earlier binary fails, which is why this was masked in every earlier
Windows run) and failed with `Timeout`/`Deadline` instead of
`ConnectionRefused`/`HopConnect`.

Two hypotheses were tested and one was disproven locally:

- Port theft by parallel fixtures was the first hypothesis. A held
  bound-but-unlistened socket was tried as a reservation mechanism and
  deterministically produced `Timeout` even in isolation — disproving
  both the reservation approach (a bound socket blackholes SYNs instead
  of refusing on macOS/Windows) and, by itself, explaining nothing
  about the runner.
- A temporary control dial (`TcpStream::connect` + 2 s budget, run
  `36014632837`) gave ground truth on the hosted runner: 8 of 10 fresh
  dropped loopback ports showed no RST within 2 s, and 2 showed refusal
  arriving at ~2 s — while the same control refuses in 0 ms locally.
  Ten consecutive identical hangs cannot be a port-allocation race; it
  is the runner's delayed-RST behavior for closed loopback ports.

The engine's `Timeout`/`Deadline` report was truthful in every case:
the dial genuinely did not complete inside the budget. No production
mapping is wrong.

## Exact correction (test-only)

`crates/eggprobe-core/tests/engine.rs`
(`http_status_is_observed_even_when_assertion_fails`):

- drain request headers until `\r\n\r\n` with a 16 KiB / 5 s bound;
- write the unchanged 503 response bytes;
- `flush` + orderly `shutdown`;
- strengthened regression: probe status `Ok`, HTTP evidence status
  exactly `503`, report `Ok`, and the 2xx finding `Failed` are all
  asserted separately.

`crates/eggprobe-core/tests/routed_qualification.rs`
(`routed_hop_connect_error_...`):

- retry with a fresh dropped port (up to 4 attempts) so rare
  parallel-fixture port theft cannot flake the suite;
- 6 s per-attempt plan deadline so the genuine refusal is observable
  even where the runner delays RST by ~2 s;
- the accepted outcome is unchanged and strict: `ConnectionRefused` /
  `HopConnect` / hop 0 / no protocol, plus the existing
  never-dials-target-directly check.

A bounded theft simulation (held socket forces the stolen-port
signature) was run locally to prove the retry bound terminates with a
clear message; the simulation was reverted and no diagnostic output
remains in the tree.

No production, schema, CLI, MSRV, audit-toolchain, or release-workflow
file was changed by C004.

## Before/after evidence

| Hosted run | Head | Windows outcome |
|---|---|---|
| `36008924756` (plan baseline) | `23da04e` | contract tests green, `http_status_is_observed...` FAILED (`Failed` vs `Ok`) |
| `36010459854` | `eeebeb2` | same HTTP test FAILED; Ubuntu/macOS/MSRV/audit green |
| `36011679685` | `8ed17de` | HTTP test **ok**; `routed_hop_connect_error...` FAILED (`Timeout` vs `ConnectionRefused`) |
| `36013075573` | `406670f` | 10 fresh-port attempts all `Timeout` — systematic, not a race |
| `36014632837` | `1fca5ea` (temp diagnostic) | control dial: no RST within 2 s on 8/10 ports, refusal at ~2 s on 2/10; 0 ms refusal locally |
| `36015806725` (qualifying) | `04c84d3` | **all green** (see below) |

## Hosted CI evidence (qualifying run)

Run: <https://github.com/eggstack/eggprobe/actions/runs/36015806725>
(head `04c84d3e4721707839fe286b6c94ecfdd568eac0`)

- quality (ubuntu-latest): job `107687917563` — green
- quality (macos-latest): job `107687917768` — green
- quality (windows-latest): job `107687917787` — green
- msrv (Rust 1.89.0): job `107687917366` — green
- audit (stable / cargo-audit 0.22.2): job `107687917580` — green (no advisories)

Windows job `107687917787` log confirms:

- `http_status_is_observed_even_when_assertion_fails ... ok`
- `routed_hop_connect_error_keeps_typed_stage_and_never_dials_target_directly ... ok`
  (routed binary finished in ~2.1 s, consistent with the ~2 s runner RST delay)
- C003 contract tests
  `plan_round_trips_against_deterministic_fixture` and
  `report_matches_deterministic_json_and_round_trips ... ok`
- every suite green: engine 7/7, routed 13/13, contract 12/12, CLI/TLS/schema remainder 0-failed

## HTTP 503 evidence + assertion separation proof

The strengthened regression now asserts all of the following on every
platform (Windows log: ok):

- `report.status == Ok` for a valid 503 response;
- `report.probes[0].status == Ok` (transport/probe success);
- HTTP evidence status is exactly `Some(503)`;
- `evaluate_assertions` yields a `Failed` finding for the 200–299 range.

A real reset, malformed response, timeout, or body failure still maps to
a probe failure through the unchanged `normalize_http_failure` path;
only the fixture lifecycle changed.

## Windows deterministic contract regression proof

Covered by Windows job `107687917787` above: both C003 CRLF tests pass
unchanged, confirming the `.gitattributes` LF policy remains intact and
C004 did not touch canonical fixtures or schemas.

## MSRV and audit outcomes

- MSRV job `107687917366` green on the closure head; product MSRV
  remains 1.89.0; local `cargo +1.89.0 check --workspace --all-targets
  --locked` passes.
- Audit job `107687917580` green with no advisories (215 crates);
  local `cargo +stable audit` exits 0. The isolated audit toolchain is
  untouched.

## Local verification (closure head, before push)

```text
cargo fmt --all -- --check                                             → ok
cargo check --workspace --all-targets --all-features --locked          → ok
cargo clippy --workspace --all-targets --all-features --locked -D warnings → ok
cargo test --workspace --all-features --locked                         → 0 failed (all suites)
cargo +1.89.0 check --workspace --all-targets --locked                 → ok
cargo tree --locked                                                    → ok
cargo +stable audit                                                    → exit 0, no advisories
cargo run -p eggprobe-core --example generate-schemas --locked         → no diff
```

Plus: routed binary 3× serial and 6× concurrent-process green locally;
theft simulation exhausts the retry bound with a clear message and was
reverted.

## Compatibility/security review

- Public compatibility effect: none. No schema bump (still `0.3`, no
  diff on regeneration), no CLI/exit-code change, no report-field
  change, no MSRV change, no release-version change.
- Security/redaction: no credential, URL, header, or hop-URI code
  touched; `cargo audit` clean; no new permissions.
- The temporary Windows diagnostic (`1fca5ea`) printed only port
  numbers, elapsed times, and already-bounded error messages to the CI
  log; it was fully reverted in `04c84d3` and no diagnostic output
  remains.
- Invariant 6 honored: no production behavior was changed to
  accommodate a fixture; invariant 3 honored: genuine transport
  failures still fail.

## Unresolved findings

None for C004. Pre-existing tracked items remain with their owners:

- `release.yml` `macos-13` label and `shellcheck SC2193` heuristic
  (actionlint) — recorded in the C003 closure, outside C004 scope.
- Hosted Windows runners delay RST for closed loopback ports by ~2 s.
  The refused-hop test now absorbs this with a 6 s per-attempt budget;
  if runner behavior ever exceeds that, the test panics with the last
  error attached rather than silently passing.

## Downstream disposition

- **C001 (conditionally closed): released from hold.** Its only
  remaining condition is creating/using an intentional valid release
  tag and obtaining the successful hosted packaging run with
  artifact/hash/native-smoke evidence. That is now the next action.
- **M002:** operational gate resolves via the C001 valid-tag hosted
  packaging evidence above.
- **M004: remains blocked**, now only on C001/M002 operational
  evidence (C004 is removed from its blocker list).
- **M003a / M003b:** unchanged, still blocked/deferred on their named
  cross-repository readiness gates; neither is a first-release
  prerequisite and C004 introduces no Eggpack/Eggup substitutes.
- No release tag was created by C004, per plan scope.

## Roadmap and registry disposition

- `plans/registry.md`: C004 marked closed at `04c84d3` with qualifying
  run `36015806725`; C001 released from hold to "next action:
  valid-tag hosted packaging evidence"; M004 blocked only on C001/M002
  operational evidence.
- Release subsystem roadmap and corrective addendum: C004 closed with
  a pointer to this record; dependency graph and milestone tables
  updated accordingly.
- C003 closure record left intact as historical evidence.
- Intended sequence from here: C001 valid-tag hosted packaging
  evidence → M002 operationally qualified → M004 final release
  qualification → first standalone archive release.
