# Release and Operational Qualification M002 — Cross-Platform Binary Packaging

Status: conditionally closed

Planning baseline: `e55a21fe73915b7a38c2fa674807354539806c60`; refresh after Release M001 and usable CLI capability close.

Source roadmap:

- `plans/subsystems/release-operational-qualification-roadmap.md#m002--cross-platform-binary-packaging`

Hard dependencies:

- Release M001 closed.
- At least one real diagnostic command exists so artifacts can receive a meaningful smoke test.

Primary class: capability + infrastructure

## 1. Objective

Produce verified standalone Eggprobe archives for the supported target matrix with checksums and release smoke evidence.

## 2. Target matrix

Initial intended targets:

- Linux x86_64 GNU;
- Linux aarch64 GNU for Pi/Le Potato-class systems;
- macOS x86_64;
- macOS arm64;
- Windows x86_64.

Add another target only with an actual build/test/operational support story.

## 3. Required production changes

- release archive layout and deterministic naming;
- binary + license/readme metadata;
- SHA-256 checksums;
- version/tag consistency validation;
- artifact extraction smoke;
- `eggprobe --version`;
- machine-readable JSON smoke using a deterministic no-internet/local fixture;
- release upload only after all required verification;
- source-build fallback documentation.

## 4. SBC requirement

Cross-compilation alone is not sufficient to claim Raspberry Pi/Le Potato operational support. Record at least one native aarch64 Linux target-class smoke before that support claim is promoted from buildable to qualified.

## 5. Tests

- archive names/version;
- checksum verification;
- extraction/path;
- executable starts;
- JSON output parses;
- architecture magic/metadata;
- macOS/Windows/Linux host-native where available;
- Linux aarch64 cross-build plus native smoke evidence.

## 6. Acceptance criteria

Published assets are traceable to a tested tag/commit and have checksums. Supported-target documentation distinguishes built, CI-tested, and target-class runtime-qualified states.

## 7. Stop conditions

Stop if cross-build artifacts cannot be smoke-tested adequately, if native libraries make portability assumptions invalid, or if release upload can occur before verification.

## 8. Closure evidence

Create `plans/closure/release-operational-qualification/002-status.md` with target table, workflow run IDs, artifact hashes, smoke outputs, SBC evidence, and unresolved target limitations.
