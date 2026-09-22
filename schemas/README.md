# Eggprobe pre-1 schemas

The canonical Rust types in `eggprobe-core` are the source of truth. The
checked-in schema artifacts are generated with `eggprobe_core::schema` and are
intentionally versioned separately from the tool version. Regenerate them with
the repository's schema generation command before changing a public field.

The pre-1 policy permits additive optional fields and new enum variants only
when consumers reject unknown required structure safely. Breaking changes or
unit changes require a schema minor revision and updated fixtures.
