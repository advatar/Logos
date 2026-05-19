namespace AnonymousForum

structure Share (F : Type) where
  x : F
  y : F
  deriving Repr

/--
Proof contract for the Shamir/Lagrange layer.

The Rust implementation fixes the concrete field and interpolation algorithm.
This Lean layer exposes the theorem the slash proof needs without importing
Mathlib in the default build: any concrete field implementation must provide
`lagrangeSound`, then downstream proofs can use the theorem name below.
-/
structure ShamirSystem (F Polynomial : Type) where
  eval : Polynomial → F → F
  interpolate : List (Share F) → Option Polynomial
  degreeLt : Polynomial → Nat → Prop
  sharesFrom : Polynomial → List (Share F) → Prop
  distinctXs : List (Share F) → Prop
  lagrangeSound :
    ∀ {p reconstructed : Polynomial} {shares : List (Share F)} {k : Nat},
      degreeLt p k →
      shares.length = k →
      distinctXs shares →
      sharesFrom p shares →
      interpolate shares = some reconstructed →
      reconstructed = p

theorem lagrange_reconstructs_original_polynomial
    {F Polynomial : Type}
    (system : ShamirSystem F Polynomial)
    {p reconstructed : Polynomial}
    {shares : List (Share F)}
    {k : Nat} :
    system.degreeLt p k →
    shares.length = k →
    system.distinctXs shares →
    system.sharesFrom p shares →
    system.interpolate shares = some reconstructed →
    reconstructed = p := by
  exact system.lagrangeSound

/--
Dependency-free concrete sanity check for the reconstruction surface.

This proves the degree-1 case over Lean integers for shares at x=0 and x=1.
The production implementation still uses the Rust finite-field Shamir code;
this theorem keeps the Lean layer from being only an abstract contract while
avoiding a large Mathlib dependency in the default evaluator build.
-/
structure AffinePolynomial where
  constant : Int
  slope : Int
  deriving Repr, DecidableEq

def affineEval (p : AffinePolynomial) (x : Int) : Int :=
  p.constant + p.slope * x

def reconstructAffineAtZeroOne (shares : List (Share Int)) : Option AffinePolynomial :=
  match shares with
  | [s0, s1] =>
      if s0.x = 0 ∧ s1.x = 1 then
        some { constant := s0.y, slope := s1.y - s0.y }
      else
        none
  | _ => none

def affineSharesFrom (p : AffinePolynomial) (shares : List (Share Int)) : Prop :=
  shares = [
    { x := 0, y := affineEval p 0 },
    { x := 1, y := affineEval p 1 }
  ]

theorem affine_reconstructs_original_polynomial
    {p reconstructed : AffinePolynomial}
    {shares : List (Share Int)} :
    affineSharesFrom p shares →
    reconstructAffineAtZeroOne shares = some reconstructed →
    reconstructed = p := by
  intro hShares hReconstructed
  cases p with
  | mk constant slope =>
      rw [hShares] at hReconstructed
      simp [reconstructAffineAtZeroOne, affineEval] at hReconstructed
      have hSlope : constant + slope - constant = slope := by
        omega
      rw [hSlope] at hReconstructed
      exact hReconstructed.symm

end AnonymousForum
