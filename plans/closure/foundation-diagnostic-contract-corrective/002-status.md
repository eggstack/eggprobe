# Foundation Diagnostic Contract Corrective C002 — Closure

Status: closed

Implementation plan:

- `plans/implementation/foundation-diagnostic-contract-corrective/002-structural-report-safety-and-authority-normalization.md`

Baseline: `24965aa0ba2696b201e9c74531920877529821d5`

## Delivered correction

- `RouteSummary::Eggress` is now a unit variant. Raw Eggress expressions can
  remain only in input `RouteSpec::Eggress(EggressRoute)` and cannot be
  constructed, deserialized, or serialized through the report route type.
- HTTP authority validation uses `url::Url`; only HTTP and HTTPS are accepted,
  userinfo is rejected, scheme-default ports are explicit, and bracketed or
  equivalent IPv6 spellings compare as the same address.
- The active machine contract is schema `0.2`. Schema `0.1` plan/report
  artifacts remain checked in unchanged and the active validator rejects them
  rather than silently reinterpreting them.
- A reproducible `generate-schemas` example regenerates the active artifacts.

## Contract evidence

Historical schema SHA-256:

```text
a39db24d46e306b4b2e019a2f12fa138cee778514329d540eea837bb120c975a  schemas/plan-0.1.json
7c3f5e6aff6b62b18d4465c98ef0d37f37b279070fbbd5294bb1bd9ddbe06d9c  schemas/report-0.1.json
```

Current schema SHA-256:

```text
42065c85c3471a91e86a87ac9c1a0d5524960fff8012e4d0561aeab62f73a255  schemas/plan-0.2.json
b0ae1ab87f33fb4a2481b3c651c5f3c5a2d95c9e6618575c373d4429e7441c7f  schemas/report-0.2.json
```

The schema generator was run twice consecutively; the second run produced no
working-tree delta. `git diff -- schemas/plan-0.1.json schemas/report-0.1.json`
was empty.

Normalization tests cover:

- HTTPS default port 443 and HTTP default port 80;
- explicit ports;
- case-insensitive DNS names;
- bracketed and equivalent IPv6 authorities;
- unsupported schemes and userinfo rejection;
- structurally secret-free Eggress report serialization.

## Verification

All required commands passed:

```text
cargo fmt --all -- --check
cargo check --workspace --all-targets --all-features --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test --workspace --all-features --locked        # 19 passed
cargo +1.89.0 check --workspace --all-targets --locked
cargo tree --locked
cargo audit
```

## Dependency and downstream disposition

The only direct dependency addition is the already-locked `url` 2.5.8 line;
no Eggfetch, Eggress, or network implementation was copied or changed.

Foundation C002 is closed and the following hard gates are satisfied:

- Transport corrective C001 is unblocked and ready;
- CLI corrective C001 is unblocked with its remaining hard dependency on
  Transport C001;
- Transport corrective C002 remains blocked on Transport C001;
- Release corrective C001 remains independently ready;
- Release M003 remains blocked on the unpublished shared `eggup` interface;
- Release M004 remains blocked on the remaining corrective qualification work.

Residual schema migration impact is explicit: consumers must opt into 0.2;
there is no implicit 0.1 compatibility loader.
