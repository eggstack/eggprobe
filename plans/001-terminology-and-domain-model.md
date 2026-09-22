# Eggprobe Canonical Terminology and Domain Model

Status: normative companion to `plans/000-long-term-specification.md`

This document defines language that implementation plans, Rust types, JSON fields, tests, CLI help, and documentation MUST use consistently.

## 1. Naming rules

1. A probe is an operation that gathers evidence. A finding is an interpretation or assertion over evidence.
2. A route describes how a connection is established. A target describes what is being diagnosed.
3. A local DNS lookup MUST NOT be called the route DNS result when the selected proxy resolves the destination remotely.
4. A successful protocol exchange and a satisfied expectation are distinct states.
5. Missing evidence, unsupported evidence, and failed evidence are distinct.
6. Display strings are not machine contracts. Structured enums and fields are.
7. Secret-bearing values MUST have redacted display forms before they cross report/log boundaries.

## 2. Top-level relationships

```text
ProbePlan
|-- Target
|-- RouteSpec
|-- ProbeSpec[]
|-- ExecutionPolicy
|   |-- timeouts
|   |-- repetition
|   `-- concurrency
`-- AssertionSpec[]

ProbeExecution
|-- ProbeAttempt[]
|   |-- Observation[]
|   |-- Timing[]
|   `-- DiagnosticError?
`-- Finding[]

ProbeReport
|-- schema/tool provenance
|-- normalized plan summary
|-- probe results
|-- findings
`-- warnings / unsupported evidence
```

## 3. Plan terms

### Probe plan

A complete, versioned request for Eggprobe execution.

A plan identifies target, route, requested probes, execution policy, and assertions. CLI arguments are compiled into a `ProbePlan` before network execution.

### Probe specification

One requested diagnostic operation such as DNS, TCP, TLS, or HTTP.

A specification describes intent. A result describes what happened.

### Execution policy

Cross-probe controls such as overall deadline, per-probe timeout, repetition count, concurrency limit, redirect policy, target policy, or retry policy.

Execution policy MUST NOT be inferred from presentation mode.

### Assertion

An explicit expectation evaluated against observations, such as an allowed HTTP status range, required ALPN, maximum latency, or required TLS version.

An assertion failure is a `Finding`, not automatically a transport error.

## 4. Target terms

### Target

The logical subject of diagnosis.

Target forms MAY include URL, host/port, hostname, or IP literal depending on the probe.

### Logical host

The hostname or IP literal requested by the user or plan before route resolution.

### Selected address

The concrete socket address actually used for a direct connection when observable.

### Origin

For HTTP(S), the scheme/host/port identity used by HTTP and TLS policy.

## 5. Route terms

### Route specification

The requested path used to establish transport.

Initial route kinds are `Direct` and `Eggress`.

### Direct route

Eggprobe directly resolves/dials the target using its direct probe machinery or delegates HTTP's direct route to Eggfetch where appropriate.

### Eggress route

A listener-free route executed by `eggress-outbound::OutboundConnector`.

The route may contain one or more proxy hops.

### Hop

One proxy-chain member traversed by Eggress.

A hop has an index and protocol. Endpoint details in reports MUST be bounded and credential-redacted.

### Route DNS

The name-resolution behavior that actually determines the destination reached by the selected route.

For remotely resolved proxy requests, the concrete remote answers may be unavailable to Eggprobe.

### Client DNS

A name lookup performed by Eggprobe on the local machine. Client DNS is not automatically route DNS.

## 6. Probe terms

### DNS probe

A diagnostic lookup whose purpose is to observe DNS behavior.

### TCP probe

A diagnostic connection attempt whose purpose is to observe stream establishment independent of HTTP.

### TLS probe

A diagnostic TLS handshake over a selected route.

### HTTP probe

An HTTP request executed through Eggfetch.

### Composite check

An orchestration operation that executes several probe specifications while preserving their individual results.

### Comparison

A derived operation that compares independent reports, commonly direct versus Eggress-routed execution.

## 7. Evidence terms

### Observation

A structured fact directly returned by Eggprobe or an underlying dependency.

Examples include selected address, HTTP status, negotiated ALPN, TLS version, Eggress failure stage, or DNS record.

### Derived observation

A deterministic value calculated from direct observations, such as aggregate latency statistics.

Derived observations MUST be labeled or structurally distinguishable from directly observed facts where ambiguity would matter.

### Unavailable evidence

A fact that may exist but cannot be observed faithfully through the selected API or route.

Unavailable is not failure.

### Unsupported capability

A requested operation that Eggprobe intentionally cannot execute under the selected route/platform/build.

Unsupported is not equivalent to network failure.

### Finding

A normalized conclusion over observations, usually produced by an explicit assertion.

Findings SHOULD carry severity or outcome without replacing source observations.

## 8. Execution terms

### Probe execution

One evaluation of a plan.

### Probe attempt

One concrete network attempt for one probe. Repetitions and retries create distinct attempts.

### Attempt identifier

A stable identifier within one report used to correlate timings, errors, and observations.

### Timing

A duration associated with a named phase or complete probe attempt.

Canonical public durations are integer microseconds unless superseded by ADR.

### Deadline

An absolute outer bound for an operation.

### Timeout

A configured duration budget or the error resulting from exceeding one.

Do not use timeout and deadline interchangeably in implementation documentation.

### Cancellation

Termination requested by the local caller or process rather than by target/network failure.

## 9. Status and error terms

### Probe status

The result state of the probe operation itself.

The initial vocabulary SHOULD include `ok`, `failed`, `unsupported`, and `cancelled`.

### Diagnostic error

A structured failure describing why a probe could not complete as requested.

It SHOULD contain stable category, stage, retry/attempt context where applicable, and a bounded redacted message.

### Error category

A stable protocol-neutral class such as DNS, timeout, connection-refused, network-unreachable, host-unreachable, authentication, TLS, protocol, policy, I/O, unsupported, or internal.

### Error stage

The point at which a failure occurred, such as resolution, direct-connect, hop-connect, hop-handshake, TLS-handshake, request, response-headers, body, or deadline.

### Overall report status

A summary of plan execution. It MUST NOT erase individual probe statuses or findings.

## 10. HTTP terms

### TTFB / response-header latency

The elapsed time from the defined request start boundary to receiving response status/headers.

Eggprobe MUST document the exact boundary it can observe and MUST NOT label a broader duration as DNS/TCP/TLS-specific timing.

### Redirect hop

One HTTP redirect response and the subsequent target selection.

Redirect following is explicit policy. Every followed hop remains observable.

### Response body sample

A bounded optional body capture requested for diagnostics. It is not enabled by default.

## 11. TLS terms

### TLS verification

Certificate and hostname verification according to the configured trust policy.

### Peer certificate summary

A sanitized bounded representation of peer certificate facts such as subject, issuer, SANs, validity interval, fingerprint, and chain length.

A summary is not the raw certificate chain.

### SNI server name

The logical server name supplied to TLS when applicable. It is distinct from the selected socket address.

## 12. Serialization terms

### Schema version

The version of the machine-readable plan/report contract, independent of the Eggprobe binary version.

### Tool version

The Eggprobe application/library version that produced a report.

### JSON mode

One complete JSON document written to stdout.

### NDJSON mode

A stream of independently parseable newline-delimited JSON records.

### Renderer

A pure presentation adapter that consumes report types and performs no network I/O.

## 13. Compatibility mappings

Avoid these ambiguous terms:

| Avoid | Use instead |
|---|---|
| test | probe, assertion, or verification depending on meaning |
| proxy result | Eggress route result or hop result |
| DNS time | client DNS time or route DNS time |
| success | probe status, assertion outcome, or overall status |
| latency | named timing phase or total duration |
| error string | diagnostic error |
| JSON output wrapper | serialized ProbeReport |
| direct proxy | direct route |
