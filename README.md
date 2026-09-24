# Eggprobe

JSON-first network diagnostics. `eggprobe-core` owns the plan/report
contract and all probing (direct DNS/TCP/TLS, Eggfetch-backed HTTP/1.1+HTTP/2,
Eggress-routed TCP/TLS/HTTP, typed assertions); the `eggprobe` binary is a
thin presentation adapter and owns no networking.

## Install

Source builds require Rust 1.89 or newer:

```text
cargo build --locked --release
./target/release/eggprobe --version
```

Release archives contain the `eggprobe` executable plus license/README
metadata; verify against the published checksum and move the binary into a
trusted directory manually. See `docs/operator.md` for install details and
the supported-target matrix.

## Usage

```text
eggprobe dns example.com --json
eggprobe tcp example.com --port 443 --json
eggprobe tls example.com --port 443 --json
eggprobe http https://example.com/ --json
eggprobe check example.com --port 443 --url https://example.com/ --json
eggprobe run plan.json
eggprobe run - --ndjson < batch.jsonl
eggprobe compare direct.json routed.json --repeat 5 --json
```

Notes:

- `--json` / `--ndjson` select machine output on stdout; without them the
  CLI prints short human text. Logs and diagnostics always go to stderr.
- `run` accepts a single plan, a JSON array of plans, or `{plans:[...]}`;
  `-` reads stdin. Batch output is one compact report per line in input
  order.
- `compare` takes a direct plan and an Eggress-routed plan with matching
  target/probes and reports separate latency distributions.
- Exit codes: `0` success, `1` negative probe/assertion outcome, `2` invalid
  invocation/plan, `3` internal failure, `130` interrupted.
- Full command, routing, privacy, and troubleshooting reference is in
  `docs/operator.md`.

## Contract

The active contract is schema `0.4`, described by `schemas/plan-0.4.json`
and `schemas/report-0.4.json`. Rust types in `eggprobe-core` are
authoritative; the binary rejects plan versions other than `0.4`.
Route credentials are input-only: reports carry only
`{"kind":"eggress"}` and human/debug output prints `eggress(<redacted>)`.

## Verification

```text
cargo fmt --all -- --check
cargo check --workspace --all-targets --all-features --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test --workspace --all-features --locked
cargo +1.89.0 check --workspace --all-targets --locked
cargo tree --locked
cargo audit
```
