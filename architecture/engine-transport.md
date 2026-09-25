# Probe Engine / Transport

Scope: `crates/eggprobe-core/src/engine.rs` (+ `Cargo.toml`, `tests/engine.rs`, `tests/routed_qualification.rs`, `domain/probe.rs`, `domain/route.rs`, `domain/timing.rs`).

## 1. Purpose: `ProbeEngine`, `TargetPolicy`, execution flow

* `ProbeEngine { target_policy: TargetPolicy }` (`engine.rs:49-54`); `Default` is `AllowPrivate` (`engine.rs:56-62`).
* `TargetPolicy`: `AllowPrivate` (permit loopback/LAN for diagnostics) vs `Strict` (reject private/reserved after resolution) (`engine.rs:40-47`). Enforced centrally in `resolve_addresses()` (`engine.rs:832-871`), covering direct DNS/TCP/TLS and direct HTTP via `PolicyDialer` (`engine.rs:714-747`). Routed (Eggress) paths delegate to Eggress typed errors.
* `is_private()` (`engine.rs:1101-1118`): IPv4 private/loopback/link-local/unspecified/multicast; IPv6 loopback/unspecified/multicast/`fc00::/7`/`fe80::/10`.
* `execute(plan)` (`engine.rs:66-95`): provenance `eggprobe/<CARGO_PKG_VERSION>`, `execution_id = exec-<AtomicU64>`; `plan.validate()` first (failure → `Failed` report, empty probes, warning); finite outer deadline via `tokio::time::timeout_at`; expiry synthesizes `Failed` + `deadline_results` + warning.
* `execute_inner()` (`engine.rs:97-120`): sequential `repetitions × probes` loop, no spawn/join. Status fold: any `Failed` → `Failed`, else any `Unsupported` → `Unsupported`, else any `Cancelled` → `Cancelled`, else `Ok`.
* `execute_probe()` (`engine.rs:122-183`): maps `ProbeSpec` → `ProbeKind`, dispatches to `dns()/tcp()/tls()/http()`, measures `total` (`DurationMicros`), `phases` always empty; HTTP adds `unavailable=["dns_phase_timing","tcp_phase_timing","tls_phase_timing"]`.
* Constants: `MAX_DNS_ANSWERS=32` (`engine.rs:36`), `MAX_BODY_SAMPLE=4096` (`engine.rs:37`).

## 2. Direct DNS / TCP / TLS

Shared resolution `addresses()` → `resolve_addresses()` (`engine.rs:185-191,832-871`): literal IP policy-checked; hostname via `lookup_host`; failure → `Dns/Resolution`; capped at 32; `Strict` + any private answer fails whole resolution (`policy_error()` = `Policy/Resolution`); empty set → `Dns/Resolution "no addresses returned"`. Messages use `error.kind().to_string()` only (redaction-safe, `engine.rs:1070-1072`).

* **DNS** (`engine.rs:193-201`): local system resolver always, even when routed. `DnsEvidence{addresses, resolution_scope: Client}`. No per-op timeout; bounded by outer deadline.
* **TCP direct** (`engine.rs:232-257`): serial `TcpStream::connect` per address; success → `TcpEvidence{peer, local, attempts}`; all-fail → `normalize_io(last, DirectConnect, Some(attempts))` mapping `ConnectionRefused/TimedOut/NetworkUnreachable/HostUnreachable` else `Io`. No `connect_timeout`; hangs until outer deadline.
* **TLS direct** (`engine.rs:276-287,374-423`): SNI from `server_name` or target host (invalid → `Tls/TlsHandshake`); **only first resolved address** (unlike TCP); `TcpStream::connect` then `tls_handshake` with webpki roots, ALPN `[h2, http/1.1]`; failure → `Tls/TlsHandshake "TLS handshake failed"` (detail discarded); success → `TlsEvidence{version, alpn, cipher_suite}`.
* **Deadline errors** (`engine.rs:881-903`): per-`ProbeSpec` `Failed/Timeout/Deadline`, `timing: None`.

## 2b. Direct UDP (`eggprobe-native::udp_exchange`, `ProbeEngine::udp`)

* `udp()` (`engine.rs`, `ProbeEngine::udp`): direct-only (Eggress → `Unsupported/PacketExchange`); resolves the first address under `TargetPolicy`; rejects unspecified/multicast/broadcast/link-local classes with `Policy/PacketExchange` before socket creation; delegates the exchange to `eggprobe-native`.
* `udp_exchange()` (`eggprobe-native/src/lib.rs`): family-correct ephemeral bind, connected send of at most `MAX_UDP_PAYLOAD_BYTES` (1200), optional single-reply window bounded by the remaining outer deadline. Local `send` success never claims remote health.
* Completed observations (`ProbeStatus::Ok` + `UdpEvidence`): `Sent` (no reply awaited), `Response` (source, byte count, sample capped at `MAX_UDP_RESPONSE_SAMPLE_BYTES` = 1400, present only when a reply was awaited), `Timeout` (silence), `Unreachable` (`ConnectionRefused/HostUnreachable/NetworkUnreachable` surfaced on the connected socket). Only bind/connect/send failures become `Failed` probe errors. No retries; cancellation drops the socket with no background task.
* Request payload bytes are input-only and never serialized into reports. CLI: `eggprobe udp <target> --port <p> [--payload <text>] [--receive]` (`eggprobe-cli/src/lib.rs`, `UdpArgs`/`plan_for_udp`); `--port` required, payload capped at 1200.

