# Release and Operational Qualification Corrective C006 — Routed-Path CryptoProvider Installation

Status: closed at `fa8ad7c` — see `plans/closure/release-operational-qualification-corrective/006-status.md` (hosted run `36038995837` fully green; M004 handed back ready)

Planning baseline: `25dc692c7e61129fe6398644c6e41f4692e33bb2`

Source roadmap:

- `plans/subsystems/release-operational-qualification-roadmap.md`
- `plans/subsystems/release-operational-qualification-corrective-addendum.md`
- `plans/002-long-term-roadmap.md#phase-7--release-packaging-and-operational-qualification`

Historical/related evidence:

- `plans/closure/release-operational-qualification-corrective/005-status.md`
  (qualified tag `v0.1.0` at `2760b8b`, hosted packaging run `36030784594`)
- `plans/implementation/release-operational-qualification/004-release-qualification-and-operator-docs.md`
  (M004 execution at `25dc692` stopped on this defect; partial verification
  evidence below is retained for M004 resumption)
- hosted C005 packaging run `36030784594` (direct-only smoke; never exercised
  a routed probe through the shipped binary)

Applicable ADRs:

- `plans/adrs/ADR-0001-json-first-core-and-transport-ownership.md`
- `plans/adrs/ADR-0002-release-producer-consumer-ownership.md`

Primary class: release qualification + invariant (no panic on a supported
command path)

## 1. Objective

Fix the deterministic routed-probe panic in the shipped `eggprobe` binary and
re-qualify the routed path, so M004 release qualification can resume and the
first standalone release can advertise the Eggress-routed commands it already
documents.

C006 must:

1. install a process-default rustls CryptoProvider (ring, matching the
   direct-path choice) on every production path that can reach Eggress
   outbound execution, before any routed probe runs;
2. add a regression that fails if the provider is ever again missing on a
   production-routed code path — it must run in a process that has NOT
   pre-installed the provider (see §5.2);
3. re-run the full verification suite plus the M004 §2 routed probes against
   the fixed binary;
4. record why all prior evidence missed the defect;
5. hand back to M004 without changing schema, CLI surface, exit-code
   contract, MSRV, or artifact layout.

## 2. Defect evidence (M004 execution, retained)

Release binary extracted from the C005-qualified archive
`eggprobe-0.1.0-x86_64-apple-darwin.tar.gz` (SHA-256 independently verified),
run on macOS x86_64:

- direct probes all correct: `dns localhost` → exit 0 `Ok`; `tcp 127.0.0.1`
  against a local listener → exit 0 `Ok`; `http http://127.0.0.1:18081/`
  against a local server → exit 0 with observed `status: 200` and explicitly
  `unavailable` per-phase timings; assertion separation holds (valid 200
  stays `Ok` evidence while a deliberately failing 500–599 finding yields
  exit 1); invalid invocation yields exit 2; `--version` prints
  `eggprobe 0.1.0`.
- direct TLS control: `tls 127.0.0.1` against a local self-signed server →
  structured `tls` failure, exit 1, no panic (direct path uses an explicit
  `builder_with_provider`).
- routed probes deterministically panic: a plan with
  `"route":{"kind":"eggress","expression":"socks5://127.0.0.1:18099"}` and a
  TCP probe against an open loopback port — with a working no-auth SOCKS5
  CONNECT relay actually listening on that port — exits 3 with:

```text
Could not automatically determine the process-level CryptoProvider from
Rustls crate features. Call CryptoProvider::install_default() ...
```

Backtrace (`RUST_BACKTRACE=1`) root-cause chain:

```text
rustls::client::ClientConfig::builder
eggress_transport_tls::client::TlsClientConfigBuilder::build
eggress_transport_tls::client::default_client_config
eggress_outbound::executor::build_chain_executor_inner
eggress_outbound::connector::OutboundConnector::from_pproxy_uri
eggprobe_core::engine::egress_connector
eggprobe_core::engine::ProbeEngine::execute_probe
```

The panic is pre-dial and independent of route validity: `from_pproxy_uri`
builds a default TLS client config without an explicit provider. Eggprobe's
own code always passes an explicit ring provider (`engine.rs`
`builder_with_provider` / `.crypto_provider` call sites); only `#[cfg(test)]`
and integration-test setups call
`rustls::crypto::ring::default_provider().install_default()`
(`engine.rs` test module, `tests/routed_qualification.rs:14`). The production
binary path never installs a process-default provider.

The identical panic reproduces on a local debug build, so this is a product
defect, not a packaging artifact.

## 3. Why prior evidence missed it

1. Every test that touches the routed path installs the process-default
   provider as a side effect of test setup. Provider installation is
   process-global and permanent, so the whole test process — including the
   CLI/binary tests — runs with a provider present, masking the production
   gap. No test ever executes a routed probe in a provider-less process.
2. All hosted smoke (C001 design, C005 run `36030784594`) is direct-only by
   design; no packaging or CI job has ever run a routed probe through the
   shipped binary.
3. The `proxy` CLI command (routed plans via `--via`) exists and is
   documented in `docs/operator.md`, so the panicking path is a supported,
   advertised command path — not an exotic input.

## 4. Readiness and dependencies

Hard dependencies: none beyond the closed C005 evidence above. The qualified
release identity (`v0.1.0` at `2760b8b`) is unaffected — the fix lands on
`main` after the tag and M004 qualifies the fixed tree; whether M004 needs a
new tag/version is an M004 disposition decision at resumption (the fix is
production code, so the `v0.1.0` artifacts do NOT contain it).

