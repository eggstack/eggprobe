# Eggprobe operator guide

## Install and verify

Release archives contain the `eggprobe` executable, this repository's license,
and README metadata. Verify the archive against `SHA256SUMS.txt` before moving
the binary into a trusted directory. Source builds require Rust 1.89 or newer.

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

The checked-in `schemas/*-0.2.json` files describe the active contract. The
`*-0.1.json` files are retained historical evidence; the binary rejects 0.1
plans rather than silently reinterpreting them. Rust types remain authoritative,
and additive evolution is permitted only when optional and redaction-safe.

## Routes and privacy

Pass Eggress route expressions with `--via` or in a plan's `route` field.
Credentials are accepted only as input. Reports contain only the structural
route marker `{"kind":"eggress"}`; debug and human output use
`eggress(<redacted>)`.
Eggress owns route parsing and hop protocol semantics. Eggprobe does not use
environment proxy variables or silently fall back to a direct connection.

## Troubleshooting

- `dns` failures are client/system resolver failures; they do not claim a
  remote proxy resolver address.
- `connection_refused`, `timeout`, `tls`, and `protocol` are structured probe
  outcomes in JSON.
- HTTP per-phase DNS/TCP/TLS timings are explicitly reported unavailable when
  the current Eggfetch public observer does not expose them.
- Exit codes are 0 for success, 1 for negative probe/assertion outcomes, 2 for
  invalid invocation/plan, 3 for internal failures, and 130 for interruption.

The shared Eggstack `eggup` interface is not yet published. Do not use an
unverified download or replacement script as an automatic updater.

`compare` requires a direct first plan and routed second plan with matching
target/probe families. It reports separate direct and routed distributions.
Latency percentiles use nearest-rank on successful probe timings only:
`ceil(p*N/100)`, clamped to the first sample.
