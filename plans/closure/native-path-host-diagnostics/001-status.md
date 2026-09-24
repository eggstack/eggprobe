# Native M001 Closure — Native Contract and Platform Substrate

Status: closed

Source plan: `plans/implementation/native-path-host-diagnostics/001-native-contract-and-platform-substrate.md`

Source roadmap: `plans/subsystems/native-path-host-diagnostics-roadmap.md`

Reviewed repository baseline: `f0971ff09cd68b0168fcdd9fad5f502d47f99309` (`main` before implementation). The plan's `8055730…` provenance baseline was stale; current `main`, architecture ownership, and dependency APIs were inspected before changes.

Implementation: `f0971ff..6e45e857e6a621f9e5a895ab7618a8b6dbaa6d6b` (pushed to `origin/main`). Hosted qualification: [CI run 36051400630](https://github.com/eggstack/eggprobe/actions/runs/36051400630), all jobs green.

## Finding

The schema 0.4 contract, internal `eggprobe-native` substrate, normalized native error vocabulary, fake backend seam, and direct-only handling for native probes are implemented. This closes the infrastructure/invariant milestone. It does not claim ICMP, UDP, traceroute, or PMTU support; those dispatch as typed unsupported results until their plans qualify and implement backends.

## Requirements and evidence

| Requirement | Evidence |
|---|---|
| Schema 0.4 is current, typed, generated, and historical schemas remain immutable | Core Rust domain types; generated `schemas/plan-0.4.json` and `report-0.4.json`; 0.3 fixture/schema retained; schema tests reject leaked internal expression fields and verify round trips. |
| Internal backend seam, safe-code boundary, normalized outcomes | `crates/eggprobe-native`; `#![deny(unsafe_code)]`; fake backend; typed completed/unsupported/permission/error normalization tests. |
| Every native plan family maps to its probe kind; no accidental Eggress direct fallback | Core dispatch and engine tests cover family mapping, typed unsupported operations, and Eggress/native rejection. |
| Dependency/API qualification | `netdev` 0.46.3 and `netroute` 0.4.0 accepted for M002; `ping-async` 1.0.2 and `ping-rs` 0.1.2 rejected for M003; `tracert` 0.12.0 rejected for M005. Evidence and reasons are in the subsystem roadmap. |
| Runtime/MSRV and dependency policy | Workspace MSRV check and `cargo tree --locked`; target compilation of `eggprobe-native` passed for Linux x86_64/aarch64, macOS x86_64/aarch64, and Windows x86_64. |

## Verification

The exact repository verification sequence passed locally:

```text
cargo fmt --all -- --check                         PASS
cargo check --workspace --all-targets --all-features --locked PASS
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings PASS
cargo test --workspace --all-features --locked     PASS (56 tests across 14 suites)
cargo +1.89.0 check --workspace --all-targets --locked PASS
cargo tree --locked                                PASS
cargo audit                                        PASS (one pre-existing allowed unmaintained `paste` advisory)
```

Hosted run 36051400630 passed Linux, macOS, and Windows format/clippy/test jobs, the Rust 1.89 MSRV check, and dependency audit. Full-workspace cross-target checks requiring unavailable local cross C compilers/SDK headers were not treated as passing; target-native crate checks above did pass.

## Review and disposition

- Compatibility/schema: current plans are exact-version validated; 0.4 is explicit and prior schema generations remain evidence. No migration path was added.
- Security/privacy: native dependency errors are normalized; no dependency display strings are forwarded. Route credentials do not enter native probe evidence. Native probes are direct-only and never silently bypass Eggress.
- Failure/cancellation/resources: the substrate introduces no persistent runtime, worker, or active long-running packet I/O. Fake backend and outer engine deadline/error handling remain deterministic.
- Documentation/operations: README, schemas, architecture, operator guide, release smoke plan, registry, and subsystem roadmap describe schema 0.4 and the new boundary.
- Unresolved findings: no M001 acceptance findings. M003's safe ICMP backend is not qualified; this is registered as a blocker and is outside this plan's completed scope.

Roadmap disposition: M001 closed; M002 unblocked and subsequently closed; M003 blocked pending a safe backend/API meeting payload, responder, permission, and cancellation requirements; M004 is independently ready after M001/M002; M005 blocked pending a truthful trace backend; M006 remains blocked on its prerequisites and PMTU controls. Execution stops before M003 per the active sequence and user instruction.
