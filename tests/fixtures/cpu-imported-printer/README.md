# Imported constructor display names

New sources and tests use MPL-2.0. `local` and `imported` preserve the exact
minimal sources recorded by the historical ragged-row printer investigation.
Other cases isolate aliases, nested modules, declared dotted names, identical
local tags across modules, imported Con/Nil/Tuple spellings and Base sugar.

Expected text comes from pinned Bend interpreter and C O3 observations, retained
separately from generated-JS printer timeouts. Runtime tags and foreign ABI names
must stay local; only pure-output display names use canonical qualified names.
