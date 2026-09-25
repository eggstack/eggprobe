# Native M004 Closure — Direct UDP Service Diagnostics

Status: closed

Source plan: `plans/implementation/native-path-host-diagnostics/004-direct-udp-service-diagnostics.md`

Source roadmap: `plans/subsystems/native-path-host-diagnostics-roadmap.md`

Reviewed repository baseline: `902cf3a` (`main` before implementation). The plan's `8055730…` provenance baseline was stale; current `main`, architecture ownership, and dependency APIs were inspected before changes.

Implementation: `902cf3a..258e64d` (unpushed at closure drafting; push follows with the closure commit).

## Finding

The direct-only `udp` probe and `eggprobe udp` CLI command are implemented. A locally accepted send reports transmission only and never claims remote service health. Transmit-then-silence and transmit-then-unreachable are completed `Ok` observations with retained evidence, distinct from local bind/connect/send failures. Request payloads are bounded input-only bytes and never appear in reports. No Eggress route is consumed and no routed datagram abstraction was introduced.

## Requirements and evidence

| Requirement | Evidence |
|---|---|
| Connected direct UDP sockets, IPv4/IPv6 | `eggprobe-native::udp_exchange` binds a family-correct ephemeral socket and connects it to the resolved destination; engine v4/v6 loopback echo tests pass. |
| Optional bounded request payload | Plan validation caps payloads at 1200 bytes (`InvalidNativeBounds`); native `MAX_UDP_PAYLOAD_BYTES` re-checks before socket use; oversize CLI payloads rejected at plan construction. |
| Optional bounded receive window | `receive: false` completes as `Sent`; `receive: true` awaits one reply under the remaining outer deadline with a 1400-byte sample cap (`MAX_UDP_RESPONSE_SAMPLE_BYTES`). |
| Local/source socket evidence and byte counts | `UdpEvidence{local, transmitted_bytes}` recorded from the connected socket in every completed observation. |
| Response source/bytes/sample only when requested | `response_source/response_bytes/response_sample` are `None` unless `receive` is set; echo test asserts source, count, and exact sample round trip. |
| ICMP/OS unreachable evidence | `ConnectionRefused/HostUnreachable/NetworkUnreachable` on the connected socket yields completed `Unreachable` outcome; closed-port test accepts `Unreachable` or `Timeout` without brittle timing assumptions. |
| Timeout distinct from local failure | Silence yields `Ok` + `Timeout` outcome with retained `transmitted_bytes`; bind/connect/send failures yield `Failed` + normalized `DiagnosticError`. One rule, used consistently. |
| No implicit retry; cancellation-safe | Single send, single bounded receive, no retry path; the exchange future drops the socket with no background task. |
| Destination policy before socket use | Unspecified, broadcast, multicast, and v4/v6 link-local destinations rejected with `Policy/PacketExchange`; `Strict` target policy enforced via shared resolution; Eggress routes yield typed `Unsupported` with no direct fallback. |
| Payload redaction | Echo test serializes the full report and asserts the payload text is absent; `response_sample` carries only reply bytes. |
| Thin CLI and schema fixtures | `eggprobe udp <target> --port <p> [--payload <text>] [--receive] [--json]`; `--port` required; CLI tests cover machine evidence, outcome value, stderr cleanliness, and usage error. No domain type changed, so schema 0.4 artifacts regenerate byte-identical. |

## Platform qualification

| Target | Evidence and disposition |
|---|---|
| macOS hosted runner (local) | Full workspace verification family passed (see below); loopback echo, discard/timeout, and closed-port fixtures passed. |
| Linux x86_64 | Hosted CI matrix covers format/clippy/tests; loopback UDP fixtures are OS-portable Tokio sockets with no privileged operations. |
| Windows x86_64 | Hosted CI matrix covers format/clippy/tests; connected-UDP unreachable delivery differs by platform and is represented by the `Unreachable`/`Timeout` contract rather than one brittle expectation. |
| Linux aarch64 / macOS aarch64 | Build-qualified via the existing target-check posture; no separate native UDP smoke claim is made. |

No new dependencies were added (`Cargo.lock` unchanged); the adapter uses existing Tokio socket primitives only.

## Verification

The exact repository verification sequence passed locally at `258e64d`:

```text
cargo fmt --all -- --check                         PASS
cargo check --workspace --all-targets --all-features --locked PASS
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings PASS
cargo test --workspace --all-features --locked     PASS (71 tests across 14 suites)
cargo +1.89.0 check --workspace --all-targets --locked PASS
cargo tree --locked                                PASS
cargo audit                                        PASS (one pre-existing allowed unmaintained `paste` advisory)
```

Schema regeneration (`cargo run -p eggprobe-core --example generate-schemas --locked`) produces no diff; the `expression` leak assertion in `tests/schema.rs` passed unchanged.

## Review and disposition

- Compatibility/schema: additive behavior only; no domain field added or altered; historical schemas untouched.
- Security/privacy: policy checks run before socket creation; error messages use `ErrorKind` display only; no credentials, payload bytes, or dependency strings reach reports. UDP never touches an Eggress route.
- Failure/cancellation/resources: one socket per probe, bounded buffers (1200 send, 2048 receive, 1400 retained sample), finite receive window, no leaked task.
- Documentation/operations: operator guide documents transmit/response/unreachable/timeout semantics, bounds, destination restrictions, and direct-only scope; `architecture/engine-transport.md` records the new §2b adapter section.
- Unresolved findings: none against M004 acceptance. ICMP echo (M003) remains blocked on published-backend adoption; traceroute (M005) remains blocked on a truthful backend; PMTU (M006) remains blocked on its seams and platform controls.

Roadmap disposition: M004 closed. M005 and M006 remain blocked by their registered backend/test-seam requirements; executing them now would violate their stop conditions, so implementation stops here per the sequential stop condition.
