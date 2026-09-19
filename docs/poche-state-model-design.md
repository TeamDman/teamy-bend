# Next Poche scope: complete micro transition system

This design records verified integration points for implementation-plan task
4.3. No state-model result is claimed by the completed scalar gate.

## Established reference

The existing `poche-check` micro graph contains 431,800 states, 549,896 labeled
edges, 176 terminal states and maximum depth 20. The scope is two seats, two
suits, three ranks, six cards, with hand schedule 1/2/1. Inspect the current
checker before reproducing those counts; an interrupted/state-limited run is
incomplete evidence.

The public getters in `poche-model/src/state.rs` expose the needed payloads.
Do not reuse `poche-interchange::StateWire`: it omits round identity and uses
completed trick history, while the strict model stores captured sets by winner.
No private-field visibility changes are needed for an exact adapter.

## Independent Bend state and action schema

Use finite constructors for two seats, three round ordinals, six cards and three
bids. A card set contains six Boolean fields, giving precisely 64 subsets.
Keep `micro.bend` standalone initially: importing scalar kernels namespaces its
private Nat, while the numeric batch shorthand currently builds unqualified
Zero/Succ values.

```text
Ledger(dealer, round, score0, score1, misses)
BidProgress = First | DealerLast(first_bid)
TrickProgress = Lead(leader) | Follow(leader, card)
Game =
  AwaitingDeal(ledger)
  Bidding(ledger, hand0, hand1, trump, undealt, progress)
  Playing(ledger, hand0, hand1, trump, undealt, bid0, bid1,
          trick, captured0, captured1, tricks0, tricks1)
  Scoring(ledger, trump, undealt, bid0, bid1,
          captured0, captured1, tricks0, tricks1)
  Finished(score0, score1, misses, winner0, winner1)
Action = DealOne(partition) | DealTwo(partition) | BidAction(seat, bid)
       | PlayAction(seat, card) | Settle | Absorb
StepResult = Rejected(reason) | Applied(next, optional_score_events)
```

Scores are bounded 0..64; misses 0..6; pot is `50 + 10*misses` cents.
Implement `initial`, `valid`, `observe`, `action_surface`, `deals_one`,
`deals_two`, `step`, `rank`, and `inspect`. The last combines validity, rank,
both observations and the action surface for efficient comparison.

Deal enumeration has 120 one-card partitions and 180 two-card partitions.
Compare those lists once, then represent an awaiting chance surface by its
round-specific tag rather than retransmitting all partitions on every state.

`step` follows `poche-model/src/semantics.rs`: enforce phase/action agreement,
scheduled partition, nondealer-first bidding, fixed bids, ownership and follow
suit; remove played cards, award both trick cards, increment the winner, and
let that winner lead. Settlement separates point and money events, rotates the
dealer and advances the schedule or finishes. Only Finished+Absorb self-loops.
Reject invalid input states and validate every successor.

Observations include phase, viewer, optional dealer/action owner/round, the
viewer's hand, both hand counts, optional trump/lead play, optional bids,
trick counts, scores and pot. Finished clears transient round and hand fields.

## Generic constructor transport prerequisite

Nat-only batch cannot return game states. Add a persistent checked-once NDJSON
command with request IDs, named entries and recursively tagged constructor
arguments. The JavaScript backend already emits compatible constructor trees.

```json
{"id":17,"entry":"step","args":[{"constructor":"AwaitingDeal","fields":[]}]}
```

This is a protocol-shape example only; the actual constructor requires its
typed ledger field. Responses carry the same ID and a `value` tree, or an
explicit error. Feed all input through `CheckedBook::evaluate` type checking.
Allow data constructors and optional Nat shorthand, never arbitrary expressions
or references. Bound request length/depth and reject missing/extra fields,
unknown constructors and duplicate/missing IDs. A persistent process is needed
for nearly one million state/edge checks.

## Adapter and deterministic first replay

Add `poche-conformance/src/bend_micro.rs` with encoding functions for Game,
ModelAction, Observation and Transition using their public getters. Compare
Bend trees with independently encoded Rust values; do not reconstruct private
Rust Game fields.

Start from the existing `compare_common_prefix()` trajectory:

1. Dealer Seat1, hands `{C0}`/`{C1}`, trump C5, remaining `{C2,C3,C4}`.
   Seat0 bids 0, Seat1 bids 1, plays C0/C1, settles to scores `[10,21]`, pot 50.
2. Hands `{C0,C5}`/`{C1,C3}`, trump C2, remaining `{C4}`. Seat1 bids 1,
   Seat0 bids 2; plays C1/C0 then C3/C5; settles to `[10,32]`, pot 60.
3. This is the existing 14-transition/28-observation/10-action-set common
   prefix. Conventional play next uses size 3; the micro model uses size 1.
4. Finish the micro schedule using the first partition, bids 0/1 and plays
   C0/C1. Expect `[20,53]`, pot 60, sole winner Seat1, followed by Absorb.

## Complete finite acceptance

Reuse `poche_check::explore(CheckScope::Micro, ExplorerConfig::default())` and
its public states, edges and shortest counterexamples. Compare each state's
validity, rank, both observations and the full independent action surface;
compare each edge's successor and settlement events. Equal initial states,
action sets and successors establish agreement over the reachable system.

Rank is zero at Finished. Otherwise add the future-round action count (14, 6,
0 for rounds 0, 1, 2) to the current phase remainder: AwaitingDeal `4+2*h`,
Bidding `bids_remaining+2*h+1`, Playing `hand0_count+hand1_count+1`,
Scoring `1`. Each nonterminal edge must decrease rank exactly one.

Check exact card partition, captures=2*tricks, immutable bids, actor order,
follow-suit legality, winner-led continuation, score/payment separation,
complete maximum-score winner masks and terminal-only absorption. Negative
controls independently introduce card duplication/deletion, changed bids,
wrong actor, illegal off-suit play, wrong leader, wrong scoring, nonterminal
self-loop and terminal redeal.

Expose `compare rust bend --scope micro-v1`. Preserve the existing scalar
scope and receipt independently. These remain finite-scope guarantees.
