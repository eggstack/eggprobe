# Release Corrective C006 Closure

Status: closed

Source implementation plan:

- `plans/implementation/release-operational-qualification-corrective/006-routed-path-cryptoprovider-installation.md`

Source roadmap/addendum:

- `plans/subsystems/release-operational-qualification-roadmap.md`
- `plans/subsystems/release-operational-qualification-corrective-addendum.md`

Planning baseline: `25dc692c7e61129fe6398644c6e41f4692e33bb2`

Implementation commits (production fix + regression only):

- `3204634` — install the ring CryptoProvider default in `egress_connector`
  (covers all production routed paths) + new process-isolated regression
  target `tests/routed_default_provider.rs`
- `fa8ad7c` — closure head: backtick fix for clippy `doc_markdown` on the new
  test header (comment-only; production code identical to `3204634`)

## Executive finding

C006 is complete. The shipped-binary routed panic is fixed at its root, a
process-isolated regression guards it, full verification is green locally
and in hosted CI, and the redaction boundary is re-verified on a routed
report. M004 is handed back ready.

Root cause (proven by backtrace): `eggress_transport_tls` builds a default
rustls client config with no explicit provider
(`ClientConfig::builder` ← `TlsClientConfigBuilder::build` ←
`default_client_config` ← `build_chain_executor_inner` ←
`OutboundConnector::from_pproxy_uri`), which panics without a
process-default provider. Eggprobe's own code always passes an explicit ring
provider, but only test setups ever installed the process default — and
provider installation is process-global, so every test process ran masked
while the shipped binary panicked (exit 3) on any Eggress-routed probe.

## Exact correction

`crates/eggprobe-core/src/engine.rs` (`egress_connector`, the single choke
point all three routed call sites funnel through):

```rust
let _ = rustls::crypto::ring::default_provider().install_default();
```

before `OutboundConnector::from_pproxy_uri`. Ring matches the direct-path
provider; installation is idempotent so embedding test setups that already
installed it are unaffected. No route parsing, hop taxonomy, redaction,
schema, CLI, exit-code, or MSRV change.

## Regression (process-isolated, per plan §5.2)

New target `crates/eggprobe-core/tests/routed_default_provider.rs` runs a
routed TCP probe against a local Eggress SOCKS5 fixture in its own test
process and never installs the provider (header documents why it must stay
a separate target).

Before/after proof:

- fix reverted: `routed_tcp_succeeds_without_preinstalled_crypto_provider`
  FAILED with the exact production panic (`Could not automatically determine
  the process-level CryptoProvider ...`);
- fix applied: the same test passes (`Ok` probe, `Tcp` evidence,
  `route == {"kind":"eggress"}`).

## Verification

Local at closure head `fa8ad7c`:

```text
cargo fmt --all -- --check                                             → ok
cargo check --workspace --all-targets --all-features --locked          → ok
cargo clippy --workspace --all-targets --all-features --locked -D warnings → ok
cargo test --workspace --all-features --locked                         → 0 failed (12 suites, incl. new regression)
cargo +1.89.0 check --workspace --all-targets --locked                 → ok
cargo tree --locked                                                    → ok
cargo +stable audit                                                    → exit 0, no advisories (215 crates)
cargo run -p eggprobe-core --example generate-schemas --locked         → no diff (schema stays 0.3)
```

Fixed-binary evidence (local `--release` build of the fixed tree, local
SOCKS5 relay + loopback target):

- routed TCP via `socks5://127.0.0.1:18099` → exit 0, report/probe `ok`,
  `route: {"kind":"eggress"}` (previously exit 3 panic);
- credentialed route (`socks5://op-user:<secret>@...`) → typed
  `hop_connect` error with `route_hop_index: 0`, report route
  `{"kind":"eggress"}`, zero credential bytes in the output (redaction
  re-verified, no panic).

Hosted ordinary CI on the closure head:

- run: <https://github.com/eggstack/eggprobe/actions/runs/36038995837>
- quality (ubuntu-latest): job `107766135648` — green
- quality (macos-latest): job `107766135548` — green
- quality (windows-latest): job `107766135709` — green
- MSRV (Rust 1.89.0): job `107766135536` — green
- audit: job `107766135080` — green

One precursor hosted run `36038372723` on `3204634` failed all three quality
jobs on clippy `doc_markdown` (missing backticks in the new test header);
fixed by comment-only `fa8ad7c`, no production change. Failed run is not
evidence and is recorded here per process.

## Acceptance criteria (plan §11)

1. ✅ Panic fixed at root (default provider installed on the production
   routed path; fixed binary proves structured routed outcomes).
2. ✅ Process-isolated regression exists and is proven to guard the defect
   (fails reverted, passes fixed).
3. ✅ No order dependence (idempotent install; existing test installs
   harmless; full suite green).
4. ✅ Full local verification green; hosted CI green on the fix commit.
5. ✅ Redaction re-verified on routed reports.
6. ✅ M004 handed back ready (see below).

## Downstream disposition

- **M004: ready for handoff again** (was blocked only on C006). M004
  resumption owns the version/tag disposition for the fixed tree: the
  qualified `v0.1.0` artifacts (tag at `2760b8b`) do NOT contain this
  production fix, so M004 must decide between a new version/tag and
  documenting the routed limitation for `0.1.0`. The `v0.1.0` tag was not
  moved by C006.
- **M003a / M003b:** unchanged, still blocked/deferred on their named
  cross-repository readiness gates.
- M004's retained partial evidence (direct probes, schema, audit, MSRV from
  the stopped execution) still stands; only the routed path needed this
  corrective.

## Roadmap and registry disposition

- `plans/registry.md`: C006 marked closed at `fa8ad7c` with qualifying run
  `36038995837`; M004 ready.
- Release subsystem roadmap and corrective addendum: C006 closed with a
  pointer to this record; milestone tables updated accordingly.
- C001–C005 closure records left intact as evidence.
