# Schema Contract and Versioning

Eggprobe is JSON-first by architecture decision (`plans/adrs/ADR-0001-json-first-core-and-transport-ownership.md:58-87,119-146`): typed `ProbePlan`/`ProbeReport` in `eggprobe-core`; CLI rendering is an adapter; Eggfetch owns HTTP; Eggress owns proxy execution. Rust types authoritative; `schemas/*-0.3.json` describe the active contract; `*-0.1/0.2` historical evidence only (`docs/operator.md:41-45`, `schemas/README.md:1-6`, `eggprobe-core/src/lib.rs:1-15`).

Mechanism: domain types derive `serde` + `schemars::JsonSchema`; `schema.rs:7-17` exposes `plan_schema()`/`report_schema()` (`schema_for!(ProbePlan/ProbeReport)`); `pretty()` deterministic pretty-prints (`schema.rs:19-28`). Workspace pins `schemars 1.2`, `serde 1.0`, `serde_json 1.0`; emitted `$schema: 2020-12`.

## Version semantics

* `SchemaVersion{major, minor}` (`domain/version.rs:11-16`): JSON `"major.minor"` (custom Serialize/Deserialize/Display/FromStr, `version.rs:39-83`), schema pattern `^[0-9]+\.[0-9]+$` (`version.rs:18-29`). `INITIAL = 0.1`, `CURRENT = 0.3` (`version.rs:31-37`); no named `0.2`.
* `ToolVersion(String)` transparent (`version.rs:85-126`): rejects empty/whitespace; schema plain `{"type":"string"}` (producer version, e.g. `"0.1.0"`); contract test proves `schema_version == "0.3"` while `tool.version == "0.1.0"` (`tests/contract.rs:265-272`).
* Gate (`domain/plan.rs:39-42`): `schema_version != CURRENT` → `UnsupportedSchema`. Restated in `schemas/README.md:12-16` + `docs/operator.md:41-44`: binary rejects earlier plans, no migration/coercion. Engine maps to `Failed` report + warning (`engine.rs:64-74`); CLI maps to invalid-invocation strings (`eggprobe-cli/src/lib.rs:221,251,304-305`). Note: `from_plan` echoes the plan's version even on rejection (`report.rs:105`) — consumers must read `warnings` + version together.

## Evolution rule and shapes

Policy (`schemas/README.md:18-20`, `docs/operator.md:44-45`): pre-1 additive optional fields + new enum variants only when consumers reject unknown required structure safely (via `deny_unknown_fields` + loud unknown-variant failures, `tests/contract.rs:245-252`); breaking/unit changes require schema minor + fixtures; new fields must be `Option`/`#[serde(default)]` + `skip_serializing_if`; credential-carrying fields must never enter reports; duration unit changes are breaking.

* **Plan** (`domain/plan.rs:12-30`, `deny_unknown_fields`): `schema_version!` (pinned `const "0.3"`, `plan-0.3.json:352-354`), `target` (host+port, `target.rs:38-89`), `route` (`direct|eggress{expression}`, input-only), `probes[]` ordered (`dns|tcp{port}|tls{port,server_name?}|http{url,method=GET}`), `execution` defaulted (30s/1/0; zero deadline/repetitions and `retries>0` rejected), `assertions[]` defaulted (five typed kinds; `min>max` rejected). HTTP authority: `http/https` only, no userinfo, host match (case/trailing-dot/IP-aware), port match with 80/443 defaults (`plan.rs:138-177`, `tests/contract.rs:166-226`).
* **Report** (`domain/report.rs:68-92`, `deny_unknown_fields`): `schema_version!` (`const "0.3"`), `tool!`, `execution_id!`, `target!`, `route!` (`direct|eggress` only), `status!` (`ok/failed/unsupported/cancelled`), `probes[]!`, `findings[]`/`warnings[]` defaulted. Children: `ProbeResult` (kind/status!; timing/error/evidence?; `unavailable[]`/`findings[]`); `ProbeEvidence` adjacently tagged (`dns/tcp/tls/http`); `DnsEvidence{addresses, resolution_scope: "client"!}` (local-resolver semantics); `TcpEvidence{peer?,local?,attempts}`; `TlsEvidence{version?,alpn?,cipher_suite?}`; `HttpEvidence{status?,protocol?,body_sample_bytes}`; `Timing{total!, phases[]}` (integer micros); `DiagnosticError{kind!,stage!,message!; attempt?,route_hop_index?,route_protocol?}` (0.3 hop provenance); `Finding{assertion_id!,severity!,outcome!,message!}`. Redaction: `RouteSpec::summary()` collapses `Eggress(expr)→Eggress` (`route.rs:19-28`); report JSON `{"kind":"eggress"}` no `expression` (`tests/contract.rs:228-243`); `Display eggress(<redacted>)`, redacted `Debug`.
* **Historical deltas:** `0.1`: free-text `AssertionSpec{id,description}`, generic version pattern, stub route/probes. `0.2`: adds `AssertionKind` (five variants), `DurationMicros`, `SchemaVersion` defs, pins `const "0.2"`, 23 report `$defs`. `0.2→0.3`: adds required `DnsEvidence.resolution_scope` + optional `route_hop_index`/`route_protocol`; plan deltas descriptive only.

