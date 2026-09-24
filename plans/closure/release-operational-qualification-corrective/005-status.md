# Release Corrective C005 Closure

Status: closed

Source implementation plan:

- `plans/implementation/release-operational-qualification-corrective/005-first-release-tag-and-hosted-packaging-qualification.md`

Source roadmap/addendum:

- `plans/subsystems/release-operational-qualification-roadmap.md`
- `plans/subsystems/release-operational-qualification-corrective-addendum.md`

Planning baseline: `ff789d4c02d112ffb8909ba90f198ceebce1b6ad`

Implementation commits (release workflow only; no production, schema, CLI, or MSRV change):

- `a87b28c` — Intel macOS packaging runner `macos-13` → `macos-15-intel`;
  smoke-step target comparison via shell variable (removes the known
  actionlint/ShellCheck literal-glob heuristic)
- `2760b8b` — closure head: smoke plan passed via file (`smoke-plan.json`)
  instead of process substitution, so the native Windows binary can read it

## Executive finding

C005 is complete. The first standalone Eggprobe release identity is qualified
end to end:

```text
tag: v0.1.0
package version: 0.1.0
commit: 2760b8b8750272ad448272fa0878ca57bbd2d880
```

Hosted packaging run `36030784594` (dispatched against exactly `v0.1.0`,
`head_sha == 2760b8b`) is fully green across all five matrix targets, every
archive checksum independently verifies, every archive contains only the
intended files, and `release-identity.txt` is byte-identical in intent across
all targets. Host-native smoke passed on Linux x86_64, macOS x86_64, macOS
arm64, and Windows x86_64; Linux aarch64 is build-qualified only. No GitHub
Release was published by C005.

C001's remaining operational condition (intentional valid release tag +
successful hosted packaging run + artifact/hash/native-smoke evidence) is
satisfied. Release M002 is promoted to operationally qualified. M004 is ready
for handoff. M003a/M003b remain deferred and were not touched.

## WP1 — Release-workflow preflight correction

Reviewed live GitHub-hosted runner documentation at execution time
(2026-09-24): standard Intel macOS labels are `macos-15-intel` and
`macos-26-intel`; `macos-13` is in deprecation/close-down. The plan's
prescribed `macos-15-intel` is the current documented stable Intel label, so
no deviation was needed.

Change (`a87b28c`, 5 insertions / 4 deletions, `.github/workflows/release.yml`
only):

1. `x86_64-apple-darwin` runner `macos-13` → `macos-15-intel`. Target triple,
   archive names, identity semantics, and `contents: read` permissions
   unchanged.
2. Smoke-step Windows comparison rewritten from the literal
   `[[ "${{ matrix.target }}" == *windows* ]]` to a shell variable
   (`target='${{ matrix.target }}'; [[ "$target" == *windows* ]]`), removing
   the known actionlint/ShellCheck literal-glob heuristic. The package step
   already used this formulation.

Checks at execution: YAML parses; shellcheck on the rewritten fragments
reports only the pre-existing expected SC2016 info notes (GitHub `${{ }}`
expressions expand in Actions before the shell sees them) — no new warnings;
yamllint reports only pre-existing line-length notes. actionlint is not
installed on the execution host; hosted CI plus the warning-clean shell
formulation cover this, and the qualifying packaging run itself is the
authoritative lint/parse proof (a malformed workflow would not dispatch).

## WP2 — Candidate commit qualification

Local verification on the closure head `2760b8b` (before tag move):

```text
cargo fmt --all -- --check                                             → ok
cargo check --workspace --all-targets --all-features --locked          → ok
cargo clippy --workspace --all-targets --all-features --locked -D warnings → ok
cargo test --workspace --all-features --locked                         → 0 failed (all suites)
cargo +1.89.0 check --workspace --all-targets --locked                 → ok
cargo tree --locked                                                    → ok
cargo +stable audit                                                    → exit 0, no advisories (215 crates)
cargo run -p eggprobe-core --example generate-schemas --locked         → no diff
```

Hosted ordinary CI on the exact candidate `2760b8b`:

