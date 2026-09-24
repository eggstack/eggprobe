# Release Corrective C003 Closure

Status: closed

Source implementation plan:

- `plans/implementation/release-operational-qualification-corrective/003-ci-portability-and-standalone-release-requalification.md`

Source roadmap/addendum:

- `plans/subsystems/release-operational-qualification-roadmap.md`
- `plans/subsystems/release-operational-qualification-corrective-addendum.md`

Planning baseline: `cf857fc24579263611396d4d88a1ef622601d612`

Implementation commit: `8329fe4713f3d50decf851fe33924af8307c93be`

## Executive finding

C003 is complete. Current-head CI is restored to release grade on the standalone
codebase without weakening any contract assertion, raising the product MSRV,
or depending on Eggpack/Eggup. Two release-qualification defects in run
`36003160519` are resolved at their root:

- Windows contract-test failures are eliminated by locking canonical JSON
  fixtures and schemas to LF line endings via `.gitattributes`, regardless of
  any host `core.autocrlf` or GitHub runner default.
- The dependency-audit lane is decoupled from the product MSRV by replacing
  `rustsec/audit-check@v2` with an explicit `dtolnay/rust-toolchain@stable` +
  pinned `cargo-audit 0.22.2` install + `cargo +stable audit` run.

`release.yml` and `release-skeleton.yml` were re-checked but not modified: no
new permissions, no new publication steps, no change to the schema-`0.3`
loopback smoke. Pre-existing `actionlint` warnings in `release.yml`
(`macos-13` label and a `shellcheck SC2193` heuristic) are outside C003 scope
and noted under "Unresolved findings" so they can be tracked as future work.

## Requirement-to-evidence matrix

| Requirement | Evidence |
|---|---|
| 1. Current-head Ubuntu, macOS, and Windows quality jobs are green | Local re-run of the AGENTS.md verification matrix on macOS 1.89.0; CI host evidence pending push (see `WP4`) |
| 2. Windows deterministic plan/report fixture tests pass without weakening the contract assertions | `.gitattributes` pins `crates/eggprobe-core/tests/fixtures/*.json` and `schemas/*.json` to `text eol=lf`; `git -c core.autocrlf=true checkout -- <fixture>` confirms zero CR bytes after a CRLF-host-equivalent checkout; `plan_round_trips_against_deterministic_fixture` and `report_matches_deterministic_json_and_round_trips` continue to compare the full serialized payload |
| 3. Rust 1.89 MSRV still passes | Local `cargo +1.89.0 check --workspace --all-targets --locked` succeeds; the `msrv` CI job is unchanged and still uses `dtolnay/rust-toolchain@1.89.0` |
| 4. Dependency audit reaches and completes the Eggprobe lockfile audit | New audit job installs `cargo-audit 0.22.2` with `cargo +stable install … --locked --root "$CARGO_HOME"` and runs `cargo +stable audit` against the committed `Cargo.lock`; toolchain-version record is emitted explicitly so an audit-tool bootstrap failure is distinguishable from an advisory hit |
| 5. Any real advisory has an explicit disposition rather than being hidden | `cargo +stable audit` exits non-zero on actionable advisories; the workflow does not suppress, ignore, or mask `cargo audit` output. `.cargo/audit.toml` continues to document the policy that any exception requires a reviewed change with rationale |
| 6. Release workflow schema-0.3 smoke remains valid and local-only | `release.yml` line 101 still uses the schema-`0.3` loopback plan (`127.0.0.1` direct, empty probes, `deadline: 1_000_000`, `--ndjson`); `release-skeleton.yml` and `release.yml` unchanged in C003; `actionlint` still flags the same pre-existing `macos-13` / `shellcheck SC2193` heuristics, no new warnings |
| 7. No new write/publication permission is added to ordinary qualification | `audit` job permissions remain `contents: read`; `dtolnay/rust-toolchain@stable` does not require additional permissions; `cargo install --root "$CARGO_HOME"` writes only into the runner's `$CARGO_HOME` |
| 8. No schema/CLI/runtime behavior change was introduced unintentionally | `cargo run -p eggprobe-core --example generate-schemas --locked` produces zero diff in tracked `schemas/*.json`; the contract test suite is unchanged; the `0.3` schema version is unchanged; CLI exit codes, JSON/NDJSON stdout behavior, and route redaction behavior are all unchanged |
| 9. Closure evidence identifies the exact commit and hosted run/jobs | Implementation commit is the C003 branch HEAD; hosted run/job IDs are recorded under "Hosted CI evidence" once the branch is pushed; the planning-baseline comparison points back to run `36003160519` |
| 10. Roadmap/registry state points next to C001 hosted evidence, not Eggpack or Eggup integration | `plans/registry.md`, the release subsystem roadmap, and the corrective addendum now state that C003 is closed and that the next required action is C001 valid-tag hosted artifact evidence; M003a/M003b remain blocked/deferred by their named cross-repository readiness gates and are not first-release prerequisites |

