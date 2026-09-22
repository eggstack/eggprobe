# Transport Corrective C002 — Routed, TLS, and Transport Qualification

Status: blocked

Planning baseline: `24965aa0ba2696b201e9c74531920877529821d5`; refresh after corrective C001 closes.

Source addendum:

- `plans/subsystems/transport-probe-engine-corrective-addendum.md#4-c002--routed-tls-and-transport-qualification`

Hard dependency: Transport corrective C001 closed.

Primary class: correctness + qualification

## 1. Objective

Replace closure-by-code-inspection with deterministic local qualification for standalone TLS, Eggress routes, and Eggfetch-over-Eggress HTTP, while restoring typed route failure provenance required by the original roadmap.

## 2. Eggress detailed error provenance

Re-check the current published Eggress 1.0.7+ public surface.

Where available, use the detailed connect API rather than `connect_tcp()` and retain:

- failure kind;
- stage;
- hop index;
- hop protocol;
- timeout/deadline distinction.

Map these into stable Eggprobe fields without discarding the source facts. Do not parse human error strings.

If the public API cannot carry a fact, mark it unavailable and document the exact upstream limitation.

## 3. Deterministic proxy fixtures

Add local in-process or sibling-testkit fixtures for:

- SOCKS5 success;
- SOCKS5 authentication failure;
- HTTP CONNECT success;
- HTTP CONNECT rejection;
- two-hop route success;
- failure at first and later hop;
- routed TCP;
- routed TLS;
- routed HTTP/1.1;
- routed HTTP/2 where the local fixture stack supports it.

The fixtures must prove no local proxy listener is introduced by Eggprobe itself and no direct fallback occurs after route failure.

## 4. Standalone TLS fixture matrix

Use a deterministic local CA/server setup to cover:

- trusted hostname success;
- hostname mismatch;
- untrusted issuer;
- negotiated TLS version/cipher;
- ALPN present/absent;
- handshake deadline/cancellation.

Add bounded certificate summary only if the current design intentionally supports it; otherwise document it as unavailable rather than claiming it.

## 5. Routed H3 semantics

Because the current Eggfetch custom dialer is byte-stream based, routed H3/QUIC must be explicitly unsupported.

Required:

- a request/plan surface capable of distinguishing requested H3 from ordinary H1/H2, if such protocol selection is exposed;
- structured `unsupported` result before any silent downgrade;
- regression test proving the route is not retried as H1/H2 when H3 was explicitly required.

If the current public CLI/plan does not expose protocol selection, document that H3 is unavailable rather than claiming a structured unsupported path already exists.

## 6. DNS semantics

For remote-resolving proxy routes, never report a local DNS answer as the resolver result used by the route. Local client DNS probe results and routed connection semantics remain separately labeled.

## 7. Required tests

All fixture cases above plus:

- credential redaction across route errors;
- detailed hop provenance;
- route deadline/cancellation;
- no direct fallback;
- no listener allocation;
- Eggfetch origin TLS SNI/hostname verification preserved over Eggress;
- H1/H2 direct/routed report schema alignment.

## 8. Acceptance criteria

- original Transport M002/M004/M005 exit conditions are demonstrated, not inferred;
- detailed Eggress provenance is retained wherever exposed;
- TLS verification negatives are locally reproducible;
- multi-hop route behavior is tested;
- unsupported H3 semantics are truthful;
- no medium-or-higher transport qualification finding remains.

## 9. Stop conditions

Stop if required qualification depends on public internet, if Eggress detailed provenance is absent from the published API, or if adding test fixtures would require copying proxy protocol implementations into Eggprobe. Prefer sibling testkit reuse or a narrow upstream test seam.

## 10. Closure evidence

Create `plans/closure/transport-probe-engine-corrective/002-status.md` with topology diagrams, fixture matrix, typed error mapping, TLS cases, H3 disposition, DNS semantics, and full verification results.
