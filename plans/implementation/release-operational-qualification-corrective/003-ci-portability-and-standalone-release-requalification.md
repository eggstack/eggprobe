# Release and Operational Qualification Corrective C003 — CI Portability and Standalone Release Requalification

Status: closed; closure record at `plans/closure/release-operational-qualification-corrective/003-status.md`

Planning baseline: `cf857fc24579263611396d4d88a1ef622601d612`

Source roadmap:

- `plans/subsystems/release-operational-qualification-roadmap.md`
- `plans/subsystems/release-operational-qualification-corrective-addendum.md`
- `plans/002-long-term-roadmap.md#phase-7--release-packaging-and-operational-qualification`

Historical/related evidence:

- `plans/closure/release-operational-qualification/001-status.md`
- `plans/closure/release-operational-qualification/002-status.md`
- `plans/closure/release-operational-qualification-corrective/001-status.md`
- `plans/closure/release-operational-qualification-corrective/002-status.md`
- GitHub Actions run `36003160519` on the planning baseline

Applicable ADRs:

- `plans/adrs/ADR-0001-json-first-core-and-transport-ownership.md`
- `plans/adrs/ADR-0002-release-producer-consumer-ownership.md`

Primary class: infrastructure + invariant

## 1. Objective

Restore a release-grade, cross-platform CI baseline for the standalone Eggprobe
release path before a first release tag is selected and used for C001 hosted
artifact qualification.

The corrective must make deterministic machine-contract fixtures portable
across Git checkout newline behavior and must ensure dependency auditing
actually audits Eggprobe's lockfile instead of failing while bootstrapping the
audit tool under the product MSRV.

This plan deliberately works around current Eggpack/Eggup instability by not
depending on either project for the first release. It MUST NOT implement local
copies of Eggpack producer machinery or Eggup update/rollback machinery.

## 2. Readiness and dependencies

Hard dependencies:

- Foundation diagnostic contract corrective work is closed.
- Transport/probe-engine corrective work is closed.
- CLI/automation corrective work is closed.
- Release corrective C002 is closed.

Operational dependencies:

- none for implementation;
- a release tag is explicitly NOT required for C003.

Downstream:

- C001's remaining hosted packaging evidence MUST wait for C003 closure;
- M004 remains blocked until C003 closes and C001/M002 hosted artifact evidence
  is complete.

C003 is ready now.

## 3. Current implementation evidence

GitHub Actions run `36003160519` at
`cf857fc24579263611396d4d88a1ef622601d612` provides the failure baseline.

Passing lanes/evidence:

- Ubuntu format, strict Clippy, and full test suite pass.
- macOS format, strict Clippy, and full test suite pass.
- Rust 1.89.0 MSRV `cargo check --workspace --all-targets --locked` passes.
- Windows format and strict Clippy pass.
- Windows reaches the full test suite; ordinary CLI/TLS tests pass before the
  contract fixture failures.

Windows failure:

- `crates/eggprobe-core/tests/contract.rs` compares
  `serde_json::to_string_pretty(...)` output directly with checked-in
  `include_str!` JSON fixtures.
- On the Windows runner the checked-in fixtures contain CRLF after checkout,
  while generated JSON contains LF.
- `plan_round_trips_against_deterministic_fixture` and
  `report_matches_deterministic_json_and_round_trips` therefore fail on
  newline bytes despite equivalent JSON content.
- This is a portability defect in canonical fixture handling, not evidence of
  a different serialized data model.

Audit failure:

- the `rustsec/audit-check@v2` job attempts to compile
  `cargo-audit 0.22.2` under Rust 1.89.0;
- the resolved tool dependency `kstring 2.0.5` requires Rust 1.96.0;
- the job exits while building the audit tool and never produces an Eggprobe
  advisory result.

Release workflow state:

- current release smoke uses schema `0.3`;
- standalone packaging remains the first-release path;
- C001 remains conditionally closed pending valid-tag hosted artifact evidence.

## 4. Invariants

The implementation MUST preserve all of the following:

1. Rust 1.89 remains the Eggprobe product MSRV unless a separate accepted plan
   changes it.
2. CI helper/audit tooling MAY use a newer compiler and MUST NOT redefine the
   product MSRV.
3. Canonical JSON schema/fixture semantics remain deterministic.
4. A Windows newline fix MUST NOT weaken schema/version/redaction assertions or
   turn exact contract tests into superficial substring tests.
5. JSON/NDJSON stdout behavior remains unchanged.
6. Release workflow smoke remains local/deterministic and must not require
   public network access.
7. Release artifacts still correspond to an explicit tag/source/version
   identity before C001 evidence is accepted.
8. Eggpack remains the future producer owner and Eggup remains the optional
   future consumer/update owner under ADR-0002.
