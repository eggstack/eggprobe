# Release and Operational Qualification — Post-Closure Corrective Addendum

Status: active; C001 conditionally closed; C002 closed; C003 closed historically; C004 closed; C005 ready

Historical planning baseline: `24965aa0ba2696b201e9c74531920877529821d5`

C003 planning baseline: `cf857fc24579263611396d4d88a1ef622601d612`

Current corrective evidence baseline: `23da04ec32d3981aa0cb899d4d5b1d120e66f75e`

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

C003 remains closed for the two defects it actually corrected. A later hosted
run discovered an additional Windows-only engine test failure after the CRLF
tests had passed, so C003's historical closure is retained and the new finding
is handled by C004 rather than rewriting C003 evidence.

## 5. C004 — Windows HTTP fixture and hosted CI requalification

Hosted run `36008924756` on closure commit
`23da04ec32d3981aa0cb899d4d5b1d120e66f75e` provides the new evidence:

- Ubuntu quality job `107664241772` — green;
- macOS quality job `107664241926` — green;
- MSRV job `107664241902` — green;
- dependency audit job `107664241849` — green and completes the actual
  Eggprobe lockfile audit;
- Windows job `107664241734` — the C003 deterministic contract tests are
  green, but `http_status_is_observed_even_when_assertion_fails` fails later
  in `tests/engine.rs` because the report is `Failed` rather than the
  expected `Ok`.

The local HTTP fixture accepts a connection, writes a 503 response, and then
drops the stream without first consuming the request or explicitly flushing /
shutting down. That is a plausible Windows socket-lifecycle portability
failure, but C004 MUST prove the failure boundary before changing production
semantics.

Implementation:

- `plans/implementation/release-operational-qualification-corrective/004-windows-http-fixture-and-hosted-ci-requalification.md`

Closure target:

- `plans/closure/release-operational-qualification-corrective/004-status.md`

Status: closed. Closure evidence:

- `plans/closure/release-operational-qualification-corrective/004-status.md`

Hosted run `36015806725` on closure head `04c84d3` is fully green
(Ubuntu `107687917563`, macOS `107687917768`, Windows `107687917787`,
MSRV `107687917366`, audit `107687917580`). Two Windows-only
test-fixture defects were corrected with no production change: the 503
fixture now drains request headers (bounded), flushes, and shuts down
orderly instead of write-and-drop (Windows RST discarded the queued
response); the refused-hop probe retries on fresh ports with a 6 s
per-attempt dial budget because the hosted Windows runner delays RST
for closed loopback ports by ~2 s. Probe-versus-assertion semantics
(503 observed as `Ok` evidence, 2xx finding `Failed`) are preserved and
more strongly asserted.

C004 is now the immediate first-release gate. It does not create a release tag
and does not adopt Eggpack or Eggup.

## 6. Downstream disposition

- Historical M003 is superseded by M003a/M003b under ADR-0002.
- M003a Eggpack producer integration is deferred and not a first-release gate.
- M003b Eggup runtime self-update is optional/deferred and not a first-release gate.
- C003 remains historical closure evidence for the CRLF/audit findings it
  corrected; C004 owns the later Windows HTTP fixture and refused-hop
  findings and is now closed.
- C004 has closed with a fully green hosted matrix.
- C005 is the registered execution plan for C001's remaining valid-tag hosted
  packaging evidence and M002 operational promotion.
- Release M004 remains blocked until C005 closes with that evidence.
- Eggpack/Eggup instability does not block this sequence and must not be worked
  around by copying their producer/consumer responsibilities into Eggprobe.


## 7. C005 — First-release tag and hosted packaging qualification

C001 is already a historical conditional-closure record. Its only remaining
condition is operational evidence that cannot be satisfied by static review:
an intentional release tag, a successful hosted packaging run, and inspected
artifacts/checksums/smoke evidence.

C005 provides a fresh baseline-specific handoff for that work instead of
rewriting C001 history.

Implementation:

- `plans/implementation/release-operational-qualification-corrective/005-first-release-tag-and-hosted-packaging-qualification.md`

Closure target:

- `plans/closure/release-operational-qualification-corrective/005-status.md`

Status: ready for handoff.

C005 also owns the narrow pre-tag release-workflow hygiene required for the
first run: replace the stale `macos-13` Intel runner with a currently
documented standard Intel runner (planned `macos-15-intel`) and remove the
known actionlint/shellcheck target-comparison warning without changing the
artifact contract.

Successful C005 closure:

- supplies C001's remaining hosted evidence;
- promotes Release M002 to operationally qualified;
- makes M004 ready;
- does not publish a GitHub Release;
- does not adopt Eggpack or Eggup.
