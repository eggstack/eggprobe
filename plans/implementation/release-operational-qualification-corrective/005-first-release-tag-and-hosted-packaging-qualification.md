# Release and Operational Qualification Corrective C005 — First-Release Tag and Hosted Packaging Qualification

Status: ready for handoff

Planning baseline: `ff789d4c02d112ffb8909ba90f198ceebce1b6ad`

Source roadmap:

- `plans/subsystems/release-operational-qualification-roadmap.md`
- `plans/subsystems/release-operational-qualification-corrective-addendum.md`
- `plans/002-long-term-roadmap.md#phase-7--release-packaging-and-operational-qualification`

Historical/related evidence:

- `plans/implementation/release-operational-qualification/002-cross-platform-binary-packaging.md`
- `plans/closure/release-operational-qualification/002-status.md`
- `plans/implementation/release-operational-qualification-corrective/001-packaging-workflow-correction-and-hosted-evidence.md`
- `plans/closure/release-operational-qualification-corrective/001-status.md`
- `plans/closure/release-operational-qualification-corrective/004-status.md`
- hosted CI run `36016882249` on the planning baseline

Applicable ADRs:

- `plans/adrs/ADR-0001-json-first-core-and-transport-ownership.md`
- `plans/adrs/ADR-0002-release-producer-consumer-ownership.md`

Primary class: release qualification + invariant

## 1. Objective

Qualify Eggprobe's first standalone binary release artifacts against one
intentional, immutable source/tag/version identity and close the only remaining
operational evidence gap from Release M002 / corrective C001.

C005 must:

1. remove known release-workflow runner/lint risks before any release tag is
   created;
2. prove the resulting candidate commit is green in ordinary hosted CI;
3. create exactly one intentional release tag matching Cargo's release version;
4. run the hosted packaging workflow against that exact tag;
5. inspect every uploaded artifact, checksum, archive layout, and
   `release-identity.txt`;
6. record native smoke evidence for host-native targets and truthful
   build-only status for Linux aarch64;
7. close C001's remaining operational condition and promote M002 to
   operationally qualified;
8. hand off to M004 without publishing a GitHub Release in this plan.

At the planning baseline the workspace version is `0.1.0`, so the expected
first release tag is `v0.1.0`. If the workspace version changes before
execution, derive the tag from the current Cargo version and update planning
evidence before tagging; do not force a mismatched tag through the workflow.

## 2. Readiness and dependencies

Hard dependencies:

- C002 is closed.
- C003's original CRLF/audit findings are closed historically.
- C004 is closed with fully green hosted cross-platform CI.
- Current `main` `ff789d4c02d112ffb8909ba90f198ceebce1b6ad` is fully
  green in hosted run `36016882249`.
- The standalone packaging implementation and C001 workflow corrections are
  present.

External operational fact reviewed for planning on 2026-09-24:

- current GitHub-hosted runner documentation lists standard Intel macOS labels
  `macos-15-intel` and `macos-26-intel`;
- `.github/workflows/release.yml` still uses `macos-13` for the
  `x86_64-apple-darwin` package job;
- therefore runner-label maintenance is required before the first tag.

Operational dependencies:

- GitHub Actions must be available for the packaging run.
- No Eggpack/Eggup interface is required.
- No native Linux aarch64 runner is required to close C005; that target remains
  build-qualified only unless separate native evidence is available.

C005 is ready now.

## 3. Current release contract

Workspace:

- package version: `0.1.0`;
- Rust edition: 2021;
- product MSRV: Rust 1.89;
- schema version: `0.3`.

Release workflow target matrix:

- `x86_64-unknown-linux-gnu` — host-native smoke;
- `aarch64-unknown-linux-gnu` — cross-build + architecture inspection only;
- `x86_64-apple-darwin` — host-native smoke;
- `aarch64-apple-darwin` — host-native smoke;
- `x86_64-pc-windows-msvc` — host-native smoke.

Workflow authority rules already implemented by C001:

- manual dispatch requires an existing `vX.Y.Z` tag;
- workflow checks out exactly that tag with full history;
- `git describe --tags --exact-match HEAD` must equal the requested tag;
- Cargo package version must equal the tag version without the `v` prefix;
- each matrix job emits `release-identity.txt` with tag, commit, and package
  version;
- host-native jobs run `eggprobe --version` and schema-0.3 local NDJSON smoke;
- Linux aarch64 is never executed on the x86_64 builder;
- each target emits exactly one versioned archive and one SHA-256 file;
- checksums are verified immediately;
- ordinary release packaging has `contents: read` only and performs no
  publication.