- run: <https://github.com/eggstack/eggprobe/actions/runs/36030124719>
- quality (ubuntu-latest): job `107736470803` — green
- quality (macos-latest): job `107736470951` — green
- quality (windows-latest): job `107736470699` — green
- MSRV (Rust 1.89.0): job `107736470848` — green
- audit: job `107736470491` — green

The intermediate hygiene commit `a87b28c` was likewise fully green
(run `36028258472`: ubuntu `107730189257`, macOS `107730189243`, Windows
`107730189359`, MSRV `107730189337`, audit `107730188983`).

Cargo workspace/package version confirmed `0.1.0` at tagging time. Before tag
creation `gh release list` was empty (no GitHub Release for the candidate
version) and `git tag -l` was empty (no pre-existing tag to collide with).

## WP3 — Release identity and tag creation

Expected identity (Cargo remained `0.1.0`, so no planning-evidence refresh was
needed):

```text
tag: v0.1.0
package version: 0.1.0
commit: 2760b8b8750272ad448272fa0878ca57bbd2d880
```

Tag-to-commit proof (local, after push):

```text
$ git rev-list -n 1 v0.1.0
2760b8b8750272ad448272fa0878ca57bbd2d880
$ git describe --tags --exact-match HEAD
v0.1.0
```

Tag-move disposition (plan §8, recorded explicitly): the tag was first created
at `a87b28c`, and the first packaging attempt (run `36028989954`) exposed the
Windows process-substitution defect below. At that point no successful
artifact set had been produced against the tag (the matrix was 4/5 with the
Windows job failing before packaging) and no GitHub Release existed, so the
unused tag was deleted locally and remotely and recreated at the fixed,
fully-green candidate `2760b8b`. The failed run ID and cause are preserved in
this record. The tag MUST NOT be moved again now that successful hosted
packaging evidence exists against it.

## WP4 — Hosted packaging execution

Dispatch: `gh workflow run "Release packages" --ref v0.1.0 -f version=v0.1.0`

Qualifying run: <https://github.com/eggstack/eggprobe/actions/runs/36030784594>
(`workflow_dispatch` on ref `v0.1.0`, `head_sha == 2760b8b`, conclusion
`success`). Full matrix green — no partial-matrix acceptance:

| Target | Job | Requested runner | Observed runner image | Build | Smoke |
|---|---|---|---|---|---|
| `x86_64-unknown-linux-gnu` | `107738693285` | `ubuntu-latest` | `ubuntu-24.04` | success | `eggprobe 0.1.0` + schema-0.3 NDJSON smoke passed |
| `aarch64-unknown-linux-gnu` | `107738693346` | `ubuntu-latest` | `ubuntu-24.04` | cross-build success | n/a (build-only; ELF aarch64 inspection passed) |
| `x86_64-apple-darwin` | `107738693296` | `macos-15-intel` | `macos-15` | success | `eggprobe 0.1.0` + schema-0.3 NDJSON smoke passed |
| `aarch64-apple-darwin` | `107738693426` | `macos-latest` | `macos-26-arm64` | success | `eggprobe 0.1.0` + schema-0.3 NDJSON smoke passed |
| `x86_64-pc-windows-msvc` | `107738693037` | `windows-latest` | `windows-2025-vs2026` | success | `eggprobe 0.1.0` + schema-0.3 NDJSON smoke passed |

Every job's `Prove tag and source authority` step passed, i.e. each job
checked out exactly `v0.1.0`, `git describe --tags --exact-match HEAD`
equaled `v0.1.0`, and `eggprobe-cli` Cargo metadata version equaled `0.1.0`
(a mismatch fails the job closed). Job success entails smoke success: the
smoke step runs under `set -euo pipefail`, and a smoke failure fails the job
— demonstrated by the precursor run below.

Failed precursor run (NOT qualification evidence, recorded per plan §8/§14):

