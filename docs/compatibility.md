# Compatibility and verification scope

The reference is Bend 2.0.5, revision
`e6676b080f25b1bc1bf5b5b7d7a17e22f8022599`. This tool currently implements a
supported subset. It is not a drop-in replacement for the full upstream CLI.

## Supported first milestone

- Dependent functions, algebraic datatypes, kinds and quantities; erased,
  affine and reusable binders; structural recursion; equality, reflexivity and
  equality rewriting. Every law must have a proof. Unsafe/foreign assumptions
  and holes (including holes discarded by reduction) are refused.
- Source `type`, `def`, `law`, nested/multiple constructor patterns, parallel
  lets, reusable binders, local module imports/aliases, literals and operators,
  pure do notation and array syntax lowering. Syntax support alone does not
  provide an absent library operation or runtime intrinsic.
- Pure normalization, typed Nat batch calls, and Rust-authored JavaScript
  generation with closures, constructors, pattern matches and simultaneous
  let scope. No TypeScript interpreter implements these operations.
- 114 selected pure Base declarations: dependent pairs/existentials, sums,
  equality helpers, Bool/Cmp, Nat arithmetic, Maybe/Result/List, word structure
  and selected String helpers. `src/syntax/base.bend` is the exact inventory.

Templates, foreign C/JS bodies, GPU calls, hub fetch/publish, implicit array-write
rebinding, effects, the full array/word/numeric library, optimized C and GPU
backends, and upstream CLI parity remain unfinished. F32 syntax/representation
does not establish arithmetic or floating-point proof support.

## Deliberate resource limits

The kernel uses a per-definition/evaluation step budget and nesting limits.
Its current AST input and dynamic recursion limits are 128; the step budget is
2,000,000. Parsing caps expression/body/do nesting, pattern compilation and
import nesting at 64; telescopes, argument lists and strings at 128; Nat literals
at 128; expression spines and intermediate term depth at 256. A combination of
constructs can reach a stricter limit before any individual limit is reached.

Limits return failure, never a successful proof. They deliberately restrict
large programs until compact representations and iterative traversals are
implemented. Evaluation performance and sharing are not at upstream parity.

## Reproduce the upstream audit

The 2026-09-19 audit covered all 1,302 fixtures: 302 expected-positive programs
checked, 551 expected-positive programs were rejected, and all 449 expected
failures were rejected. There were zero abnormal exits and zero accepted
expected-failure fixtures. These are acceptance counts, not a parity percentage.

Use an existing separate upstream checkout at the reference revision:

```powershell
cargo build --locked
cargo run --example audit_upstream -- <bend-reference>
```

The auditor runs each of the 1,302 upstream fixtures with `#|` expectations in a
separate native CLI process. It compares check acceptance only. A rejected
negative fixture can fail for a different reason; that does not establish
diagnostic parity. A runtime-error fixture can legitimately pass checking.
Runtime values, exact diagnostics, compiled behavior and performance require
separate comparisons. Abnormal exits are counted separately from rejections.

The complete audit output can be stored under ignored `target/`. Do not publish
raw diagnostics containing local paths. A snapshot executable may be supplied
as the second argument to keep development builds free during a long audit.

## Poche integration

The Poche integration is maintained in that project's `models/bend/`,
`crates/poche-conformance/src/bend.rs` and `docs/bend-conformance.md`. The native
gate checks seven equality terms and compares 15,503 scalar cases against
independent Rust rules, including the conventional scoring/trick oracle.
Both a false equality and a well-typed scoring mutant must expose their defects.

This establishes the documented finite scalar domains and two universal
definitional equalities. The entire game state machine, full deck, network
protocol, privacy, UI and money-transfer implementation remain outside that
receipt. Full state-model integration remains in the active implementation plan.
