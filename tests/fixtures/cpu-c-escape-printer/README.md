# Executable C escape printer

The three unchanged pinned `show` originals and authored ASCII/control/Unicode
controls qualify the executable C language printer. The strict compiler JSON
encoder and raw IO/file encoding are separate contracts and remain unchanged.

`cases.json` records exact pinned printed bytes and existing native normal-form
text separately. Scalar output is loaded, checked and compared as a complete
normalized structure. Non-scalar constructor values print as braced hex in C;
JavaScript still refuses them and the port literal decoder still refuses their
readback. This is corresponding-C fidelity, not cross-runtime parity or a new
scalar policy exception.

The unchanged 95-character ASCII control exceeds the existing kernel input-depth
guard. It remains unqualified. Additional short ASCII controls cover the range
without claiming the failed aggregate passed. Literal length and all other
resource limits remain unchanged.

C helper probes use the generated runtime with TB_NO_MAIN to seed existing
counter boundaries; they do not alter production budgets. The full-u64 probe
checks buffer/accounting safety, not language acceptance of a 64-bit Char.

Historical baseline records preserve invalid UTF-8 output as base64 rather than
lossy text. Earlier failed launch and harness attempts remain in local receipts.
