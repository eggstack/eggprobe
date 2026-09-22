# Eggprobe

Eggprobe is a pre-release, JSON-first network diagnostics project. The
canonical plan/report contract lives in `eggprobe-core`; the `eggprobe` binary
is a thin presentation adapter and does not own networking.

The repository currently implements the foundation contract only: typed plan
and report values, deterministic JSON fixtures, route redaction, and the
`--help`/`--version` CLI smoke surface. DNS, TCP, TLS, HTTP, proxy execution,
schema generation, and assertions are intentionally not implemented yet.

The minimum supported Rust version is 1.89.0.

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

The project uses the JSON-first/core ownership described in
`plans/adrs/ADR-0001-json-first-core-and-transport-ownership.md`. Machine
output is the contract; human rendering remains downstream.
