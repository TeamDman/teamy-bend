# Imported Nat literal control

These two authored MPL-2.0 files preserve the original exploratory reproducer
byte for byte. They are not an upstream inventory fixture. The imported module
defines a distinct Nat datatype. Pinned Bend expands numeric Nat sugar to the
fixed Zero/Succ constructors, while explicit names resolve in the module.

The correction must reject the literal at constructor checking, before reaching
reflexivity. Its raw constructor diagnostic is not yet an exact upstream match.
See [the pinned numeric parser](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/bend.ts#L2210).
