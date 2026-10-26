# Generic borrowed-array runtime qualification

`original/array_borrow_generic.bend` and `original/array_borrow_generic_sum.bend`
are unchanged copies of `bendlang/bend` tests/reg fixtures at
`e6676b080f25b1bc1bf5b5b7d7a17e22f8022599`. They retain Apache-2.0 licensing and
attribution; see `licenses/Apache-2.0.txt` and `NOTICE`. `controls/sum-second.bend`
is an Apache-2.0 derivative of the latter, changing only the initial read index
and corresponding expected result, with an added attribution comment.
New authored controls and Rust test code use MPL-2.0.

The generic alias control reads before cloning, swaps one clone to the opposite
sum arm, then observes the retained read, displaced cell, changed cell and
untouched clone. It covers U32/String arguments and both initial arms. Formatting
allocates nonempty strings/list representations while other retained cells stay
live. Expected output includes every selected payload and list in order.

These are observable generic-source ownership checks. The port may specialize
erased arguments; the results do not establish identical borrow ABIs or physical
layout transitions. Strict IO proof exclusion, stable-layout guards, and the
previous depth-32 generated-C resource refusal remain separate.