9. No Eggpack/Eggup code is copied into Eggprobe as a temporary workaround.
10. No release/tag/publication is performed by C003 itself.

## 5. In scope

### 5.1 Canonical newline policy for machine fixtures

Establish an explicit repository policy for JSON contract/schema fixture line
endings so Windows checkout cannot mutate canonical fixture bytes underneath
exact serialization tests.

Preferred direction:

- add a narrow `.gitattributes` rule that forces LF for checked-in JSON
  schemas and deterministic JSON fixtures used as machine contracts;
- keep exact deterministic serialization comparisons when practical;
- if test-side normalization is still required for a platform edge case,
  normalize only line-ending representation and continue comparing the complete
  serialized payload.

Do not replace full contract comparisons with partial-field assertions.

Review at minimum:

- `crates/eggprobe-core/tests/fixtures/*.json`;
- `schemas/*.json`;
- generated schema/fixture regeneration behavior;
- any release smoke fixture embedded in workflow YAML.

### 5.2 Audit-toolchain isolation

Make the dependency-audit lane use a toolchain suitable for the audit tool
while auditing the repository's committed `Cargo.lock`.

Requirements:

- the audit lane must not accidentally inherit Rust 1.89 solely because the
  repository product toolchain is pinned there;
- use an explicit, reproducible audit-tool installation/execution path;
- prefer a maintained stable toolchain for CI tooling and a locked/pinned
  `cargo-audit` installation where that improves reproducibility;
- the job must clearly distinguish "audit tool failed to install/run" from
  "Eggprobe dependency advisory found";
- preserve read-only repository permissions;
- do not suppress advisories merely to make CI green.

If the existing action cannot reliably satisfy these requirements, replacing
that action with an explicit install-and-run sequence is in scope.

### 5.3 Current-head CI requalification

After corrections:

- run the ordinary quality matrix on Ubuntu, macOS, and Windows;
- run the Rust 1.89 MSRV lane;
- run the dependency audit and prove it reaches an actual lockfile audit;
- preserve strict Clippy;
- preserve `--locked` product verification;
- re-check release workflow syntax and schema-0.3 machine-output smoke;
- ensure no release packaging behavior regresses while fixing CI.

### 5.4 Planning/qualification reconciliation

At closure:

- record the implementation commit and successful hosted CI run IDs;
- record Windows contract-test evidence;
- record the audit command/toolchain/tool version and audit outcome;
- update the active registry's qualified/current baseline language;
- leave C001 conditionally closed until valid-tag hosted artifacts exist;
- leave M004 blocked until C001/M002 operational evidence is subsequently
  complete.

## 6. Out of scope

- creating or pushing a release tag;
- publishing GitHub Releases;
- collecting C001 final hosted artifact hashes;
- executing M004 final release qualification;
- adopting Eggpack;
- implementing Eggup self-update;
- adding package-manager distribution;
- changing schema semantics or bumping schema version merely for newline
  handling;
- changing product MSRV;
- Phase 8 ICMP/traceroute/MTU/UDP work;
- Phase 9 QUIC/H3 work.

If fixing the failures requires one of those changes, stop and register
separate work.

## 7. Ordered work packages

### WP1 — Reproduce and pin the failure boundary

1. Re-run or inspect the current Windows and audit failures.
2. Confirm the two Windows failures are newline-only by comparing normalized
   fixture contents without changing the test contract.
3. Confirm the audit job fails before an Eggprobe dependency audit result is
   produced.
4. Record runner/toolchain versions relevant to the failure.

### WP2 — Make JSON fixtures checkout-portable

1. Add the narrow canonical line-ending policy.
2. Re-normalize affected tracked fixture/schema files if required.
3. Preserve exact JSON serialization expectations.
4. Add/adjust regression coverage only as needed to prove CRLF checkout cannot
   reproduce the failure.
5. Re-run schema generation and require no semantic schema drift.

### WP3 — Isolate audit tooling from product MSRV

1. Select the explicit CI toolchain used to build/run `cargo-audit`.
2. Make audit-tool installation reproducible enough for CI.
3. Run the audit against the committed workspace lockfile.
4. Preserve failure on real actionable advisories.
5. Keep the separate Rust 1.89 MSRV lane unchanged.

### WP4 — Cross-platform and release-workflow regression pass

1. Run format/check/Clippy/test locally where available.
2. Push and obtain green Ubuntu/macOS/Windows hosted matrix results.
3. Obtain green MSRV and dependency-audit jobs.
4. Verify the release workflow still uses schema `0.3` and local-only machine
   smoke.
5. Verify workflow changes do not add write permissions or publication steps.

### WP5 — Closure and handoff

1. Create
   `plans/closure/release-operational-qualification-corrective/003-status.md`.