## Generation and verification

* Generate: `cargo run -p eggprobe-core --example generate-schemas --locked` (`schemas/README.md:8-10`); impl `examples/generate-schemas.rs:6-19` writes only `*-0.3.json`, patching `schema_version` to `{"const":"0.3"}` before pretty-print + newline.
* Artifacts: active `schemas/plan-0.3.json` (`ProbePlan`, `additionalProperties: false`), `schemas/report-0.3.json` (`ProbeReport`); historical `*-0.1/0.2.json` immutable.
* Tests: determinism + `!contains("expression")` (`tests/schema.rs:3-12`); fixture round-trips byte-for-byte (`tests/contract.rs:90-112`, `fixtures/plan.json`, `fixtures/report.json` with `resolution_scope: client`); redaction (Debug/Display/JSON/human, arbitrary future syntax, `route == {"kind":"eggress"}`); validation taxonomy (target/port mismatch, IPv6/scheme defaults, userinfo rejection, unknown variants, whitespace targets, empty tool version, schema vs tool distinctness).

## Key Code References

`schema.rs:7-28`; `domain/version.rs:11-140`; `domain/plan.rs:12-275,32-177`; `domain/report.rs:14-132`; `domain/probe.rs:8-142`; `domain/error.rs:6-83`; `domain/route.rs:8-74`; `domain/timing.rs:8-54`; `domain/target.rs:38-120`; `domain/finding.rs:6-42`; `lib.rs:16-28`; engine/CLI gates: `engine.rs:64-74`, `eggprobe-cli/src/lib.rs:221,251,304-305`; generator: `examples/generate-schemas.rs:6-19`; tests: `tests/schema.rs`, `tests/contract.rs`, `fixtures/plan.json`, `fixtures/report.json`; schemas: `schemas/plan-0.3.json:320-368`, `schemas/report-0.3.json:706-766`; policy: `schemas/README.md`, `docs/operator.md:41-45`, ADR-0001.

## Review Checklist / Risks

1. Regenerate before merge when domain types change; confirm CI checks drift (none described in files reviewed).
2. Never edit `0.1`/`0.2` — they are rejection-policy evidence.
3. Generator hard-codes `"0.3"` separately from `CURRENT` — derive patch value from `CURRENT` to avoid divergence.
4. Strict equality means future additive `0.4` rejected until binary updated — document whether minors should ever be compatible.
5. Echoed version on rejection — treat `schema_version` + `warnings` together.
6. `deny_unknown_fields` makes additive fields breaking for old binaries — consumers must gate on `schema_version` before parsing.
7. Redaction structural, not textual — new credential-adjacent fields need summary type/exclusion + redacted `Debug` + schema-test asserting absence from JSON/human/`Debug`.
8. New required `resolution_scope` — old `0.2` reports fail `0.3` validation; retain `report-0.2.json` for archives.
9. Optional `route_hop_index`/`route_protocol` absence ≠ hop 0/direct; use `unavailable[]` for gaps.
10. Units/defaults (micros, 30s/1/0) changes require minor + fixtures.
11. Don't assert exact warning/stderr text externally; assert `status`/`schema_version`/machine fields.
12. Key on `schema_version`, use `tool` for provenance/debugging only.
