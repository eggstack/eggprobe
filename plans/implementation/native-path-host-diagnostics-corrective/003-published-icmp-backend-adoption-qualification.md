# Native Path/Host Diagnostics Corrective C003 — Published ICMP Backend Adoption Qualification

Status: blocked — C002 closure plus published ping-async release

Repository baseline: `d9c954ca4967796eb322b6f4e4cde738e3f29f49`

Source roadmap:

- `plans/subsystems/native-path-host-diagnostics-roadmap.md`

Prerequisite plan:

- `plans/implementation/native-path-host-diagnostics-corrective/002-ping-async-upstream-icmp-contract-enablement.md`

Downstream capability plan:

- `plans/implementation/native-path-host-diagnostics/003-icmp-echo-diagnostics.md`

Applicable ADR:

- `plans/adrs/ADR-0003-native-diagnostics-platform-and-subject-boundary.md`

Primary work class: dependency adoption qualification

## 1. Objective

Qualify a published crates.io `ping-async` release containing the C002-accepted ICMP evidence interface before Eggprobe M003 takes a production dependency on it.

This separates two concerns cleanly:

- C002 proves and upstreams the interface;
- C003 proves the immutable published artifact is suitable for Eggprobe;
- M003 performs Eggprobe integration and exposes the capability.

## 2. Readiness gate

C003 becomes ready only when BOTH are true:

1. C002 has closure evidence identifying the accepted upstream commit/API;
2. crates.io has published a release containing that commit/interface.

The source tree's `Cargo.toml` version or an untagged Git commit is not sufficient.

At the planning baseline, docs.rs still reports `ping-async 1.0.2` as the latest published release, so this plan is blocked.

## 3. Invariants

- no production git dependency;
- no vendored fork hidden in Eggprobe;
- Rust 1.89 remains the Eggprobe MSRV;
- Linux x86_64/aarch64, macOS x86_64/arm64, and Windows x86_64 remain the target contract;
- Eggprobe-authored code remains `unsafe_code = "forbid"`;
- dependency-specific enums/types remain behind `eggprobe-native`;
- qualification must test semantics, not merely compilation.

## 4. In scope

- confirm the crates.io package contains the exact C002 interface;
- inspect package source and lockfile resolution;
- license/security/supply-chain review;
- Rust 1.89 build qualification;
- supported-target build qualification;
- host-native loopback/error/cancellation checks on Linux/macOS/Windows;
- confirm payload-length, responder, sequence, RTT, and outcome semantics;
- decide whether the published release is accepted for M003.

## 5. Out of scope

- exposing `eggprobe icmp`;
- core engine dispatch;
- CLI changes;
- schema changes;
- traceroute implementation;
- PMTU implementation;
- depending on a Git SHA while waiting for crates.io.

## 6. Ordered work packages

### WP1 — Published artifact provenance

1. Record crate version, checksum, release date, repository tag/commit, license, and dependency graph.
2. Prove the package contains the C002-accepted API/behavior.
3. Reject a release that has the same version text but not the accepted source.

### WP2 — MSRV and target builds

Using a disposable qualification harness or temporary branch, verify the dependency under:

```text
Rust 1.89
x86_64-unknown-linux-gnu
aarch64-unknown-linux-gnu
x86_64-apple-darwin
aarch64-apple-darwin
x86_64-pc-windows-msvc
```

Record whether any target is build-qualified only.

### WP3 — Runtime semantic qualification

Prove through public API only:

- requested payload lengths at 0, 1, 7, 8, default 56, and maximum 1024 produce correct wire/request semantics;
- loopback echo returns actual responder, sequence, and RTT;
- local deadline produces the deadline outcome, not Time Exceeded;
- a deterministic TTL/hop-limit-expiry fixture produces Time Exceeded separately where supported;
- unreachable classification preserves available subtype/code;
- permission/resource creation failures return errors and do not panic;
- cancellation releases request registration/background state.

Do not access dependency-private modules to manufacture passing evidence.

### WP4 — Dependency hygiene

Run/record:

- `cargo tree` for duplicate socket/runtime/platform crates;
- `cargo audit`;
- license review;
- feature/default-feature review;
- dependency size/compile-impact note.

A duplicate dependency is not automatically disqualifying, but any duplicate `tokio`, `socket2`, Windows binding, or packet crate line must be understood and documented.

### WP5 — Adoption disposition

If all hard requirements pass:

1. write C003 closure;
2. update registry/roadmap so M003 becomes `ready`;
3. record the exact accepted crate version/range and feature configuration;
4. hand off to M003.

If any hard requirement fails, keep M003 blocked and register a narrow corrective or fallback-backend plan.

## 7. Required verification

Qualification evidence must include exact commands/output or hosted run IDs for:

```text
cargo +1.89.0 check
cargo check --target x86_64-unknown-linux-gnu
cargo check --target aarch64-unknown-linux-gnu
cargo check --target x86_64-apple-darwin
cargo check --target aarch64-apple-darwin
cargo check --target x86_64-pc-windows-msvc
cargo tree
cargo audit
```

Plus public-API runtime fixtures described in WP3.

## 8. Acceptance criteria

C003 closes only when the immutable published crate:

1. contains the C002-accepted API;
2. builds under Rust 1.89;
3. builds for every Eggprobe target;
4. preserves the required evidence using public API only;
5. has acceptable runtime semantics on Linux/macOS/Windows;
6. has acceptable dependency/license/security posture;
7. requires no Eggprobe unsafe code, vendoring, or git pin.

## 9. Stop conditions

Stop if:

- publication omits/reverts required C002 behavior;
- the release raises effective MSRV above 1.89;
- a target cannot build;
- Windows/macOS/Linux runtime behavior cannot be normalized truthfully;
- the release panics on expected OS failure;
- dependency risk is unacceptable.

## 10. Closure evidence

Create:

`plans/closure/native-path-host-diagnostics-corrective/003-status.md`

Include crate version/checksum/tag/commit, C002 API correspondence, MSRV/target matrix, runtime fixture results, dependency review, and final M003 readiness disposition.
