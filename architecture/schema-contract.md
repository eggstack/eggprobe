# Schema Contract and Versioning

Rust types in `eggprobe-core` own the JSON contract. The active artifacts are
`schemas/plan-0.4.json` and `schemas/report-0.4.json`; 0.1, 0.2, and 0.3 are
immutable historical evidence. The binary accepts only the exact current
version and does not migrate older plans.

## Version semantics

`SchemaVersion` serializes as `major.minor`; `CURRENT` is `0.4`. `ToolVersion`
is separate producer metadata such as `0.1.1`. Plan validation rejects any
schema version other than `CURRENT`. Report construction preserves the input
version when reporting an invalid plan, with a warning explaining rejection.

Schema 0.4 adds native plan families for route inspection, ICMP echo, direct
UDP, traceroute, and path-MTU discovery. Their evidence remains typed and
backend-neutral. Route reports distinguish kernel-observed source/interface
facts from route-table candidates; interface MTU and path MTU are separate.
Native plans are direct-only and cannot fall back from Eggress.

## Evolution rules

- All input and output structs deny unknown fields.
- Additive fields must be optional/defaulted and omitted when absent.
- Units and address/fact provenance are explicit.
- Route credentials never enter reports or diagnostics.
- Schema changes require regenerated artifacts and exact-byte fixtures.
- Historical schema files and 0.3 fixtures remain unchanged.

Regenerate active artifacts with:

```text
cargo run -p eggprobe-core --example generate-schemas --locked
```

The generator writes only 0.4 schemas and pins `schema_version` to `0.4`.
