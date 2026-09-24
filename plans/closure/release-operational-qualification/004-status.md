# Release M004 Closure — Release Qualification and Operator Documentation

Status: closed — first standalone archive release qualified (`0.1.1` / `v0.1.1`).

Source implementation plan:

- `plans/implementation/release-operational-qualification/004-release-qualification-and-operator-docs.md`

Source roadmap:

- `plans/subsystems/release-operational-qualification-roadmap.md#m004--release-qualification-and-operator-documentation`

Related evidence:

- `plans/closure/release-operational-qualification-corrective/005-status.md`
  (qualified `v0.1.0` at `2760b8b`, hosted packaging run `36030784594`)
- `plans/closure/release-operational-qualification-corrective/006-status.md`
  (routed-path CryptoProvider production fix + process-isolated regression)
- Stopped M004 execution at `25dc692` (direct-probe/schema/audit/MSRV partial
  evidence retained; stopped on the shipped-binary routed panic that C006
  fixed at its root).

## Executive finding

M004 is complete. One concrete release candidate is qualified end to end:

```text
tag: v0.1.1
package version: 0.1.1
commit: 53ea53d14560c150d0ebc10de83eca10de37202d
```

Ordinary hosted CI is fully green on the exact candidate, hosted packaging
is fully green across all five declared targets against exactly the tag,
every archive checksum independently verifies, every archive contains only
the intended files, and the extracted host-native shipped binary passes the
full §2 battery — including the routed probes that the `v0.1.0` artifacts
could not run (C006 fix proven present in the packaged artifact), the
credential redaction corpus, JSON/NDJSON machine parsing, the error/exit
taxonomy, and schema compatibility (`0.3` report, `tool.version 0.1.1`,
no `expression` anywhere). Audit is clean, MSRV source check passes,
operator documentation reconciles claim-by-claim with no excess, and no
GitHub Release was published (publication remains a separate decision;
the workflow has no publication capability).

## Version/tag disposition (M004 decision, was explicitly open)

**Fix-forward as patch `0.1.1` / tag `v0.1.1`; `v0.1.0` untouched.**

Rationale: the qualified `v0.1.0` artifacts (tag at `2760b8b`) predate the
C006 production fix, and the `v0.1.0` tag MUST NOT move now that successful
hosted packaging evidence exists against it (C005 record §WP3). The delta
`v0.1.0..v0.1.1` is exactly three commits — C006 production fix `3204634`,
comment-only clippy fix `fa8ad7c`, and the version bump `53ea53d` (workspace
`Cargo.toml` `0.1.0` → `0.1.1`, `Cargo.lock` member entries, plus making the
CLI `--version` test version-robust via `env!("CARGO_PKG_VERSION")` instead
of a hardcoded `"eggprobe 0.1.0"`). Patch is the correct semver class: a
production bug fix (deterministic routed panic → structured evidence) with
no schema change (stays `0.3`), no CLI surface change, no exit-code change,
and no MSRV change. Test fixtures carrying `"version": "0.1.0"` are explicit
test-input data (constructed with `ToolVersion::new("0.1.0")`), not package
metadata, and correctly needed no change.

## Candidate commit qualification

Local strict verification on `53ea53d` (before tag creation):

```text
cargo fmt --all -- --check                                             → ok
cargo check --workspace --all-targets --all-features --locked          → ok
cargo clippy --workspace --all-targets --all-features --locked -D warnings → ok
cargo test --workspace --all-features --locked                         → 0 failed (all suites, incl. C006 regression)
cargo +1.89.0 check --workspace --all-targets --locked                 → ok
cargo tree --locked                                                    → ok
cargo +stable audit                                                    → exit 0, no advisories (215 crates)
cargo run -p eggprobe-core --example generate-schemas --locked         → no diff (schema stays 0.3)
```

One finding during local verification (fixed before commit, not a product
defect): the version bump exposed a hardcoded `"eggprobe 0.1.0"` expectation
in `crates/eggprobe-cli/tests/cli.rs::version_is_available`. Fixed
test-only by deriving the expectation from `env!("CARGO_PKG_VERSION")`.
Post-fix full suite green.

Hosted ordinary CI on the exact candidate `53ea53d`:

