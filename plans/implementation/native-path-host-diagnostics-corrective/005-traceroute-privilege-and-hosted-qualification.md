# Native Path and Host Diagnostics Corrective C005 — Traceroute Privilege and Hosted-Platform Qualification

Status: ready for handoff

Repository baseline: `0ce9597aa2acad9a61c45a70c3ffaf56333cf3d5`

Source roadmap:

- `plans/subsystems/native-path-host-diagnostics-roadmap.md`
- `plans/002-long-term-roadmap.md#phase-8--native-path-and-host-diagnostics`

Failed/partial closure record:

- `plans/closure/native-path-host-diagnostics/005-status.md`

Hosted failure evidence:

- GitHub Actions run `36138451900`
- Ubuntu quality job `108082118033` — failed
- Windows quality job `108082118099` — failed
- macOS quality job `108082118186` — passed
- Rust 1.89 MSRV job `108082117794` — passed
- dependency audit job `108082117983` — passed

Long-term requirements:

- `plans/000-long-term-specification.md#4-architectural-principles`
- `plans/000-long-term-specification.md#8-probe-families`
- `plans/000-long-term-specification.md#11-security-model`
- `plans/000-long-term-specification.md#12-timeout-repetition-and-concurrency-model`
- `plans/000-long-term-specification.md#14-platforms-and-packaging`
- `plans/000-long-term-specification.md#15-deferred-and-future-capabilities`

Applicable ADRs:

- `plans/adrs/ADR-0001-json-first-core-and-transport-ownership.md`
- `plans/adrs/ADR-0003-native-diagnostics-platform-and-subject-boundary.md`

Primary work class: invariant + corrective capability qualification

## 1. Objective

Correct Native M005's invalid cross-platform privilege assumption and restore
release-grade hosted CI without weakening traceroute semantics or pretending
that ordinary Linux/Windows processes can execute Trippy's raw-socket receive
path without the privileges the backend requires.

The correction must:

1. preserve the existing Eggprobe-owned structured trace evidence model;
2. select Trippy privilege mode according to actual platform/host capability
   rather than forcing `Unprivileged` everywhere;
3. use Trippy's published privilege-discovery seam where elevated/raw-socket
   execution is required;
4. normalize unavailable privilege to Eggprobe's existing typed
   `PermissionDenied` error instead of a generic I/O failure;
5. make routine hosted tests truthful when GitHub runners do not grant the
   required Linux/Windows privilege;
6. obtain a fully green Ubuntu/macOS/Windows/MSRV/audit matrix before current
   M005 qualification is restored.

No schema change is expected.

## 2. Why the historical M005 evidence was incomplete

M005's closure was drafted before the implementation commit was pushed. The
record states that Linux and Windows were covered by the hosted CI matrix, but
the first hosted run containing the traceroute implementation disproves that
claim.

Run `36138451900` on the current baseline fails the same CLI smoke on both
Ubuntu and Windows:

```text
trace_command_reaches_loopback_with_structured_hops
assertion failed: output.status.success()
```

macOS passes.

The current test does not print bounded stdout/stderr on failure, so the CI log
does not show the normalized diagnostic. Source inspection and upstream
qualification nevertheless identify the architectural defect:

- Eggprobe's tracer builder always sets
  `trippy_core::PrivilegeMode::Unprivileged`;
- Trippy 0.13's privilege guide explicitly states unprivileged mode is
  currently supported only on macOS;
- Trippy states Linux always needs privilege for the tracing receive path and
  Windows always needs privilege;
- Linux support uses root/`CAP_NET_RAW`; Windows checks for an elevated token;
- `trippy-privilege 0.13.0` exposes `Privilege::acquire_privileges`,
  `Privilege::discover`, `has_privileges`, `needs_privileges`, and
  `drop_privileges`;
- `trippy-core` exposes `Builder::drop_privileges(true)`, dropping elevated
  privilege after its channel/socket setup.

Upstream references reviewed for this corrective:

- Trippy 0.13 privilege guide:
  `https://github.com/fujiapple852/trippy/blob/master/docs/src/content/docs/0.13.0/guides/privileges.md`
- Trippy issue #101:
  `https://github.com/fujiapple852/trippy/issues/101`
- `trippy-privilege 0.13.0` public API:
  `https://docs.rs/trippy-privilege/0.13.0/trippy_privilege/`

