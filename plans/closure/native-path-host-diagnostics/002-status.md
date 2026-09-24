# Native M002 Closure — Target Route, Interface, and Egress-MTU Evidence

Status: closed

Source plan: `plans/implementation/native-path-host-diagnostics/002-target-route-interface-and-egress-mtu.md`

Source roadmap: `plans/subsystems/native-path-host-diagnostics-roadmap.md`

Reviewed repository baseline: `f0971ff09cd68b0168fcdd9fad5f502d47f99309`. M001 implementation and accepted dependency APIs were re-inspected before M002 work.

Implementation: `f0971ff..6e45e857e6a621f9e5a895ab7618a8b6dbaa6d6b` (pushed to `origin/main`). Hosted qualification: [CI run 36051400630](https://github.com/eggstack/eggprobe/actions/runs/36051400630), all jobs green.

## Finding

The direct-only `route` command and target-scoped report evidence are implemented. The report distinguishes kernel-observed target/source facts from interface metadata and bounded, correlated route candidates. Candidate correlation does not claim authoritative policy-route selection.

## Requirements and evidence

| Requirement | Evidence |
|---|---|
| Target policy and address validation before observation | Route engine resolves under existing policy, rejects unsupported target classes and IPv6 link-local without scope semantics; strict-target test confirms rejection before backend action. |
| Source address without application payload | Kernel UDP socket connect used only to obtain local source selection; no application payload is sent. Loopback IPv4/IPv6 tests exercise observations. |
| Interface identity/state/address/MTU | `netdev` 0.46.3 adapter maps available interface facts; optional fields remain absent when unavailable; list size is bounded. |
| Route table correlation | `netroute` 0.4.0 supplies read-only route entries; longest-prefix/candidate correlation preserves ties, gateway absence, and ambiguity; candidate list is capped and truncation exposed. |
| Honest route semantics and route policy | Report vocabulary labels observed source separately from correlated candidates. It does not emit `selected_route`; policy routing, source-specific rules, and OS route-policy decisions are not claimed. |
| Thin CLI and direct-only behavior | `eggprobe route <target> --json` constructs a plan and renders core output; native requests under Eggress are explicitly unsupported with no direct fallback. |

## Platform qualification

| Target | Evidence and disposition |
|---|---|
| Linux x86_64 | Hosted workspace format/clippy/tests passed; route loopback tests passed. |
| macOS hosted runner | Hosted workspace format/clippy/tests passed; `eggprobe-native` target check passed for x86_64 and aarch64. |
| Windows x86_64 | Hosted workspace format/clippy/tests passed; `eggprobe-native` target check passed. |
| Linux aarch64 | `eggprobe-native` target compilation passed; build-qualified, without host-native route smoke evidence. |
| macOS aarch64 | `eggprobe-native` target compilation passed; hosted CI exercises the macOS host (architecture is runner-provided); no separate local native route smoke claim is made. |

Hosted run 36051400630 passed all platform matrix jobs, MSRV, and audit. Full-workspace cross checks were unavailable locally where SDK/C cross-compilers were absent; this is not represented as native runtime qualification.

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

`rtk git diff --check` passed before the implementation commit. Hosted run 36051400630 completed successfully across Ubuntu, macOS, Windows, MSRV, and audit jobs.

## Review and disposition

- Compatibility/schema: additive route evidence is part of schema 0.4 and generated fixtures; no historical schema files were rewritten.
- Security/privacy: active source selection is target-scoped; private-target policy runs first. No route mutation, shell parsing, user payload, or credential-bearing route information is exposed.
- Failure/cancellation/resources: local metadata reads are bounded; address/interface/route candidate lists are bounded; missing optional facts and ambiguity remain explicit. No persistent state is changed.
- Documentation/operations: operator docs explain route provenance, direct-only scope, MTU meaning, platform availability, and route-policy limitations.
- Unresolved findings: OS policy-routing decisions cannot be proven by route-table enumeration and are explicitly outside this evidence claim.

Roadmap disposition: M002 closed. M003 remains blocked by ICMP backend qualification. M004 is ready after M001/M002. M005 and M006 remain blocked by their registered backend/test-seam requirements. Per the requested sequential stop condition, no subsequent milestone implementation began.
