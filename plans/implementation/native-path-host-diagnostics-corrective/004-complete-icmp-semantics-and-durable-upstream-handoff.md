# Native Path/Host Diagnostics Corrective C004 — Complete ICMP Semantics and Durable Upstream Handoff

Status: closed

Repository baseline: `f82eb8b30e65e003fc95520e37a338f9e80a002c`

Closure: `plans/closure/native-path-host-diagnostics-corrective/004-status.md`

Source roadmap:

- `plans/subsystems/native-path-host-diagnostics-roadmap.md`

Corrected historical evidence:

- `plans/implementation/native-path-host-diagnostics-corrective/002-ping-async-upstream-icmp-contract-enablement.md`
- `plans/closure/native-path-host-diagnostics-corrective/002-status.md`

Downstream plans:

- `plans/implementation/native-path-host-diagnostics-corrective/003-published-icmp-backend-adoption-qualification.md`
- `plans/implementation/native-path-host-diagnostics/003-icmp-echo-diagnostics.md`
- `plans/implementation/native-path-host-diagnostics/005-traceroute-path-diagnostics.md`

Applicable ADR:

- `plans/adrs/ADR-0003-native-diagnostics-platform-and-subject-boundary.md`

Primary work class: corrective upstream implementation + evidence durability

## 1. Objective

Correct the over-closure of C002 before any published `ping-async` artifact can be treated as eligible for Eggprobe.

C002 successfully proved the upstream direction and produced a useful additive prototype, but its closure record also documents three unmet hard requirements:

1. `with_payload_len(0..=1024)` accepts the Eggprobe range, but the current packet path still relies on an 8-byte timestamp correlation payload, so requested lengths below eight bytes are not yet proven to be the actual on-wire Echo payload length;
2. `IcmpEchoReply::responder()` exists in the prototype surface, but Unix receive/error paths and Windows reply semantics are not yet wired end-to-end to a truthful responder value;
3. the prototype commit `968cdec0654c2c7ef674750aba0513df071a6aba` and its patch/PR text were recorded only in transient local `/tmp/opencode` paths and are not reachable from the reviewed upstream repository or a durable public/user-controlled fork.

These are implementation/evidence defects, not merely outstanding publication evidence. C003 MUST remain blocked until C004 closes.

## 2. Current authority and status correction

The historical C002 closure file is preserved unchanged as evidence of what was actually attempted.

Active planning authority SHALL treat:

- C002: **corrective required**;
- C004: **ready** and current authority for the unfinished upstream work;
- C003: **blocked on C004 closure plus an immutable crates.io publication containing the C004-qualified interface**;
- M003: **blocked on C003 closure**;
- M004: independently ready and unaffected.

C004 does not invalidate C002's useful research, test results, or API direction. It corrects only the claim that the upstream implementation leg was substantially complete.

## 3. Invariants

- no Eggprobe production Git dependency on `ping-async`;
- no vendored runtime fork inside Eggprobe;
- no schema bump: schema 0.4 already expresses M003;
- exact requested ICMP Echo payload length must be observable on wire for every accepted value 0..=1024;
- short payloads must not weaken concurrent-request, cancellation, or stale-reply protection;
- responder evidence must be the actual observed responder when the platform exposes it, otherwise unavailable/None;
- network Time Exceeded must remain distinct from local deadline expiry;
- expected platform/permission/resource failures must return typed errors rather than panic;
- Rust 1.89 remains the qualification MSRV;
- Eggprobe-authored code remains `unsafe_code = "forbid"`;
- no public-Internet endpoint is the sole correctness oracle.

## 4. In scope

### 4.1 Recover or reconstruct the C002 prototype

At execution start:

1. check whether the original local branch/patch still exists;
2. if it exists, verify its HEAD and diff against upstream `b3769ef1b1e8828cf3de14944cb2f80120b2e893`;
3. if it does not exist, reconstruct the additive C002 surface from the closure record and current upstream source;
4. rebase/adapt onto current upstream `master` if upstream moved, recording any semantic conflicts.

Do not treat a missing `/tmp` artifact as loss of authority; C002 closure text is enough to reconstruct intent, but the reconstructed code must be requalified from scratch.

### 4.2 Exact payload-length semantics

Complete the public payload-length contract so an Eggprobe request for N payload bytes produces exactly N ICMP Echo payload bytes for every N in 0..=1024.

The implementation must preserve:

- concurrent requests;
- cancellation safety;
- stale-reply rejection;
- sequence exhaustion/wrap behavior;
- IPv4 and IPv6;
- Linux ping-socket identifier behavior;
- Windows IP Helper request-data behavior.

The implementation technique is deliberately not prescribed. Acceptable approaches may include revised identifier/sequence allocation, a correlation token only when the requested payload has room, socket/requestor generation changes, or another design that can be proved correct.

It is NOT acceptable to:

- silently round short payloads up to eight bytes;
- claim the configured length while transmitting a different length;
- disable stale-reply protection for short payloads;
- forbid 0..7 after schema 0.4 already accepts them unless a separate schema corrective is first approved.

### 4.3 Truthful responder plumbing

Complete the responder path through the public reply.

Unix requirements:

- use the actual source address from the receive operation for Echo Reply where available;
- preserve the outer ICMP-error responder for Time Exceeded/unreachable messages where available;
- on Linux error-queue paths, inspect and preserve offender/source metadata such as the kernel-provided offender address when trustworthy;
- do not substitute the requested target for an intermediate-router response.

Windows requirements:

- verify the semantics of the native reply address returned by the IP Helper APIs for IPv4 and IPv6;
- wire the address to `responder()` when it is truly the response source;
- keep the requested target as a separate request field/semantic;
- return `None` when the API does not provide a trustworthy responder for a specific outcome.

### 4.4 Structured outcome preservation

Retain the C002 `IcmpOutcome` direction, including:

- `EchoReply`;
- `LocalTimeout`;
- `TimeExceeded`;
- destination/network/host/no-route subclasses where directly distinguishable;
- `Other` or equivalent bounded fallback.

Preserve ICMP type/code internally or publicly as needed to avoid future lossy remapping. Do not regress to `TimeExceeded == TimedOut`.

### 4.5 Durable upstream handoff

Before C004 may close, the corrected source MUST be durably reachable outside a transient local filesystem.

At least one of these must exist:

1. an opened upstream pull request against `hankbao/ping-async`;
2. a branch in a durable user-controlled fork with immutable commit SHA and repository URL, plus an upstream-ready PR description;
3. if neither remote action is possible because of permissions/tooling, a committed patch artifact inside Eggprobe planning evidence plus exact base commit and apply instructions, followed by a separately registered external-handoff blocker.

Preferred disposition is an actual upstream PR. Merely recording a local path under `/tmp` is not closure evidence.

A committed patch under Eggprobe, if used as last-resort evidence, is planning/evidence material only and MUST NOT become a vendored production dependency.

## 5. Out of scope

- adding `ping-async` to Eggprobe production manifests;
- implementing M003 in Eggprobe;
- waiting for crates.io publication;
- implementing full traceroute;
- implementing PMTU;
- requiring reply TTL/hop-limit for M003;
- redesigning Eggprobe schema 0.4;
- importing Synvoid or Eggsec as runtime dependencies.

## 6. Ordered work packages

### WP1 — Artifact recovery and upstream rebase

1. Recover the C002 local branch/patch if still present.
2. Compare it against the C002 closure description.
3. Fetch/reinspect current upstream `master`.
4. Rebase or reconstruct the patch on the current upstream baseline.
5. Record old prototype SHA, new base SHA, and new corrective branch SHA.

### WP2 — Short-payload correlation design

1. Document why the existing timestamp-in-payload design fails exact 0..7 semantics.
2. Choose a correlation design that preserves stale-reply safety for zero-length payloads.
3. Add deterministic unit tests for sequence allocation/reuse/wrap and stale replies.
4. Implement exact payload construction for all lengths 0..=1024.
5. Ensure the historical default payload behavior remains compatible for existing callers of `new()`.

### WP3 — Responder plumbing

1. Unix Echo Reply: preserve receive source.
2. macOS/Unix ICMP errors: preserve outer responder where available.
3. Linux error queue: preserve offender/source metadata when the kernel supplies it.
4. Windows IPv4/IPv6: verify native reply-address semantics and wire them without relabeling the target.
5. Add platform-neutral public API tests and target-specific parser/adapter tests.

### WP4 — Outcome and failure regression

Re-run and extend tests proving:

- `LocalTimeout` is only local deadline expiry;
- `TimeExceeded` is a received network outcome;
- unreachable subtypes remain stable;
- permission/resource failures return errors;
- cancellation cleans registrations;
- router/background task failures remain bounded and persistent;
- no panic on expected OS failure.

### WP5 — Wire-level semantic verification

