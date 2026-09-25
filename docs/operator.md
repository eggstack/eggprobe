# Eggprobe operator guide

## Install and verify

Ordinary Eggprobe releases use qualified archives and published checksums.
Release archives contain the `eggprobe` executable, this repository's license,
and README metadata. Verify the archive against `SHA256SUMS.txt` before moving
the binary into a trusted directory. Source builds require Rust 1.89 or newer.
The archive is installed manually; no automatic download or replacement script
is part of this installation path.

The current qualification matrix builds Linux x86_64/aarch64, macOS x86_64/
arm64, and Windows x86_64. Host-native CI covers the three desktop/server
families; Linux aarch64 is build-qualified until a native SBC smoke run is
recorded.

## Quick start

```text
eggprobe dns example.com --json
eggprobe tcp example.com --port 443 --json
eggprobe tls example.com --port 443 --json
eggprobe http https://example.com/ --json
eggprobe route example.com --json
eggprobe udp example.com --port 53 --json
eggprobe check example.com --port 443 --url https://example.com/ --json
```

Every invocation has a finite deadline. Ctrl+C or task cancellation does not
retry a request. HTTP 4xx/5xx responses are observations; an assertion may
classify them as a failed expectation without erasing the response evidence.

## Plans and automation

`eggprobe run plan.json` emits one full JSON report. `eggprobe run - --ndjson`
reads stdin and emits one independently parseable final record per batch plan.
`--concurrency` bounds active plan executions and output remains in input order;
`--fail-fast` stops admitting new plans after the first negative result while
awaiting work already started. Input is bounded to 4 MiB and batches to 256
plans. Logs and diagnostics go to stderr; stdout is machine data only when
`--json` or `--ndjson` is selected.

The checked-in `schemas/*-0.4.json` files describe the active contract. The
`*-0.1.json`, `*-0.2.json`, and `*-0.3.json` files are retained historical evidence; the
binary rejects earlier plan versions rather than silently reinterpreting them.
Rust types remain authoritative, and additive evolution is permitted only when
optional and redaction-safe.

## Routes and privacy

Pass Eggress route expressions with `--via` or in a plan's `route` field.
Credentials are accepted only as input. Reports contain only the structural
route marker `{"kind":"eggress"}`; debug and human output use
`eggress(<redacted>)`.
Eggress owns route parsing and hop protocol semantics. Eggprobe does not use
environment proxy variables or silently fall back to a direct connection.

## Local route evidence

`eggprobe route <target> --json` reports the target address, the local source
address selected by the kernel for a no-payload UDP socket, its correlated
interface addresses/state/MTU when available, and at most 32 most-specific
route-table candidates. A candidate is correlation evidence, not proof of the
kernel's selected route; policy routing may make the table view incomplete.
Interface MTU describes the local link and is not path MTU. Native probes are
direct-only, so a route plan using `--via` reports `unsupported`.

## Direct UDP evidence

`eggprobe udp <target> --port <port> [--payload <text>] [--receive] --json`
sends one bounded datagram (at most 1200 payload bytes) from a connected UDP
socket and reports local transmission separately from reply observation:

- without `--receive`, a locally accepted send completes with outcome
  `sent`; this proves local transmission only, never remote service health;
- with `--receive`, one reply is awaited under the outer deadline with
  outcome `response` (source, byte count, and a bounded 1400-byte sample),
  `timeout` (silence is an observation, not a local failure), or
  `unreachable` (the OS surfaced ICMP/host feedback on the connected
  socket);
- bind, connect, and send failures fail the probe with a structured error;
  there is no implicit retry and no background receive task survives the
  request;
- unspecified, broadcast, multicast, and link-local destinations are
  rejected by policy before any socket is created, as are Eggress-routed
  (`--via`) UDP plans;
- request payload bytes are input-only and never reproduced in reports.

## Troubleshooting

- DNS probe answers identify the client/system resolver scope and do not claim
  to be the resolver results used by a remote Eggress hop.
- `connection_refused`, `timeout`, `tls`, and `protocol` are structured probe
  outcomes in JSON.
- HTTP per-phase DNS/TCP/TLS timings are explicitly reported unavailable when
  the current Eggfetch public observer does not expose them.
- DNS probe results are labeled as client resolver evidence. They do not state
  which DNS resolver an Eggress route used remotely.
- HTTP currently negotiates HTTP/1.1 or HTTP/2. Plans have no HTTP/3 request
  selector, and Eggprobe's byte-stream Eggress dialer does not support QUIC.
- Exit codes are 0 for success, 1 for negative probe/assertion outcomes, 2 for
  invalid invocation/plan, 3 for internal failures, and 130 for interruption.

Shared producer-side release construction, bootstrap installers, and generated
release CI belong to Eggpack. Eggprobe may adopt those interfaces in the future
after they are stable and qualified. In-place runtime self-update is a separate
optional Eggup consumer integration. Eggprobe does not currently provide a
self-update command, and that does not block ordinary archive releases. Do not
use an unverified download or replacement script as an automatic updater.

`compare` requires a direct first plan and routed second plan with matching
target/probe families. It reports separate direct and routed distributions.
Latency percentiles use nearest-rank on successful probe timings only:
`ceil(p*N/100)`, clamped to the first sample.

## Native diagnostics

`eggprobe-native` is the internal backend crate for direct path and host
evidence. Implemented primitive surface today:

- DNS, TCP, TLS, and Eggfetch-backed HTTP (engine + Eggress byte-stream
  routing);
- `route <target>` for target-scoped local source/interface/MTU evidence
  and bounded route-table candidate correlation. Direct-only; route
  candidates are correlation evidence, not proof of the kernel's
  selected policy route.

Schema `0.4` reserves additional native probe families but the binary
returns typed unsupported results for them until each milestone
qualifies its backend:

- ICMP echo — `M003`, blocked on a safe backend;
- direct UDP service checks — `M004`, implemented (`eggprobe udp`,
  connected sockets, bounded payload/reply, transmit/reply/timeout/
  unreachable semantics);
- traceroute / path tracing — `M005`, blocked on a truthful backend;
- active path-MTU discovery — `M006`, blocked on test seams and
  trustworthy PMTU feedback.

See `plans/subsystems/native-path-host-diagnostics-roadmap.md` for the
dependency graph and milestone disposition.

## Release identity

The qualified release of record is `v0.1.1` (tag at `53ea53d`); closure
evidence is at `plans/closure/release-operational-qualification/004-status.md`.
The current `main` branch contains unreleased Phase 8 native work
(`6e45e85` — Native M001/M002). Release/package workflows MUST NOT publish
or recreate the changed `main` tree under the `v0.1.1` tag. The next
qualification cycle must select a new version before any artifact
publication.
