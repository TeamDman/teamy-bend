# No-main checking fixtures

These nine `.bend` files are unchanged copies from
[bendlang/bend at e6676b080f25b1bc1bf5b5b7d7a17e22f8022599](https://github.com/bendlang/bend/tree/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/tests).
Copyright 2026 HigherOrderCO; Apache-2.0. See NOTICE and
`licenses/Apache-2.0.txt`. Their relative paths preserve the upstream location.

`normal-forms.tsv` records actual pinned interpreter normal forms for all 30
source-defined functions. These are checking/normalization cases, not executable
programs: the default command reports checking success, named definitions can
normalize, and standalone C/JS emission refuses a missing entry. The two modules
without Base meet the upstream Base prerequisite refusal first. Compiler diagnostic
text differences remain distinct from rejection/artifact agreement.