- run: <https://github.com/eggstack/eggprobe/actions/runs/36040792687>
- quality (ubuntu-latest): job `107772167471` — green
- quality (macos-latest): job `107772167546` — green
- quality (windows-latest): job `107772167380` — green
- MSRV (Rust 1.89.0): job `107772167336` — green
- audit: job `107772167146` — green

Before tag creation: `git tag -l` showed only `v0.1.0` (no `v0.1.1`
collision) and `gh release list` was empty. Tag-to-commit proof (local,
after push):

```text
$ git rev-list -n 1 v0.1.1
53ea53d14560c150d0ebc10de83eca10de37202d
```

## Hosted packaging execution

Dispatch: `gh workflow run "Release packages" --ref v0.1.1 -f version=v0.1.1`

Qualifying run: <https://github.com/eggstack/eggprobe/actions/runs/36041400748>
(`workflow_dispatch` on ref `v0.1.1`, `head_sha == 53ea53d`, conclusion
`success`). Full matrix green — no partial-matrix acceptance:

| Target | Job | Requested runner | Observed runner image | Build | Smoke |
|---|---|---|---|---|---|
| `x86_64-unknown-linux-gnu` | `107774184046` | `ubuntu-latest` | `ubuntu-24.04` | success | `eggprobe 0.1.1` + schema-0.3 NDJSON smoke passed |
| `aarch64-unknown-linux-gnu` | `107774183958` | `ubuntu-latest` | `ubuntu-24.04` | cross-build success | n/a (build-only; ELF aarch64 inspection passed) |
| `x86_64-apple-darwin` | `107774184115` | `macos-15-intel` | `macos-15` | success | `eggprobe 0.1.1` + schema-0.3 NDJSON smoke passed |
| `aarch64-apple-darwin` | `107774183833` | `macos-latest` | `macos-26-arm64` | success | `eggprobe 0.1.1` + schema-0.3 NDJSON smoke passed |
| `x86_64-pc-windows-msvc` | `107774184228` | `windows-latest` | `windows-2025-vs2026` | success | `eggprobe 0.1.1` + schema-0.3 NDJSON smoke passed |

Every job's `Prove tag and source authority` step passed (checkout exactly
`v0.1.1`, `git describe --tags --exact-match HEAD` equals `v0.1.1`,
`eggprobe-cli` Cargo metadata version equals `0.1.1`; a mismatch fails the
job closed). Job success entails smoke success (smoke runs under
`set -euo pipefail`).

## Artifact download and independent inspection

All five uploaded artifact bundles downloaded via `gh run download`.
Per-target evidence (archive byte size; SHA-256 independently recomputed
locally with `shasum -a 256` over the archive file; every value matched the
emitted `.sha256` file):

| Artifact container | Archive | Archive bytes | SHA-256 (independently verified) |
|---|---|---|---|
| `eggprobe-x86_64-unknown-linux-gnu` | `eggprobe-0.1.1-x86_64-unknown-linux-gnu.tar.gz` | 4508782 | `38cc319d58bc186f9514d5e2c7e7282fc3c6f678bec38b2fbb4d15771f399ee4` MATCH |
| `eggprobe-aarch64-unknown-linux-gnu` | `eggprobe-0.1.1-aarch64-unknown-linux-gnu.tar.gz` | 4460782 | `3126a724ef44486670095aa0963db2f6a368fb5fcf05eeacf0e21c5f99702537` MATCH |
| `eggprobe-x86_64-apple-darwin` | `eggprobe-0.1.1-x86_64-apple-darwin.tar.gz` | 4698246 | `c1970b96b24c1651446e3ea0c18ff25dec254c5a2588c09b5269daafcd409d26` MATCH |
| `eggprobe-aarch64-apple-darwin` | `eggprobe-0.1.1-aarch64-apple-darwin.tar.gz` | 4454317 | `7df5112d216a5b3187e353c4a0c2b964268239d567ab38f1c3bfc43a4f8e9d28` MATCH |
| `eggprobe-x86_64-pc-windows-msvc` | `eggprobe-0.1.1-x86_64-pc-windows-msvc.zip` | 4169326 | `7398dfafb5f97a2d471624a8becd225f6b504dec34327549210317af3a9165a4` MATCH |

