# Eggprobe machine schemas

The canonical Rust types in `eggprobe-core` are the source of truth. The
checked-in schema artifacts are generated with `eggprobe_core::schema` and are
intentionally versioned separately from the tool version. Regenerate the active
artifacts with:

```text
cargo run -p eggprobe-core --example generate-schemas --locked
```

`plan-0.1.json` / `report-0.1.json`, `plan-0.2.json` / `report-0.2.json`, and
`plan-0.3.json` / `report-0.3.json` are immutable historical evidence.
`plan-0.4.json` and `report-0.4.json` are the active contract. The binary
rejects earlier plans rather than silently reinterpreting them. Version 0.4
adds native diagnostic vocabulary and evidence types.

The pre-1 policy permits additive optional fields and new enum variants only
when consumers reject unknown required structure safely. Breaking changes or
unit changes require a schema minor revision and updated fixtures.
