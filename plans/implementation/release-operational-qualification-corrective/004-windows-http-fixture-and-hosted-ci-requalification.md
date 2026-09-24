# Release and Operational Qualification Corrective C004 — Windows HTTP Fixture and Hosted CI Requalification

Status: ready for handoff

Planning baseline: `23da04ec32d3981aa0cb899d4d5b1d120e66f75e`

Source roadmap:

- `plans/subsystems/release-operational-qualification-roadmap.md`
- `plans/subsystems/release-operational-qualification-corrective-addendum.md`
- `plans/002-long-term-roadmap.md#phase-7--release-packaging-and-operational-qualification`

Historical/related evidence:

- `plans/closure/release-operational-qualification-corrective/003-status.md`
- GitHub Actions run `36008924756`
- Windows quality job `107664241734`
- Ubuntu quality job `107664241772`
- macOS quality job `107664241926`
- MSRV job `107664241902`
- dependency-audit job `107664241849`

Applicable ADRs:

- `plans/adrs/ADR-0001-json-first-core-and-transport-ownership.md`
- `plans/adrs/ADR-0002-release-producer-consumer-ownership.md`

Primary class: infrastructure + invariant

## 1. Objective

Close the post-C003 Windows-only HTTP integration failure and obtain the fully
green hosted CI evidence required before Eggprobe creates/uses an intentional
first-release tag for C001 packaging qualification.

The corrective must determine whether
`http_status_is_observed_even_when_assertion_fails` is failing because the
test fixture uses non-portable TCP/HTTP connection teardown semantics or
because Eggprobe/Eggfetch behaves differently on Windows. Fix the proven root
cause without weakening the product invariant that an observed HTTP status
such as 503 is successful probe evidence while a separate status-range
assertion may fail.

C004 also corrects the planning disposition created when C003 was closed before
hosted Windows evidence existed. C003 remains historical evidence for the CRLF
and audit-tool defects it actually fixed; C004 owns the later finding.

## 2. Readiness and dependencies

Hard dependencies:

- Foundation diagnostic contract corrective work is closed.
- Transport/probe-engine corrective work is closed.
- CLI/automation corrective work is closed.
- Release corrective C002 is closed.
- C003 implementation is present and its original CRLF/audit defects are
  demonstrably corrected in hosted run `36008924756`.

Operational dependencies:

- none for implementation;
- no release tag is required or permitted as qualification evidence while C004
  remains open.

Downstream:

- C001's remaining valid-tag hosted packaging evidence is on hold until C004
  closes;
- M004 remains blocked until C004 closes and C001/M002 operational evidence is
  complete.

C004 is ready now.

## 3. Current evidence and failure boundary

Hosted run `36008924756` on
`23da04ec32d3981aa0cb899d4d5b1d120e66f75e` proves:

- Ubuntu quality job `107664241772` passes;
- macOS quality job `107664241926` passes;
- Rust 1.89 MSRV job `107664241902` passes;
- dependency audit job `107664241849` passes and reaches the real Eggprobe
  lockfile audit;
- Windows job `107664241734` passes formatting, strict Clippy, CLI tests,
  TLS fixture tests, and all 12 deterministic contract tests, including the
  two CRLF failures C003 targeted;
- the Windows job then fails
  `crates/eggprobe-core/tests/engine.rs::http_status_is_observed_even_when_assertion_fails`.

Observed assertion:

```text
left: Failed
right: Ok
```

The test creates a local `TcpListener`, accepts one socket, writes:

```text
HTTP/1.1 503 Service Unavailable\r\n
Content-Length: 0\r\n
\r\n
```

and then drops the stream. It does not consume the client's HTTP request,
explicitly flush the response, or perform an orderly shutdown.

That fixture lifecycle is a plausible explanation for a Windows-specific reset
or incomplete-response observation: closing a socket while receive-side request
data remains unread can produce platform-dependent teardown behavior. This is
a hypothesis, not an accepted root cause. The implementation must prove it.

## 4. Invariants

The corrective MUST preserve:

1. HTTP transport success and assertion outcome remain separate concepts.
2. Receiving a valid HTTP 503 response is represented as successful HTTP probe
   evidence with status 503, not as a transport failure merely because a
   2xx assertion fails.
3. A real connection reset, malformed response, timeout, body read failure, or
   protocol failure remains a probe failure.
4. Tests must use deterministic local fixtures and no public network.
5. The fixture must have bounded reads and cannot wait indefinitely for a
   malformed client request.
6. No production behavior may be changed solely to accommodate a defective
   test fixture.
7. If production behavior is actually wrong on Windows, the fix must preserve
   Linux/macOS semantics and receive dedicated regression coverage.