Archive integrity: all four `.tar.gz` extracted cleanly; the `.zip` passed
`testzip` with no errors.

Archive content inventory (exact; root directory matches
`eggprobe-<version>-<target>` in every case):

- all `.tar.gz`: `<root>/`, `<root>/LICENSE`, `<root>/eggprobe`,
  `<root>/README.md`, `<root>/release-identity.txt`
- `.zip`: `eggprobe-0.1.1-x86_64-pc-windows-msvc/`, `eggprobe.exe`,
  `LICENSE`, `README.md`, `release-identity.txt`

No archive contains source trees, build directories, credentials, or
unrelated files.

`release-identity.txt` is identical in intent across all five targets:

```text
tag=v0.1.1
commit=53ea53d14560c150d0ebc10de83eca10de37202d
package_version=0.1.1
```

## Shipped-binary qualification battery (§2)

Binary under test: `eggprobe` extracted from the qualified
`eggprobe-0.1.1-x86_64-apple-darwin.tar.gz` (checksum verified above),
on macOS x86_64. Loopback fixtures only: local HTTP server `:18081`,
loopback TCP acceptor `:18101`, local SOCKS5 CONNECT relay `:18099`.

| # | Check | Result |
|---|---|---|
| 1 | `--version` | `eggprobe 0.1.1` (matches tag/package) |
| 2 | `dns localhost --json` | exit 0, `status ok`, `route.direct` |
| 3 | `tcp 127.0.0.1:18101 --json` (open) | exit 0, `status ok`, `Tcp` evidence |
| 4 | `tcp 127.0.0.1:9 --json` (closed) | exit 1, `status failed`, typed `connection_refused` at stage `direct_connect` with bounded message |
| 5 | `tls 127.0.0.1:9 --json` (closed) | exit 1, `status failed`, no panic |
| 6 | `tcp` without `--port` | exit 2 (invalid invocation) |
| 7 | `http http://127.0.0.1:18081/ --json` | exit 0, observed `status: 200`; per-phase timings explicitly `unavailable` |
| 8 | `http` + failing `http_status_range 500..=599` plan | exit 1, observed 200 preserved, typed finding `HTTP status 200 is outside 500..=599` (assertion separation) |
| 9 | `check 127.0.0.1 --port 18081 --url ...` | exit 0, probes `[dns, tcp, http]` |
| 10 | `check` with URL/target port mismatch | exit 2 with clear message (contract guard) |
| 11 | `run` single plan (file) | pretty JSON single report, `schema_version 0.3` |
| 12 | `run {plans:[...]} --ndjson` (2 plans) | 2 lines, input order (`ok` then `failed`), `execution_id` rewritten `batch-0`/`batch-1`, exit 1 |
| 13 | **routed TCP plan via `socks5://127.0.0.1:18099`** | **exit 0, `status ok`, `route {"kind":"eggress"}` — C006 fix proven present in the packaged artifact (the `v0.1.0` artifact panicked here with exit 3)** |
| 14 | `proxy 127.0.0.1 --port 18101 --via ... --json` | exit 0, `status ok`, `route {"kind":"eggress"}` |
| 15 | credentialed route `socks5://op-user:<secret>@...` | exit 0 structured; **zero secret bytes** in JSON stdout, stderr, and human rendering; report route `{"kind":"eggress"}`; no `expression` key in any report |
| 16 | `compare direct routed --repeat 3 --json` | exit 0, `kind comparison`, `connection_policy cold`, separate direct/routed distributions |
| 17 | SIGINT during a hanging probe | exit 130, no partial report |
| 18 | shipped report vs `schemas/report-0.3.json` | all required keys present, `schema_version 0.3` matches schema `const`, `tool.version 0.1.1` (≠ schema version, as documented) |

Tooling note (not a product finding): an early minimal SOCKS5
hand-rolled fixture wedged routed runs via TCP read-framing races
(short `recv(4)` → handler returned without replying/closing). Replaced
with an exact-length-read relay; all routed results above are against the
robust fixture. The product behaved correctly throughout (structured
outcomes, no panic); only the throwaway fixture was at fault.

## Target-claim reconciliation

