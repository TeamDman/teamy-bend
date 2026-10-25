<!-- SPDX-License-Identifier: MPL-2.0 -->
# Typed compile-time binders

The unchanged upstream originals are attributed in NOTICE.md. Missing-colon
controls cover compile-time, erased and reusable binders; comma, closer and EOF;
imported/multiline source; datatype parameters and constructor fields. Expected
stderr comes from the pinned checking adapter. Missing name/type and missing
compile-time argument controls retain their distinct load/check refusals.

Four pure controls and two unchanged IO programs exercise typed compile-time
values/types, specialization replay, erased/reusable arguments and bare quantity
binders. Pure normalization, executable native IO and emitted C/JS execution
are separate observations. Strict proof checking excludes IO.

`comptime/wrong_type.bend` retains an unresolved checking diagnostic difference;
it is not in the exact-diagnostic manifest. Public ParseError fields and Display
remain unchanged. Formatted CLI failures use the existing status-bearing Ok
result with exit_code 1; external invoke consumers remain unaudited.