## 2c. Direct traceroute (`eggprobe-native::trace_path`, `ProbeEngine::trace`)

* Backend: `trippy-core =0.13.0` (exact pin) plus direct `trippy-privilege =0.13.0` (same line, no duplication), UDP, classic single-flow strategy, fixed per-trace source port (atomic window `43534..44534`, isolating concurrent traces) with per-probe destination ports. No `netdev` duplication, no reverse-DNS crate in its tree. ICMP echo mode stays reserved for the M003 backend.
* Privilege mode (`eggprobe-native/src/lib.rs`, `select_privilege_mode`/`current_privilege_mode`/`trace_capability`): macOS uses the backend's documented unprivileged mode; Linux requires already-effective privilege (effective `CAP_NET_RAW`) discovered read-only via `trippy_privilege::Privilege::discover()`, else bounded `PermissionDenied`. Windows refuses with `Unsupported` before any backend contact: `trippy-core 0.13.0`'s privileged run corrupts heap memory there (fail-fast abort while dropping a parsed `UnknownExtension` on an elevated hosted runner) instead of returning an error. Eggprobe never acquires/raises/clears privilege, so `drop_privileges(false)` is set (the tracer must not clear an effective set it did not raise) and concurrent traces cannot race on capability state. Unprivileged support follows upstream documentation per target, never runtime inference.
* `trace()` (`engine.rs`, `ProbeEngine::trace`): direct-only (Eggress → `Unsupported/HopProbe`); first-address resolution under `TargetPolicy`; same destination-class rejection as UDP (`Policy/HopProbe`); remaining outer budget divided across rounds (`max_round_duration = per_round − 100ms` margin) so silent traces finish inside the budget and report `MaxHops`; `read_timeout` is fixed at 100ms because it only sets backend loop-wakeup granularity, never reply cutoffs; engine timeout is a defensive backstop. `PermissionDenied` maps to `PermissionDenied/HopProbe` with the fixed `TRACE_PERMISSION_MESSAGE`, `Unsupported` to `Unsupported/HopProbe` with `TRACE_UNSUPPORTED_MESSAGE`; other failures keep `"UDP trace failed"`.
* `trace_path()` (`eggprobe-native/src/lib.rs`): privilege discovery and channel construction share one blocking worker on `spawn_blocking`, rounds collected via `run_with`; bounded `timeout` keeps partial rounds with `completed: false` instead of failing. Build/run errors normalize to `PermissionDenied` (privilege) or `Io` with fixed messages — dependency `Display` (addresses, interfaces) never forwarded.
* `observe_round`/`summarize_trace` (pure, scripted-test-covered): silent `Awaited` probes → `TimedOut` attempts (evidence, not failure); target replies → `DestinationReached`; router TTL-expired → `TimeExceeded`; router unreachable → `DestinationUnreachable`; unsent (`NotSent`/`Skipped`) probes are not attempts. Termination: `Deadline` (incomplete) > `DestinationReached` > `Unreachable` > `MaxHops`. Hops capped at `max_hops`, TTL-ordered.
* CLI: `eggprobe trace <target> [--max-hops 1-64] [--attempts 1-5]` (`TraceArgs`/`plan_for_trace`); no port flag exists (ports are backend-internal and invisible in reports).

## 3. Eggfetch-backed HTTP

Deps (`eggprobe-core/Cargo.toml:12-14`): `eggfetch-core 0.2.0` (`advanced-routing,standard-http1,standard-http2,tls-rustls`), no QUIC/H3; `eggress-core`/`eggress-embed` 1.0.8; `tokio 1.47`, `rustls 0.23`, `tokio-rustls 0.26`.

* `http()` (`engine.rs:309-371`): method parse (invalid → `Protocol/Request`); builder `automatic_decompression(false)`, `max_decoded_body_size(4096)`; dialer `PolicyDialer` (direct) vs `RouteDialer` (Eggress, bad expression → `Protocol/HopHandshake`); `request.send_detailed()` → `normalize_http_failure()` on error; success records `status`, `protocol` (`"{:?}"` of version, e.g. `HTTP/1.1`), `body_sample_bytes = min(len,4096)`.
* `PolicyDialer` (`engine.rs:714-747`): enforces `Strict` inside eggfetch path; serial connect; `Rejected` on policy, `Other` on DNS.
* `RouteDialer` (`engine.rs:749-782`): `connect_tcp_timeout_detailed` → `EgressStream`; maps `Timeout→Timeout`, `Authentication→Authentication`, `Policy→Rejected`, `Other→Other`, rest → `Connection`.
* Origin TLS (https) happens inside eggfetch; H1/H2 via ALPN default negotiation; body is a bounded retained sample, not full length; per-phase timings unavailable by observer seam.
* `normalize_http_failure()` (`engine.rs:905-998`): timeout → `Timeout/Deadline`; network kind (`Dns/ConnectionRefused/Io`); routed transport unwrapped to `diagnostic_from_route_error`; `DialErrorKind` mapping; string-kind fallback (`tls→Tls/TlsHandshake`, `protocol→Protocol/ResponseHeaders`, `invalid_*→Protocol/Request`, `unsupported→Unsupported/Request`, else `Other/ResponseHeaders`).

