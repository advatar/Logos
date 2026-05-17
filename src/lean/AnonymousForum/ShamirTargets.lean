/-
Proof targets for the next Lean milestone.

The current Lean files prove the structural moderation/slash invariants. The
next milestone is a finite-field polynomial formalization with these theorems:

1. lagrange_reconstructs_original_polynomial

   Given K distinct x-coordinates and shares yᵢ = P(xᵢ), where degree(P) < K,
   interpolation returns P exactly.

2. fewer_than_k_shares_are_ambiguous

   Given t < K shares, there are multiple degree < K polynomials consistent
   with those shares, supporting the Shamir part of the unlinkability argument.

3. duplicate_share_x_values_are_rejected

   Slash verification must reject repeated x-coordinates before interpolation.

No production correctness claim should cite this file until these targets are
implemented without `sorry`.
-/