## 4. Invariants

The implementation MUST preserve:

1. One source commit, one Cargo version, and one release tag identify all
   qualified artifacts.
2. A release tag MUST NOT be moved after successful hosted packaging evidence
   exists.
3. A tag/version mismatch fails closed.
4. No artifact may be qualified from a dirty or different source tree than the
   tag points to.
5. All host-native targets execute both `--version` and machine-output smoke.
6. Linux aarch64 remains explicitly build-qualified, not runtime-qualified,
   unless native aarch64/SBC evidence is separately captured.
7. Checksums cover archive files, not staging directories.
8. Archive contents are deterministic in shape and contain only the intended
   binary + release metadata/documentation.
9. The schema remains 0.3 for this release unless a separate plan changes it
   before tagging.
10. Rust 1.89 remains the product MSRV.
11. Ordinary packaging remains read-only and does not create/publish a GitHub
    Release.
12. No Eggpack producer or Eggup consumer/update logic is copied into Eggprobe.
13. Failed or cancelled hosted runs are not qualification evidence.
14. M004 does not begin until C005 closure records successful artifacts.

## 5. In scope

### 5.1 Pre-tag release-workflow hygiene

Before creating the first tag:

- replace the stale `macos-13` runner used for
  `x86_64-apple-darwin` with `macos-15-intel`;
- if GitHub's documented standard Intel label changes before execution, use the
  current documented stable Intel label and record it in closure evidence;
- preserve `x86_64-apple-darwin` as the target triple;
- remove the known actionlint/ShellCheck SC2193 heuristic around the Windows
  target comparison by assigning the matrix target to a shell variable before
  comparison or an equivalently clear shell-safe formulation;
- run workflow syntax/actionlint checks and require no new warnings; the
  release workflow should be warning-clean if the pre-existing two findings
  are fully resolved;
- do not alter archive names, target triples, release identity semantics, or
  permission scope merely for cleanup.

If runner migration requires material packaging changes beyond the label,
stop and register separate corrective work.

### 5.2 Candidate commit qualification before tag creation

After any release-workflow hygiene change:

- run the full ordinary hosted CI matrix;
- require Ubuntu/macOS/Windows quality, Rust 1.89 MSRV, and dependency audit all
  green on the exact candidate commit;
- rerun schema generation and require no unintended schema diff;
- confirm Cargo workspace/package version;
- confirm no GitHub Release exists for the candidate version;
- confirm the intended tag is not already attached to a different commit.

Do not tag a candidate with failing or incomplete ordinary CI.

### 5.3 Release identity and tag creation

Expected identity while Cargo remains `0.1.0`:

```text
tag: v0.1.0
package version: 0.1.0
commit: <fully-green candidate commit>
```

Create the tag only after WP1/WP2 pass.

Tag policy:

- the tag identifies the exact candidate commit;
- do not force-move it after a successful packaging run;
- do not reuse a tag that points at another commit;
- if the tag exists unexpectedly, inspect it and stop on mismatch;
- if a tagged packaging attempt exposes a repository defect after artifacts
  have been successfully produced/uploaded, fix forward with a new version/tag
  rather than silently moving the old tag;
- any exceptional deletion/recreation before artifact production must be
  explicitly recorded and must not occur if a GitHub Release or successful
  distributable artifact set has been published.

### 5.4 Hosted packaging execution

Dispatch `.github/workflows/release.yml` with the exact release tag.

Require all matrix jobs to complete successfully:

- Linux x86_64;
- Linux aarch64;
- macOS x86_64;
- macOS arm64;
- Windows x86_64.

For every job record:

- job/run ID;
- runner label;
- checked-out tag;
- checked-out commit;
- Cargo package version;
- build conclusion;
- smoke conclusion where host-native;
- archive name;
- checksum name;
- uploaded artifact name.

Do not accept a partial matrix as release qualification.

### 5.5 Artifact download and independent inspection

Download every uploaded GitHub Actions artifact.

For each target:

1. record artifact container name and size;
2. locate the versioned archive and its checksum;
3. independently recompute SHA-256 and compare with the emitted checksum file;
4. test archive integrity;
5. list archive contents;
6. verify the root directory/archive naming matches
   `eggprobe-<version>-<target>`;
7. verify contents include only:
   - `eggprobe` or `eggprobe.exe`;
   - `LICENSE`;
   - `README.md`;
   - `release-identity.txt`;
8. inspect `release-identity.txt` and require the same tag/commit/package
   version in every target archive and top-level artifact metadata;
