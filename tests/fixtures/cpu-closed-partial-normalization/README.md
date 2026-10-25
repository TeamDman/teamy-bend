# Closed partial-application normalization controls

Authored MPL-2.0 fixtures. The unchanged original is
`../cpu-comptime-rebind/positive/local.bend`; native IO is separately observed
with `../cpu-comptime-rebind/positive/local_io.bend`.

`before.json` preserves exact compact pre-fix streams, fixture hashes and the
actual binary/source identity. Expected stdout/stderr files come from the pinned
upstream interpreter, not from the candidate port. Tests compare every normalized
constructor/application/binder node separately from exact printed text. The
negative fixtures retain rejection; `true_proof` previously failed incorrectly.
Open function results and proof results are normalization evidence, not automatic
claims of executable printer applicability. Resource, sharing and cancellation
controls are in the kernel and integration tests; no guard or budget is raised.
