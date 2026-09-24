# CLI / Automation

`eggprobe-cli` is the thin command-line adapter. It owns argument parsing, plan construction, batch orchestration, output selection, and exit codes. It owns no networking.

Crate: `crates/eggprobe-cli/Cargo.toml:1-20` (`eggprobe` bin at `src/main.rs`); deps `clap`, `eggprobe-core` (path), `serde`, `serde_json`, `tokio`. Module doc `lib.rs:1`: "Thin Clap adapter over eggprobe-core."

## Subcommands, args, plan builders

Top level `Cli { command: Option<Command> }` (`lib.rs:20-26`); no subcommand → `(0, "Run eggprobe --help…")`. `Command`: `Dns|Tcp|Tls|Http|Proxy|Check|Run|Compare` (`lib.rs:30-39`); dispatch `execute()` (`lib.rs:193-218`).

* `dns/tcp` — `PrimitiveArgs` (`lib.rs:43-55`): positional `target`, `--port/-p?`, `--via?`, `--timeout-ms` default 30_000, `--strict-target`, `--json`. `plan_for_dns` ignores `--port` (always `None` + `Dns` probe); `plan_for_tcp` requires `--port` (`lib.rs:424-442`).
* `tls` — `TlsArgs` flattens `PrimitiveArgs` + `--server-name?` (`lib.rs:58-63`); requires port (`lib.rs:443-455`).
* `http` — `HttpArgs` (`lib.rs:66-78`): positional `url`, `--via?`, `--method` default `GET`, `--timeout-ms`, `--strict-target`, `--json`. `parse_url_host` → `target(host, None)`; URL port stays inside `ProbeSpec::Http.url` (`lib.rs:456-468,508-526`).
* `proxy` — `ProxyArgs` (`lib.rs:81-91`): `target` + required `--port` + required `--via`, no `--strict-target` (always `AllowPrivate`, `lib.rs:211`).
* `check` — `CheckArgs` (`lib.rs:94-112`): `target` + required `--port`, optional `--url`, `--expect-status-min/max` default 200/299. Rejects `min > max`; always `[Dns, Tcp]` plus `Http{GET}` + `http-status: HttpStatusRange` when `--url` present (`lib.rs:478-507`).
* Shared helpers: `route()` (`None→Direct`, empty→error, else `Eggress`), `target()` (`TargetSpec::new`), `execution()` (`deadline=timeout_ms*1000µs saturating, repetitions 1, retries 0`; no CLI knob), `base()` (`schema_version: CURRENT`), `run_single()` (validate → engine with `Strict` iff flag → `evaluate_assertions` → `exit_code` + `render`).

## `run` automation

`RunArgs` (`lib.rs:115-123`): positional `input`, `--ndjson`, `--concurrency` default 4, `--fail-fast`. Entry `run_input()` (`lib.rs:235-285`).

* Bounds: `MAX_INPUT_BYTES = 4 MiB`, `MAX_BATCH_SIZE = 256` (`lib.rs:16-17`); `read_bounded()` (`lib.rs:371-386`): `-` reads stdin, `take(MAX+1)` + UTF-8 check; `parse_plans()` tries single `ProbePlan`, then `Vec<ProbePlan>`, then `{plans:[...]}` (`lib.rs:356-370`); concurrency guard 1–64, batch guard 256.
* Single-plan fast path (`len==1 && !ndjson`): `ProbeEngine::default()` (always `AllowPrivate`), findings attached, pretty JSON regardless of flags (`lib.rs:243-248`).
* Batch path: pre-validate all; sliding-window `JoinSet` bounded by concurrency; `join_next` records `(index, report)`, folds exit code, stores `reports[index]`; `fail_fast` stops admitting new plans but awaits started work; output iterates in index order, skips `None` (fail-fast truncation), serializes compact `to_string` per line with overwritten `execution_id = batch-{index}` (`lib.rs:249-294`).
* Contract (`docs/operator.md:33-39`): one report per plan, independently parseable lines, input-order output, logs to stderr, stdout machine-data-only under `--json`/`--ndjson`.

