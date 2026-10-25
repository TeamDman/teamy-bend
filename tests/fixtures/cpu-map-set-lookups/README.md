<!-- SPDX-License-Identifier: MPL-2.0 -->
# Map/set and list/string lookups

The originals are byte-identical copies of `tests/run/map_set_ops.bend` and
`tests/run/list_string_lookups.bend` from upstream Bend
`e6676b080f25b1bc1bf5b5b7d7a17e22f8022599`, covered by the Apache-2.0 attribution
in NOTICE. Authored edge wrappers are MPL-2.0 and import the unchanged originals.

Map/set tests cover the bundled API, including empty versus NUL-character keys;
NUL remains internal and is observed through numeric lookup results. Unicode key
ordering and duplicate Set entries are distinct observations. List.get/set are
bundled APIs, while sfind/strim/seq are fixture-local algorithms. The original's
comment about absent String.contains is stale and does not define library scope.

These are IO executable contracts. Strict proof checking refuses IO separately;
that is not a production failure or a runtime match. Golden output is checked
against pinned generated C O3 and JavaScript before qualification.
