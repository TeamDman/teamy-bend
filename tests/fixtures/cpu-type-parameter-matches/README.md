# Type-parameter matching

Two unchanged Apache-2.0 originals from pinned Bend
`e6676b080f25b1bc1bf5b5b7d7a17e22f8022599`:
`tests/flatten/type_param_match_000` and `_001`. Derived full graphs, normal
forms and diagnostics retain attribution. New tests/adapters/wrappers use MPL-2.0.

Actual standalone builds require Base; typed function adapters then refuse
printing. Kind probes check Type, Data, the local Nat datatype and Nat -> Nat
as the leading Type argument. Passing Z{} instead must reject before execution
or artifact generation even though A is unused. Data fits Type under the pinned
kind order; function types have kind Type.

The 72 application inputs use Nat and Nat -> Nat as distinct leading arguments.
Fixture 000 uses all nine ordered pairs from [], [S{Z{}}], [Z{}, S{Z{}}] for each
type argument (18 inputs), returning the first list's tail or [] regardless of
the second list. Fixture 001 uses all 27 ordered triples of Nat depths 0,1,2 for
each type argument (54 inputs): F retains b,d when c is zero; G retains b and d
and c's predecessor otherwise. Projection tags F=0 and G=1 are distinct from the
complete constructor results. Original graphs, closed results, projections,
checking and actual runtime lanes remain separate evidence. Earlier upstream
imported-constructor printer failures are not waived.
