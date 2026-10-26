# Raw apostrophe lexer controls

These MPL-2.0 controls exercise the pinned Bend parser at
`e6676b080f25b1bc1bf5b5b7d7a17e22f8022599`. They are authored regressions,
not copied upstream inventory entries. The `triple-quote`, `triple-argument`
and `escaped-quote` originals preserve the earlier diagnosis byte-for-byte.

The pinned character parser reads one code before requiring its closing quote;
therefore `'''` denotes code 39. Optional argument separators also allow six
or nine consecutive quotes to supply two or three character arguments. Token
spelling and byte spans must remain intact for template replay.

`cases.json` identifies 12 positive and 17 negative controls. The file named
`positive/unused-template.bend` is deliberately retained where the exploration
created it: both implementations reject its empty character template; the
manifest classifies it as negative. `escaped/` supplies equivalent successful
controls for complete normalized-value graph and native-result comparisons.
The IO control is executable and is excluded from strict proof checking.

Fifteen negative controls retain the previous public raw load-error fields.
Two observable raw errors change: an invalid declaration name contains the
new three-quote token, and a successfully repaired imported module exposes a
later root-file grammar error. These are not claimed as unchanged diagnostics.
Invalid quote counts remain rejected. Canonical parser depth and literal-size
guards remain in force; a pre-existing global lexical refusal is restored on
failed parses, so its public message can precede an internal depth refusal.
Generated runtime comparisons are recorded separately from native pure
normalization; a matching phase is not an exact diagnostic match.