Prove exact request payload size at representative boundaries:

```text
0, 1, 7, 8, 32, 56, 1024
```

Use deterministic packet-construction tests and, where feasible, local capture/test harnesses. The proof must validate packet bytes/length, not just the value returned by `payload_len()`.

Also prove:

- reply correlation for each short-payload boundary;
- multiple concurrent short-payload requests;
- cancellation followed by sequence reuse;
- delayed/stale reply does not satisfy a newer request.

### WP6 — Platform qualification

On Rust 1.89:

- Linux x86_64 build + host-native tests where runner supports ICMP ping sockets;
- Linux aarch64 build qualification;
- macOS x86_64/arm64 build, with host-native runtime on available architecture;
- Windows x86_64 MSVC build and host-native runtime where available.

A build-only target must be labeled build-qualified, never runtime-qualified.

### WP7 — Durable upstream publication of the patch

1. Push the corrective branch to a durable fork or open the upstream PR.
2. Record repository, branch, commit SHA, base upstream SHA, and PR URL/number when available.
3. Preserve PR description and any maintainer-requested changes.
4. Confirm the durable SHA contains the exact tested code.
5. Only then write C004 closure.

## 7. Required tests

At minimum, upstream tests must cover:

- payload length 0;
- payload length 1;
- payload length 7;
- payload length 8;
- payload length 56;
- payload length 1024;
- invalid payload >1024;
- concurrent short-payload sends;
- stale-reply rejection for short payloads;
- sequence wrap/exhaustion behavior;
- IPv4/IPv6 echo responder mapping;
- Time Exceeded responder mapping;
- unreachable responder/outcome mapping where available;
- local timeout vs Time Exceeded;
- cancellation cleanup;
- Windows handle-creation/resource error path;
- Linux error-queue parsing;
- regression of existing loopback/concurrency tests.

## 8. Verification

Record exact results for:

```text
cargo +1.89.0 fmt --all -- --check
cargo +1.89.0 clippy --all-targets --all-features -- -D warnings
cargo +1.89.0 test
cargo +1.89.0 build --target x86_64-unknown-linux-gnu
cargo +1.89.0 build --target aarch64-unknown-linux-gnu
cargo +1.89.0 build --target x86_64-apple-darwin
cargo +1.89.0 build --target aarch64-apple-darwin
cargo +1.89.0 build --target x86_64-pc-windows-msvc
cargo audit
```

Cross-target clippy/check may also be recorded where toolchain support permits.

The closure must include a semantic matrix for payload length and responder behavior by platform, not just compile results.

## 9. Acceptance criteria

C004 closes only when all are true:

1. a durable corrective branch/PR/patch artifact exists outside transient `/tmp`;
2. exact on-wire payload lengths 0..=1024 are implemented, with boundary tests;
3. short payloads retain stale-reply and concurrency safety;
4. truthful responder identity is wired end-to-end where each platform exposes it, and unavailable is explicit otherwise;
5. local deadline and received Time Exceeded remain distinct;
6. existing cancellation/concurrency/error guarantees remain green;
7. Rust 1.89 and supported target builds pass;
8. the exact tested corrective commit is the durable handoff commit;
9. no Eggprobe production dependency/schema/runtime code changes occur;
10. registry/roadmap update C002 to historical corrective-required, C004 to closed, and C003 to blocked only on upstream acceptance/merge plus crates.io publication of the C004-qualified surface.

## 10. Stop conditions

Stop and register a narrower architecture decision if:

- exact zero-length payload semantics cannot coexist with reliable reply correlation on one or more required platforms;
- truthful responder identity is fundamentally unavailable on a required platform and schema semantics need revision;
- the required behavior would force a breaking upstream API rather than an additive one;
- Rust 1.89 can no longer build current upstream;
- expected platform failures panic;
- the upstream project is inactive/unwilling to accept the necessary semantics and maintaining a shared Eggstack ICMP crate becomes the more sustainable path.

## 11. Closure evidence

Create:

`plans/closure/native-path-host-diagnostics-corrective/004-status.md`

The closure must record:

- recovered/reconstructed prototype provenance;
- current upstream base;
- durable branch/PR/patch location;
- final tested commit SHA;
- exact payload-length evidence;
- responder behavior matrix;
- outcome/cancellation/error regression evidence;
- MSRV/target matrix;
- upstream review state;
- precise C003 readiness gate.

C002's existing closure file remains unchanged and is referenced as historical evidence corrected by C004.
