# Base identity and prelude-free constructors

`main_forged_io.bend` and `base_prelude.bend` are unchanged Apache-2.0 fixtures
from bendlang/bend at `e6676b080f25b1bc1bf5b5b7d7a17e22f8022599`:

- `tests/io/main_forged_io.bend`, SHA-256 `09526a3908faa7db1a8d9304de818dc542d09ebd6ebcbbafcecc0df662651934`
- `tests/import/base_prelude.bend`, SHA-256 `6b0aebbc5f983acc99b98e1bc8895a7edabc5880f77e3230901be544069c781d`

See the repository's upstream attribution and license notices. New application,
negative and import adapters carry MPL-2.0 headers. Expected graph/text/error
observations come from the pinned interpreter; they are not a replacement for
checking actual compiler applicability. Graphs include every constructor child
and binder position. The four depth applications append their fragments to the
unchanged prelude-free original. The two negative fragments append to that
original or the forged-IO original, as specified in `support/base_identity.rs`.

A user family named `IO` is ordinary data. The Base import adapter distinguishes
its identity from Base's actual effect family. The original forged-IO comment's
historical compiler prediction is not an oracle: pinned JS generates and runs,
whereas pinned C generation throws the retained TypeError. No C execution is
claimed after that failure. Both executable backends reject prelude-free builds
with the recorded Base requirement; the port's separate strict-pure C/JS API is
compared with the interpreter's complete constructor results.

Negative diagnostics cover unknown names, context/import origins, type and erased
positions, first missing-reference order and a wrong constructor domain. These
check rejection and stderr/status/artifact contracts without changing guards,
resource policy or approved runtime demand semantics.
