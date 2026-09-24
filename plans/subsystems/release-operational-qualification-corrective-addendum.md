# Release and Operational Qualification — Post-Closure Corrective Addendum

Status: active; C001 conditionally closed; C002 closed; C003 closed

Historical planning baseline: `24965aa0ba2696b201e9c74531920877529821d5`

Current corrective evidence baseline: `cf857fc24579263611396d4d88a1ef622601d612`

Historical evidence:

- `plans/closure/release-operational-qualification/001-status.md`
- `plans/closure/release-operational-qualification/002-status.md`
- `plans/subsystems/release-operational-qualification-roadmap.md`

## 1. Why this addendum exists

Release M002 was conditionally closed before the packaging workflow was actually exercised. Static review found correctness defects that must be fixed before hosted artifact evidence is meaningful:

- the Linux aarch64 cross-build job attempts to execute the aarch64 binary on an x86_64 runner;
- checksum generation operates over `dist/*` while unpacked directories are present;
- the required `workflow_dispatch.version` input is ignored in favor of `GITHUB_REF_NAME`;
- tag/version/source checkout consistency is not proven;
- there are no hosted workflow/artifact hashes yet.

The historical conditional closure record remains evidence of what was implemented, but M002 is not considered operationally qualified until this corrective closes.

## 2. C001 — Packaging workflow correction and hosted evidence

Implementation:

- `plans/implementation/release-operational-qualification-corrective/001-packaging-workflow-correction-and-hosted-evidence.md`

Closure:

- `plans/closure/release-operational-qualification-corrective/001-status.md`

This plan was dependency-ready and executed independently of Foundation C002.
It is conditionally closed pending a hosted dispatch against an existing
release tag; the repository currently has no tags.

The product-correctness correctives are now closed. C001 remains operationally
conditional on hosted artifact evidence against a valid tag.

## 3. C002 — Release boundary and documentation cleanup

A later cross-repo planning review established the durable producer/consumer
boundary recorded in ADR-0002:

- Eggpack owns producer release construction/evidence/bootstrap/CI;
- Eggup owns optional consumer deployment/update transactions;
- Eggprobe owns product policy.

The historical Release M003 mixed these responsibilities and incorrectly
appeared as a first-release Eggup blocker.

Implementation:

- `plans/implementation/release-operational-qualification-corrective/002-release-boundary-and-documentation-cleanup.md`

Closure target:

- `plans/closure/release-operational-qualification-corrective/002-status.md`

Status: closed. Closure evidence:

- `plans/closure/release-operational-qualification-corrective/002-status.md`

C002 is documentation/planning-adjacent cleanup only. It does not create a tag,
publish a release, or implement Eggpack/Eggup integration.

## 4. C003 — CI portability and standalone release requalification

After C002 closed, current-head CI exposed two release-qualification defects
that were not visible in the earlier local/hosted evidence:

- Windows checks out the deterministic JSON fixtures with CRLF and the contract
  tests compare those bytes against LF-only `serde_json` output, causing two
  otherwise semantically identical fixtures to fail;
- the dependency-audit action attempts to build its own `cargo-audit` tooling
  under Eggprobe's Rust 1.89 product toolchain, and the resolved audit-tool
  dependency graph now requires a newer compiler.

GitHub Actions run `36003160519` on
`cf857fc24579263611396d4d88a1ef622601d612` is the evidence baseline.
Ubuntu, macOS, Clippy, and the Rust 1.89 MSRV lane pass; Windows contract tests
and the audit-tool bootstrap fail.

Implementation:

- `plans/implementation/release-operational-qualification-corrective/003-ci-portability-and-standalone-release-requalification.md`

Closure target:

- `plans/closure/release-operational-qualification-corrective/003-status.md`

Status: closed. Closure evidence:

- `plans/closure/release-operational-qualification-corrective/003-status.md`

C003 was a hard first-release gate; it is intentionally local to Eggprobe and
introduced no Eggpack producer integration or Eggup self-update. The
cross-platform CI barrier it repaired (Windows CRLF fixtures + audit-tool
bootstrap under the product MSRV) is now removed: canonical JSON
fixtures/schemas are LF-locked via `.gitattributes`, and the `audit` job is
rewritten around an explicit stable toolchain, a pinned `cargo-audit`
install, and `cargo +stable audit`. The next required action is C001's
valid-tag hosted artifact evidence; M004 remains blocked only on that gate.

## 5. Downstream disposition

- Historical M003 is superseded by M003a/M003b under ADR-0002.
- M003a Eggpack producer integration is deferred and not a first-release gate.
- M003b Eggup runtime self-update is optional/deferred and not a first-release gate.
- C003 is closed; the cross-platform CI barrier is removed.
- The next required action is C001's remaining valid-tag hosted artifact
  evidence, which promotes M002 to operationally qualified.
- Release M004 then becomes ready for the first standalone archive release.
- Eggpack/Eggup instability does not block this sequence and must not be worked
  around by copying their producer/consumer responsibilities into Eggprobe.
