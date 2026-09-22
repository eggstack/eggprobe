# Transport and Probe Engine Roadmap

Status: blocked pending foundation corrective C001

Long-term references:

- `plans/000-long-term-specification.md#7-route-model`
- `plans/000-long-term-specification.md#8-probe-families`
- `plans/000-long-term-specification.md#11-security-model`
- `plans/000-long-term-specification.md#12-timeout-repetition-and-concurrency-model`
- `plans/002-long-term-roadmap.md#phase-1--direct-route-dns-and-tcp-diagnostics`
- `plans/002-long-term-roadmap.md#phase-2--tls-diagnostics-and-eggfetch-http-integration`
- `plans/002-long-term-roadmap.md#phase-3--eggress-listener-free-route-integration`
- `plans/002-long-term-roadmap.md#phase-6--diagnostic-observability-refinement`

Related ADRs:

- `plans/adrs/ADR-0001-json-first-core-and-transport-ownership.md`

## 1. Purpose and ownership boundary

This subsystem owns network execution beneath `ProbePlan -> ProbeReport`.

Eggprobe owns probe-specific DNS/direct TCP/standalone TLS operations and orchestration. Eggfetch remains the HTTP owner. Eggress remains the proxy-chain owner.

The subsystem must provide one route vocabulary and normalize evidence without erasing source-specific facts.

## 2. Work classification

### Invariants

- HTTP uses Eggfetch.
- Proxy chains use Eggress.
- No listener process is required for Eggfetch-over-Eggress composition.
- Remote proxy DNS is not fabricated from local DNS.
- Unsupported routed H3 fails explicitly.
- Retries are opt-in and visible as attempts.
- Every network probe has a finite outer deadline.
- Cancellation releases resources.

### Capabilities

- DNS;
- TCP;
- TLS;
- HTTP/1.1 and HTTP/2;
- direct and Eggress routes;
- multi-hop proxy diagnosis;
- later direct H3 qualification.

### Infrastructure

- route/dial abstraction;
- error normalization;
- timeout/deadline utilities;
- local fixtures;
- Eggfetch custom-dialer adapter;
- Eggress outbound adapter;
- probe event/timing collectors.

### Polish

- richer certificate analysis;
- per-hop success timing;
- precise per-request DNS/TCP/TLS phase timing;
- latency summaries.

## 3. Non-goals

- no inbound proxy server;
- no MITM;
- no port scanning;
- no packet capture;
- no shell-command probe backend;
- no routed H3 through a TCP-only seam;
- no general-purpose replacement for Eggfetch/Eggress.

## 4. Current state

Planning inspection found:

### Eggfetch

Current 0.2.0 core exposes:

- `Client` / `ClientBuilder`;
- advanced-routing `Dialer`, `DialTarget`, `DialStream`, `DialError`;
- `ConnectionMetadata` with local/peer address, transport kind, TLS version/cipher/ALPN/SNI;
- phase-aware timeout/error types;
- transport metrics;
- HTTP/1.1, HTTP/2, optional HTTP/3.

The public `DialStream` is an erased Tokio `AsyncRead + AsyncWrite + Send + Unpin` stream. Eggfetch retains HTTP framing and origin TLS above it.

Current trace documentation states connector-level DNS/connect/TLS events are not uniformly emitted per request; aggregate metrics cover those connector events instead.

### Eggress

Current 1.0.7 `eggress-outbound` exposes:

- `OutboundConnector::direct()`;
- `OutboundConnector::from_chain(...)`;
- optional `from_pproxy_uri(...)`;
- `connect_tcp_detailed(...)`;
- `connect_tcp_timeout_detailed(...)`;
- `OutboundInfo` local/peer/hop-count metadata;
- `OutboundConnectErrorKind`;
- `OutboundConnectStage`;
- hop index/protocol diagnostics.

`eggress_core::BoxStream` is also `AsyncRead + AsyncWrite + Send + Unpin`, making direct in-process adaptation to Eggfetch's dialer feasible.

### Evidence gaps

- Eggfetch lacks complete per-request DNS/TCP/TLS timing events across all transports.
- Eggress does not yet expose equivalent successful per-hop timing events.
- Routed QUIC/H3 needs a datagram transport contract, not the byte-stream adapter.

## 5. Target architecture

```text
ProbeEngine
   |
   +--> DnsProbe
   |
   +--> RouteDialer
   |      +--> DirectRoute
   |      `--> EggressRoute -> OutboundConnector
   |
   +--> TlsProbe(RouteDialer)
   |
   `--> HttpProbe
           |
           `--> eggfetch Client
                  |
                  +--> direct/native route where appropriate
                  `--> custom EggressDialer for routed H1/H2
```

The adapter may retain Eggress's typed source error inside Eggfetch's custom transport error when possible, then normalize it into Eggprobe's report taxonomy at the outer boundary.

## 6. Dependency graph

```text
foundation M001 historical closure
    |
    v
foundation corrective C001
    |
    v
M001 direct route + DNS/TCP
    |
    +--> M002 standalone TLS
    |
    +--> M004 Eggress route core
    |
    v
M003 Eggfetch HTTP
    |
    +--------+
             v
M005 HTTP over Eggress H1/H2
             |
             v
M006 observability refinement
```

- M001 hard-depends on foundation corrective C001; historical foundation M001 closure alone is no longer sufficient.
- M002 hard-depends on transport M001.
- M003 hard-depends on M001; M002 is a soft dependency because HTTP-associated TLS remains Eggfetch-owned.
- M004 hard-depends on M001 and has an interface dependency on Eggress 1.0.7 public APIs.
- M005 hard-depends on M003 and M004.
- M006 is an interface/operational upstream enhancement after working consumer paths exist.

## 7. Milestones

