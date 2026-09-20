# Compatibility and verification scope

The reference is Bend 2.0.5, revision
`e6676b080f25b1bc1bf5b5b7d7a17e22f8022599`. This tool currently implements a
supported subset. It is not a drop-in replacement for the full upstream CLI.

## Supported surface

- Dependent functions, algebraic datatypes, kinds and quantities; erased,
  affine and reusable binders; structural recursion; equality, reflexivity and
  equality rewriting. Every ordinary law must have a proof. Unsafe/foreign
  assumptions and holes in checked terms (including holes discarded by
  reduction) are refused.
- Source `type`, `def`, `law`, closed compile-time templates, nested/multiple
  constructor patterns, parallel lets, reusable binders, local module
  imports/aliases, literals and operators,
  pure do notation and array syntax lowering. Syntax support alone does not
  provide an absent library operation or runtime intrinsic.
- Pure normalization, typed Nat batch calls, persistent constructor calls,
  and Rust-authored JavaScript/C11 generation with closures, constructors,
  pattern matches and simultaneous let scope. Persistent calls use a separate
  lazy native data runtime after checking their arguments. No TypeScript
  interpreter implements these operations.
- 290 selected pure Base source declarations: dependent pairs/existentials, sums,
  equality helpers, Bool/Cmp, Nat arithmetic, Maybe/Result/List, word structure
  and Map, Char/String operations, List templates and pure Word/U32 helpers.
  `src/syntax/base.bend` is the exact inventory (254 definition forms including
  14 templates, 18 laws and 18 datatypes). Templates enter the checked book only
  when instantiated, so check-report counts differ from source-form counts.

Foreign C/JS bodies, GPU calls, hub fetch/publish,
effects, the full array/word/numeric library, optimized C and GPU
backends, and upstream CLI parity remain unfinished. F32 syntax/representation
does not establish arithmetic or floating-point proof support.

## Deliberate resource limits

Compile-time template arguments must be closed. Specialization uses structural
keys that preserve binder identity and share alpha-equivalent arguments.
Instances retain ordinary erased-argument type checks, affine/reusable resource
checks, source ordering and structural descent. Like upstream, unused template
bodies are parsed but not type-checked until instantiated. The complete module
graph allows at most 256 instances and 16 nested specializations; each key is
limited to 2,048 bytes, 128 term levels and 4,096 visited term nodes.

The kernel uses a per-definition/evaluation step budget and nesting limits.
Its current AST input and dynamic recursion limits are 128; the step budget is
2,000,000. Parsing caps expression/body/do nesting, pattern compilation and
import nesting at 64; telescopes, argument lists and strings at 128; Nat literals
at 128; expression spines and intermediate term depth at 256. A combination of
constructs can reach a stricter limit before any individual limit is reached.

Limits return failure, never a successful proof. They deliberately restrict
large programs until compact representations and iterative traversals are
implemented. Evaluation performance and sharing are not at upstream parity.

The persistent data runtime has separate arena, continuation and output limits
documented in [the protocol](data-protocol.md). It does not replace the proof
normalizer. The baseline C backend limits instruction generation to 250,000
nodes and defaults to 2,000,000 runtime steps, 64 MiB of tracked allocations,
nesting depth 1,024 and 8 MiB of output. Generated programs release their arena
after either success or failure. C and JavaScript emit an explicit erased marker
for proof/type results; the constructor-only data protocol rejects such results.

## Reproduce the upstream audit

The 2026-09-19 audit covered all 1,302 fixtures: 349 expected-positive programs
checked, 504 expected-positive programs were rejected, and all 449 expected
failures were rejected. There were zero abnormal exits and zero accepted
expected-failure fixtures. These are acceptance counts, not a parity percentage.

The template/collection slice adds five accepted positives over the previous
344-positive audit. All 32 upstream comptime fixtures still reject because the
positive programs require other unfinished features; this category does not
establish template parity. The 13 focused template tests, import tests and
native/JavaScript/C execution comparisons provide the supported evidence.

The earlier numeric/parser expansion exposed a debug Windows stack overflow in `run/fuel_loops.bend`
was reduced to a large unary Nat, fixed with a bounded CLI worker stack, and
rerun through this complete audit. Kernel proof limits remain unchanged.

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

The separate `micro-v1` model now covers the complete six-card, two-seat 1/2/1
state schema. Its trajectory gate passes 21 transitions, 22 inspections, all
300 deal partitions and eight rejection controls. A distributed sample passes
1,000 states and 1,000 edges. Seven compiled witnesses and two well-typed model
mutants also pass their expected-value checks.

The final native comparison with the clean `6802c1e` release completed on
2026-09-19: all 431,800 reachable states, 549,896 labeled edges and 863,600
player observations matched. It also checked all 300 deal partitions and
eight rejection controls. Its 981,720 requests completed in 1,658.848 seconds.
This is exhaustive conformance for the registered six-card, two-seat 1/2/1
graph, not an inductive proof or full-deck/network verification.

The receipt fingerprints the actual executable, frozen model, compiled Rust
adapter/oracle sources and transcript. The checked model BLAKE3 is
`3798ecaad4f3bff64d9648452c0f7f2090bde1294a5beb8c505d8bce4fedf226`;
the transcript BLAKE3 is
`a273a13c52d7e36f956c30b0675cfefeb60ace620dece023b0beb361b0bbed39`.
The Poche integration remains in its local checkout; these results do not
imply that its changes have been published.
