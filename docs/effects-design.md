# Effects and foreign execution

This records the full implementation contract and the remaining work.
It follows Bend 2.0.5 revision `e6676b080f25b1bc1bf5b5b7d7a17e22f8022599`.
The existing strict checker and Poche constructor protocol remain the proof path.

The first native console slice is implemented: separate loader/check products,
exact IO continuations and request values, ordinary IO helpers and three console
handlers. Ten upstream output/status comparisons and strict boundary regressions
pass. The arbitrary foreign interfaces, generated-target effect drivers and the
remaining inventory below are still incomplete.

## Checking contracts

Ordinary laws must be filled and live references obey declaration order.
Foreign definitions instead provide signatures and target-specific source
imports whose implementations are trusted. Upstream permits foreign results
of `IO(Type)` and even `IO({0n == 1n : Nat})`; accepting the signature does
not prove that equality. Executable checking therefore needs a separate result
type from `CheckedBook`. A foreign result must never become strict proof evidence.

The loader must retain foreign import paths, actual parameter arity and source
origin. Only the embedded Base loader can mint Base origin. Matching a familiar
name must not authorize an intrinsic or opaque unfilled law. Keep the existing
pure Base checked; do not expose upstream's harness facility for skipping a
number of declaration events as a strict-checking option.

Upstream's foreign return check requires a direct reference to the actual Base
`IO`, after traversing the function telescope and stripping annotations. It
does not unfold a return alias. Entry-point detection does unfold aliases.
These two rules need distinct tests. `@unsafe` also has precise execution-only
exceptions to descent and reusable-domain checking; ordinary checking must not
inherit them.

## Runtime representation

Preserve the source continuation encoding:

```bend
type IO.OP<-R: Type> is Type:
  Emit{value: R}
  Halt{code: U32, message: String}

def IO(A):
  @-R: Type -> @k: (A -> IO.OP<R>) -> IO.OP<R>
```

`IO.pure`, `IO.bind`, `IO.die`, `IO.pass` and `IO.try` are ordinary definitions.
A foreign call produces a runtime request carrying its arguments and
continuation. That request is distinct from both public constructors. Only
the effect driver executes it. Matching a request as `IO.OP`, including through
a fallback arm, must fail without performing the effect.

The driver supplies the terminal `Emit` continuation to a filled nullary main.
An `Emit` payload is discarded: `IO.pure(U32, 7)` exits successfully. `Halt`
writes its message and a newline to stderr, supplies the exit code and prevents
subsequent effects. A foreign main is rejected. A user datatype called `IO`
does not select this driver. Preserve separate pure-main behavior.

The first implementation stage adds native `run` and bundled console handlers:
print, write and print_err. Preserve UTF-8 bytes, embedded NUL, newline rules,
stdout/stderr ordering, cancellation and bounded resources. This stage does not
complete arbitrary foreign execution or the remaining effects.

The native runtime is lazy. Its console decoder rejects invalid Unicode scalar
values, but a discarded `IO.pure(Char, Chr{55296})` result is never decoded and
currently succeeds. Upstream JavaScript validates that constructor eagerly and
fails. Preserve this as an open execution-semantics difference when implementing
the generated JavaScript driver; do not conflate source Char validation with
the separate raw foreign-string contract below.

## Arbitrary foreign interfaces

Each backend chooses the first matching `.js` or `.c` import, resolves it
relative to the declaring file, and deduplicates canonical source paths.
Effects from a shared source share state. Symbols use local declaration names,
not importing namespaces. Only live parameters cross the foreign boundary;
live proof/type values are different from erased parameters.

Full JavaScript compatibility requires upstream's native values and callbacks:
BigInt Nat, numeric U32/F32, native Bool/Char/String/Array, null proof/type values,
and named constructor fields. Foreign results are trusted raw. Upstream tests
deliberately accept truthy numeric Bool values, negative BigInt Nat values and
lone-surrogate strings. The typed Poche decoder is not a replacement for this
interface. Returning undefined can suspend execution through saved continuations.

Full C compatibility requires the upstream effect registration/callback ABI,
constructor IDs and arities, namespaced constructor aliases and host inclusion
rules. Its GNU constructor attributes need an appropriate toolchain or a
faithful adapter on Windows. Rust-generated JavaScript may run in a JavaScript
engine; compilation must remain implemented in Rust. Neither target can be
declared complete from the bundled native console handlers.

## Remaining inventory and validation

The complete foreign Base inventory has 34 functions:

| Group | Functions |
| --- | --- |
| Console | IO.print, IO.write, IO.print_err |
| Environment and tasks | IO.get_env, IO.spawn, IO.sleep, IO.now |
| Channels | Chan.new, Chan.send, Chan.recv, Chan.close |
| Files | File.open, File.read, File.read_bytes, File.write, File.close |
| TCP | TCP.listen, TCP.accept, TCP.connect, TCP.send, TCP.recv |
| UDP | UDP.bind, UDP.send_to, UDP.recv_from, UDP.poll |
| Network cleanup | Socket.close, Listener.close |
| Windows | Window.open, Window.frame, Window.set_title, Window.close |
| Audio | Audio.open, Audio.write, Audio.close |

The scheduler, opaque handle ownership, ordinary IO.fork/join and App helpers,
and 37 unfilled numeric Base primitives remain required. Preserve asynchronous
suspension, channel close wakeups and tasks that outlive main.

Start actual-output regression testing with upstream `tests/io/hello_print`,
`hello_end_to_end`, `print_write`, `print_utf8_law`, `halt_utf8_law`,
`request_dead_fall`, `request_out_of_band`, `main_alias_io`, `main_forged_io`,
`main_foreign`, and `tests/parse/foreign_refill`. Add console NUL and emitted
nonzero-value/zero-exit cases. Capture stdout, stderr and exit status separately.
Then cover foreign arity, shared module state, runtime-name collisions, all
`marshal_*` fixtures and missing-target imports. Scheduler tests must cover
spawn lifetime, sleep order, rendezvous, channel closure and deadlock.

Reference implementation areas: `bend2/bend.ts` foreign checking/loading;
`bend2/main.ts` entry dispatch; `bend2/comp.ts` IO detection, marshalling and
drivers; `bend2/base.bend` declarations; `bend2/effs/` target implementations.
The strict proof and typed-session regressions must continue passing throughout.