9. record archive byte size and SHA-256 in closure evidence.

No artifact should contain source trees, build directories, credentials, or
unrelated files.

### 5.6 Smoke and target-claim evidence

Hosted workflow evidence is authoritative for:

- Linux x86_64 native `--version` + schema-0.3 NDJSON smoke;
- macOS x86_64 native smoke;
- macOS arm64 native smoke;
- Windows x86_64 native smoke.

Linux aarch64:

- require successful cross-build;
- require architecture inspection to report aarch64/arm64;
- verify archive/checksum like other targets;
- label it **build-qualified only**;
- do not claim Raspberry Pi/SBC runtime qualification from cross-build evidence.

If a native aarch64/SBC test is available during execution, it may be recorded
as additional evidence, but C005 closure must not depend on it.

### 5.7 Operational promotion

On successful evidence:

- create
  `plans/closure/release-operational-qualification-corrective/005-status.md`;
- record C001's remaining hosted evidence as satisfied;
- update planning state so Release M002 is **operationally qualified**;
- keep historical C001 and M002 closure records intact;
- update registry/roadmap/addendum;
- mark M004 ready for handoff;
- do not publish a GitHub Release in C005.

## 6. Out of scope

- GitHub Release publication;
- package-manager publication;
- Eggpack adoption;
- Eggup self-update;
- installer/bootstrap generation;
- signing/notarization unless separately planned;
- native Linux aarch64/SBC qualification as a closure requirement;
- changing diagnostic behavior;
- changing schema semantics;
- raising MSRV;
- Phase 8/9 feature work.

Publication is deliberately deferred to the final M004 qualification decision
so artifacts are inspected before becoming the broadly consumable release.

## 7. Ordered work packages

### WP1 — Release-workflow preflight correction

1. Inspect current GitHub-hosted runner documentation.
2. Change Intel macOS packaging runner from `macos-13` to
   `macos-15-intel` (or the current supported stable Intel label if it has
   changed).
3. Rewrite the shell target comparison to remove the known SC2193 heuristic.
4. Run YAML/actionlint/shell syntax checks.
5. Confirm permissions remain `contents: read`.
6. Confirm artifact contract is unchanged.

### WP2 — Qualify the exact candidate commit

1. Run the full ordinary hosted CI matrix.
2. Require all jobs green.
3. Verify schema regeneration produces no unintended diff.
4. Confirm Cargo release version.
5. Confirm intended release tag/release identity is unused or correctly points
   to the exact candidate.
6. Record the candidate commit.

### WP3 — Create immutable candidate tag

1. Derive tag from Cargo version.
2. Expected first tag at current baseline: `v0.1.0`.
3. Create tag at the exact WP2 candidate commit.
4. Fetch/inspect the tag and prove it resolves back to that commit.
5. Do not move it after successful hosted artifact generation.

### WP4 — Dispatch and qualify hosted packaging

1. Dispatch `Release packages` using the exact tag.
2. Wait for the matrix to complete.
3. Record run/job IDs and per-target conclusions.
4. On failure:
   - classify runner/infrastructure vs repository/workflow defect;
   - do not treat failed/cancelled jobs as evidence;
   - do not force-move an already-qualified tag;
   - register separate corrective work if the fix exceeds C005's narrow
     release-workflow portability scope.

### WP5 — Download and inspect artifacts

1. Download all five target artifact bundles.
2. Recompute checksums independently.
3. Verify archive integrity and exact contents.
4. Verify release identity is identical across all targets.
5. Record archive size/hash/target.
6. Verify no unintended files or secrets are present.

### WP6 — Reconcile target claims

1. Mark host-native targets runtime-smoke-qualified.
2. Mark Linux aarch64 build-qualified only.
3. Ensure README/operator supported-target wording does not exceed evidence.
4. Do not imply SBC runtime qualification without native evidence.

### WP7 — Closure and M004 handoff

1. Create C005 closure record.
2. Update registry/roadmap/addendum.
3. Mark C001's remaining evidence satisfied.
4. Mark M002 operationally qualified.
5. Change M004 from blocked to ready.
6. Preserve M003a/M003b deferred status.
7. Stop before publication.

## 8. Failure, cancellation, restart, and tag semantics

A cancelled workflow is not evidence.

An infrastructure failure may be rerun against the same immutable tag if the
source/workflow content being executed is unchanged and the failure is clearly
external.

A repository/workflow defect discovered after tag creation requires explicit
disposition:

- if no successful artifact set has been produced and no GitHub Release has
  been published, maintainers may decide whether the unused tag can be deleted
  and recreated; record that decision and all failed run IDs;
