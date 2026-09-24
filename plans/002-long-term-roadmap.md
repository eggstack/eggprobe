# Eggprobe Long-Term Implementation Roadmap

Status: execution roadmap for `plans/000-long-term-specification.md`

Terminology: `plans/001-terminology-and-domain-model.md`

This roadmap is dependency-ordered, not calendar-ordered. Each phase MUST leave the repository coherent and MUST include bounded implementation plans, tests, documentation, and closure evidence before dependent phases are treated as available.

## Cross-phase execution rules

Every phase MUST:

1. preserve the JSON-first core and thin-CLI boundary;
2. retain explicit ownership between Eggprobe, Eggfetch, and Eggress;
3. serialize only truthful observations and explicit unavailable/unsupported states;
4. keep stdout machine-clean in JSON/NDJSON modes;
5. bound deadlines, concurrency, strings, and captured payloads;
6. preserve TLS verification and secret redaction by default;
7. use local deterministic fixtures for routine qualification;
8. avoid implicit retries and hidden route fallbacks;
9. keep platform limitations explicit;
10. record closure evidence before advancing hard dependencies.

## Phase 0 — Repository foundation and diagnostic contract

### Objective

Create the Rust workspace and stable internal boundaries before networking capability expands.

### Deliverables

- Rust 1.89 workspace.
- `eggprobe-core` and `eggprobe-cli` crates.
- Canonical `ProbePlan`, `ProbeReport`, target, route, status, observation, timing, finding, and diagnostic-error types.
- Serde JSON round trips.
- Schema/version fields from the first executable build.
- Renderer boundary that cannot initiate network I/O.
- Baseline CI, formatting, Clippy, test, audit, and dependency-policy hooks.
- Initial dependency decisions for Eggfetch, Eggress, Tokio, Serde, Clap, and DNS/TLS support.

### Dependencies

None.

### Exit criteria

- A no-network example plan can be parsed and serialized through the same core types the CLI uses.
- JSON output is deterministic enough for fixtures.
- The workspace compiles on the declared MSRV.
- No networking implementation lives in the CLI crate.

## Phase 1 — Direct route, DNS, and TCP diagnostics

### Objective

Establish the route abstraction and the smallest independent probes.

### Deliverables

- Direct route abstraction with explicit local/private target policy.
- System-resolver DNS probe with A/AAAA evidence and elapsed time.
- TCP probe with IPv4/IPv6 address attempts, selected peer/local address, deadlines, cancellation, and stable error mapping.
- Bounded repetition and aggregate statistics without hiding attempts.
- Local loopback fixtures for success/refusal/timeout/cancellation.

### Dependencies

Phase 0.

### Exit criteria

- `eggprobe dns` and `eggprobe tcp` operate entirely through `eggprobe-core`.
- Machine reports distinguish client DNS from route DNS.
- Private/loopback targets are usable by the CLI under explicit diagnostics policy.

## Phase 2 — TLS diagnostics and Eggfetch HTTP integration

### Objective

Add application-layer diagnosis without duplicating an HTTP client.

### Deliverables

- Explicit TLS probe over the route abstraction using Rustls-compatible policy.
- Negotiated version/cipher/ALPN/SNI evidence.
- Bounded peer-certificate summary when available.
- Eggfetch-backed HTTP probe.
- Redirects disabled by default; observable when enabled.
- HTTP/1.1 and HTTP/2 qualification.
- Accurate response-header/TTFB timing at the boundary Eggfetch exposes.
- Response bodies disabled by default and bounded when requested.

### Dependencies

Phases 0 and 1.

### Exit criteria

- HTTP requests use Eggfetch public APIs.
- TLS and HTTP failures map into Eggprobe structured errors without string parsing.
- Direct local fixture servers prove H1/H2/TLS behavior.
- No second HTTP framing implementation exists.

## Phase 3 — Eggress listener-free route integration

### Objective

Run TCP/TLS/HTTP probes through real Eggress chains in-process.

### Deliverables

- `EggressRoute` backed by `eggress-outbound::OutboundConnector`.
- Adapter compatible with Eggfetch's custom `Dialer` byte-stream contract.
- Direct use of Eggress typed error kind/stage/hop/protocol metadata.
- Credential-redacted route representation.
- Local SOCKS5/HTTP CONNECT chain fixtures.
- Multi-hop route qualification.
- Explicit remote-DNS/unavailable-DNS semantics.
- Structured unsupported response for route/protocol combinations that cannot be represented faithfully.

### Dependencies

Phase 1. HTTP-over-route validation also depends on Phase 2.

### Exit criteria