## `compare`

`CompareArgs` (`lib.rs:126-133`): positional `direct`, `routed`, `--repeat` default 1, `--json`. Entry `run_compare()` (`lib.rs:296-341`).

* Inputs: single `ProbePlan` each via `read_bounded` (no batch support); repeat guard 1–100; both `validate()`d; `validate_comparison()`: direct must be `Direct`, routed must be `Eggress(_)`, same `target` + `probes` (`lib.rs:343-354`).
* Execution: cold loop `2*repeat` sequential with fresh `ProbeEngine::default()`; output tags `"connection_policy": "cold"`.
* `statistics()` (`lib.rs:535-581`): samples = successful probe `total.as_micros()`; counts `successes/failures/unsupported` from `ReportStatus`; empty → no min/max/percentiles; else sorted + `percentile(p) = sorted[(ceil(p*N/100).max(1)-1)]`, emits `{attempts,samples,successes,failures,unsupported,min,max,median,p50,p95}` (`median == p50`).
* `comparable_delta()` (`lib.rs:583-599`): median absolute (`routed-direct` i128) + relative `{numerator,denominator}` (null when `direct==0` or medians absent).
* Output envelope `{kind:comparison, repetitions, connection_policy:cold, direct, routed, delta, attempts:{direct:[reports], routed:[reports]}}`; `--json` → pretty, else short human `comparison repetitions…` (`lib.rs:321-340`).

## Exit codes, `CliError`, signals

Core (`assertions.rs:124-156`): `0 Success`, `1 Negative` (Failed/Unsupported or any non-Passed finding), `2 Invalid`, `3 Internal`, `130 Interrupted`. CLI: `CliError::Invalid→2`, `Internal→3` (`lib.rs:135-158`); only three `Internal` sites (JoinSet join, batch serialize ×2, compare serialize). `combine_exit_codes()` precedence `130 > 3 > 2 > 1 > 0` (`lib.rs:601-609`). `main.rs:1-21`: `select!{ execute vs ctrl_c → eprintln("interrupted") + exit(130) }`; `Ok → print!` stdout + code; `Err → eprintln!("eggprobe: …")` stderr + `error.code()`.

## Key Code References

Adapter doc/imports: `lib.rs:1-14`; bounds: `lib.rs:16-17`; CLI types: `lib.rs:20-133`; errors: `lib.rs:135-180`; dispatch: `lib.rs:193-233`; batch: `lib.rs:235-294`; compare: `lib.rs:296-354,535-599`; builders: `lib.rs:387-526`; I/O/parse/render: `lib.rs:356-386,527-533,601-614`; binary/signals: `main.rs:1-21`; manifest: `eggprobe-cli/Cargo.toml`; tests: `tests/cli.rs`, `tests/render.rs:7-30`; operator: `docs/operator.md:19-81`.

## Review Checklist / Risks

* Single-plan `run` fast path skips explicit `validate()` (relies on engine-level check) — confirm or add.
* `dns --port` silently discarded; consider rejecting/warning.
* `proxy`/`run`/`compare` always `AllowPrivate` — no strict policy in automation paths.
* `http` target port always `None`; URL ports live only in probe spec — verify policy implications.
* Pretty (single/compare-json) vs compact (batch NDJSON) duality — parsers must tolerate both.
* Fail-fast truncates output (`None` skipped) — callers must not assume 1:1 line mapping.
* Hand-rolled `parse_url_host` (no `url` crate) — check IPv6 zones, trailing dots, userinfo, uppercase schemes.
* Single-JSON `render()` uses `.expect("report serializes")` — panics instead of `Internal` (batch/compare return `Internal`).
* `compare` sequential `2*repeat` with default 30 s deadlines can run very long; full reports embedded per attempt inflate stdout.
* Per-file 4 MiB bound means `compare` allows ~8 MiB total; post-parse memory (256 plans → reports) unbounded beyond plan cap.
* Human (non-`--json`) output shares stdout with machine data — wrappers must pass `--json`/`--ndjson` before parsing stdout, read diagnostics from stderr.
