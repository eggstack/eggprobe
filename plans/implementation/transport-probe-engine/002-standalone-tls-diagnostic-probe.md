# Transport and Probe Engine M002 — Standalone TLS Diagnostic Probe

Status: closed

Planning baseline: `e55a21fe73915b7a38c2fa674807354539806c60`; refresh when M001 closes.

Source roadmap:

- `plans/subsystems/transport-probe-engine-roadmap.md#m002--standalone-tls-diagnostic-probe`

Hard dependency: Transport M001 closed.

Primary class: capability

## 1. Objective

Perform a standalone TLS client handshake over the established route abstraction and report only negotiated/verified facts that are directly observable.

## 2. Required production changes

- Rustls-based TLS client path owned by Eggprobe for standalone TLS only;
- verification enabled by default using the reviewed trust-root policy;
- explicit SNI/server-name selection;
- negotiated TLS version, cipher suite, ALPN, and peer/local transport metadata;
- bounded peer-certificate summary: chain length, subject/issuer/SANs/validity/fingerprint where a small audited parser can provide them;
- structured TLS verification/handshake errors;
- deadline/cancellation inherited from route execution.

Do not expose raw private/session secrets or unbounded certificate blobs.

## 3. Work packages

A. Select minimal Rustls/trust/certificate parsing dependency set compatible with Rust 1.89.
B. Implement handshake over direct route stream.
C. Add sanitized certificate summary.
D. Normalize errors/timing into report types.
E. Add local CA/server fixtures for trusted, untrusted, expired/not-yet-valid if practical, and hostname mismatch.

## 4. Tests

- trusted local certificate;
- hostname mismatch;
- untrusted CA;
- ALPN negotiated and absent;
- TLS version/cipher facts present only when observed;
- cancellation/deadline;
- certificate summary bounds/redaction;
- IPv6 route reuse where available.

## 5. Acceptance criteria

Verification is on by default; disabling it, if supported at all, is an explicit unsafe plan/CLI option. No second TCP dialer is introduced. TLS errors are typed and no certificate/session secret leaks.

## 6. Stop conditions

Stop if the certificate parser materially expands dependency/security surface without clear value, if TLS policy conflicts with later Eggfetch ownership, or if direct-route semantics must be redesigned.

## 7. Closure evidence

Create `plans/closure/transport-probe-engine/002-status.md` with trust policy, fixture certificates, negotiated facts, negative cases, dependency/audit results, and platform notes.
