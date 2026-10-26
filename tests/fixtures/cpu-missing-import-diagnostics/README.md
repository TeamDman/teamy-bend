# Missing-import diagnostics

The five import originals are unchanged from the pinned upstream files in NOTICE.
They stop at missing files while loading. Their comments do not qualify actual
IO, proof chaining, cycles or diamond loading.

Ten exact negatives cover the originals, first missing sibling, an import before
Base, CRLF/comments, EOF and the immediate nested importer. Goldens retain exact
pinned stderr; before.json preserves compact pre-fix evidence with machine roots
replaced. Invalid-source execution remains unrun.

Two positive programs exercise real present imports and diamond deduplication.
Their generated C/JS execution and complete port normalized graphs are separate
from pure-run normalization and the upstream interpreter's text.

Unit controls preserve public ParseError fields/Display, missing-root and other
filesystem errors, canonicalization-before-depth ordering, the 64-level guard,
cycle refusal, dangling-symlink spelling and raw source excerpts. Ambiguous
lexical paths and hash/Unicode tokenization keep unformatted fallbacks. The
Windows-relative spelling fallback remains unqualified; absolute POSIX import
spelling does not depend on the importing file's host separators.

Optional private diagnostic metadata and derived Debug may change. Newly
formatted CLI refusals use the existing Ok(CliOutput)/exit_code1 boundary;
external invoke callers remain unaudited. No public type field, filesystem
operation, resolution rule, semantic exception or resource limit changes.
