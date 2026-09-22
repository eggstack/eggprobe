# Transport M006 closure

Status: closed — no upstream change required

Source plan: `plans/implementation/transport-probe-engine/006-upstream-diagnostic-observability-refinement.md`
Implementation: `792ad65524a869d2ec22ba88dd9daea427c74cab`

## Evidence-driven disposition

Working M003–M005 consumers demonstrate that the current public Eggfetch
observer exposes response and negotiated metadata but not portable per-phase
DNS/TCP/TLS timings. Eggress exposes route connection metadata but not a stable
per-hop timing observer required for parity. Eggprobe records those facts as
explicit `unavailable` fields; it does not infer timings or parse display text.

No sibling project change was justified by the current evidence. The gap is
documented in `docs/operator.md` and remains eligible for a future additive
upstream observer plan.

