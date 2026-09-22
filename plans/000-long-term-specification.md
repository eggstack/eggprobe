# Eggprobe Long-Term Architecture and Product Specification

Status: canonical long-term implementation directive

Companion documents:

- `plans/001-terminology-and-domain-model.md`
- `plans/002-long-term-roadmap.md`
- `plans/003-planning-process.md`
- `plans/adrs/ADR-0001-json-first-core-and-transport-ownership.md`

This document defines the intended end state for Eggprobe. It establishes product scope, ownership boundaries, machine-readable contracts, probe semantics, security properties, interoperability requirements, and closure criteria. The keywords MUST, MUST NOT, REQUIRED, SHOULD, SHOULD NOT, and MAY are normative.

## 1. Product definition

Eggprobe is a Rust-native network diagnostics engine and CLI for repeatable, evidence-oriented diagnosis of DNS, connection establishment, TLS, HTTP, and proxy/path behavior.

The machine-readable core is the product boundary. The CLI is a thin adapter over the same typed request and report model used by tests, future library consumers, automation, agents, and CI.

Eggprobe is intended to answer questions such as:

- Can this target be resolved and reached?
- Which address and route were actually used?
- Where did connection setup time go?
- What TLS and HTTP properties were negotiated?
- Does behavior differ between direct and proxied paths?
- Which Eggress hop failed, at which stage, and with what stable failure category?
- Did an explicit diagnostic expectation fail even though the network operation itself completed?
- Can another program consume the complete result without scraping terminal text?

## 2. Primary product goals

Eggprobe MUST provide:

1. A versioned JSON-first diagnostic request and report contract.
2. A CLI that is useful interactively without becoming the owner of diagnostic semantics.
3. Standalone DNS, TCP, TLS, and HTTP diagnostics.
4. Composite checks that preserve the evidence from their constituent probes.
5. Direct and Eggress-routed TCP/HTTP(S) execution without spawning a loopback proxy service.
6. First-class Eggfetch integration for HTTP behavior and Eggress integration for outbound proxy chains.
7. Stable, typed failure classification without requiring consumers to parse display strings.
8. Explicit timing semantics and a distinction between observed evidence, derived findings, and unavailable information.
9. Deterministic secret redaction and bounded machine-readable output.
10. Repeatable plan execution for CI, agents, scripts, and batch diagnostics.
11. Linux, macOS, and Windows support wherever the underlying probe is portable.
12. Small, auditable Rust ownership boundaries rather than duplicated general-purpose networking stacks.

## 3. Non-goals

Eggprobe is not initially:

- a vulnerability scanner or port scanner;
- a packet capture or full protocol analyzer;
- a replacement for Wireshark, tcpdump, nmap, or a browser developer-tools stack;
- a persistent monitoring daemon;
- a general load-testing system;
- a traffic interception/MITM proxy;
- a replacement for Eggfetch or Eggress;
- a DNS server or recursive resolver;
- a generic proxy service;
- a topology discovery crawler;
- a shell wrapper around `curl`, `dig`, `ping`, or `traceroute`.

Those systems MAY be complemented by Eggprobe, but Eggprobe MUST NOT absorb their complete product scope.

## 4. Architectural principles

### 4.1 JSON first, CLI second

The canonical execution boundary MUST be a typed Rust input model producing a typed Rust report model. JSON serialization is a first-class representation of those types.

Human-oriented terminal output MUST be rendered from the report. It MUST NOT contain facts that are unavailable in the machine-readable representation.

### 4.2 One diagnostic engine

The CLI, tests, future bindings, and future service adapters MUST use the same `eggprobe-core` execution path. The CLI MUST NOT maintain a second implementation of routing, timing, errors, assertions, or result interpretation.

### 4.3 Explicit transport ownership

Eggprobe owns diagnostic orchestration and probe-specific direct primitives.

Eggfetch owns HTTP request semantics, HTTP protocol negotiation, origin TLS performed as part of HTTP, redirects when explicitly enabled, body handling, and its associated transport policy.

Eggress owns proxy-chain parsing/execution and proxy-hop protocol semantics.

Eggprobe MUST compose those libraries at supported public interfaces instead of copying their general-purpose HTTP, SOCKS, CONNECT, relay, or proxy implementations.

### 4.4 Evidence before inference

Reports MUST distinguish:

- facts directly observed by Eggprobe or a dependency;
- facts derived deterministically from observations;
- facts that are unavailable on the selected route.

Eggprobe MUST NOT fabricate remote-DNS answers, per-hop timings, certificate facts, HTTP/3 route facts, or socket metadata when the underlying layer cannot observe them.

### 4.5 Diagnostic operations are not transparent retries

Retries MUST be opt-in and represented as distinct attempts. A diagnostic failure MUST NOT disappear behind an implicit retry policy.

Redirect following MUST likewise be explicit and the redirect chain MUST remain observable.

### 4.6 Local diagnostics are allowed by policy, not accident

