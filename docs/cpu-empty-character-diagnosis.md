# Empty-character and raw-apostrophe diagnosis

Local investigation from reviewed checkpoint `878c0bb3860b8fd590c0949680a2ea263ff048d9`.
No production change, new semantic exception, dependency/toolchain change or remote operation.
Upstream remains `e6676b080f25b1bc1bf5b5b7d7a17e22f8022599`.

The retained `''` case is still a load-time diagnostic difference. Pinned
`parse_char` consumes the second apostrophe as code 39, then the Char parser
requires another apostrophe immediately. With only two quotes it reports
`expected : a closing '` and the following character (newline, space, `#` or `)`), or end of input.
The port lexes a zero-code quoted token, then rejects its cardinality; raw cursor
and message remain unchanged in this investigation.

This exposes one underlying **acceptance defect**, reproduced by a standalone
literal and the same literal in an argument:

```bend
import Base
def main() -> Char: '''
```

Pinned upstream accepts this raw apostrophe literal and the same literal in a
function argument. Both forms normalize to the escaped apostrophe spelling and
generate C and JavaScript programs that exit 0 with stdout bytes `27 5c 27 27 0a`
(the printed spelling of an apostrophe Char); stderr is empty. The port splits an
empty literal from an unmatched third quote and refuses loading with
`a string or character literal cannot cross a line`. Both C/JS emissions are
refused; **port execution of these two programs was not possible and was not run**.

The explicit escaped spelling `\'` inside a Char literal is an accepted control.
Its generated C/JS outputs agree across upstream and port. Pure port CLI output
is `Chr{39}` while upstream prints the literal spelling; this is distinct
presentation evidence, not a complete normalized-graph comparison or actual IO run.

## Bounded evidence

[Compact observations and source/runner hashes](cpu-empty-character-diagnosis.json.gz)
retain all 12 sources, 110 observations and 32 deterministic repeat pairs, zero timeouts.
The 12-case checking subset is **two acceptance mismatches, one jointly accepted
control, nine jointly rejected controls**. Of the nine rejections, one diagnostic
matches exactly and eight differ. Positive checking stdout differs by CLI contract
and is not counted as a diagnostic failure.

Sixteen generated executions passed (two repeats each): upstream C/JS for both
raw-apostrophe cases and the escaped control, plus port C/JS for the escaped control.
Four C builds used the pinned compiler with upstream's `-O3` profile; port C also
passed strict warnings. Children were bounded to 30 seconds. Existing 11/10/1 GiB
workspace preflight/active/temporary floors remained satisfied. No Rust build or
full gate was run for this documentation-only diagnosis; prior 1,327/0/49 results
belong to source `6c26f207626582deb04be75885325cfc9389197d`.

The probes reused the reviewed non-Tracy executable from that source
(`b6cee7475ec2510955ff3e44966adabac35c7100ce8f90eb50361950a65981a3`), not a
newly built binary labeled as 878c0bb. That checkpoint adds documentation only.
An initial runner syntax error occurred before any child execution; its log and
script remain local. Raw evidence and generated artifacts remain local; compact
receipts substitute explicit roots while retaining raw-stream hashes.

## Pinned basis and next step

- [parse_char](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/bend.ts#L1588) consumes an unescaped code point, including apostrophe.
- [Char parser](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/bend.ts#L1930) then requires the closing quote without skipping trivia.
- [Base Char](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/base.bend#L60) stores a U32 code. The guide illustrates `'c'` but does not separately specify triple quotes.

No new semantic-policy choice is apparent: the parser and both generated backends
agree, and the accepted escaped control has the same apostrophe value. The next
implementation should correct character tokenization, not merely format an error
for an upstream-accepted input. Preserve existing rejected-input raw errors,
first-error ordering, template replay and literal/depth guards; test empty, escaped,
three/four quotes and argument contexts separately. In particular, four quotes
currently yield the port cardinality refusal while upstream reports an unexpected
top-level quote after a complete apostrophe literal. Blind retokenization can
change that retained raw error; the implementation must address this explicitly. This diagnosis-only continuation
has **not** implemented or validated that change.

The denied main update remains pending; reviewed 878c0bb and its backups are untouched.
No retry, alternative publication route or further remote read/write occurred.
All prior unwaived parity decisions and census limitations remain unchanged.