## 4. Eggress routing

* Listener-free client-only: only `OutboundConnector::from_pproxy_uri(expression)` (`engine.rs:820-822`); test proxies use ephemeral `EggressService`. Unsupported syntax → `Protocol/HopHandshake "invalid or unsupported Eggress route"` (cause discarded).
* Byte-stream TCP dialer only (`connect_tcp_timeout_detailed` + `EgressStream` `AsyncRead+AsyncWrite` adapter, `engine.rs:784-818`); no QUIC/H3; no env-proxy fallback; routed failures never fall back to direct (tested). Multi-hop `scheme://addr__scheme://addr` supported.
* `diagnostic_from_route_error()` (`engine.rs:1000-1046`): kind mapping (Timeout/Dns/ConnectionRefused/NetworkUnreachable/HostUnreachable/Authentication/Tls/Protocol/Policy else Other), stage mapping (DirectConnect/HopConnect/HopHandshake/Deadline else Other), fixed `routed ...` messages, provenance `route_hop_index` + `route_protocol`. Redaction: `RouteSummary::{Direct,Eggress}` only, `from_plan` never copies raw route, `Debug`/`Display` redacted.
* Routed TLS preserves SNI + hostname verification over `EgressStream`.

## 5. Concurrency / deadline / cancellation

* Runtime `tokio 1.47` (`rt-multi-thread,net,time,io-util,macros,signal,sync`); binary `#[tokio::main]` (`eggprobe-cli/src/main.rs:1`).
* No engine-level parallelism; repetitions serial. No pooling.
* Finite deadline: validation rejects `0`; CLI builds `timeout_ms*1000µs`; engine `timeout_at`; routed sub-deadline `route_connect_timeout` reserves `min(10ms, remaining/100)` so Eggress returns typed `Deadline` before outer fires (`engine.rs:824-830`).
* Cancellation: futures drop-cancelled (sockets closed; tested for routed + standalone TLS). Engine never emits `Cancelled` today; arm is defensive.
* Ctrl+C → `eprintln("interrupted")` + exit 130, no retry, no partial report (`eggprobe-cli/src/main.rs:3-10`). Retries contractually forbidden (`UnsupportedRetries`, `plan.rs:49-53`); CLI hardcodes `retries: 0`; `DiagnosticError.attempt` records per-address attempts only.

## 6. Key Code References

Engine: `engine.rs:40-62` (policy/engine), `66-95` (execute+deadline), `97-120` (sequential+fold), `122-183` (probe dispatch+timing), `193-201` (dns), `203-257` (tcp), `259-307` (tls), `309-371` (http), `374-423` (handshake), `714-747` (PolicyDialer), `749-782` (RouteDialer), `784-830` (EgressStream+connector+timeout), `832-871` (resolve), `881-903` (deadline_results), `905-998` (http normalization), `1000-1046` (route errors), `1073-1118` (normalize_io/policy/is_private). Deps: `eggprobe-core/Cargo.toml:12-23`. Tests: `tests/engine.rs`, `tests/routed_qualification.rs`, TLS/H2 fixtures `engine.rs:425-712`.

## 7. Review Checklist / Risks

* Direct TCP/TLS have no per-connection timeout — one hanging address stalls the plan; surfaces as generic `Timeout/Deadline` with `timing: None`.
* TLS tries only the first address while TCP tries all — inconsistent failover.
* `Strict` fails closed on any private answer in a 32-answer set.
* Routed DNS is still local (`Client` scope) — callers may misread as proxy-observed.
* No per-phase timings; HTTP lists them as `unavailable`.
* HTTP body is a 4 KiB sample, decompression off.
* Error detail intentionally lossy (TLS/IO/HTTP collapsed) — good for redaction, bad for debugging.
* Eggress parse cause discarded — typo vs unsupported scheme indistinguishable.
* `is_private` coverage narrow; `AllowPrivate` default permits everything — confirm `Strict` wiring in production.
* Short-deadline reserve race (`min(10ms, remaining/100)`).
* No retries, no parallelism — transient blips fail; multi-probe latency sums.
* `execution_id` in-process only (`AtomicU64`, resets per process).
* Mixed-state fold hides `Cancelled` under `Failed`.
* Ctrl+C yields no report (exit 130, partial results discarded).
* No QUIC/H3/env-proxy — confirm product coverage expectations.
* `MAX_DNS_ANSWERS=32` truncation silently drops records.