This is not evidence that the M005 structured hop model is wrong. It is
evidence that backend execution-mode qualification was wrong.

## 3. Readiness and dependencies

Hard dependencies:

- Native M001 is closed.
- The M005 structured trace contract and adapter implementation exist.
- `trippy-core = 0.13.0` remains pinned and builds under Rust 1.89.
- Existing `NativeErrorKind::PermissionDenied` and core
  `DiagnosticErrorKind::PermissionDenied` normalization already exist.

No upstream code change is required to begin C005.

Interface dependency:

- published `trippy-privilege 0.13.0`, from the same Trippy release line as
  `trippy-core 0.13.0`.

Operational dependency:

- hosted GitHub Actions Linux/macOS/Windows evidence is required for closure.

C005 is ready now.

M003's separate `ping-async` publication blocker and M006's separate PMTU
DF/PTB-control blocker do not block C005.

## 4. Invariants

C005 MUST preserve:

1. no subprocess `traceroute`, `tracert`, `ping`, or shell parsing;
2. no public-network dependency in routine correctness tests;
3. ordered per-hop/per-attempt evidence and silent-attempt truth;
4. valid destination replies remain successful trace evidence;
5. lack of local privilege is an execution failure, not a silent hop or
   destination timeout;
6. privilege failure maps to `PermissionDenied` at
   `DiagnosticStage::HopProbe`;
7. no generic OS/dependency display string is copied into machine output;
8. macOS may use the backend's unprivileged mode;
9. Linux/Windows MUST NOT request backend unprivileged mode where upstream
   documents it unsupported;
10. Linux privileged execution uses only already-held/permitted capability or
    root privilege; Eggprobe must not invoke `sudo`, modify file capabilities,
    or persist privilege changes;
11. Windows privileged execution requires an already elevated process;
12. elevated privilege, if temporarily acquired by the backend integration,
    is dropped after channel/socket establishment using the backend-supported
    lifecycle and MUST NOT leak beyond the trace operation;
13. target policy is applied before any active trace;
14. max hops, attempts, packet/read/round/outer deadlines remain bounded;
15. schema 0.4, JSON/NDJSON cleanliness, and CLI exit-code semantics remain
    unchanged;
16. Rust 1.89 remains the product MSRV;
17. routed Eggress traces remain unsupported/direct-only;
18. M005's historical closure record is retained rather than rewritten.

## 5. Required production changes

### 5.1 Add the published privilege seam explicitly

Add a direct exact-line dependency appropriate for the existing exact
`trippy-core` pin:

```toml
trippy-privilege = "=0.13.0"
```

Do not rely on reaching a transitive crate through an undeclared dependency.

Re-run duplicate/dependency/security review after adding it. Because
`trippy-core 0.13.0` already uses the same crate line, this should not create
a second Trippy privilege implementation.

### 5.2 Introduce one bounded trace execution-mode decision

Create a small internal Eggprobe-owned helper that determines one of:

- `Unprivileged` — backend mode is valid without elevated privilege;
- `Privileged` — required privilege is available/acquired for the current
  trace operation;
- `PermissionDenied` — platform requires privilege and the current process
  cannot supply it;
- `Unsupported` — only if the backend/platform genuinely cannot support the
  requested trace family independent of privilege.

Do not expose Trippy's privilege types through Eggprobe public contracts.

Expected policy for the currently supported host families:

- **macOS:** use Trippy unprivileged mode for the current UDP trace path.
- **Linux:** execute privilege acquisition/discovery on the same blocking
  worker that will create the trace channel. If `CAP_NET_RAW` is available
  in the permitted/effective state (or process privilege otherwise qualifies),
  use `PrivilegeMode::Privileged`; otherwise return
  `NativeErrorKind::PermissionDenied` before starting the trace.
- **Windows:** inspect the current token. If elevated, use
  `PrivilegeMode::Privileged`; otherwise return
  `NativeErrorKind::PermissionDenied`.
- **other targets:** keep current support claims truthful; do not infer support
  from compilation alone.

If upstream runtime facts differ from these documented semantics at execution,
stop and record the discrepancy rather than hard-coding around it.

### 5.3 Keep acquisition and drop lifecycle on the trace worker

`trace_path` already moves blocking Trippy work into `spawn_blocking`.
Privilege discovery/acquisition and channel construction must occur inside that
same blocking operation.

When privileged mode is used:

