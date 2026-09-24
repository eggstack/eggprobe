# Release and Operational Qualification M004 — Release Qualification and Operator Documentation

Status: blocked — awaiting corrective C004, then C001 valid-tag hosted artifact evidence

Planning baseline: historical `e55a21fe73915b7a38c2fa674807354539806c60`; refresh at execution against current release artifacts.

Source roadmap:

- `plans/subsystems/release-operational-qualification-roadmap.md#m004--release-qualification-and-operator-documentation`

Hard dependency: Release M002 operationally qualified.
Corrective dependencies: Release corrective C002 is closed. C003 remains
historical closure evidence for its CRLF/audit corrections, but post-closure
hosted run `36008924756` exposed an additional Windows HTTP engine-fixture
failure. Release corrective C004 must close with a fully green hosted matrix
before M004 can execute.
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

1. Release corrective C004 closes with a fully green hosted CI matrix,
   including Windows proof that successfully observed HTTP error responses
   remain probe-success observations while assertions can fail independently;
2. Release corrective C001 then obtains hosted artifact evidence against a
   valid release tag and Release M002 is operationally qualified; and
3. Release corrective C002 remains closed. C003 remains historical evidence
   for the CRLF/audit defects it fixed, with the later Windows finding tracked
   through C004.

Eggpack producer adoption (M003a) and Eggup self-update (M003b) are not first
release gates. No temporary local replacement for either shared subsystem is
part of M004.