A diagnostics tool legitimately targets loopback, LAN, private, link-local, and lab networks. Direct-probe policy MUST therefore be explicit and must not accidentally inherit an SSRF-oriented default that makes local diagnosis impossible.

Embedding consumers MUST be able to select a stricter target policy.

### 4.7 Stable machine contracts, flexible presentation

Field names, enums, schema versions, error kinds, and presence/absence semantics are compatibility surfaces. Table layout, colors, prose, and terminal formatting are not.

### 4.8 Fail closed on unsupported routing semantics

If a requested transport combination cannot be implemented faithfully, Eggprobe MUST return a structured unsupported result rather than silently downgrade or choose another route.

This specifically applies to routed HTTP/3 until a real datagram/QUIC transport seam exists.

## 5. Canonical execution model

The target model is:

```text
ProbePlan
  |
  +--> Target
  +--> Route
  +--> Probe specification(s)
  +--> Timeouts / repetition / assertions
  |
  v
ProbeEngine
  |
  +--> DNS probe owner
  +--> route/dial owner
  |      +--> direct TCP
  |      `--> Eggress OutboundConnector
  +--> TLS probe owner
  +--> Eggfetch HTTP adapter
  |
  v
ProbeReport
  |
  +--> observations
  +--> timings
  +--> attempts
  +--> findings/assertions
  +--> errors/unsupported facts
  `--> provenance/version
          |
          +--> JSON / NDJSON
          `--> human CLI renderer
