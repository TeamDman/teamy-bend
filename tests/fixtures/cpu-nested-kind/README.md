<!-- SPDX-License-Identifier: MPL-2.0 -->
# Nested kind and annotation source locations

`deeper-kind.bend` is the unchanged project-authored reproducer retained from the
previous checkpoint. The thirteen negative controls reject during checking;
expected diagnostics come from pinned upstream
`e6676b080f25b1bc1bf5b5b7d7a17e22f8022599`. They cover quantified domains/codomains,
annotation types/values, multiline and imported locations, nested arrows,
reusable grades, and first-domain failure before an undefined codomain.

The correction records weak, external child-source anchors for quantified types
and annotations and registers quantified atom locations. It neither changes terms
nor redirects errors through a broad parent fallback. Independently sourced
replacements keep their own locations; unsourced substituted variables use the
specific original child edge. Reference sharing and evaluation remain unchanged.

`before-receipt.json` preserves exact pre-fix process streams from tested source
`ddd4f0a9bcc6d8b224b1c17d45fe63b91f6712ac` / checkpoint
`d3259a053be6164305028fbbf427b767105f1b4a`. Command roots are placeholders.
The fixture manifest is exact negative checking/compilation-boundary evidence,
not a claim that rejected programs executed.
