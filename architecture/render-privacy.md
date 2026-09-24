# Render + Privacy/Redaction

Machine JSON is the contract. Human text is downstream and lossy. `eggprobe-core` owns pure renderers; CLI selects a path and returns `(code, output)`; `main` does I/O (`eggprobe-cli/src/lib.rs:1-2,188-218`, `main.rs:1-20`, `eggprobe-core/src/render.rs:7-9`, `domain/report.rs:68-92`).

## Render paths

* **Single (`dns/tcp/tls/http/proxy/check`, single `run`):** `render(report, json)` (`eggprobe-cli/src/lib.rs:527-533`) — `json` → `to_string_pretty` (same as `ProbeReport::to_json`, `report.rs:117-125`); else `render_human` (`render.rs:9-39`): `{name} {version} — {status lowercase}`, `Target: {host}`, `Route: {RouteSummary:?}` (never raw expression), per-probe `- {kind}: {status}` + `error/witness/unavailable` lines. Single-plan `run` always pretty JSON (`lib.rs:243-248`). Wrapper `render_report()` (`lib.rs:610-614`) for tests/embedders.
* **Batch `run` (NDJSON):** `run_input()` (`lib.rs:235-285`) — bounds 4 MiB/256 plans, concurrency 1–64; accepts single/`Vec`/`{plans}`; concurrent `JoinSet`, input-order `reports[index]`, `combine_exit_codes`, `--fail-fast` stops admitting but awaits started; output compact `to_string` per line with `execution_id = batch-{index}`. Verified input-ordered + pure (`tests/cli.rs:64-93`).
* **`compare`:** `run_compare()` (`lib.rs:296-341`) — validates direct-`Direct` + routed-`Eggress` + same target/probes; cold `repeat` loop; nearest-rank `ceil(p*N/100)` stats + median delta; envelope `{kind:comparison, repetitions, connection_policy:cold, direct, routed, delta, attempts:{direct,routed}}`; `--json` pretty else short human.
* **stdout/stderr:** `Ok → print!` stdout + code; `Err → eprintln!("eggprobe: …")` stderr + `error.code()`; Ctrl+C → stderr + 130 (`main.rs:1-20`). Codes: 0 success, 1 negative probe/assertion, 2 invalid, 3 internal, 130 interruption (`docs/operator.md:68-69`, `assertions.rs:122-156`, `lib.rs:150-158,601-609`). Risk: human output also goes to stdout — only `--json`/`--ndjson` stdout is machine-contract (`docs/operator.md:38-39`).

## Privacy / redaction model

Credentials are input-only. Reports are structurally incapable of holding them.

* **Type boundary:** `RouteSpec{Direct, Eggress(EggressRoute{expression})}` never copied to reports (`route.rs:8-17`); `ProbeReport.route: RouteSummary{Direct,Eggress}` only (`report.rs:44-52,80-81`), serializing as `{"kind":"eggress"}`; `summary()` discards the string (`route.rs:19-28`); `from_plan()` enforces (`report.rs:94-115`); all engine paths use `from_plan`.
* **Debug/Display/human:** `RouteSpec::Debug/Display` and `EggressRoute::Debug` emit `<redacted>` via `Redacted` helper (`route.rs:30-74`); human renderer prints `RouteSummary` Debug only (`render.rs:10-17`); operator contract `{"kind":"eggress"}` / `eggress(<redacted>)` (`docs/operator.md:47-54`). Tests: fixture `super-secret`/`fixture-token` absent from `plan:?`, `to_string()`, `to_json()`, `render_human` (`tests/contract.rs:115-141`); arbitrary future syntax collapsed (`:143-158`); no `"expression"` in report/schema JSON (`:228-243`, `tests/schema.rs:3-12`); live auth failures secret-free (`tests/routed_qualification.rs:190-199,272-316`).
* **Error hygiene:** `DiagnosticError.message` bounded, credential-stripped (`domain/error.rs:64-73`); routed failures use fixed `diagnostic_from_route_error` strings + only `hop_index`/`protocol` copied (`engine.rs:1000-1046`); direct failures use `safe_io_message` (`kind().to_string()`) or fixed strings, never dependency `to_string()` with URLs (`engine.rs:920-998,1070-1093`); bad Eggress syntax → generic `invalid or unsupported Eggress route`. Hop provenance `route_hop_index` + `route_protocol` (e.g. `socks5`) safe to expose; host/port/credentials never are.
* **Resolver scope:** `DnsEvidence{addresses, resolution_scope}` documents local-system-resolver semantics, not remote hop (`domain/probe.rs:50-59`); only variant `Client` (`:61-67`); engine always sets `Client` even for routed plans (`engine.rs:193-201`, tested `:431-450`); operator docs warn DNS answers are client evidence (`docs/operator.md:58-65`). Eggress owns route parsing; no env-proxy fallback (`:53-54`).

## Key Code References

Renderer: `render.rs:9-43`; route boundary/redaction: `domain/route.rs:8-75`; report envelope: `domain/report.rs:44-132`; DNS scope: `domain/probe.rs:50-67`; error hygiene: `domain/error.rs:64-83`, `engine.rs:193-371,1000-1093`; CLI selectors/batch/compare/main: `eggprobe-cli/src/lib.rs:220-341,387-397,527-533,601-614`, `main.rs:1-20`; exit codes: `assertions.rs:122-156`; tests: `contract.rs:100-158,228-243`, `routed_qualification.rs:180-316,431-450`, `schema.rs:3-12`, `eggprobe-cli/tests/render.rs:7-30`, `cli.rs:30-145`; operator: `docs/operator.md:33-81`.

## Review Checklist / Risks

* New report/evidence/error/finding fields must be redaction-safe (no expression, userinfo URLs, headers, hop URIs); schema `!contains("expression")` only guards the field name — review strings manually.
* New secret-holding types need manual `Debug`/`Display` via `Redacted`; derived `Debug` leaks.
* Never forward dependency `error.to_string()`/`{:?}` into `DiagnosticError.message`; use fixed tables.
* `{evidence:?}`/`{route:?}` in human renderer safe today (addresses/versions/statuses + `RouteSummary`) — re-test every new evidence variant.
* Pretty vs compact discipline (single pretty, batch compact NDJSON + rewritten `batch-N`, compare pretty aggregate) — don't change without updating consumers/tests.
* stdout purity: never `println!` progress/logs on success path; stderr only; scripts must pass `--json`/`--ndjson` before parsing stdout.
* DNS scope: new remote-hop/DNS observers need distinct `DnsResolutionScope` variants; never relabel remote answers as `Client`.
* Hop provenance: copy only index + normalized protocol.
* HTTP userinfo already rejected at plan validation — replicate for any new URL-holding specs.
* `render_human` coverage thin (header/target only) — consider locking `Route: Eggress`, evidence, and `unavailable` lines.