- No loopback proxy listener is started merely to connect Eggfetch and Eggress.
- Direct and Eggress routes produce the same top-level report vocabulary.
- Proxy failures identify stable kind and stage.
- H1/H2 over Eggress are qualified.
- Routed H3 is explicitly unsupported rather than silently downgraded.

## Phase 4 — Composite diagnosis, assertions, and comparison

### Objective

Turn primitives into useful diagnostic workflows while retaining raw evidence.

### Deliverables

- `check` orchestration.
- Explicit assertions for status, protocol, TLS properties, and timing thresholds.
- Direct-versus-route comparison.
- Repeat-count support with distributions/summary statistics.
- Clear separation between probe status and assertion findings.
- Coarse stable CLI exit-code mapping.

### Dependencies

Phases 1–3.

### Exit criteria

- Composite workflows preserve every child probe result.
- An observed HTTP error status and an assertion failure remain distinguishable.
- Comparison never reuses or mutates one route's observations as the other's.

## Phase 5 — Plan files, NDJSON, and batch automation

### Objective

Make Eggprobe a reliable automation component rather than only an interactive CLI.

### Deliverables

- Versioned JSON plan input from file/stdin.
- JSON Schema generated from canonical Rust types.
- NDJSON event/result stream for repeated or batch execution.
- Bounded batch concurrency.
- Stable stdin/stdout behavior suitable for `jq`, CI, agents, and subprocess use.
- Golden schema/report compatibility fixtures.

### Dependencies

Phases 0 and 4.

### Exit criteria

- A saved plan can reproduce the same execution semantics as equivalent CLI arguments.
- stdout is parseable with no diagnostic prose contamination.
- Unknown future optional fields and non-exhaustive enum handling are documented.

## Phase 6 — Diagnostic observability refinement

### Objective

Close evidence gaps that require improved upstream hooks rather than inference.

### Deliverables

Subject to upstream acceptance and published interfaces:

- per-request Eggfetch DNS/TCP/TLS diagnostic observer events;
- successful Eggress per-hop connect/handshake events;
- richer Eggfetch peer-certificate summaries;
- precise phase timing integration into Eggprobe reports.

### Dependencies

Phases 2 and 3 provide the consumer contract first.

### Exit criteria

- Phase timings are reported only from real observer events.
- Coarse/fallback fields remain backward-compatible.
- Eggprobe does not fork upstream network stacks to obtain diagnostics.

## Phase 7 — Release, packaging, and operational qualification

### Objective

Ship a small cross-platform binary with repeatable qualification evidence.

### Deliverables

- prebuilt Linux x86_64/aarch64, macOS x86_64/arm64, and Windows x86_64 artifacts where CI supports them;
- checksums and release provenance;
- shell completions;
- release smoke tests;
- MSRV and supported-target CI;
- dependency/security audit;
- operator documentation;
- optional later producer integration with Eggpack once the required producer interfaces are stable;
- optional later runtime self-update through Eggup consumer deployment machinery, using Eggpack release evidence where applicable.

### Dependencies

Phases 0–5. Phase 6 is not required for an initial release if evidence limitations are documented truthfully.

### Exit criteria

- a clean machine can run the published binary without a Rust toolchain;
- machine-readable compatibility fixtures pass against the release build;
- supported platform limitations are explicit;
- lack of optional Eggpack producer adoption or Eggup runtime self-update does not block an otherwise qualified standalone archive release;
- any adopted shared release path follows ADR-0002 ownership: Eggpack for producer construction/evidence, Eggup for consumer deployment/rollback, Eggprobe for product policy.

## Phase 8 — Native path and host diagnostics

### Objective

Add high-value non-HTTP diagnostics after the core contract is stable.

### Candidate deliverables

- ICMP echo with native platform backends;
- traceroute/path tracing without terminal-output scraping;
- route-table and interface inspection;
- MTU/path-MTU probes;
- UDP service checks.

### Dependencies

Stable schema and release discipline from Phases 5–7.

### Exit criteria

Each added primitive has a typed cross-platform contract and explicit platform-specific unsupported states.

## Phase 9 — Direct and routed QUIC/HTTP/3 expansion

### Objective

Add HTTP/3/path evidence only when transport composition is truthful.

### Deliverables

- qualify direct H3 through Eggfetch;
- define a datagram/QUIC route abstraction if Eggfetch and Eggress expose compatible public seams;
- preserve QUIC-specific metrics without pretending stream metadata applies;
- qualify routed H3 only after real UDP/datagram proxy behavior exists.

### Dependencies

Upstream public interfaces and Phase 6 evidence.

### Exit criteria

No H3 route is reported as supported through a TCP-only dialer.

## Program completion

The roadmap is substantially complete when Phases 0–7 are closed with evidence. Phases 8–9 are capability expansion and MUST NOT block a useful first major Eggprobe release.
