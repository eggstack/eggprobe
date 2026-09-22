# Foundation Diagnostic Contract Corrective C002 — Structural Report Safety and Authority Normalization

Status: closed

Planning baseline: `24965aa0ba2696b201e9c74531920877529821d5`

Source corrective addendum:

- `plans/subsystems/foundation-diagnostic-contract-corrective-addendum.md#6-c002--structural-report-safety-and-authority-normalization`

Historical evidence:

- `plans/closure/foundation-diagnostic-contract-corrective/001-status.md`
- `plans/closure/foundation-diagnostic-contract/002-status.md`

Primary class: invariant + corrective

## 1. Objective

Repair two contract defects discovered after C001/M002 closure before more transport or release qualification builds on the pre-1 schema:

1. make it structurally impossible for a `ProbeReport` route summary to carry an arbitrary raw Eggress expression;
2. replace hand-written HTTP authority parsing with canonical URL semantics, including scheme-default ports and IPv6 authorities.

The correction must be represented as an explicit schema evolution rather than silently changing the existing checked-in `0.1` contract.

## 2. Current evidence

At the planning baseline:

- `RouteSummary::Eggress { expression: String }` can be directly constructed or deserialized with any string, including secrets. Normal report construction inserts `"<redacted>"`, but the type itself does not enforce the security invariant.
- `report-0.1.json` therefore permits arbitrary `expression` content.
- `ProbePlan::validate()` uses a local `http_authority()` parser.
- `target.port = 443` plus `http://example.com/` does not conflict because the implicit HTTP port is not normalized.
- bracketed IPv6 authorities and other URL edge cases are not delegated to a standards-compliant URL parser.

## 3. Required contract decision

### Report route summary

Preferred shape:

```rust
enum RouteSummary {
    Direct,
    Eggress,
}
```

A structured Eggress variant containing only intrinsically non-secret fields may be used instead if there is a current, demonstrated consumer need, but **no free-form route string field is allowed**.

Raw route input remains confined to `RouteSpec::Eggress(EggressRoute)`.

### URL authority

Use a maintained URL parser rather than a local substring parser. Normalize:

- host, including IPv4 and bracketed IPv6;
- explicit port;
- scheme-default port (80 for HTTP, 443 for HTTPS);
- scheme restrictions;
- userinfo policy.

The canonical target authority rule remains: `ProbePlan.target` is authoritative and every HTTP probe must resolve to the same logical host/port after normalization.

## 4. Schema/version disposition

This is a breaking machine-contract change to the checked-in pre-1 schema. Do not mutate schema `0.1` in place.

Required:

- advance the schema contract to `0.2`;
- retain the existing `schemas/*-0.1.json` artifacts as historical compatibility evidence;
- generate `schemas/plan-0.2.json` and `schemas/report-0.2.json`;
- update golden fixtures to `0.2`;
- document whether the binary rejects `0.1` plans or supports an explicit compatibility loader. Do not silently reinterpret `0.1`.

Tool version remains independent.

## 5. Work packages

A. Replace free-form report route expression with a structurally secret-free variant.
B. Introduce canonical URL authority normalization and remove local authority parsing.
C. Add default-port, IPv6, case/normalization, invalid-scheme, and userinfo tests.
D. Advance schema version and generate retained `0.1` plus current `0.2` artifacts.
E. Update contract fixtures, README/operator/schema docs, and stale core crate documentation.
F. Audit every public report/domain type for another free-form field that violates a claimed structural security invariant.

## 6. Required regression cases

- deserializing an Eggress report cannot inject a route URI/password/token because no field can carry one;
- `https://example.com/` matches target `example.com:443`;
- `http://example.com/` conflicts with target `example.com:443`;
- explicit `https://example.com:8443/` matches only target port 8443;
- bracketed IPv6 with and without explicit port normalizes correctly;
- unsupported/non-HTTP schemes fail plan validation;
- URL userinfo behavior is explicit and tested;
- schema `0.1` remains checked in and unchanged;
- current fixtures/schema identify `0.2`.

## 7. Verification

```bash
cargo fmt --all -- --check
cargo check --workspace --all-targets --all-features --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test --workspace --all-features --locked
cargo +1.89.0 check --workspace --all-targets --locked
cargo tree --locked
cargo audit
```

Also regenerate schemas twice and verify deterministic byte-for-byte output.

## 8. Acceptance criteria

- report route types cannot hold arbitrary route input;
- no hand-written URL authority parser remains;
- implicit and explicit ports have one canonical interpretation;
- schema breaking change is represented by `0.2`, not hidden under `0.1`;
- historical `0.1` schema evidence is preserved;
- no medium-or-higher foundation finding remains.

## 9. Stop conditions

Stop if the correction requires route execution behavior, if a compatibility layer would ambiguously reinterpret `0.1`, or if a URL parser raises MSRV/security concerns that have not been reviewed.

## 10. Closure evidence

Create `plans/closure/foundation-diagnostic-contract-corrective/002-status.md` with:

- old/new schema hashes;
- explicit breaking-change rationale;
- URL normalization matrix;
- structural route safety proof;
- dependency delta;
- full verification results;
- downstream unblock disposition.