Operational dependencies: a local SOCKS5 fixture for regression/verification
(the engine test module already embeds an Eggress listener pattern that can
be reused; no internet endpoint required).

## 5. In scope

### 5.1 Production fix

- Install the ring default CryptoProvider exactly once before Eggress
  outbound execution is reachable in production. Preferred location: a single
  idempotent call at the top of the routed execution path in
  `eggprobe-core::engine` (so library consumers are covered too), e.g. next
  to `egress_connector`, using the existing `let _ =
  rustls::crypto::ring::default_provider().install_default();` idiom already
  used in tests. A CLI-`main`-only installation is NOT sufficient (library
  path would stay broken).
- Do not change the provider selection: ring, matching the direct path. Do
  not add `aws-lc-rs`. Do not change route parsing, hop error taxonomy,
  redaction, schema, CLI flags, or exit codes.

### 5.2 Regression evidence

The regression MUST execute a routed probe in a process where no test setup
has installed the provider. Acceptable forms (pick one, document why it is
process-isolated):

- a new integration test binary (separate `tests/` target) that performs a
  routed TCP probe against a local SOCKS5 fixture and never calls
  `install_default` itself; or
- a CLI-level test that spawns the built binary as a child process with a
  local SOCKS5 fixture and asserts a structured (non-panic) outcome.

Rationale: adding the case to the existing `routed_qualification` binary
would NOT regress — that binary installs the provider globally at setup.

The regression asserts: routed probe against a local fixture completes with
a typed `ProbeEvidence` or typed `DiagnosticError` (never exit 3 / panic
output), and the report route remains `{"kind":"eggress"}` with no
credential leak.

### 5.3 Re-qualification

- Full ordinary verification suite at the fix commit (fmt, check, clippy
  `-D warnings`, workspace tests, MSRV check, tree, audit, schema regen
  no-diff).
- The M004 §2 routed probes re-run against the fixed binary (local SOCKS5 +
  open loopback target): expect structured success, not a panic.
- Hosted ordinary CI green on the fix commit before handoff to M004.

## 6. Out of scope

- Schema changes (stays `0.3`); CLI surface changes; exit-code changes; MSRV
  changes.
- Retagging `v0.1.0` or rebuilding release archives — that disposition
  belongs to M004 resumption, which must decide whether the fixed tree needs
  a new version/tag or whether the fix ships in the next release. C006 must
  NOT move the `v0.1.0` tag.
- Eggpack/Eggup work; installer/self-update work.
- Changing Eggress dependency version (the dependency behaves correctly once
  the host process provides the default it documents).

## 7. Ordered work packages

### WP1 — Fix provider installation

1. Add one idempotent ring-default-provider installation covering all
   production routes into `egress_connector` / routed execution.
2. Keep the change minimal and commented briefly (why process-global
   installation is required: Eggress builds a default client config).

### WP2 — Process-isolated regression

1. Add the regression in a provider-less process (new test target or
   binary-spawning CLI test) with a local SOCKS5 + loopback fixture.
2. Prove it fails before the fix (stash/conditional check) and passes after.

### WP3 — Full verification + hosted CI

1. Local: fmt, check, clippy, test, MSRV, tree, audit, schema regen.
2. Fixed-binary routed probes (M004 §2 set) green.
3. Push; require fully green hosted ordinary CI on the fix commit.

### WP4 — Closure and M004 handoff

1. Create `plans/closure/release-operational-qualification-corrective/006-status.md`.
2. Update registry/roadmap/addendum: C006 closed, M004 ready again.
3. Hand back to M004 with the explicit open question recorded: M004 owns the
   version/tag disposition for the fixed tree.

## 8. Failure and stop conditions

Stop and register further work if the panic trace implicates anything other
than the missing process-default provider, if the fix requires changing the
Eggress dependency version or provider selection, or if any routed outcome
loses its typed stage/provenance or redaction boundary.

## 9. Compatibility and migration

Public product compatibility effect: routed probes change from deterministic
panic (exit 3) to structured evidence/errors. Direct behavior unchanged. No
schema bump. MSRV unchanged (ring is already a direct dependency).

## 10. Required verification

```text
cargo fmt --all -- --check
cargo check --workspace --all-targets --all-features --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test --workspace --all-features --locked
cargo +1.89.0 check --workspace --all-targets --locked
cargo tree --locked
cargo +stable audit
cargo run -p eggprobe-core --example generate-schemas --locked  (no diff)
```

Plus: new regression green; fixed-binary local routed probe structured (no
panic); hosted ordinary CI fully green on the fix commit.

## 11. Acceptance criteria

C006 closes only when:

1. the shipped-binary routed panic is fixed at its root (process-default
   provider installed on the production path);
2. a process-isolated regression exists and is proven to guard the defect;
3. no test anywhere depends on installation order (the fix is idempotent;
   existing `install_default` calls in tests remain harmless);
4. full local verification green; hosted CI green on the fix commit;
5. redaction boundary re-verified on a routed report (`{"kind":"eggress"}`,
   no credential bytes in any rendering);
6. M004 is handed back ready with version/tag disposition explicitly open.

## 12. Closure evidence

Create `plans/closure/release-operational-qualification-corrective/006-status.md`
with: fix commit; exact code change; regression location and
before/after proof; local + hosted verification outcomes; fixed-binary
routed evidence; redaction re-verification; M004 handoff note.