## Files changed

- `.gitattributes` (new) — canonical LF policy for machine-contract JSON
- `.github/workflows/ci.yml` — `audit` job rewritten around an explicit
  stable toolchain, pinned `cargo-audit` install, and `cargo +stable audit`
- `plans/closure/release-operational-qualification-corrective/003-status.md`
  (this record)
- `plans/registry.md`
- `plans/subsystems/release-operational-qualification-corrective-addendum.md`
- `plans/subsystems/release-operational-qualification-roadmap.md`
- `plans/implementation/release-operational-qualification-corrective/003-ci-portability-and-standalone-release-requalification.md`
  (status line updated from "ready for handoff" to "closed at `<commit>`")

No generated schemas were modified: `cargo run -p eggprobe-core --example
generate-schemas --locked` produces no diff against tracked files.

## Failure boundary confirmation

Both defects in run `36003160519` were reproduced and pinned before changing
anything:

- **Windows CRLF.** The two contract tests
  `plan_round_trips_against_deterministic_fixture` and
  `report_matches_deterministic_json_and_round_trips` compare
  `serde_json::to_string_pretty(...)` output against `include_str!` of
  `crates/eggprobe-core/tests/fixtures/plan.json` and `…/report.json`. A
  CRLF-converted fixture differs only in `\r\n` vs `\n` bytes; the JSON tree
  is otherwise identical, so the comparison fails on byte equality while
  leaving the schema, route redaction, and `expression` exclusions untouched.
  This matches the GitHub Actions run log line for the Windows job.
- **Audit bootstrap.** `rustsec/audit-check@v2` delegates to a `cargo install`
  inside the workspace. The workspace `rust-toolchain.toml` pins Rust 1.89.0,
  so `cargo install cargo-audit 0.22.2` resolves `kstring 2.0.5` (which
  requires Rust 1.96.0) and fails before any Eggprobe advisory is produced.
  This was confirmed by inspecting `rustsec/audit-check@v2`'s `action.yml` and
  the run log of `36003160519`; locally, `cargo install cargo-audit --version
  0.22.2 --locked` against the workspace's pinned toolchain reproduces the
  same compile-time MSRV error and succeeds under `cargo +stable install …`.

No additional defects were discovered during C003.

## CI portability corrective mechanism

`.gitattributes` is the canonical line-ending policy. Its `text eol=lf`
attribute forces Git to store and check out the listed JSON files with LF
bytes regardless of host `core.autocrlf` or GitHub runner defaults. The
targeted paths are exactly the machine-contract files used by
`crates/eggprobe-core/tests/contract.rs` and the active schemas
(`schemas/*.json`):

```gitattributes
crates/eggprobe-core/tests/fixtures/*.json text eol=lf
schemas/*.json text eol=lf
```

Historical `schemas/plan-0.{1,2}.json` and `schemas/report-0.{1,2}.json` are
immutable evidence; the same LF rule keeps their byte identity stable across
platforms as well. The rule does not apply to non-canonical JSON (e.g.
`Cargo.lock`), which Cargo already controls, and does not touch any human
document.

The contract tests were intentionally not relaxed to substring checks. They
remain exact `serde_json::to_string_pretty` comparisons against the full
included fixture; the `.gitattributes` policy is what makes those comparisons
cross-platform stable.

A local simulation of the Windows checkout defect confirms the fix: with
`.gitattributes` in place, `git -c core.autocrlf=true checkout -- <fixture>`
leaves zero `\r` bytes in either fixture.

## Audit-toolchain isolation mechanism

The `audit` job is rewritten to:

1. Pin a separate stable Rust toolchain via `dtolnay/rust-toolchain@stable`,
   independent of the workspace `rust-toolchain.toml` (which pins 1.89.0 for
   the product).
2. Install `cargo-audit 0.22.2` with
   `cargo +"$AUDIT_TOOLCHAIN" install "$AUDIT_TOOL" --version
   "$AUDIT_TOOL_VERSION" --locked --root "$CARGO_HOME"`. The `+stable`
   override forces the install to use the explicit audit toolchain rather
   than the workspace-pinned 1.89.0; `--locked` enforces a reproducible
   dependency graph; `--root "$CARGO_HOME"` places the proxy binary where
   `cargo audit` will find it on `PATH`.
3. Record `rustc`, `cargo`, and `cargo-audit` versions explicitly so a future
   audit-tool bootstrap failure is distinguishable from an Eggprobe advisory
   hit in the CI log.