8. Rust 1.89 remains the product MSRV.
9. The C003 LF policy and isolated audit toolchain remain intact.
10. JSON/schema/CLI contracts remain unchanged unless the investigation proves
    a genuine contract defect; any such defect requires a separate plan.
11. Eggpack/Eggup remain deferred; no local replacement of their responsibilities
    is introduced.
12. No release tag or public release is created by C004.

## 5. In scope

### 5.1 Reproduce and classify the Windows failure

Use the existing failing test as the primary reproducer.

Instrument or temporarily enrich test diagnostics enough to distinguish:

- response headers not observed;
- connection reset / early EOF;
- response body read failure;
- Eggfetch protocol error;
- policy/dialer error;
- assertion-layer behavior after a valid response.

Do not leave secret-bearing or noisy ad hoc diagnostics in production output.

### 5.2 Make the local HTTP fixture protocol-correct and portable

If fixture lifecycle is confirmed as the root cause, replace the write-and-drop
server behavior with a bounded HTTP/1 fixture that:

1. accepts the connection;
2. reads request bytes until the end-of-headers delimiter or a small bounded
   limit/deadline;
3. validates only what the test needs (do not build a second HTTP server);
4. writes the complete 503 response;
5. flushes the response;
6. performs an orderly write shutdown when appropriate;
7. joins/cleans up the fixture task deterministically.

Prefer a small reusable test helper if other engine tests duplicate the same
unsafe write-and-drop pattern, but do not create a general server framework.

### 5.3 Preserve probe-versus-assertion semantics

The regression must explicitly prove all of the following:

- HTTP probe result status is `Ok`;
- HTTP evidence contains status `503`;
- report status reflects probe execution rather than converting the assertion
  into a transport/probe failure at this stage;
- `evaluate_assertions` produces a failed finding for the configured
  200-299 status range;
- CLI exit/finding behavior remains governed by the existing assertion layer.

If the existing intended report-status semantics differ from those statements,
stop and reconcile against the canonical specification before changing code.

### 5.4 Hosted current-head requalification

Obtain one hosted CI run on the implementation head in which all are green:

- Ubuntu quality;
- macOS quality;
- Windows quality;
- Rust 1.89 MSRV;
- dependency audit.

The Windows job must include successful execution of the previously failing
HTTP test and the deterministic contract tests fixed by C003.

### 5.5 Planning reconciliation

At closure:

- create
  `plans/closure/release-operational-qualification-corrective/004-status.md`;
- record the exact implementation commit, hosted run ID, and job IDs;
- keep C003's closure record intact as historical evidence;
- mark C004 closed only after hosted Windows evidence is green;
- release C001 from hold and make valid-tag hosted packaging evidence the next
  action;
- keep M004 blocked until C001/M002 operational evidence completes.

## 6. Out of scope

- creating/pushing a release tag;
- publishing a GitHub Release;
- collecting C001 archive hashes/artifacts;
- executing M004;
- changing HTTP semantics merely to satisfy a fixture;
- replacing Eggfetch HTTP ownership;
- changing Eggress routing behavior;
- changing schema version or public report fields;
- raising the product MSRV;
- adopting Eggpack or implementing Eggup self-update;
- Phase 8/9 feature work;
- broad cleanup of unrelated `release.yml` actionlint warnings unless one
  directly prevents C004 hosted qualification.

The pre-existing `macos-13` runner-lifecycle risk and SC2193 heuristic remain
tracked release-workflow findings. They may receive a separate bounded cleanup
plan if they become active blockers.

## 7. Ordered work packages

### WP1 — Pin the actual Windows error

1. Inspect run `36008924756` and reproduce locally where possible.
2. Add focused test-only diagnostics or an isolated reproducer if required.
3. Determine whether failure occurs while parsing headers, reading the zero-byte
   body, handling socket close/reset, or evaluating the report.
4. Record the exact Eggfetch/Eggprobe error if the probe fails.
5. Do not change production code until the failure boundary is known.

### WP2 — Correct fixture lifecycle if proven

1. Add bounded request-header consumption.
2. Write and flush the complete response.
3. Perform orderly shutdown/cleanup.
4. Keep the fixture minimal and deterministic.
5. Search nearby HTTP fixtures for the same write-and-drop lifecycle and fix
   only equivalent instances that are demonstrably exposed to the same issue.

### WP3 — Production correction only if required

If a protocol-correct fixture still fails on Windows:

1. identify the production-level Eggfetch/Eggprobe Windows behavior difference;
2. preserve valid HTTP status evidence across platforms;
3. add a focused regression that fails before the production fix;
4. keep source error classification truthful;
5. stop if the required correction belongs upstream in Eggfetch and register
   that upstream dependency instead of copying/forking HTTP logic.

### WP4 — Broad verification

Run:

