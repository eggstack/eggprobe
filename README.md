# Eggprobe

Eggprobe is a pre-release, JSON-first network diagnostics project. The
canonical plan/report contract lives in `eggprobe-core`; the `eggprobe` binary
is a thin presentation adapter and does not own networking.

The repository implements the pre-1 diagnostic contract, direct DNS/TCP/TLS
probes, Eggfetch-backed HTTP, listener-free Eggress routing, typed assertions,
plan-file/NDJSON automation, and reproducible release scaffolding. Report route
summaries remain opaque until Eggress parses route input; credentials are never
printed by human or machine renderers.

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
output is the contract; human rendering remains downstream. `eggprobe run -`
accepts a bounded plan from stdin, and `--ndjson` emits one final report per
batch item. The shared Eggstack updater is not yet integrated, so releases use
manual archive installation and checksum verification.