- use Trippy's documented acquisition/discovery path;
- configure the tracer to drop privileges after channel/socket creation where
  supported (the `trippy-core` builder exposes
  `drop_privileges(true)`);
- ensure all early returns/errors have a defined privilege disposition;
- do not mutate process-global privilege state outside the bounded tracer
  lifecycle;
- add regression review/tests that a failed builder/run cannot leave privilege
  elevated.

If the upstream API cannot provide a safe acquisition/drop lifecycle for an
embedded multi-threaded process, STOP and narrow Linux support to
already-effective privilege discovery instead of adding unsafe or global
privilege manipulation.

### 5.4 Normalize privilege errors before dependency strings escape

Map:

- explicit privilege preflight denial;
- Trippy privilege acquisition/discovery denial;
- backend `PrivilegeError`;

to:

```text
NativeErrorKind::PermissionDenied
-> DiagnosticErrorKind::PermissionDenied
-> DiagnosticStage::HopProbe
```

Machine output should use a fixed bounded message such as
`"UDP trace requires additional local privilege"`.

Do not expose usernames, token details, capabilities, interface names, OS error
display strings, or dependency-formatted error text in the public report.

## 6. Test correction and hosted evidence strategy

### 6.1 Improve the failing CLI smoke's diagnostics

The current CLI smoke asserts only `output.status.success()`, hiding the
actual JSON/stderr on failure.

Change the test helper/assertion so failures include bounded diagnostic context
in the test panic while preserving stdout/stderr contract assertions.

Diagnostic text exists only in test failure output; do not make production
stdout noisy.

### 6.2 Separate deterministic contract tests from host privilege smoke

Cross-platform correctness MUST NOT depend on ordinary CI runners having raw
socket privilege.

Deterministic tests must cover, through injected/pure privilege facts or a
small internal selection helper:

- macOS-style unprivileged selection;
- Linux privilege available -> privileged mode;
- Linux privilege unavailable -> `PermissionDenied`;
- Windows elevated -> privileged mode;
- Windows non-elevated -> `PermissionDenied`;
- privilege acquisition/discovery error -> bounded typed failure;
- no `Unsupported` substitution for a mere privilege deficit.

Existing scripted/fake-backend trace tests continue to prove:

- ordered hops;
- silent attempts;
- destination reached;
- unreachable;
- deadline;
- max-hops;
- serialization.

### 6.3 Make the live loopback smoke host-policy-aware

The live CLI/native loopback test is an environmental smoke, not the sole
contract test.

It must accept only outcomes justified by the host's actual privilege state:

- when the operation is executable, require success,
  `destination_reached`, hop 1, structured evidence, and clean stderr;
- when required privilege is unavailable, require a non-success diagnostic
  with `PermissionDenied` at `HopProbe`;
- no generic I/O failure, timeout, empty output, panic, or silent skip is an
  acceptable substitute.

Prefer deriving the expectation from the same internal capability decision
rather than hard-coding "Linux always fails CI" or "Windows always fails CI".
Tests must remain valid when run as root/elevated locally.

macOS hosted CI should continue to demonstrate the successful unprivileged
loopback path.

Ordinary Ubuntu/Windows hosted CI may demonstrate the typed permission-denied
path unless those runners actually expose the required privilege.

### 6.4 Optional privileged platform smoke

If a safe existing CI environment can run a Linux job with `CAP_NET_RAW`
without weakening repository security posture, an additional privileged smoke
MAY prove the Linux success path.

It is not required for C005 closure if deterministic privileged-mode tests plus
truthful ordinary-runner permission evidence exist.

Do not add a privileged Windows runner merely to close C005.

## 7. In scope

- direct `trippy-privilege 0.13.0` dependency;
- trace privilege-mode selection;
- bounded acquisition/discovery/drop lifecycle;
- permission normalization;
- test diagnostics;
- deterministic privilege-selection tests;
- host-policy-aware loopback smoke;
- Linux/macOS/Windows hosted requalification;
- planning/docs correction of M005 platform claims.

## 8. Out of scope

- changing schema 0.4 trace fields;
- changing the trace protocol away from UDP merely to avoid privilege policy;
- implementing a new traceroute stack;
- upstreaming Linux/Windows unprivileged support to Trippy;
- invoking `sudo`, `setcap`, UAC elevation, or persistent privilege
  configuration automatically;
