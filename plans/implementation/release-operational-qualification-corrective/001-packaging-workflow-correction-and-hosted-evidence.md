# Release Corrective C001 — Packaging Workflow Correction and Hosted Evidence

Status: ready for handoff

Planning baseline: `24965aa0ba2696b201e9c74531920877529821d5`

Source addendum:

- `plans/subsystems/release-operational-qualification-corrective-addendum.md#2-c001--packaging-workflow-correction-and-hosted-evidence`

Primary class: correctness + release qualification

## 1. Objective

Correct the cross-platform packaging workflow, then replace static workflow inspection with real hosted build/artifact evidence.

## 2. Version/ref authority

Choose one authoritative release identity.

Preferred manual workflow contract:

- required input is an existing tag such as `v0.1.0`;
- checkout that exact tag/ref;
- derive archive version from the checked-out tag;
- verify it matches Cargo package version before building;
- fail closed on mismatch.

Alternatively use tag-push only. Do not accept a version input that is ignored.

## 3. Cross-architecture smoke behavior

Do not execute a Linux aarch64 binary on an x86_64 runner.

For cross-built artifacts:

- inspect architecture/file metadata;
- archive/checksum successfully;
- label as build-qualified only.

Runtime smoke for Linux aarch64 requires a native/self-hosted aarch64/SBC lane and remains a separate evidence condition until available.

Host-native macOS/Linux x86_64/Windows artifacts should execute `--version` and a no-public-network machine-output smoke.

## 4. Archive/checksum correction

Package exactly one versioned archive per matrix target.

Generate SHA-256 for archive files only, never directories. Prefer target-local manifest generation with explicit archive filename rather than globbing the whole `dist` directory.

Verify the checksum immediately after creation.

## 5. Artifact layout

Each target artifact should contain:

- versioned archive;
- checksum manifest scoped clearly enough to avoid ambiguous duplicate filenames;
- optional metadata file recording commit/tag/target.

Do not upload unrelated intermediate directories unless intentionally documented.

## 6. Hosted evidence

After workflow correction:

- run the workflow through the supported trigger;
- record run IDs and job conclusions;
- inspect uploaded artifact names/sizes;
- retrieve artifact metadata where connector access permits;
- record archive hashes;
- demonstrate native `--version`/JSON smoke on host-native targets;
- keep Linux aarch64 explicitly build-qualified until native target evidence exists.

If a hosted runner/toolchain failure is external, record it as an operational blocker rather than marking the corrective closed.

## 7. CI/release permissions

Maintain least privilege:

- ordinary packaging does not need release write permission;
- any later GitHub Release publication should be a separate gated job with explicit `contents: write`;
- PR workflows never need release secrets.

## 8. Required verification

- workflow syntax;
- tag/Cargo version mismatch negative case;
- host-native version/JSON smoke;
- cross-arch no-execution behavior;
- archive extraction;
- checksum verify;
- artifact inspection;
- Rust 1.89 source build remains independently qualified.

## 9. Acceptance criteria

- manual/tag release identity cannot drift from built source/version;
- Linux aarch64 cross-build does not attempt native execution on x86_64;
- checksum generation succeeds and covers archives only;
- at least one hosted packaging workflow produces inspectable artifacts;
- support documentation distinguishes host-native qualified from build-only targets;
- no unresolved medium-or-higher release workflow finding remains.

## 10. Closure evidence

Create `plans/closure/release-operational-qualification-corrective/001-status.md` with exact workflow run IDs, per-target results, hashes, archive contents, smoke results, permission review, and remaining SBC-native evidence condition.
