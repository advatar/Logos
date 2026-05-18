import AnonymousForum.Shamir
import AnonymousForum.Slash

namespace AnonymousForum

/-- Compatibility alias for older references to the proof-target file. -/
theorem shamir_target_lagrange
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
  exact lagrange_reconstructs_original_polynomial system

end AnonymousForum