- once successful artifacts have been produced against the tag, do not
  force-move it;
- if code/workflow changes are required after successful artifact production,
  bump version and qualify a new tag.

Never overwrite evidence from a failed run to make it appear successful.

## 9. Compatibility and migration

Expected public product compatibility effect: none.

The planned release-workflow changes affect runner selection/lint cleanliness,
not binary behavior or artifact contract.

Expected release identity at current baseline is `v0.1.0` / package
`0.1.0`. A version change before execution is allowed only if the workspace
and tag remain aligned and planning evidence is refreshed.

## 10. Required verification

Pre-tag:

```text
cargo fmt --all -- --check
cargo check --workspace --all-targets --all-features --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test --workspace --all-features --locked
cargo +1.89.0 check --workspace --all-targets --locked
cargo +stable audit
cargo run -p eggprobe-core --example generate-schemas --locked
actionlint .github/workflows/ci.yml .github/workflows/release.yml
```

Hosted candidate CI:

- Ubuntu quality green;
- macOS quality green;
- Windows quality green;
- MSRV green;
- audit green.

Hosted release packaging:

- all five matrix jobs green;
- host-native `--version` smoke green;
- host-native schema-0.3 NDJSON smoke green;
- Linux aarch64 architecture inspection green;
- all uploads present.

Artifact verification:

- archive integrity;
- independent SHA-256 match;
- exact archive contents;
- consistent release identity;
- recorded size/hash metadata.

## 11. Documentation updates

Before closure, reconcile only claims affected by evidence:

- supported-target matrix;
- manual archive installation instructions;
- Linux aarch64 build-only wording;
- release qualification status.

Do not add:

- Eggpack installer claims;
- Eggup self-update claims;
- package-manager installation claims;
- native SBC qualification unless actually demonstrated.

## 12. Acceptance criteria

C005 closes only when all are true:

1. release workflow no longer depends on stale `macos-13`;
2. release workflow lint/shell hygiene is clean enough that known pre-existing
   warnings are resolved or explicitly justified;
3. exact candidate commit has fully green ordinary hosted CI;
4. one intentional release tag matches Cargo version and candidate commit;
5. hosted packaging succeeds for all five declared targets;
6. tag/commit/package identity is identical across all jobs/artifacts;
7. host-native smoke passes on Linux x86_64, macOS x86_64, macOS arm64, and
   Windows x86_64;
8. Linux aarch64 cross-build and architecture inspection pass;
9. every archive checksum independently verifies;
10. every archive contains only the intended files;
11. artifact names, sizes, and SHA-256 values are recorded;
12. no medium-or-higher packaging/provenance/security finding remains;
13. C001's outstanding hosted evidence is explicitly recorded as satisfied;
14. M002 is promoted to operationally qualified;
15. M004 is made ready;
16. no GitHub Release is published by C005.

## 13. Stop conditions

Stop and register separate work if:

- release workflow changes require altering the artifact contract materially;
- tag/Cargo/source identity cannot be made consistent;
- a host-native binary fails machine-output smoke;
- archive checksum or contents are inconsistent;
- a real product/runtime defect appears;
- a real security advisory appears;
- current GitHub runner availability requires dropping an advertised target;
- release publication/signing/notarization becomes necessary to continue;
- Eggpack/Eggup integration would be required.

## 14. Closure evidence

Create:

- `plans/closure/release-operational-qualification-corrective/005-status.md`

The closure record MUST include:

- planning baseline;
- workflow-hygiene implementation commit;
- exact fully-green candidate commit;
- tag name and tag-to-commit proof;
- ordinary CI run/jobs for the candidate;
- release packaging run and all matrix job IDs;
- per-target runner labels;
- per-target smoke results;
- artifact container names;
- archive filenames;
- archive sizes;
- independently verified SHA-256 values;
- archive content inventory;
- release-identity contents/proof across targets;
- Linux aarch64 build-only disposition;
- permissions/security review;
- any failed/cancelled precursor run IDs and dispositions;
- explicit statement that no GitHub Release was published;
- explicit C001/M002 promotion;
- downstream disposition: M004 ready.

## 15. Handoff notes

This is the immediate next release plan.

Do not skip the pre-tag workflow hygiene because a tag should not be created
against a release workflow with a known stale runner label.

Do not publish the release during this plan. The goal is to produce and inspect
a trustworthy artifact set first. M004 owns the final release-candidate
qualification and operator-document reconciliation immediately afterward.

Eggpack M003a and Eggup M003b remain deferred and outside the first-release
critical path.
