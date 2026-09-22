# Transport and Probe Engine M003 — Eggfetch-Backed HTTP Probe

Status: blocked

Planning baseline: `e55a21fe73915b7a38c2fa674807354539806c60`; refresh after Transport M001 closes.

Source roadmap:

- `plans/subsystems/transport-probe-engine-roadmap.md#m003--eggfetch-backed-http-probe`

Hard dependency: Transport M001 closed.
Soft dependency: Transport M002 for shared terminology only; HTTP-associated TLS remains Eggfetch-owned.

Primary class: capability

## 1. Objective

Add HTTP diagnostics through published `eggfetch-core` APIs without duplicating HTTP framing, pooling, redirects, or origin TLS.

## 2. Dependency posture

At implementation time, re-check the current crates.io Eggfetch release and its feature graph. The researched baseline is 0.2.0.

Enable only required features, expected to include HTTP/1.1, HTTP/2, Rustls/WebPKI policy, high-level URL/request support as needed, and relevant metadata. Do not enable proxy, compression, HTTP/3, multipart, environment proxy, or retries incidentally.

## 3. Required behavior

- HTTP GET/HEAD or explicitly allowed safe methods;
- redirects off by default;
- explicit redirect following with every hop observable;
- response status/version and safe selected headers;
- response-header/TTFB timing only at the boundary actually observable;
- bounded optional body sample; no full body by default;
- connection/TLS metadata mapped from Eggfetch without claiming unavailable DNS/TCP/TLS phase timings;
- finite request deadline;
- Eggfetch retry policy disabled unless a later explicit plan owns retries;
- typed Eggfetch failures normalized to Eggprobe errors while retaining useful source detail safely.

HTTP status 4xx/5xx is an observed HTTP result, not a transport failure.

## 4. Work packages

A. Qualify crates.io Eggfetch version/features/MSRV/security.
B. Add a narrow Eggfetch adapter in `eggprobe-core`.
C. Implement HTTP plan validation and request construction.
D. Map connection/TLS/response metadata into typed evidence.
E. Add redirect/body bounds.
F. Add local H1/H2/TLS fixture servers and docs.

## 5. Tests

- H1 and H2 success;
- 4xx/5xx successful observation semantics;
- redirects disabled and explicitly followed;
- redirect loop/max-hop;
- header/body bounds including chunked body;
- TLS verification through Eggfetch;
- timeout/cancellation;
- no Eggfetch retry owner;
- no environment proxy;
- no compression unless explicitly selected later;
- no live internet dependency.

## 6. Acceptance criteria

Every HTTP request flows through Eggfetch. No direct Hyper HTTP implementation exists in Eggprobe. Report timings are truthful. Body and redirect behavior are bounded. Dependency feature tree is recorded.

## 7. Stop conditions

Stop if the current Eggfetch release lacks required public APIs, requires Git/path override, regresses Rust 1.89, or forces unrelated optional features. Record an upstream interface blocker rather than copying Eggfetch internals.

## 8. Closure evidence

Create `plans/closure/transport-probe-engine/003-status.md` with exact Eggfetch version/features, local fixture matrix, error mapping, redirects/body policy, timing limitations, audit/MSRV results, and source-ownership review.
