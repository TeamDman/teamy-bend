# Executable JavaScript

`compile --executable FILE --output program.cjs` checks execution contracts
and generates a standalone CommonJS program. Run it with Node.js. The compiler
is implemented in Rust and consumes typed expressions from `ExecutableBook`;
foreign signatures cannot produce a strict `CheckedBook` or a proof result.
Unsafe declarations remain unsupported.

## Entry points and output

The entry is `main`. Actual Base IO, including aliases, selects the IO driver.
`Emit` discards its payload and returns status zero; `Halt` writes its message
and sets the exit status. Missing `main` prints `All terms check.`. Supported
pure result types print Bend values, including native numerics. Functions and
dependent result shapes that cannot be described by the printer reject during
compilation. Default strict `compile` retains its constructor JSON interface.

Console output uses synchronous UTF-8 writes, preserves embedded NUL and retries
partial/interrupted writes. Pending foreign requests have a private identity.
Ordinary matching, including a fallback arm, rejects a request before its effect
runs. Source Char constructors validate Unicode scalars eagerly; this also
catches a discarded invalid Char, unlike the current lazy native `run` backend.

## Synchronous foreign calls

The backend selects the first JavaScript import for each reachable foreign
declaration. It deduplicates canonical paths and embeds sources in one shared
lexical scope, ordered by live reference discovery from `main`. Symbols use
their declaring module's local names. Missing selected sources are errors;
later imports do not silently replace them. Unreachable foreign files are not
needed. Compilation reads source but does not execute it. Running the generated
program gives that source ordinary Node host access.

The foreign interface follows the reference JavaScript representations:

| Bend value | JavaScript representation |
| --- | --- |
| Nat | BigInt |
| U32 / F32 | Number, with binary32 rounding for F32 arithmetic |
| Bool | Boolean |
| Char / String | String |
| Array | Array |
| Other datatype | Object with local `$` constructor tag and named live fields |
| Live type or proof | `null` |
| Erased argument or field | Omitted |
| Function | Curried closure that evaluates before returning to host code |

Foreign return values remain raw. The interface deliberately preserves upstream
cases such as truthy numeric Bool values, negative BigInt Nat values and lone
surrogate strings. It does not apply the typed data protocol's validation to
these values. Runtime assumptions about foreign results do not establish proofs.

The synchronous driver passes live arguments and the trailing continuation.
An undefined return, promise or `_need` readiness hook fails explicitly.
Resuming saved continuations, timers, channels, host scheduling and the remaining
Base effects require further work. Synchronous companion helpers `io_bytes`,
`io_text`, `io_out`, `io_errs`, `io_done` and `io_tup` are provided. Native
system FFI and scheduling helpers reject explicitly pending the platform layer.
Foreign sources depending on upstream's Bun/POSIX system driver still require
that missing compatibility layer.

## Limits and verification

Generated Bend evaluation shares a 2,000,000-transition budget, allows 512
nested non-tail calls and limits native arrays to 131,072 elements. Tail calls
use a trampoline. Pure output limits are 96 levels, 16,384 visited nodes and
8 MiB of text; string construction and each console write also have an 8 MiB
byte limit. These are fail-closed execution limits, not upstream performance
parity. Arbitrary host JavaScript runs outside Bend's evaluation budget.

Regressions cover compile-time non-execution, unchanged output files after
failed checks, UTF-8/NUL, exit handling, callbacks, erased arguments and fields,
live proof/type nulls, shared source state, import order and request rejection.
Actual-output comparisons use separately generated upstream JavaScript and
capture stdout, stderr and exit status. Strict proof rejection and Poche
conformance checks remain separate gates.

A fixed release candidate passes 34 unchanged upstream console, foreign,
marshalling, import and numeric programs: 31 agree exactly on stdout, stderr
and status; three expected failures agree on stdout/status with different
diagnostic wording. A separate 34-case numeric matrix passes exactly, covering
31 generated boundary cases and the three upstream numeric programs already
included above. This includes quiet and signaling NaN payloads. Eleven focused
emitter tests and 18 foreign/driver tests supplement these comparisons.

The full rewrite still requires asynchronous scheduling, executable C, the
remaining numeric/library contracts, GPU support and the rest of the upstream
CLI. See the [implementation plan](implementation-plan.md).
