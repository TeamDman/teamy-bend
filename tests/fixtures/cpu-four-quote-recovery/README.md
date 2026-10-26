# Four-quote recovery controls

Authored MPL-2.0 fixtures compared with pinned Bend
`e6676b080f25b1bc1bf5b5b7d7a17e22f8022599`. Original four-quotes remains
unchanged in `cpu-empty-character-context`.

These eight adjacent load refusals distinguish declaration, group, argument and
list contexts from earlier grammar/escape/duplicate errors and a genuine parser
depth limit. Five diagnostics match exactly after bounded recovery. Argument and
list recovery still stops at an incomplete atom; those two differences are not
waived. The guard fixture keeps the port's 64-level refusal, distinct from the
upstream syntax diagnostic. No case supplies executable success evidence.