```

No renderer may initiate network operations.

## 6. Canonical report contract

Every top-level report MUST include:

- schema version;
- Eggprobe version;
- execution identifier;
- start/end or total-duration facts in a stable representation;
- normalized target;
- normalized route description with secrets removed;
- overall execution status;
- ordered probe results;
- explicit findings/assertion outcomes;
- warnings and unavailable/unsupported facts where applicable.

Durations SHOULD serialize as integer microseconds unless a later accepted ADR changes the public unit. Floating-point seconds MUST NOT be the canonical machine format.

Each probe result MUST identify:

- probe kind;
- attempt number when repeated;
- status;
- duration;
- observations;
- structured failure when present;
- whether the result is direct evidence or a derived conclusion.

## 7. Route model

Eggprobe MUST distinguish at least:

### Direct route

Eggprobe establishes the target connection directly.

The direct route MAY resolve through the operating system resolver or an explicitly selected DNS resolver depending on the probe.

### Eggress route

Eggprobe delegates stream establishment to `eggress-outbound::OutboundConnector`.

The route description MUST be credential-redacted. The report SHOULD preserve hop count and, where Eggress exposes it safely, hop protocol/stage/failure metadata.

Destination DNS semantics MUST remain truthful. If the proxy resolves the destination remotely, Eggprobe MUST NOT present a local lookup as the route's actual DNS answer.

### Future datagram route

UDP/QUIC routing is a distinct capability. It MUST NOT be modeled as if a byte-stream dialer were sufficient.

## 8. Probe families

### 8.1 DNS

The DNS probe owns explicit name-resolution diagnostics.

The initial implementation SHOULD support the system resolver path and A/AAAA records. Later work MAY add explicit resolvers, additional record types, transport choice, DNSSEC evidence, and encrypted DNS.

A DNS probe MUST record which resolver mode was requested. It MUST distinguish client-side resolution from remote proxy resolution.

### 8.2 TCP

The TCP probe establishes a connection and records target, selected peer address, local address when available, elapsed time, route metadata, and stable failure category.

It MUST support IPv4 and IPv6 targets where the platform does.

### 8.3 TLS

The TLS probe performs an explicit client handshake over an established route.

It SHOULD expose negotiated version, cipher, ALPN, SNI/server name, verification result, and a sanitized peer-certificate summary when safely available.

Private key material, session secrets, and unbounded certificate dumps MUST NOT enter ordinary reports.

### 8.4 HTTP

HTTP probes MUST use Eggfetch rather than a second HTTP implementation.

They MUST support at least HTTP/1.1 and HTTP/2. Direct HTTP/3 MAY be added when qualified against Eggfetch. HTTP/3 over an Eggress byte-stream route is unsupported until a real datagram route exists.

HTTP evidence SHOULD include status, negotiated protocol, relevant connection metadata, redirect chain when enabled, response-header timing/TTFB where accurately observable, and bounded body facts when explicitly requested.

Response bodies MUST NOT be emitted by default.

### 8.5 Proxy/path

Proxy diagnostics MUST use Eggress public listener-free outbound APIs.

Failures SHOULD retain Eggress's stable kind, stage, hop index, and protocol facts rather than collapsing them to one generic connection error.

Successful per-hop timing MUST only be reported when Eggress exposes a truthful observer seam.

### 8.6 Composite checks

A composite check runs a declared set of probes and preserves each child result.

A composite MUST NOT turn absence of optional evidence into a fabricated failure. Expectations are evaluated separately as findings.

### 8.7 Direct-versus-route comparison

Comparison is a derived operation over independent probe reports. It MUST preserve both source reports or their stable identifiers and MUST NOT mutate either route to make them artificially comparable.

## 9. Error, status, and assertion semantics

A network operation completing successfully is distinct from an expectation passing.

For example, an HTTP 503 is a successfully observed HTTP response. It is an assertion failure only when the user requested an expectation such as `status in 200..399`.

The machine report MUST therefore distinguish:

- execution/probe status;
- structured diagnostic errors;
- warnings/unavailable evidence;
- expectation findings.

The CLI exit-code surface SHOULD remain coarse while JSON contains the detailed taxonomy.

Initial exit-code intent:

- `0` — execution completed and all requested assertions passed;
- `1` — execution completed but one or more diagnostic assertions failed, or a requested probe produced a negative diagnostic result;
- `2` — invalid invocation, input plan, or configuration;
- `3` — internal Eggprobe execution failure that is not an ordinary target/network result;
- `130` — interruption.

Any change to these meanings after public release requires compatibility review.

## 10. Serialization and automation

Eggprobe MUST support:

- one complete JSON report on stdout;
- NDJSON for streaming/batch operation;
- human terminal rendering;
- stdin plan input;
- stable output that does not mix logs with data.

When JSON/NDJSON is selected:

- stdout MUST contain only the selected machine format;
- logs, tracing, progress, and warnings not represented as report data MUST go to stderr;
- secrets MUST remain redacted;
- serialization MUST be deterministic enough for fixtures and schema tests.

Published schema artifacts SHOULD be generated from canonical Rust types rather than maintained as an independent source of truth.

## 11. Security model

Eggprobe is an active network client and MUST assume targets and peers can be hostile.

Required properties include:

- bounded DNS names, URLs, headers, bodies, certificate summaries, error strings, and batch sizes;
- no credential leakage in route displays, errors, JSON, logs, or panic paths;
- TLS verification enabled by default;
- explicit unsafe flags for disabled verification;
- clear target-policy controls for embedded use;
- no automatic execution of remote content;
- no shell invocation as a canonical probe backend;
- no passive acceptance of environment proxy settings unless the user explicitly requests that policy;
- safe cancellation and cleanup of sockets/tasks;
- bounded concurrency.

## 12. Timeout, repetition, and concurrency model

Every network probe MUST have a finite default outer deadline.

Phase-specific timeouts MAY exist where the underlying implementation can enforce and identify them correctly.

Repeated probes MUST be represented as individual attempts. Aggregate statistics MAY be derived afterward.

Batch execution MUST have explicit concurrency limits and MUST NOT create unbounded tasks or file descriptors.

Cancellation MUST release route resources, pool permits, and child tasks promptly.

## 13. Compatibility and schema evolution

The public schema begins at major version `1` only when the initial contract is intentionally declared stable. Pre-1 development MAY use `0.x` schema versions but MUST still carry an explicit schema version.

Within a stable major schema:

- new optional fields MAY be added;
- new non-exhaustive enum values MAY be added if consumers are instructed to tolerate them;
- existing field meaning MUST NOT change silently;
- required fields MUST NOT be removed;
- units MUST NOT change.

The Rust public API SHOULD use non-exhaustive enums or private fields/accessors where forward evolution is expected.

## 14. Platforms and packaging

The intended supported binary targets are:

- Linux x86_64;
- Linux aarch64, including SBC-class systems;
- macOS x86_64;
- macOS arm64;
- Windows x86_64.

The workspace MSRV target is Rust 1.89 unless a separately reviewed dependency requirement changes it.

Prebuilt release artifacts SHOULD be installable without a Rust toolchain. Release provenance, checksums, and update/install ownership SHOULD align with shared Eggstack machinery when available rather than being copied independently.

## 15. Deferred and future capabilities

Later roadmaps MAY add:

- ICMP echo;
- traceroute/path tracing;
- route-table and interface inspection;
- MTU/path-MTU diagnostics;
- UDP service probes;
- explicit DNS servers and DNSSEC;
- richer certificate-chain analysis;
- direct HTTP/3;
- routed HTTP/3 after an explicit datagram/QUIC composition seam exists;
- repeated latency distributions and export formats;
- library bindings.

These capabilities MUST preserve the JSON-first and transport-ownership invariants.

## 16. End-state acceptance definition

The long-term product is coherent when:

- a program can submit a versioned plan and consume a versioned report without terminal scraping;
- the CLI exposes the same engine without separate semantics;
- direct and Eggress-routed diagnostics share stable report concepts;
- HTTP behavior is delegated to Eggfetch;
- proxy-chain behavior is delegated to Eggress;
- unsupported evidence is explicit rather than fabricated;
- errors, findings, and exit codes have stable documented meanings;
- concurrency, timeouts, cancellation, redaction, and output size are bounded;
- major supported platforms have reproducible qualification evidence;
- each public capability has closure evidence rather than only implementation claims.