- public Internet traces in routine CI;
- ICMP trace mode (still tied to M003/backend work);
- routed Eggress traceroute;
- PMTU/DF/PTB work;
- reverse DNS, ASN, GeoIP, Paris/Dublin/MTR expansion;
- publication/release work.

## 9. Ordered work packages

### WP1 — Capture the real current failure

1. Preserve run `36138451900` and job IDs in closure evidence.
2. Improve the failing live test's bounded failure diagnostics.
3. Re-run focused Linux/Windows-equivalent paths where possible.
4. Confirm the failure is privilege-mode/backend setup and not target-policy,
   schema, or CLI parsing.
5. Record the exact normalized pre-fix error.

If the observed failure contradicts the upstream privilege model, stop and
reassess before implementing WP2.

### WP2 — Add platform-aware privilege selection

1. Add exact `trippy-privilege 0.13.0`.
2. Add a small internal execution-mode helper.
3. Use unprivileged mode only where upstream supports it.
4. Use privileged mode only when required privilege is actually present.
5. Return typed `PermissionDenied` otherwise.
6. Keep privilege types internal.

### WP3 — Bound acquisition/drop behavior

1. Keep privilege handling inside the blocking trace worker.
2. Use backend-supported drop-after-channel creation.
3. Prove error paths do not retain elevated privilege.
4. Do not use shell/system elevation.
5. Stop if safe embedded privilege lifecycle cannot be proven.

### WP4 — Regression suite

Add deterministic coverage for mode selection and error normalization, then
retain/re-run all M005 fake/scripted trace tests.

Strengthen the live CLI smoke to distinguish:

- successful structured loopback trace; versus
- truthful typed privilege denial.

### WP5 — Broad verification

Run:

```text
cargo fmt --all -- --check
cargo check --workspace --all-targets --all-features --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test --workspace --all-features --locked
cargo +1.89.0 check --workspace --all-targets --locked
cargo tree --locked
cargo +stable audit
cargo run -p eggprobe-core --example generate-schemas --locked
```

Require no schema diff.

Review the dependency tree for duplicate Trippy privilege versions and any new
advisory/license issue.

### WP6 — Hosted requalification

Push the correction and require one same-head hosted run with:

- Ubuntu quality green;
- macOS quality green;
- Windows quality green;
- Rust 1.89 MSRV green;
- dependency audit green.

The closure record must state, per OS, whether the live traceroute smoke
demonstrated:

- successful unprivileged execution;
- successful privileged execution; or
- correctly typed permission denial.

"Test passed because live traceroute was skipped" is not acceptable evidence.

### WP7 — Planning reconciliation

After hosted evidence exists:

- create
  `plans/closure/native-path-host-diagnostics-corrective/005-status.md`;
- mark C005 closed;
- restore M005 current qualification only to the platform dispositions actually
  proven;
- retain the historical M005 closure unchanged;
- update the subsystem roadmap and `plans/registry.md`;
- keep M003/C003 blocked on their existing upstream ICMP publication gate;
- keep M006 blocked on its independent PMTU-control gap.

## 10. Failure, cancellation, restart, and concurrency semantics

A cancelled or infrastructure-failed hosted run is not qualification evidence.

A live trace returning permission denial on a host that lacks required
privilege is an expected environmental disposition, not a flaky skip.

A live trace returning timeout on loopback where privilege preflight says the
operation is executable is a failure requiring investigation.

Privilege acquisition and trace execution must remain within one bounded
blocking operation. The outer Eggprobe deadline remains authoritative.

Concurrent trace calls must not corrupt privilege state, source-port allocation,
or backend resources. If the chosen Trippy privilege lifecycle is not safe
under concurrent embedded traces, stop and serialize only the privileged
setup/teardown critical section or restrict to already-effective privilege;
do not silently introduce process-wide races.

## 11. Compatibility and migration

Expected public schema effect: none.

Expected behavior correction:

- macOS retains live unprivileged UDP traceroute;
- Linux without required privilege returns explicit permission denial instead
  of attempting an unsupported backend mode;
- Linux with qualifying privilege may execute the privileged trace path;
- Windows non-elevated processes return explicit permission denial;
- elevated Windows processes may execute the privileged trace path.

This is a correction of execution truth, not a new schema generation.

Operator documentation must state these privilege requirements explicitly.

## 12. Security review

C005 touches privilege-sensitive behavior and therefore requires explicit
review:

