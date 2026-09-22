# Transport Corrective C001 — Policy, Deadline, Retry, and Error Semantics

Status: blocked

Planning baseline: `24965aa0ba2696b201e9c74531920877529821d5`

Source addendum:

- `plans/subsystems/transport-probe-engine-corrective-addendum.md#3-c001--execution-policy-deadline-retry-and-error-semantics`

Hard dependency:

- Foundation corrective C002 closed.

Primary class: correctness + security + invariant

## 1. Objective

Make the working probe engine enforce the same target, deadline, retry, and result semantics across every direct transport path before deeper routed qualification.

## 2. Strict target policy must govern the actual HTTP connection

Current direct HTTP sends the URL to Eggfetch's native connector after only plan validation. A strict-policy preflight lookup followed by a second Eggfetch lookup would remain vulnerable to DNS rebinding and is not sufficient.

Preferred correction:

- implement a direct policy-aware Eggfetch `Dialer` that performs the actual DNS/address selection and TCP connection under Eggprobe's `TargetPolicy`;
- let Eggfetch continue to own origin TLS and HTTP above that stream;
- use the same private/reserved-address classification as direct TCP/TLS;
- ensure every resolved address considered by the dialer is policy-checked;
- never silently fall back to Eggfetch's native resolver when strict policy is requested.

If Eggfetch's public dialer seam cannot preserve origin TLS correctly for the direct case, stop and record the interface blocker rather than using a preflight-only policy check.

## 3. Retry semantics

A nonzero `ExecutionPolicy.retries` must not be silently ignored.

For this corrective pass, either:

A. implement visible retry attempts with a stable attempt representation in the current `0.2` contract; or
B. reject `retries > 0` during plan validation with an explicit unsupported/validation error and retain actual retry implementation for a later capability milestone.

Do not implement hidden retries.

Given the goal of a bounded corrective pass, option B is preferred unless Foundation C002 deliberately added an attempt model.

## 4. Deadline and cancellation semantics

- one outer execution deadline;
- no nested request deadline may exceed the remaining outer budget;
- per-probe operations must be cancellation-safe by drop;
- a deadline must produce a structured terminal state rather than only a top-level warning where the failed operation is identifiable;
- execution ID must be allocated once and remain identical across success, failure, and timeout paths.

## 5. Direct TCP evidence

On successful direct TCP connection, record both peer and `TcpStream::local_addr()` when available.

Preserve per-address attempt count and deterministic address ordering evidence where portable.

## 6. Error normalization

Expand direct I/O normalization where portable:

- connection refused;
- timed out;
- network unreachable;
- host unreachable;
- DNS;
- policy;
- generic I/O.

Messages remain bounded and secret-free. Do not parse OS display text to infer categories when `ErrorKind` or source typing is unavailable.

## 7. Required tests

- strict direct HTTP rejects loopback/private destinations on the actual dial path;
- permissive HTTP can reach a local deterministic fixture;
- DNS rebinding seam/property test proves policy is evaluated on addresses actually dialed, not only preflight input;
- nonzero retries are either visibly implemented or rejected;
- outer deadline covers DNS/TCP/TLS/HTTP work coherently;
- cancellation drops pending work without background tasks;
- connection-refused and controlled timeout classifications;
- local/peer addresses on successful direct TCP;
- stable single execution ID on success and timeout.

## 8. Verification

Run full locked/MSRV/Clippy/test/audit gates plus deterministic local network tests. No public endpoint is closure evidence.

## 9. Acceptance criteria

- strict policy cannot be bypassed by direct HTTP;
- ignored retry configuration is eliminated;
- deadline/cancellation semantics are represented truthfully;
- direct TCP captures local address when observable;
- execution ID is stable;
- no unresolved medium-or-higher C001 finding remains.

## 10. Closure evidence

Create `plans/closure/transport-probe-engine-corrective/001-status.md` with the policy-path diagram, retry disposition, deadline/cancellation matrix, error mapping, local fixture evidence, and exact dependency versions.
