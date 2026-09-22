# ADR-0001: JSON-First Core and Transport Ownership

Status: accepted

Date: 2026-09-22

Decision owners: project maintainers

Related specification sections:

- `plans/000-long-term-specification.md#4-architectural-principles`
- `plans/000-long-term-specification.md#5-canonical-execution-model`
- `plans/000-long-term-specification.md#7-route-model`
- `plans/000-long-term-specification.md#10-serialization-and-automation`

Affected subsystem roadmaps:

- `plans/subsystems/foundation-diagnostic-contract-roadmap.md`
- `plans/subsystems/transport-probe-engine-roadmap.md`
- `plans/subsystems/cli-automation-roadmap.md`

## Context

Eggprobe is a greenfield network diagnostics project intended to reuse Eggstack network components rather than recreate them.

Current sibling surfaces provide a clean composition opportunity:

- Eggfetch 0.2.0 is an async-first HTTP engine with HTTP/1.1, HTTP/2, optional HTTP/3, Rustls, typed timeouts/errors, connection metadata, and an `advanced-routing` custom `Dialer` whose stream is `AsyncRead + AsyncWrite + Send + Unpin`.
- Eggress 1.0.7 exposes listener-free `eggress-outbound::OutboundConnector`, whose `BoxStream` has the same async byte-stream capabilities and whose detailed connect API exposes stable failure kind, stage, hop index, and protocol facts.
- Eggfetch aggregate transport metrics do not currently provide a complete per-request DNS/TCP/TLS timing trace.
- Eggress detailed failures are stronger than its successful per-hop timing surface.
- HTTP/3 is datagram/QUIC based and therefore cannot truthfully traverse a TCP byte-stream dialer.

A founding decision is needed before crate/module and schema work begins.

## Decision drivers

- machine-readable automation is a primary use case;
- CLI behavior must not become the architecture;
- Eggfetch and Eggress should remain canonical owners of their existing protocol stacks;
- direct diagnostic primitives must remain possible even when they are not HTTP;
- reports must not invent missing evidence;
- transport composition must not require spawning a localhost proxy;
- future schema evolution must be deliberate.

## Considered options

### Option A — CLI-first application with optional JSON formatting

Build commands directly and serialize whatever each command prints.

Benefits: fast initial CLI.

Costs: duplicates semantics across commands, makes JSON unstable, makes future embedding difficult, and encourages display-string parsing.

### Option B — JSON-first core with explicit transport ownership

Define typed plan/report contracts in `eggprobe-core`, make CLI rendering an adapter, use Eggfetch for HTTP, Eggress for proxy chains, and own only probe-specific primitives in Eggprobe.

Benefits: one execution model, strong automation contract, direct library reuse, testability, and clear ownership.

Costs: slightly more up-front type/schema work.

### Option C — Start Eggress as a loopback proxy and point Eggfetch at it

Benefits: minimal adapter code.

Costs: extra local socket/listener, port allocation, weaker error/hop evidence, more moving parts, and a false service boundary.

### Option D — Implement all networking directly in Eggprobe

Benefits: maximum instrumentation control.

Costs: duplicates mature HTTP/proxy/TLS behavior, expands security/maintenance scope, and weakens Eggstack reuse.

## Decision

Select Option B.

Eggprobe SHALL use a JSON-first typed core. The CLI SHALL compile arguments into `ProbePlan`, invoke the core, and render `ProbeReport`.

Ownership is:

- Eggprobe: diagnostic domain model, execution orchestration, DNS probe, direct TCP probe, explicit standalone TLS probe, timing/finding/assertion logic, report/schema, renderer.
- Eggfetch: HTTP protocol behavior and HTTP-associated origin TLS.
- Eggress: proxy-chain parsing and execution.
- Eggprobe adapters: composition glue, including an Eggress-backed implementation of Eggfetch's custom dialer where supported.

Eggprobe SHALL NOT start an Eggress listener merely to connect Eggfetch to Eggress.

Direct and routed execution SHALL expose the same Eggprobe report vocabulary, but unavailable route-specific facts SHALL remain unavailable rather than inferred.

Routed HTTP/3 SHALL remain unsupported until a compatible public datagram/QUIC composition seam exists. Direct HTTP/3 is a separate later qualification.

## Consequences

### Positive

- machine contracts are stable from project inception;
- tests and CLI use one engine;
- Eggfetch/Eggress improvements can be inherited rather than copied;
- proxy failure provenance remains structured;
- local fixture testing is straightforward;
- later bindings or service adapters do not require architectural inversion.

### Negative

- some diagnostic phase timings initially remain coarse until upstream observer hooks exist;
- standalone TLS requires a small probe-specific Rustls path in addition to Eggfetch's HTTP TLS path;
- adapters must normalize error taxonomies without erasing source detail.

### Neutral or deferred

- exact crate decomposition beyond `eggprobe-core` and `eggprobe-cli`;
- Python/FFI bindings;
- native ICMP/traceroute;
- routed QUIC/H3.

## Compatibility and migration

The repository is greenfield, so there is no legacy user contract to migrate.

The initial schema MUST carry an explicit version before the first release. Future incompatible schema changes require a major schema-version change or an accepted compatibility ADR.

## Security and reliability implications

- TLS verification remains enabled by default.
- Route displays and errors must use credential-redacted forms.
- Direct local/private diagnostics are allowed under explicit diagnostic target policy.
- Embedded consumers may choose stricter target policy.
- JSON/NDJSON stdout must not contain logs.
- Unsupported route combinations fail explicitly.
- Retries are opt-in and attempts remain visible.

## Verification

The architecture is conformant when:

- CLI crate contains no independent network execution path;
- core JSON round-trip fixtures exist;
- HTTP probe implementation imports and uses Eggfetch rather than Hyper directly for HTTP semantics;
- proxy-chain implementation imports and uses `eggress-outbound`;
- an Eggress-to-Eggfetch in-process adapter is proven without a listener;
- tests assert unsupported routed H3 rather than fallback;
- redaction fixtures prove credentials do not enter reports.

## Supersession

None.
