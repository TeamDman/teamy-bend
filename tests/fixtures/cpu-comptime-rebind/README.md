<!-- SPDX-License-Identifier: MPL-2.0 -->
# Compile-time reference pattern diagnostics

Reference-valued compile-time arguments retain exact invalid-pattern diagnostics at original per-use spans. Cases cover assignment, match rows, imported/qualified arguments, multiline and nested fields, and earlier shared uses. Original fixtures remain byte-identical. Lambda-binder diagnostics remain a separate unresolved observation. Constructor-valued compile bindings are not qualified by these reference controls.

Public ParseError fields and Display stay unchanged. The existing CLI structured boundary returns Ok(CliOutput) with exit_code1; external invoke consumers remain unaudited. Generated execution, pure normalization and strict IO checking refusals are distinct observations.

The local positive exposes an unchanged normalization gap: upstream returns `12n`, while port normalization and pure run_main retain `Nat.add(11n, 1n)`. Complete port graphs are compared against that explicitly retained observation. Generated C/JS still execute to `12n`; a separate explicit IO wrapper exercises native execution to `12`. This is not a normalizer correction or parity exception.
