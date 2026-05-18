import AnonymousForum.Basic

namespace AnonymousForum

def VerifySlash (cfg : ForumConfig) (registry : Registry) (bundle : SlashBundle) (commitment : Nat) : Prop :=
  ValidSlashBundle cfg bundle ∧ Active registry commitment

theorem slash_sound
    {cfg : ForumConfig}
    {registry : Registry}
    {bundle : SlashBundle}
    {commitment : Nat} :
    VerifySlash cfg registry bundle commitment →
      commitment ∈ registry.registered ∧
      commitment ∉ registry.revoked ∧
      bundle.certs.length = cfg.K := by
  intro h
  unfold VerifySlash Active at h
  exact ⟨h.2.1, h.2.2, h.1.1⟩

theorem slash_bundle_certificate_thresholds
    {cfg : ForumConfig}
    {registry : Registry}
    {bundle : SlashBundle}
    {commitment : Nat} :
    VerifySlash cfg registry bundle commitment →
      ∀ cert, cert ∈ bundle.certs → cfg.N ≤ cert.signers.length := by
  intro h cert certIn
  unfold VerifySlash at h
  exact (h.1.2 cert certIn).2.1

end AnonymousForum
