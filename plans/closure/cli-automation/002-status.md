# CLI M002 closure

Status: closed

Source plan: `plans/implementation/cli-automation/002-composite-check-assertions-and-exit-codes.md`
Implementation: `792ad65524a869d2ec22ba88dd9daea427c74cab`

## Finding and evidence

Typed `AssertionKind` values are evaluated by a pure core function. Composite
`check` preserves child probe evidence and separates findings from transport
errors. HTTP 503 is observed successfully while a 2xx assertion fails in the
local fixture. Exit mapping is 0 success, 1 negative/unsupported, 2 invalid,
3 reserved internal, and 130 interruption.

The evaluator returns `Unavailable` when evidence is absent; it never invents
timings or protocol facts. Workspace tests and strict verification passed.

