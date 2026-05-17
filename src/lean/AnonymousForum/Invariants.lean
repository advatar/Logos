import AnonymousForum.Basic

namespace AnonymousForum

theorem validCertificate_implies_threshold
    {cfg : ForumConfig} {cert : Certificate} :
    ValidCertificate cfg cert → cfg.N ≤ cert.signers.length := by
  intro h
  exact h.2.1

theorem validCertificate_implies_all_signers_are_moderators
    {cfg : ForumConfig} {cert : Certificate} :
    ValidCertificate cfg cert → ∀ signer, signer ∈ cert.signers → signer ∈ cfg.moderators := by
  intro h
  exact h.2.2

theorem validSlashBundle_implies_k_certificates
    {cfg : ForumConfig} {bundle : SlashBundle} :
    ValidSlashBundle cfg bundle → bundle.certs.length = cfg.K := by
  intro h
  exact h.1

theorem validSlashBundle_implies_each_certificate_valid
    {cfg : ForumConfig} {bundle : SlashBundle} :
    ValidSlashBundle cfg bundle → ∀ cert, cert ∈ bundle.certs → ValidCertificate cfg cert := by
  intro h
  exact h.2

theorem revoked_commitment_is_in_revocation_list
    {registry : Registry} {commitment : Nat} :
    commitment ∈ (revoke registry commitment).revoked := by
  unfold revoke
  simp

theorem revoked_commitment_not_active
    {registry : Registry} {commitment : Nat} :
    ¬ Active (revoke registry commitment) commitment := by
  intro h
  unfold Active revoke at h
  simp at h

end AnonymousForum