- run `36028989954` (`v0.1.0` at `a87b28c`): Linux x86_64, Linux aarch64,
  macOS x86_64, macOS arm64 green; Windows job `107732646417` failed in
  `Run host-native smoke` with `eggprobe: The system cannot find the path
  specified. (os error 3)`. Root cause: repository/workflow defect — the
  smoke step used bash process substitution (`run <(printf ...)`), which
  yields a `/dev/fd/63` path that the native Windows `eggprobe.exe` cannot
  open. `--version` itself printed `eggprobe 0.1.0` correctly, so this was
  purely a plan-handoff mechanism defect, not a product defect. Fixed by
  `2760b8b` (plan via `smoke-plan.json` file); the fixed smoke logic was
  verified locally before commit and passes on all host-native runners in
  the qualifying run.
- Older cancelled runs `35768825481` and `35766690963` (2026-09-22, pre-C005,
  `workflow_dispatch` on `main`): cancelled, never a complete matrix, not
  evidence.

## WP5 — Artifact download and independent inspection

All five uploaded artifact bundles downloaded via `gh run download`.
Per-target evidence (archive byte size and SHA-256 independently recomputed
locally with SHA-256 over the archive file; every value matched the emitted
`.sha256` file):

| Artifact container (Actions bytes) | Archive | Archive bytes | SHA-256 (independently verified) |
|---|---|---|---|
| `eggprobe-x86_64-unknown-linux-gnu` (4509015) | `eggprobe-0.1.0-x86_64-unknown-linux-gnu.tar.gz` | 4514536 | `df45c8ecc50817e2d33cc8747b33a1b59baba004f59f115ded2865d1e6ef9b76` MATCH |
| `eggprobe-aarch64-unknown-linux-gnu` (4443375) | `eggprobe-0.1.0-aarch64-unknown-linux-gnu.tar.gz` | 4459657 | `7b85542e70f1a724d8b89a5a589a0c5276849fca87708f9ea75bc1a4de7582f9` MATCH |
| `eggprobe-x86_64-apple-darwin` (4683961) | `eggprobe-0.1.0-x86_64-apple-darwin.tar.gz` | 4693130 | `5092aec18e712802269ae7034087dc771dd7b8a91b64f95dac69d0538411b288` MATCH |
| `eggprobe-aarch64-apple-darwin` (4455419) | `eggprobe-0.1.0-aarch64-apple-darwin.tar.gz` | 4460628 | `3b68440018ddaa414ea2fd8ea92313f63c86e691aa9b8d0b9e36a8bde3bb873c` MATCH |
| `eggprobe-x86_64-pc-windows-msvc` (4165335) | `eggprobe-0.1.0-x86_64-pc-windows-msvc.zip` | 4169388 | `ed81a9cd5cc78faada7f1e1707c4b17df5b6212e5dab4e5c6300b8547ea8c658` MATCH |

Archive integrity: all four `.tar.gz` extracted cleanly via tar listing; the
`.zip` passed `testzip` with no errors.

Archive content inventory (exact; root directory matches
`eggprobe-<version>-<target>` in every case):

- all `.tar.gz`: `<root>/`, `<root>/LICENSE`, `<root>/eggprobe`,
  `<root>/README.md`, `<root>/release-identity.txt`
- `.zip`: `eggprobe-0.1.0-x86_64-pc-windows-msvc/`,
  `eggprobe.exe`, `LICENSE`, `README.md`, `release-identity.txt`

No archive contains source trees, build directories, credentials, or
unrelated files.

`release-identity.txt` is identical in intent across all five targets:

```text
tag=v0.1.0
commit=2760b8b8750272ad448272fa0878ca57bbd2d880
package_version=0.1.0
```

## WP6 — Target-claim reconciliation

Disposition:

- Linux x86_64, macOS x86_64, macOS arm64, Windows x86_64:
  **runtime-smoke-qualified** (native `--version` + machine-output smoke in
  the qualifying hosted run).
- Linux aarch64: **build-qualified only** — successful cross-build on
  `ubuntu-24.04` plus `file` inspection reporting `ELF 64-bit LSB pie
  executable, ARM aarch64`; never executed on the x86_64 builder. No
  Raspberry Pi/SBC runtime qualification is claimed.