2. Include a requirement-to-evidence matrix and hosted run/job identifiers.
3. Update roadmap/registry status from C003 ready/active to closed.
4. Mark the next action as C001 valid-tag hosted packaging evidence.
5. Do not mark M002 or M004 closed in this corrective.

## 8. Failure, cancellation, restart, and contention semantics

CI jobs remain independently restartable.

A cancelled/infra-failed hosted job is not passing evidence. Re-run it and
record the successful attempt.

If GitHub runner newline behavior changes, the repository policy remains the
authority; do not special-case a single runner image.

If audit tooling becomes temporarily unavailable upstream, keep the lane
failing/blocked rather than silently skipping the audit. A time-bounded,
documented exception would require separate review.

Concurrent changes to schemas, fixtures, `Cargo.lock`, MSRV, or release
workflows invalidate cached C003 evidence and require requalification at the
new head.

## 9. Compatibility and migration

Expected public compatibility effect: none.

- no schema version bump is expected;
- no CLI spelling or exit-code change is expected;
- no report field change is expected;
- no route behavior change is expected;
- no archive naming/layout change is expected.

A `.gitattributes` addition is repository hygiene, not a machine-contract
change, provided regenerated semantic JSON remains identical.

The audit lane may use a compiler newer than 1.89; this does not change the
documented source-build MSRV because the product MSRV lane remains explicit and
must continue to pass.

## 10. Required tests and evidence

At minimum:

- Windows:
  - `cargo test --workspace --all-features --locked`;
  - both deterministic contract fixture tests pass.
- Linux/macOS:
  - existing quality matrix remains green.
- MSRV:
  - `cargo +1.89.0 check --workspace --all-targets --locked` or the equivalent
    hosted lane passes.
- Audit:
  - `cargo audit` actually runs against the current `Cargo.lock`;
  - record audit tool version and compiler/toolchain used to run it.
- Contract/schema:
  - deterministic plan/report fixture tests pass;
  - schema generation produces no unintended semantic drift.
- Release workflow:
  - YAML is valid;
  - schema `0.3` smoke remains parseable;
  - no public-network dependency is introduced;
  - permissions remain read-only for qualification/build jobs.

Broad verification command family:

```text
cargo fmt --all -- --check
cargo check --workspace --all-targets --all-features --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test --workspace --all-features --locked
cargo +1.89.0 check --workspace --all-targets --locked
cargo tree --locked
cargo audit
```

The implementation may run `cargo audit` under the separately selected audit
toolchain; record that distinction explicitly.

## 11. Documentation updates

Update only documentation made stale by the actual correction.

Required planning updates at closure:

- this corrective status;
- corrective addendum;
- release subsystem roadmap;
- `plans/registry.md`;
- C003 closure record.

Do not add Eggpack/Eggup installation/update claims.

## 12. Acceptance criteria

C003 closes only when all are true:

1. current-head Ubuntu, macOS, and Windows quality jobs are green;
2. Windows deterministic plan/report fixture tests pass without weakening the
   contract assertions;
3. Rust 1.89 MSRV still passes;
4. dependency audit reaches and completes the Eggprobe lockfile audit;
5. any real advisory has an explicit disposition rather than being hidden;
6. release workflow schema-0.3 smoke remains valid and local-only;
7. no new write/publication permission is added to ordinary qualification;
8. no schema/CLI/runtime behavior change was introduced unintentionally;
9. closure evidence identifies the exact commit and hosted run/jobs;
10. roadmap/registry state points next to C001 hosted evidence, not Eggpack or
    Eggup integration.

## 13. Stop conditions

Stop and register separate work if:

- Windows failures reveal actual serialization/schema divergence rather than
  newline conversion;
- a real RustSec advisory requires a dependency upgrade with non-trivial
  product compatibility impact;
- fixing audit requires raising Eggprobe's product MSRV;
- release smoke uncovers a runtime/network correctness regression;
- a schema bump becomes necessary;
- the standalone release path would require implementing Eggpack/Eggup-owned
  machinery locally.

## 14. Closure evidence

Create:

- `plans/closure/release-operational-qualification-corrective/003-status.md`

The closure record MUST contain:

- baseline and implementation commit;
- hosted CI run and relevant job identifiers;
- Windows fixture root cause and exact corrective mechanism;
- audit-toolchain/tool version and completed audit result;
- schema/contract regression results;
- release-workflow review;
- security/permission review;
- unresolved findings, if any;
- explicit downstream disposition:
  C001 hosted valid-tag evidence next, then M004.

## 15. Handoff notes

This is the immediate next implementation plan.

Do not create a first-release tag merely to satisfy C001 while C003 is open.
Do not wait for Eggpack/Eggup stabilization. The intended first release remains
a standalone archive/checksum release under the existing Eggprobe workflow,
with Eggpack producer adoption and Eggup self-update deferred until their
interfaces are closure-backed and a later product decision adopts them.
