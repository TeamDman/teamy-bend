# Four-quote recovery controls

Authored MPL-2.0 fixtures compared with pinned Bend
`e6676b080f25b1bc1bf5b5b7d7a17e22f8022599`. Original four-quotes remains
unchanged in `cpu-empty-character-context`.

These eight adjacent load refusals distinguish declaration, group, argument and
list contexts from earlier grammar/escape/duplicate errors and a genuine parser
depth limit. At f64eb0a, five diagnostics matched after bounded recovery.
Argument and list remained different; subsequent tracing distinguished the
argument's preceding-declaration lowering barrier from the list's terminal atom. The guard fixture keeps the port's 64-level refusal, distinct from the
upstream syntax diagnostic. No case supplies executable success evidence.

The f64eb0a checkpoint measured five exact/three differing contexts. Terminal
incomplete-character metadata subsequently qualifies the list context (six/two).
The original preceding-function argument and depth refusal remain different.
See ../cpu-incomplete-character/README.md for the bounded mechanism and limits.
