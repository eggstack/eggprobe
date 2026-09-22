# CLI and Automation M002 — Composite Check, Assertions, and Exit Codes

Status: closed

Planning baseline: `e55a21fe73915b7a38c2fa674807354539806c60`; refresh after CLI M001 closes.

Source roadmap:

- `plans/subsystems/cli-automation-roadmap.md#m002--composite-check-assertions-and-exit-codes`

Hard dependency: CLI M001 closed.

Primary class: capability

## 1. Objective

Add a composite `check` workflow, typed assertion evaluation, and stable coarse process exit semantics while preserving raw probe evidence.

## 2. Required production changes

- core-owned assertion evaluation over completed `ProbeReport`;
- typed assertion variants rather than free-form expression parsing;
- `check` plan composition for common DNS/TCP/TLS/HTTP sequences;
- findings recorded separately from probe errors;
- exit mapping:
  - 0 completed + assertions satisfied;
  - 1 negative probe/assertion outcome;
  - 2 invalid invocation/plan;
  - 3 internal Eggprobe failure;
  - 130 interruption;
- assertion examples: HTTP status range, required HTTP version, required ALPN/TLS version, max total/TTFB/connect timing, certificate-expiry threshold only when evidence exists.

Unavailable evidence must produce an explicit unavailable finding according to assertion policy, not a fabricated pass/fail.

## 3. Work packages

A. Replace placeholder assertion declaration with typed variants.
B. Implement pure assertion evaluator.
C. Implement composite `check` plan construction.
D. Add exit-code reducer over report/finding state.
E. Update human/JSON output and docs.

## 4. Tests

- HTTP 503 observed successfully but status assertion fails;
- probe failure separate from assertion failure;
- unavailable evidence result;
- multiple assertions with mixed outcomes;
- exit codes 0/1/2/3 and interruption path;
- composite child evidence preserved in order;
- assertion evaluation deterministic and side-effect free.

## 5. Acceptance criteria

Assertions never alter or hide probe observations. Exit codes remain coarse and JSON contains detailed reasons. `check` uses existing probes rather than a second diagnostic implementation.

## 6. Stop conditions

Stop if assertion evaluation requires free-form scripting, if a timing assertion depends on unobservable phase timing, or if exit-code requirements cannot be derived from canonical report state.

## 7. Closure evidence

Create `plans/closure/cli-automation/002-status.md` with assertion matrix, exit-code integration evidence, composite report fixture, unavailable-evidence cases, and compatibility review.