- `docs/operator.md` already states exactly this ("Linux aarch64 is
  build-qualified until a native SBC smoke run is recorded"); no
  documentation change was required and none was made. No Eggpack installer,
  Eggup self-update, package-manager, signing/notarization, or SBC claims
  were added.

## Permissions/security review

- Ordinary packaging workflow permissions remain `contents: read` only
  (verified in the final `release.yml`); no publication step exists and none
  was added.
- No Eggpack producer or Eggup consumer/update logic was copied into
  Eggprobe; the diff to `release.yml` is runner selection + shell-safe
  comparison + file-based smoke handoff only.
- `DiagnosticError`/redaction boundaries untouched; report/schema JSON
  contains no `expression` (schema regen produced no diff; schema remains
  `0.3`).
- `cargo +stable audit` clean (215 crates, no advisories) on the tagged
  commit.
- Explicit statement: **no GitHub Release was published by C005**
  (`gh release list` empty after the qualifying run; the workflow has no
  publication capability).

## Acceptance criteria (plan §12)

1. ✅ `macos-13` gone from `release.yml`; Intel job ran on `macos-15`
   (label `macos-15-intel`).
2. ✅ Known actionlint/ShellCheck literal-glob heuristic removed via shell
   variable; YAML/shell checks show no new warnings.
3. ✅ Exact candidate `2760b8b` fully green in ordinary hosted CI
   (run `36030124719`).
4. ✅ One intentional tag `v0.1.0` matches Cargo `0.1.0` and resolves to
   `2760b8b`.
5. ✅ Hosted packaging succeeds for all five declared targets
   (run `36030784594`).
6. ✅ Tag/commit/package identity identical across all jobs/artifacts.
7. ✅ Host-native smoke passes on all four host-native targets.
8. ✅ Linux aarch64 cross-build + `ARM aarch64` architecture inspection pass.
9. ✅ Every archive checksum independently verifies.
10. ✅ Every archive contains only the intended files.
11. ✅ Artifact container/archive names, sizes, and SHA-256 values recorded
    above.
12. ✅ No medium-or-higher packaging/provenance/security finding remains
    (the one workflow defect found was fixed and re-qualified; audit clean).
13. ✅ C001's outstanding hosted evidence explicitly recorded as satisfied
    (see below).
14. ✅ M002 promoted to operationally qualified (see below).
15. ✅ M004 made ready (see below).
16. ✅ No GitHub Release published by C005.

## Downstream disposition

- **C001 (conditionally closed): remaining evidence satisfied.** The
  historical conditional-closure record
  (`plans/closure/release-operational-qualification-corrective/001-status.md`)
  is left intact; this record supplies its outstanding condition: intentional
  valid release tag `v0.1.0` + successful hosted packaging run `36030784594`
  + artifact/hash/native-smoke evidence above.
- **M002: operationally qualified.** Historical conditional closure
  (`plans/closure/release-operational-qualification/002-status.md`) retained;
  its operational gate (C001 hosted evidence) is now met.
- **M004: ready for handoff** (was blocked only on C005 closure). No longer
  blocked.
- **M003a / M003b:** unchanged, still blocked/deferred on their named
  cross-repository readiness gates; neither is a first-release prerequisite
  and C005 introduces no Eggpack/Eggup substitutes.
- No release tag was moved after successful artifact production; any future
  workflow/product defect fix requires a new version/tag (fix-forward), never
  a silent move of `v0.1.0`.

## Roadmap and registry disposition

- `plans/registry.md`: C005 marked closed at `2760b8b` with qualifying run
  `36030784594` and tag `v0.1.0`; C001 operational condition satisfied; M002
  operationally qualified; M004 ready.
- Release subsystem roadmap and corrective addendum: C005 closed with a
  pointer to this record; milestone tables updated accordingly.
- C001–C004 closure records and the M002 historical conditional closure left
  intact as evidence.
- Intended sequence from here: M004 final release-candidate qualification →
  first standalone archive release decision (publication owned by M004, not
  C005).
