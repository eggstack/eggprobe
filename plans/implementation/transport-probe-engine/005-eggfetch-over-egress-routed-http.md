# Transport and Probe Engine M005 — Eggfetch over Eggress for Routed HTTP(S)

Status: blocked

Planning baseline: `e55a21fe73915b7a38c2fa674807354539806c60`; refresh after dependencies close.

Source roadmap:

- `plans/subsystems/transport-probe-engine-roadmap.md#m005--eggfetch-over-egress-for-routed-https`

Hard dependencies:

- Transport M003 Eggfetch HTTP closed.
- Transport M004 Eggress route core closed.

Primary class: capability

## 1. Objective

Compose Eggress's listener-free byte stream with Eggfetch's public custom-dialer seam so HTTP/1.1 and HTTP/2 can be diagnosed through proxy chains in-process.

## 2. Ownership contract

- Eggress establishes the physical route and owns hop protocol semantics.
- Eggfetch owns HTTP framing, destination/origin TLS, SNI, hostname verification, response processing, and connection semantics above the dialed stream.
- Eggprobe owns adapter/error normalization/report orchestration.

Do not move origin TLS into Eggress merely to simplify the adapter.

## 3. Required production changes

Implement an Eggfetch `Dialer` backed by the established Eggress route connector. Map target host/port without performing an unintended local DNS lookup when the route expects remote resolution.

Preserve Eggress failure provenance through the Eggfetch adapter when possible; if Eggfetch's error carrier cannot retain typed source data, maintain a side-channel/adapter error object within Eggprobe rather than parsing display text.

Explicitly reject routed HTTP/3/QUIC because the dialer is a byte-stream contract.

## 4. Work packages

A. Requalify current Eggfetch Dialer and Eggress BoxStream compatibility.
B. Implement zero-listener adapter.
C. Integrate into HTTP client builder without enabling Eggfetch proxy features.
D. Validate TLS/SNI/Host identity across proxy routes.
E. Add routed H1/H2 fixtures and negative H3 case.
F. Reconcile report route/DNS/error evidence.

## 5. Tests

- HTTP and HTTPS through SOCKS5;
- HTTP and HTTPS through CONNECT;
- at least one multi-hop chain;
- origin TLS hostname verification and SNI preservation;
- Host header/logical origin preserved;
- proxy authentication failure retains hop provenance;
- remote destination DNS is not locally fabricated;
- routed H3 returns `unsupported`, never downgrade;
- cancellation/deadline/no direct fallback;
- no listener allocation.

## 6. Acceptance criteria

Routed H1/H2 uses one in-process stream adapter; no duplicate HTTP or proxy implementation exists. Direct and routed reports share the same schema while preserving route-specific evidence. Unsupported H3 is explicit.

## 7. Stop conditions

Stop if either published public API no longer supports direct stream composition, if origin TLS identity cannot be preserved, or if error provenance can only be recovered by parsing human strings.

## 8. Closure evidence

Create `plans/closure/transport-probe-engine/005-status.md` with adapter ownership diagram, package versions/features, H1/H2/TLS/multi-hop fixture evidence, DNS semantics, H3 negative case, and no-listener/no-fallback proof.
