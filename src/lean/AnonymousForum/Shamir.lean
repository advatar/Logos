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

end AnonymousForum