- Linux x86_64, macOS x86_64, macOS arm64, Windows x86_64:
  **runtime-smoke-qualified** (native `--version` + machine-output smoke in
  the qualifying hosted run), plus M004's full shipped-binary battery on
  macOS x86_64 above.
- Linux aarch64: **build-qualified only** — successful cross-build on
  `ubuntu-24.04` plus `file` inspection (`ARM aarch64`); never executed.
  No SBC runtime qualification is claimed.
- `docs/operator.md` states exactly this; no target-claim change was
  required and none was made.

## Documentation reconciliation (§3, claim-by-claim)

- Installation/source-build: archives contain binary + license + README;
  checksums published per archive; manual install; source builds need
  Rust ≥ 1.89 — all verified true. No auto-download/replacement script
  exists in the path.
- Supported target table: matches the reconciliation above.
- Command quick start (`dns/tcp/tls/http/check --json`): every command
  exercised against the shipped binary (rows 2–9).
- JSON/schema usage: single-plan pretty JSON, batch NDJSON in input order
  with rewritten `execution_id`, stdin input, stderr/machine-stdout
  separation — all verified (rows 11–12). Active schema `0.3`; binary
  rejects other plan versions (contract suite green).
- Proxy credential handling: credentials input-only, reports carry only
  `{"kind":"eggress"}`, human/debug print redacted forms — verified on
  the shipped binary (row 15), including the human `Route: Eggress` line
  with no secret bytes.
- Timeout/cancellation: finite deadlines on every invocation; SIGINT →
  exit 130 with no partial report (row 17).
- Privacy/security: redaction corpus green in-suite plus shipped-binary
  proof (row 15); report JSON contains no `expression` (row 18).
- Troubleshooting/error taxonomy: structured `connection_refused` /
  stage evidence (row 4); HTTP phase timings explicitly `unavailable`
  (row 7); client-scope DNS labeling, H1/H2-only, exit-code table — all
  match observed behavior. No `update`/self-update command exists in
  `--help`, consistent with "does not currently provide a self-update
  command".
- Update instructions: none advertised (M003b not closed) — correctly
  absent, per plan §3.
- No documentation claim exceeds qualified behavior. No doc change was
  required and none was made.

## Security review

- Packaging workflow permissions remain `contents: read` only; no
  publication step exists and none was added. **No GitHub Release was
  published by M004** (`gh release list` empty after the qualifying run).
- No Eggpack producer or Eggup consumer/update logic copied into Eggprobe;
  the `v0.1.0..v0.1.1` delta is the C006 routed fix + version bump +
  test-only version-robustness fix.
- `cargo +stable audit` clean on the tagged commit (215 crates, 0
  advisories); `cargo tree --locked` reviewed; MSRV source check passes
  on 1.89.0.
- Stop conditions (§5): no checksum/provenance mismatch, no schema drift,
  no secret leak, no supported-platform smoke failure, no advisory without
  disposition, no unclosed-capability documentation claim. None triggered.

## Acceptance criteria (plan §4)

1. ✅ No documentation claim exceeds qualified behavior (matrix above).
2. ✅ Release artifacts and schemas correspond to one source
   commit/version (`53ea53d` / `0.1.1` / `v0.1.1`; schema `0.3` no-diff).
3. ✅ No unresolved medium-or-higher release finding remains (the one
   local test-hardcoding finding was fixed and re-qualified; the
   fixture race was tooling-only).

## Downstream disposition

- **First standalone archive release: qualified.** The `0.1.1` artifacts
  above are the qualified release. Any publication/distribution step is a
  separate decision outside this plan (no GitHub Release exists).
- **M003a / M003b:** unchanged, still blocked/deferred on their named
  cross-repository readiness gates; neither was a prerequisite and M004
  introduces no Eggpack/Eggup substitutes.
- `v0.1.0` tag and artifacts remain intact as historical evidence
  (C005 record); they are superseded for consumption by `0.1.1` because
  they lack the routed-path fix.

## Roadmap and registry disposition

- `plans/registry.md`: M004 marked closed at `v0.1.1` with qualifying
  packaging run `36041400748`; release subsystem notes the qualified
  `0.1.1` release.
- Release subsystem roadmap: M004 closed with a pointer to this record;
  milestone tables updated accordingly.
- C001–C006 closure records left intact as evidence.
