# Transport and Probe Engine M004 — Eggress Listener-Free Route Core

Status: blocked

Planning baseline: `e55a21fe73915b7a38c2fa674807354539806c60`; refresh after Transport M001 closes.

Source roadmap:

- `plans/subsystems/transport-probe-engine-roadmap.md#m004--eggress-listener-free-route-core`

Hard dependency: Transport M001 closed.
Interface dependency: current published Eggress listener-free outbound APIs.

Primary class: capability + infrastructure

## 1. Objective

Add Eggress-backed route execution for stream probes without starting a proxy listener, while making Eggress the canonical parser/redactor for route syntax and retaining typed hop failure provenance.

## 2. Dependency posture

Re-check the current crates.io Eggress release at implementation time; researched baseline is 1.0.7.

Prefer narrow dependencies on:

- `eggress-outbound`;
- `eggress-uri` only where needed;
- `eggress-core` types only when public adapter types require them.

Feature-gate extended protocols. Do not enable legacy/insecure features by default.

## 3. Required production changes

- replace foundation opaque route handling at execution time with Eggress canonical parse/validation;
- preserve raw credentials only in input/config ownership; use Eggress redacted representation for display/report;
- implement `EggressRouteDialer` over `OutboundConnector::connect_tcp_detailed` or current equivalent;
- map `OutboundInfo` local/peer/hop count;
- preserve `OutboundConnectErrorKind`, stage, hop index, protocol in Eggprobe's structured source details;
- define destination-resolution semantics: local client DNS evidence is not claimed as remote proxy DNS;
- no automatic direct fallback after proxy failure.

## 4. Work packages

A. Qualify Eggress package/features and URI grammar.
B. Replace temporary route-summary logic with canonical Eggress redaction.
C. Implement route/dial adapter.
D. Normalize errors while retaining hop provenance.
E. Add SOCKS5, HTTP CONNECT, auth failure, and multi-hop local fixtures.
F. Add secret-leak/property tests for arbitrary supported chain syntax.

## 5. Tests

- SOCKS5 success/failure;
- HTTP CONNECT success/failure;
- authentication rejection;
- malformed route;
- multi-hop success and failure at each hop;
- no direct fallback;
- remote-DNS mode does not fabricate local route answers;
- credentials never appear in Debug/Display/report/logs;
- cancellation/deadline;
- no local listener/socket opened except intended outbound connections.

## 6. Acceptance criteria

Eggress owns proxy syntax and hop protocols. Eggprobe owns only orchestration/normalization. Multi-hop errors retain stable provenance. Route output is credential-safe for all supported Eggress syntax.

## 7. Stop conditions

Stop if the published Eggress interface requires a listener, lacks typed detailed errors needed for truthful reports, or requires insecure/legacy defaults. Propose the narrow upstream seam rather than forking protocol code.

## 8. Closure evidence

Create `plans/closure/transport-probe-engine/004-status.md` with Eggress version/features, chain fixtures, redaction corpus, failure provenance matrix, DNS semantics, no-fallback/no-listener evidence, MSRV/audit results.
