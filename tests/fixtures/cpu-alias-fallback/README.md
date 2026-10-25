<!-- SPDX-License-Identifier: MPL-2.0 -->
# Missing alias-member fallback

Authored controls for `parse_reso` at pinned Bend
`e6676b080f25b1bc1bf5b5b7d7a17e22f8022599`. Both files under `original` are unchanged
copies of the retained authored reproducer; they are not upstream inventory files.

The resolver selects one candidate: the alias-qualified name when an alias exists,
otherwise the current namespace-qualified name. A known candidate wins. An unknown
candidate falls back directly to the raw spelling, with no second namespace search.
Controls cover existing imported definitions/constructors/templates, raw-name law
completion and specialization, nested raw Base-name fallback, undotted names and a
separate native IO program. A nested negative proves that a local qualified name
does not rescue an unknown alias member when its raw name is undefined.

Seven exact rejection goldens come from the pinned checker. Unknown-template
rejection is separate: both implementations refuse during loading, but their
messages differ. It is not an exact diagnostic pass or a new policy exception.
The IO program remains excluded from strict proof checking. Existing foreign and
builtin-origin controls remain in the repository suite; these fixtures do not
qualify foreign host execution.
