# Demand and private-request contract review

These five authored MPL-2.0 programs are independent, small reproducers for
unresolved native/compiled and upstream C/JS disagreements at Bend pin
`e6676b080f25b1bc1bf5b5b7d7a17e22f8022599`.

They are research inputs, not assertions that one observed behavior is approved.
Each declares its own main and needs only Base. Run each through native execution
and generated C/JS, separately comparing upstream C and JS. Use the documented
CPU profile `--threads 1 --gpu off`; retain raw streams and status twice with
bounded execution. No program should print the synthetic `SECRET` request text.

The diagnostic correction does not change their behavior. The decision and
measured results are recorded in `docs/cpu-demand-contract.md` at the repository
root. Do not silently accept one oracle when the two upstream lanes disagree.