### M001 — Direct route, DNS, and TCP primitives

Class: capability + infrastructure.

Objective:

Implement direct route policy plus deterministic DNS/TCP diagnostics.

Exit conditions:

- system-resolver A/AAAA diagnostics;
- IPv4/IPv6 TCP attempts;
- selected/local address when observable;
- finite deadline/cancellation;
- explicit private/local target policy;
- stable errors;
- local fixtures.

### M002 — Standalone TLS diagnostic probe

Class: capability.

Objective:

Negotiate TLS over the route abstraction independently of HTTP.

Exit conditions:

- verification on by default;
- TLS version/cipher/ALPN/SNI facts;
- bounded peer cert summary where feasible;
- local trusted/untrusted/name-mismatch fixtures;
- no secret/session key exposure.

### M003 — Eggfetch-backed HTTP probe

Class: capability.

Objective:

Expose HTTP diagnostics without another HTTP implementation.

Exit conditions:

- Eggfetch dependency uses the narrow required feature set;
- H1/H2 qualified;
- redirects explicit/observable;
- bounded body policy;
- status and response-header timing;
- typed Eggfetch errors normalized;
- local fixtures.

### M004 — Eggress listener-free route core

Class: capability + infrastructure.

Objective:

Use Eggress chains for TCP/TLS route establishment.

Exit conditions:

- native chain parsing/validation path selected;
- typed failure kind/stage/hop/protocol retained;
- credentials redacted;
- SOCKS5 and HTTP CONNECT fixtures;
- multi-hop fixture;
- no local listener.

### M005 — Eggfetch over Eggress for routed HTTP(S)

Class: capability.

Objective:

Adapt Eggress streams to Eggfetch's custom dialer for H1/H2.

Exit conditions:

- origin TLS remains Eggfetch-owned;
- proxy chain remains Eggress-owned;
- direct/routed report contracts align;
- remote-DNS semantics truthful;
- H1/H2 route fixtures pass;
- routed H3 returns structured unsupported.

### M006 — Upstream diagnostic observability refinement

Class: polish + infrastructure.

Objective:

Obtain precise phase/per-hop evidence through upstream public observers where valuable.

Candidate work:

- Eggfetch per-request DNS/TCP/TLS observer;
- Eggress successful hop connect/handshake observer;
- peer-certificate summary seam.

This milestone MUST NOT block a truthful initial release if coarse/unavailable evidence is clearly represented.

## 8. Cross-cutting requirements

### Storage and migration

None.

### Protocol and compatibility

Normalize source errors without discarding source metadata. New source enum values must map safely to `other`/unknown-compatible representations until explicitly handled.

### Security and authorization

TLS verification defaults on. Route credentials never enter report/log output. Local/private target policy is explicit. Environment proxy use is not implicit.

### Concurrency, cancellation, and recovery

All futures must be cancellation-safe by drop. Batch concurrency is owned elsewhere but probe primitives must not leak background tasks.

### Observability and audit

Named timings require truthful boundaries. Unknown/unavailable evidence is first-class.

### Performance and resource use

No eager body buffering by default. Bounds on DNS answers, certificate data, headers, body samples, attempts, and hops.

### Documentation and operations

Every route/probe documents what it can and cannot observe.

## 9. Verification strategy

Use local deterministic servers/proxies/resolvers wherever practical:

- loopback TCP success/refusal/timeout;
- local DNS fixture where explicit resolver support lands;
- local Rustls CA/server fixtures;
- local H1/H2 servers;
- local SOCKS5/CONNECT proxies;
- multi-hop chain fixture;
- cancellation and deadline tests;
- redaction property tests.

Live public internet endpoints are exploratory only.

## 10. Risks and decision points

- Standalone TLS certificate parsing crate choice should remain small and auditable.
- System DNS APIs may not expose the actual configured resolver server portably; report mode rather than inventing an address.
- Eggress DNS-rebinding policy differs from a local diagnostic CLI's needs; routed behavior must respect Eggress semantics while direct diagnostic policy remains explicit.
- HTTP/3 requires separate qualification.
- Precise phase timing depends on upstream observer surfaces.

A new ADR is required before inventing a generic datagram routing abstraction that changes cross-crate ownership.

## 11. Completion definition

The subsystem closes when direct and Eggress-routed DNS/TCP/TLS/HTTP diagnostics are truthful, bounded, cancellable, locally testable, and represented through one report contract without duplicated HTTP/proxy implementations.

M006 may remain deferred if its unavailable evidence is explicitly represented and does not block useful diagnosis.

## 12. Milestone status

| Milestone | Status | Implementation plan | Closure record | Blockers |
|---|---|---|---|---|
| M001 direct route, DNS, and TCP primitives | blocked | `plans/implementation/transport-probe-engine/001-direct-route-dns-and-tcp-primitives.md` | pending | hard: foundation corrective C001 |
| M002 standalone TLS diagnostic probe | blocked | `plans/implementation/transport-probe-engine/002-standalone-tls-diagnostic-probe.md` | pending | hard: M001 |
| M003 Eggfetch-backed HTTP probe | blocked | `plans/implementation/transport-probe-engine/003-eggfetch-backed-http-probe.md` | pending | hard: M001 |
| M004 Eggress listener-free route core | blocked | `plans/implementation/transport-probe-engine/004-eggress-listener-free-route-core.md` | pending | hard: M001 |
| M005 Eggfetch over Eggress for routed HTTP(S) | blocked | `plans/implementation/transport-probe-engine/005-eggfetch-over-egress-routed-http.md` | pending | hard: M003 + M004 |
| M006 upstream diagnostic observability refinement | blocked | `plans/implementation/transport-probe-engine/006-upstream-diagnostic-observability-refinement.md` | pending | interface evidence: consumer paths M003-M005 |