- no automatic sudo/UAC invocation;
- no file capability mutation;
- no persistent elevation;
- no privilege retention after tracer channel creation;
- no user-controlled capability names;
- no dependency/OS error strings in JSON;
- target policy still precedes privilege acquisition/network send;
- trace bounds remain unchanged;
- no new write permissions or secrets.

If using `Privilege::acquire_privileges` would create process/thread-wide
side effects not safely bounded in Eggprobe's runtime model, do not use it.
Prefer discovery of already-effective privilege and return
`PermissionDenied` otherwise.

## 13. Required tests

At minimum:

- platform-mode decision: macOS unprivileged;
- Linux effective/permitted raw privilege -> privileged;
- Linux no raw privilege -> permission denied;
- Windows elevated -> privileged;
- Windows non-elevated -> permission denied;
- privilege API error -> bounded typed error;
- backend `PrivilegeError` -> permission denied;
- no permission deficit becomes `Unsupported`;
- loopback trace success when capability decision says executable;
- loopback typed permission failure when capability decision says unavailable;
- IPv4/IPv6 deterministic trace contract coverage remains;
- ordered/silent/unreachable/deadline/max-hop tests remain green;
- direct-only route rejection remains;
- schema fixtures/regeneration unchanged;
- concurrent trace regression remains bounded.

## 14. Documentation updates

Update:

- native diagnostics roadmap platform matrix/Trippy qualification language;
- operator traceroute privilege requirements;
- architecture/backend notes if they currently claim universal unprivileged UDP
  trace;
- registry current state.

Remove or correct any statement that Trippy 0.13 unprivileged UDP mode needs no
capabilities on all supported platforms.

## 15. Acceptance criteria

C005 closes only when:

1. the Ubuntu/Windows failure from run `36138451900` is root-caused with
   bounded diagnostic evidence;
2. Trippy unprivileged mode is no longer selected on platforms where upstream
   documents it unsupported;
3. required Linux/Windows privilege is discovered/handled without automatic
   elevation;
4. unavailable privilege maps to Eggprobe `PermissionDenied/HopProbe`;
5. no generic dependency/OS text leaks into the report;
6. macOS unprivileged live loopback trace remains green;
7. Linux/Windows live smoke is truthful for actual runner privilege state;
8. deterministic tests prove both privileged-success selection and denied
   paths independent of runner policy;
9. all existing M005 structured trace semantics remain green;
10. full Ubuntu/macOS/Windows hosted quality jobs are green on one head;
11. Rust 1.89 MSRV and dependency audit are green on that same head;
12. schema regeneration produces no diff;
13. no privilege leak or unsafe elevation mechanism is introduced;
14. current planning no longer calls M005 fully qualified from the historical
    pre-hosted closure alone.

## 16. Stop conditions

Stop and register separate work if:

- `trippy-privilege 0.13.0` cannot provide a safe embedded privilege
  lifecycle under Eggprobe concurrency;
- privileged mode requires process-global mutation that cannot be bounded;
- the actual Linux/Windows failure persists after correct privilege-mode
  preflight;
- successful Linux/Windows tracing requires a Trippy upstream change;
- schema semantics must change;
- Rust 1.89 can no longer be supported;
- a new security advisory or unacceptable dependency appears;
- fixing the issue would require a homegrown traceroute implementation.

## 17. Closure evidence

Create:

- `plans/closure/native-path-host-diagnostics-corrective/005-status.md`

The closure record MUST include:

- planning baseline and implementation commit;
- pre-fix run/job IDs;
- captured pre-fix failure classification;
- upstream privilege references/versions;
- exact privilege-mode policy;
- acquisition/drop security review;
- deterministic mode-selection tests;
- Linux/macOS/Windows live-smoke disposition;
- qualifying hosted run and job IDs;
- MSRV/audit results;
- dependency-tree review;
- schema no-diff proof;
- compatibility/security findings;
- remaining platform limitations;
- downstream M005/M006/M003 disposition.

## 18. Handoff notes

This is the immediate repository-quality corrective because current `main`
is red and M005's current qualification claim is not supported by hosted
evidence.

Do not weaken the CLI smoke into an unconditional success-or-any-error test.
Permission denial is acceptable only when it is the truthful typed result of
the host privilege state.

Do not conflate this with M006. Restoring traceroute qualification does not
provide the missing DF/PTB controls required for active PMTU discovery.

Do not reopen M005's historical closure record. C005 is the current corrective
authority.