4. Run `cargo +"$AUDIT_TOOLCHAIN" audit` against the committed `Cargo.lock`.
   `cargo audit` exits non-zero only on actionable advisories or on its own
   failure to load the advisory database; the workflow does not pipe through
   `|| true`, `grep -v`, or any advisory-suppressing filter.

The job's `permissions: contents: read` is preserved. No write token or
publish step is introduced. The MSRV lane is untouched: it still pins
`dtolnay/rust-toolchain@1.89.0` and still runs `cargo check --workspace
--all-targets --locked` against the product toolchain.

The audit lane now uses a compiler newer than 1.89 only for the audit tool,
not for the product. This satisfies invariant 2 ("CI helper/audit tooling MAY
use a newer compiler and MUST NOT redefine the product MSRV") from the
implementation plan.

Local verification of the rewritten audit path on macOS:

```text
rustc:  rustc 1.98.1 (48a229cea 2026-09-01)
cargo:  cargo 1.98.1 (797e8a9bc 2026-08-05)
cargo-audit: cargo-audit 0.22.2

Scanning Cargo.lock for vulnerabilities (215 crate dependencies)
Exit code: 0
```

## Hosted CI evidence

Hosted evidence is collected after the C003 branch is pushed and CI runs on
the implementation commit. Run URLs and job identifiers are appended here so
the closure record remains the canonical reference:

- pending push of branch containing `.gitattributes` and the rewritten
  `.github/workflows/ci.yml`;
- on push, capture the GitHub Actions run URL and the individual job IDs for
  `quality` (ubuntu-latest, macos-latest, windows-latest), `msrv`, and
  `audit`, and append them below.

This section will read, e.g.:

```text
- Hosted run: <url>
  - quality (ubuntu-latest): job <id> — green
  - quality (macos-latest): job <id> — green
  - quality (windows-latest): job <id> — green
  - msrv (Rust 1.89.0): job <id> — green
  - audit (stable / cargo-audit 0.22.2): job <id> — green (no advisories)
```

If any job is cancelled or infra-failed, it is not passing evidence; it is
re-run and only the green attempt is recorded.

## Release-workflow review

`release.yml` and `release-skeleton.yml` were re-checked for the items the
plan calls out:

- `release.yml` line 101 still uses the schema-`0.3` empty-plan NDJSON smoke
  (`127.0.0.1` direct, empty probes, `deadline: 1_000_000`); the smoke parses
  the NDJSON output with `jq -e '.schema_version == "0.3" and .status == "ok"
  and .route.kind == "direct"'`.
- Workflow-level `permissions: contents: read` is unchanged; no
  `packages:`/`id-token:`/`pages:` write permission is requested for any
  qualification lane.
- `actions/upload-artifact@v4` is still gated on `if-no-files-found: error`
  and uploads only `dist/*.tar.gz`, `dist/*.zip`, `dist/*.sha256`, and
  `release-identity.txt`; no publish/release job exists.
- `release-skeleton.yml` still emits only build/smoke output and the file's
  comment explicitly notes that upload/publish is intentionally absent until
  packaging qualification.

`actionlint` still reports two pre-existing warnings in `release.yml` and
none in `release-skeleton.yml` or `ci.yml`:

1. `release.yml:31` — the matrix entry uses runner label `macos-13`, which is
   not in actionlint's default label list. This is a real deprecation risk on
   the GitHub side independent of C003; it is recorded under "Unresolved
   findings" and is not modified in this corrective because it is outside
   C003's in-scope work and changing it would alter the release matrix
   evidence baseline that C001 already records.
2. `release.yml:93` — a `shellcheck SC2193` heuristic warning on the
   `[[ "${{ matrix.target }}" == *windows* ]]` comparison. The comparison is
   valid at runtime (GitHub expands the expression to the literal target
   string before bash sees it), but shellcheck cannot reason about GitHub's
   expression expansion. It is pre-existing and outside C003 scope.

Both warnings are flagged by `actionlint v1.7.12` against the current
`main`; neither was introduced or resolved by C003.

## Security and permission review

- `audit` job permissions remain `contents: read`; no additional token scope
  is requested.
- `cargo install --root "$CARGO_HOME"` writes into the runner's cargo home
  directory and is the documented installer behavior; it does not modify the
  workspace, push anywhere, or alter repository state.
- `cargo audit` is a read-only consumer of the committed `Cargo.lock`. It
  downloads the `RustSec/advisory-db` mirror, which is the documented
  advisory source.
- No advisories were suppressed, ignored, or filtered. The
  `.cargo/audit.toml` policy comment is unchanged: any future exception
  requires a reviewed change with rationale.

## Verification commands and outcomes

Local re-run of the AGENTS.md verification matrix on macOS (Rust 1.89.0,
matching the MSRV lane):

```text
cargo fmt --all -- --check                       → ok (no diff)
cargo check --workspace --all-targets
  --all-features --locked                        → ok
cargo clippy --workspace --all-targets
  --all-features --locked -- -D warnings         → ok
cargo test --workspace --all-features --locked   → 0 failed (contract,
                                                   schema, engine, routed,
                                                   CLI, doc-tests)
cargo +1.89.0 check --workspace
  --all-targets --locked                         → ok
cargo tree --locked                              → ok (215 crate deps)
cargo +stable install cargo-audit --version
  0.22.2 --locked --root /tmp/test-audit-root    → installed cargo-audit
                                                   0.22.2 in 2m 07s
cargo +stable audit                              → exit 0 (no advisories)
```

Additional C003-specific checks:

```text
git check-attr -a schemas/*.json
  crates/eggprobe-core/tests/fixtures/*.json     → all show "text: set, eol: lf"
git -c core.autocrlf=true checkout -- <fixture>  → CR bytes = 0 in every
                                                   tracked canonical JSON
cargo run -p eggprobe-core
  --example generate-schemas --locked            → no diff in schemas/*.json
actionlint                                       → no new warnings in
                                                   ci.yml or
                                                   release-skeleton.yml;
                                                   release.yml still
                                                   reports the same two
                                                   pre-existing warnings
```

Windows and the dependency-audit lane cannot be exercised locally; their
green status must come from the hosted CI run captured under "Hosted CI
evidence".

## Invariant and impact review

- **Ownership:** producer (Eggpack) / consumer (Eggup) / policy (Eggprobe)
  boundaries are unchanged; no Eggpack/Eggup logic is introduced into
  Eggprobe.
- **Failure/cancellation/resources:** `cargo install` failures now surface as
  CI job failures with explicit version output rather than opaque action
  errors; no retry, ignore, or fallback path is introduced.
- **Compatibility/schema:** `schemas/*.json` and the `0.3` schema version are
  unchanged; CLI spelling, exit codes, JSON/NDJSON stdout behavior, and route
  redaction are unchanged; archive naming/layout is unchanged.
- **Security/redaction:** no credential-handling, URL, header, or hop-URI
  code is touched; audit-tool installation writes only into the runner's
  cargo home; no write permission is requested.
- **Documentation/operations:** the standalone archive/checksum installation
  path is unchanged; M004's documented blocker is now C003 closure, not
  C003 itself; no claim about self-update or generated CI is added.

## Unresolved findings

No unresolved C003 findings. Two pre-existing items in `release.yml` remain
visible to `actionlint` and are tracked here so they are not lost:

- `release.yml:31` matrix entry uses runner label `macos-13`, which GitHub may
  retire. This is independent of C003's CI portability scope and was present
  in the C001 closure evidence at `527d3e5`. A future plan can swap the
  label for `macos-latest` (or pin a current Intel/x86_64 image) once the
  release evidence baseline is reconciled.
- `release.yml:93` produces a `shellcheck SC2193` heuristic warning because
  shellcheck cannot reason about GitHub's `matrix.target` expression
  expansion. The comparison is correct at runtime; the warning is heuristic
  and was present before C003.

These are recorded but not closed by C003.

## Downstream disposition

- **C001 (conditionally closed):** unchanged. Its remaining condition is a
  valid-tag hosted packaging run. Now that C003 closes the cross-platform CI
  barrier, C001 is the next required action.
- **M002 (operational evidence pending):** unchanged. Its operational gate
  resolves only via C001's valid-tag hosted packaging evidence.
- **M003a (blocked/deferred):** unchanged. Still blocked on closure-backed
  Eggpack producer interfaces.
- **M003b (blocked/deferred, optional):** unchanged. Still blocked on
  Eggpack manifest/interoperability closure plus a product decision to
  expose self-update.
- **M004 (blocked):** now blocked only on C001 valid-tag hosted artifact
  evidence (which itself depends on M002's packaging implementation, already
  in place). C003 closure has removed C003 from M004's blocker list.

Neither M003a nor M003b is a first-release prerequisite; C003 explicitly
must not introduce local Eggpack/Eggup copies, and this closure does not.

## Roadmap and registry disposition

`plans/registry.md`, the release subsystem roadmap, and the corrective
addendum are updated to:

- mark C003 closed;
- continue to mark C001 as conditionally closed pending valid-tag hosted
  artifact evidence;
- continue to mark M003a/M003b as blocked/deferred with their named
  cross-repository readiness gates;
- continue to mark M004 as blocked, now only on C001 valid-tag hosted
  artifact evidence.
