# Quote and grammar diagnostic controls

These MPL-2.0 controls exercise the pinned Bend parser at
`e6676b080f25b1bc1bf5b5b7d7a17e22f8022599`. The `.stderr` files retain exact
upstream load failures. `cases.json` records unchanged raw Rust parser fields;
these differ from the optional formatted diagnostics by design.

Quantified names consume one `+` or `-` before the name upstream. Ordinary,
already-quantified, compile-time and existential positions provide controls
against applying that diagnostic offset to unrelated grammar. Later unclosed
literals and a preceding raw apostrophe test first-error attribution.

The original four cases remain in `cpu-empty-character-context`. Three now have
exact diagnostic matches. `four-quotes` remains an explicit difference: legacy
tokenization succeeds, so reconstructing another diagnostic parse would need a
separate design for isolated, bounded recovery. No such recovery is implemented.

Checking, evaluation and all executable emitters must refuse these inputs;
rejection does not establish executable semantic parity. Accepted apostrophe,
template and IO controls remain in the adjacent regression suite.