```text
cargo fmt --all -- --check
cargo check --workspace --all-targets --all-features --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test --workspace --all-features --locked
cargo +1.89.0 check --workspace --all-targets --locked
cargo tree --locked
cargo +stable audit
```

Also re-run schema generation and require no unintended diff.

### WP5 — Hosted evidence and closure

1. Push implementation.
2. Require all five hosted lanes to complete successfully.
3. Capture run/job identifiers.
4. Create C004 closure evidence only after hosted results exist.
5. Update registry/roadmap/addendum to C004 closed.
6. Move the next action to C001 valid-tag hosted packaging evidence.

## 8. Failure, cancellation, restart, and contention semantics

A cancelled or infrastructure-failed Windows run is not qualification evidence.

A flaky rerun is not sufficient by itself if the root cause remains unknown.
The implementation must remove nondeterminism or explicitly demonstrate a
runner/environment failure outside the repository.

Fixture reads must have a bound/deadline so a malformed or changed client
request cannot hang CI.

If concurrent changes touch Eggfetch/Eggress versions, engine HTTP handling,
fixture helpers, `Cargo.lock`, CI workflows, or schema files, rebase/reinspect
and rerun qualification on the new head.

## 9. Compatibility and migration

Expected public compatibility effect: none.

No schema bump, CLI change, report-field change, archive-layout change, or
release-version change is expected.

A test-fixture-only fix has no user migration. If production code must change,
the closure record must explain why behavior was previously incorrect and prove
that the public semantics match the canonical HTTP observation/assertion model.

## 10. Required tests

At minimum:

- the existing
  `http_status_is_observed_even_when_assertion_fails` test passes on Windows,
  Linux, and macOS;
- assert HTTP evidence status is exactly 503;
- assert probe/result status is successful for the valid HTTP response;
- assert the 2xx status-range assertion fails independently;
- deterministic contract fixture tests remain green on Windows;
- strict-policy HTTP test remains green;
- direct/routed HTTP/TLS qualification tests remain green;
- no public-network test is added;
- audit and MSRV lanes remain green.

If a fixture helper is introduced, add focused tests only where they improve
failure localization; do not overbuild the test harness.

## 11. Documentation and planning updates

No user-facing docs should change for a fixture-only correction.

Required closure-time planning updates:

- C004 implementation plan status;
- C004 closure record;
- corrective addendum;
- release subsystem roadmap;
- `plans/registry.md`;
- M004 blocker language if necessary.

Do not rewrite C003 closure history. The current planning documents should state
that C003 fixed its original findings and C004 handled the subsequent hosted
Windows finding.

## 12. Acceptance criteria

C004 closes only when:

1. the Windows failure is root-caused, not merely rerun away;
2. the HTTP fixture uses portable, bounded lifecycle semantics if it was the
   defect;
3. a valid HTTP 503 remains successful probe evidence;
4. assertion failure remains separate from transport/probe failure;
5. Windows contract tests remain green;
6. Windows full workspace tests are green;
7. Ubuntu and macOS full quality jobs are green;
8. Rust 1.89 MSRV is green;
9. dependency audit is green and actually executes the Eggprobe lockfile audit;
10. one hosted run provides all of the above on the same implementation head;
11. no schema/CLI/release-archive behavior changes unintentionally;
12. C001 remains unexecuted until this evidence exists.

## 13. Stop conditions

Stop and register separate work if:

- the failure reproduces with a protocol-correct fixture and is proven to be an
  Eggfetch upstream defect requiring public API/behavior changes;
- correcting it requires changing canonical HTTP status/report semantics;
- a schema bump is required;
- product MSRV must change;
- a real security advisory appears;
- release workflow or runner availability becomes a separate active blocker;
- Eggpack/Eggup integration would be required.

## 14. Closure evidence

Create:

- `plans/closure/release-operational-qualification-corrective/004-status.md`

The closure record MUST include:

- planning baseline and implementation commit;
- root-cause analysis;
- exact fixture or production correction;
- before/after evidence for the failing test;
- hosted run and all relevant job IDs;
- HTTP 503 evidence + assertion separation proof;
- Windows deterministic contract regression proof;
- MSRV and audit outcomes;
- compatibility/security review;
- unresolved findings;
- downstream disposition: C001 next, then M004.

## 15. Handoff notes

This is the immediate next implementation plan.

Do not create the first release tag while C004 is open. Do not reinterpret the
C003 closure record as proof that the whole hosted matrix was green; it remains
historical evidence for the specific CRLF/audit corrections.

The intended first-release sequence remains:

```text
C004 fully green hosted CI
        |
        v
C001 valid-tag hosted packaging evidence
        |
        v
M002 operational qualification
        |
        v
M004 final release qualification
        |
        v
first standalone archive release
```

Eggpack M003a and Eggup M003b remain deferred and outside this path.
