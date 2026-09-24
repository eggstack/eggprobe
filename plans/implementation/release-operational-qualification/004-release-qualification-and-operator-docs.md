# Release and Operational Qualification M004 — Release Qualification and Operator Documentation

Status: closed at `v0.1.1` — see `plans/closure/release-operational-qualification/004-status.md` (hosted packaging run `36041400748` fully green; first standalone archive release qualified as `0.1.1`)

Planning baseline: historical `e55a21fe73915b7a38c2fa674807354539806c60`; refresh at execution against current release artifacts.

Source roadmap:

- `plans/subsystems/release-operational-qualification-roadmap.md#m004--release-qualification-and-operator-documentation`

Hard dependency: Release M002 operationally qualified.
Corrective dependencies: Release corrective C002 is closed. C003 remains
historical closure evidence for its CRLF/audit corrections, and release
corrective C004 is closed with a fully green hosted matrix (closure:
`plans/closure/release-operational-qualification-corrective/004-status.md`,
run `36015806725`). M004 now waits only on C001 valid-tag hosted artifact
evidence.
Conditional dependency: M003b only if runtime self-update is advertised in the
release being qualified. M003a Eggpack producer adoption is not required for
the first standalone archive release.

Additional planning dependency: Release corrective C002 (closed) established
the current Eggpack/Eggup ownership model in operator documentation.

Primary class: invariant + polish

## 1. Objective

Perform a release-candidate qualification pass that reconciles code, schema, artifacts, documentation, security evidence, and supported-platform claims before the first broadly consumable release.

## 2. Required qualification

- run strict workspace verification on release commit;
- verify generated schema/fixtures against release binary;
- verify all archives/checksums;
- perform install/extract smoke on supported hosts;
- execute representative local DNS/TCP/TLS/HTTP/proxy commands that are implemented by that release;
- verify JSON/NDJSON machine parsing;
- verify redaction corpus;
- run audit/dependency review;
- check MSRV source build;
- confirm version/tag/help/docs match;
- document unsupported/deferred capabilities truthfully.

## 3. Operator documentation

Provide:

- installation/source-build instructions;
- supported target table;
- command quick start;
- JSON/schema usage;
- proxy credential handling guidance;
- timeout/cancellation behavior;
- privacy/security notes;
- troubleshooting/error taxonomy;
- update instructions only if optional M003b self-update is closed and advertised.

## 4. Acceptance criteria

No documentation claim exceeds qualified behavior. Release artifacts and schemas correspond to one source commit/version. No unresolved medium-or-higher release finding remains.

## 5. Stop conditions

Stop release qualification on checksum/provenance mismatch, schema drift, secret leak, failing supported-platform smoke, security advisory without disposition, or documentation claiming an unclosed capability.

## 6. Closure evidence

Create `plans/closure/release-operational-qualification/004-status.md` with a requirement-to-evidence matrix, artifact hashes, platform results, schema/redaction tests, security review, docs reconciliation, and final release disposition.


## 8. Current unblock condition

For the first standalone release, this plan becomes ready when:

1. Release corrective C005 closes with:
   - one intentional valid release tag;
   - successful hosted packaging across the declared target matrix;
   - verified archive/checksum/release-identity evidence;
   - native smoke evidence on host-native targets;
   - Linux aarch64 retained as build-qualified unless native SBC evidence is
     separately obtained;
2. C005 records C001's remaining operational evidence as satisfied and promotes
   Release M002 to operationally qualified; and
3. Release corrective C002 remains closed; C003/C004 remain historical closure
   evidence for the CI portability findings they corrected.

Eggpack producer adoption (M003a) and Eggup self-update (M003b) are not first
release gates. No temporary local replacement for either shared subsystem is
part of M004.
