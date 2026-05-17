namespace AnonymousForum

structure ForumConfig where
  K : Nat
  N : Nat
  moderators : List Nat
  deriving Repr, DecidableEq

structure Certificate where
  signers : List Nat
  deriving Repr, DecidableEq

structure SlashBundle where
  certs : List Certificate
  deriving Repr, DecidableEq

/-- A certificate is valid when it has N distinct signers, and every signer is in the forum moderator set. -/
def ValidCertificate (cfg : ForumConfig) (cert : Certificate) : Prop :=
  cert.signers.Nodup ∧
  cfg.N ≤ cert.signers.length ∧
  ∀ signer, signer ∈ cert.signers → signer ∈ cfg.moderators

/-- A slash bundle is valid when it has exactly K valid certificates. -/
def ValidSlashBundle (cfg : ForumConfig) (bundle : SlashBundle) : Prop :=
  bundle.certs.length = cfg.K ∧
  ∀ cert, cert ∈ bundle.certs → ValidCertificate cfg cert

structure Registry where
  registered : List Nat
  revoked : List Nat
  deriving Repr, DecidableEq

/-- A commitment is active when registered and not revoked. -/
def Active (registry : Registry) (commitment : Nat) : Prop :=
  commitment ∈ registry.registered ∧ commitment ∉ registry.revoked

/-- Registry state transition used by slash. -/
def revoke (registry : Registry) (commitment : Nat) : Registry :=
  { registered := registry.registered, revoked := commitment :: registry.revoked }

end AnonymousForum
