# Persistent data protocol

Run `teamy-bend serve <program.bend>` to load and check one program, then call
its definitions over standard input. The process reads one JSON request per
line and flushes one JSON response per line. End standard input to finish.
Standard output always uses this protocol, including when another global
output format was selected. Diagnostics go to standard error.

## Requests and responses

```json
{"id":1,"entry":"identity","args":[{"constructor":"Succ","fields":[{"constructor":"Zero","fields":[]}]}]}
```

Arguments are constructor trees. The kernel checks their constructors, fields,
types and number against the definition on every call. References, variables,
expressions and host callbacks are not part of the protocol. Natural numbers
use the program's constructors; there is no numeric shorthand. Unknown object
fields, duplicate fields and trailing content after the request object reject.

```json
{"id":1,"value":{"constructor":"Succ","fields":[{"constructor":"Zero","fields":[]}]},"error":null}
{"id":2,"value":null,"error":"undefined definition missing"}
```

Each response has `id`, `value` and `error`. Exactly one of `value` or `error`
is non-null. A malformed request has a null ID. Error text is diagnostic, not
a stable machine-readable error code. Results must be constructor data;
functions, types and proof terms cannot be returned as data.

Request IDs are unsigned 64-bit integers and must strictly increase within
the session. A decoded request consumes its ID even if its typed call fails.
Malformed requests and repeated or decreasing IDs do not advance the last ID.
An ID of `18446744073709551615` therefore permits no subsequent request.

Malformed JSON and typed-call errors receive error responses and the session
continues. Oversized lines and invalid UTF-8 are fatal framing errors: the
process emits an error response, then exits unsuccessfully. An invalid source
program fails before accepting calls.

## Resource bounds and isolation

- A request line is at most 1 MiB, including its newline when present.
- Raw JSON nesting is at most 208 levels.
- Constructor depth is at most 96, with the root counted as depth zero.
- All arguments share a budget of 16,384 constructor nodes; each response has
  the same independent node budget.
- The kernel's own input, evaluation and nesting limits also apply.

These are ceilings, not guarantees that every value below them evaluates.
Each call has fresh evaluation budgets and private aliases. Checked
declarations are shared immutably between calls. After argument validation,
data calls use a native lazy runtime that shares evaluated values within that
call. Proof checking and ordinary `eval` retain their separate normalizer.

The data runtime allows at most 2,000,000 steps, 131,072 retained thunk slots,
131,072 retained environment slots and 4,096 pending continuation frames.
Unreachable values are reclaimed at evaluator safe points; live IDs never move.
Collection traces pending frames, globals, environments and scoped caller roots
without evaluating terms. Its work consumes the same step budget and checks
cancellation. Caller roots are also bounded at 131,072 entries. A full live
graph still fails, and a large transition can exhaust the remaining space before
the next safe point. These caps now bound retained slots rather than cumulative
allocation over the whole call.

Output keeps the depth and node limits above. Each failed call discards its
arena, so a resource-limit error does not poison later calls. The CLI uses a
bounded worker stack on Windows and a bounded stdin queue so cancellation
remains responsive while input is idle.
