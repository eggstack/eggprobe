# Native Path/Host Diagnostics Corrective C004 Closure — Complete ICMP Semantics and Durable Upstream Handoff

Status: closed

Source implementation plan:

- `plans/implementation/native-path-host-diagnostics-corrective/004-complete-icmp-semantics-and-durable-upstream-handoff.md`

Corrective baseline:

- Eggprobe planning baseline `f82eb8b30e65e003fc95520e37a338f9e80a002c`
- Current upstream base `b3769ef1b1e8828cf3de14944cb2f80120b2e893` (`ping-async` 1.2.0)
- Historical prototype recorded by C002: `968cdec0654c2c7ef674750aba0513df071a6aba` (not durably reachable; reconstructed from the C002 closure and current upstream source)

## Executive finding

C004 is complete. The corrective patch is committed, pushed to a durable user fork, and opened as an upstream pull request. It implements exact payload lengths from 0 through 1024 without silently padding short requests, preserves correlation for both replies and payload-free ICMP errors, exposes truthful responder/outcome evidence, and keeps the requested target separate from the observed responder.

No Eggprobe production code, schema, dependency, or runtime behavior changed. C003 remains blocked until the upstream surface is accepted/merged and an immutable crates.io release contains it.

## Durable upstream handoff

- Repository: `https://github.com/dbowm91/ping-async`
- Branch: `eggprobe/c004-icmp-semantics-and-durable-handoff`
- Tested commit: `040771431b6e0b3f66bb17629636c8a3b4956b45`
- Upstream pull request: https://github.com/hankbao/ping-async/pull/9
- Upstream base: `master` at `b3769ef1b1e8828cf3de14944cb2f80120b2e893`
- PR state at closure: open; maintainer acceptance/merge is a downstream C003 prerequisite, not a C004 implementation prerequisite.

The PR description is retained in the handoff work area and records the exact Rust 1.89 verification commands and the build-qualified/runtime-qualified boundary.

## Implementation evidence

### Payload and correlation

- `with_payload_len` accepts exactly `0..=1024` on Unix and Windows and rejects larger values with `InvalidInput`.
- Unix constructs the ICMP payload vector with exactly the requested length. Payloads of at least eight bytes retain the timestamp prefix; payloads of 0–7 contain no hidden correlation bytes.
- Unix short-payload sequence numbers come from a process-wide non-reusing 16-bit space. Long-payload sequence numbers are monotonic per requestor and are not reused. Exhaustion returns an error.
- Exact sent payload bytes are retained for reply matching. ICMP errors, which do not echo payload bytes, cannot collide with a later request because sequence numbers are not reused.
- Windows passes a dynamically sized request-data buffer and sizes the IP Helper reply buffer for the maximum accepted payload.
- `IcmpEchoReply::sequence()` is populated on Unix and remains unavailable on Windows because the parsed IP Helper reply does not expose the sequence.

### Responder and outcome semantics

| Platform/path | Echo responder | ICMP-error responder | Sequence | Structured outcome |
|---|---|---|---|---|
| macOS | `recv_from` source | outer IPv4 ICMP-error source when present | populated | type/code mapped; Time Exceeded is `TimeExceeded` with coarse `Unreachable` |
| Linux | `recv_from` source | error-queue offender/source when supplied | populated | `ee_type`/`ee_code` mapped through the same ICMP outcome table |
| Windows | parsed IP Helper reply `Address` | unavailable on immediate/local failures | unavailable | IP Helper status mapped; driver timeout is `LocalTimeout`, TTL/time-exceeded statuses are `TimeExceeded` |

- `IcmpEchoStatus::TimedOut` is reserved for local deadline/driver-local timeout semantics.
- Received ICMP Time Exceeded is not collapsed into local timeout.
- Unspecified or wrong-family responder addresses are returned as `None`.
- `destination()` remains the requested target.

## Required test and build evidence

Rust 1.89 commands completed successfully:

```text
cargo +1.89.0 fmt --all -- --check
cargo +1.89.0 check --workspace --all-targets --all-features --locked
cargo +1.89.0 clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo +1.89.0 test --workspace --all-features --locked
cargo +1.89.0 build --target x86_64-unknown-linux-gnu --locked
cargo +1.89.0 build --target aarch64-unknown-linux-gnu --locked
cargo +1.89.0 build --target x86_64-apple-darwin --locked
cargo +1.89.0 build --target aarch64-apple-darwin --locked
cargo +1.89.0 build --target x86_64-pc-windows-msvc --locked
cargo audit
```

Host-native macOS aarch64 tests passed, including 62 unit tests, 6 integration tests, and 6 doc tests. Coverage includes payload boundaries `0, 1, 7, 8, 32, 56, 1024`, invalid lengths, concurrent short requests, stale replies and stale ICMP errors, IPv4/IPv6 responder mapping, macOS outer responder mapping, Linux error-queue code/responder parsing, local timeout versus received Time Exceeded, cancellation, loopback, and existing concurrency/resource regressions.

Linux x86_64/aarch64, macOS x86_64, and Windows x86_64 MSVC were build-qualified from this macOS environment. Linux and Windows host-native runtime tests were not available here and are not claimed. Linux error-queue tests are compile-checked for the target and remain covered by the upstream CI/runtime matrix when run natively.

## Planning disposition

- C002 remains **corrective required** as historical evidence; its closure file is unchanged.
- C004 is **closed** by this record and the durable PR handoff.
- C003 remains **blocked** on upstream acceptance/merge and publication of a crates.io release containing commit `040771431b6e0b3f66bb17629636c8a3b4956b45` or an explicitly reconciled equivalent.
- M003 remains **blocked** on C003 closure and must consume the published qualified package, never the fork or Git SHA.
- M004 remains independently ready.
- M005/M006 remain blocked for their previously registered reasons.

## Repository changes

Only planning evidence was changed in Eggprobe:

- `plans/closure/native-path-host-diagnostics-corrective/004-status.md`
- `plans/registry.md`
- `plans/subsystems/native-path-host-diagnostics-roadmap.md`

No production Rust, schema, Cargo manifest, lockfile, or release artifact changed.
