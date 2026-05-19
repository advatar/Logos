# Logos

LP-0016 anonymous forum starter implementation.

The current starter app lives in `src/`. It includes the Rust protocol workspace, Python simulator, Lean proof modules, LEZ/SPEL registry crate, feature-gated RISC0 host/guest crates, and a Basecamp flow harness. See `src/README.md` and `REPO.md` for the detailed repository notes.

License: MIT. See `LICENSE`.

Proof-stack highlights:

- **RISC0:** the primary submitted zero-knowledge proof path. The local gate
  runs the real membership prover with `RISC0_DEV_MODE=0` and records proof
  performance evidence.
- **Lean 4:** a mechanically checked proof surface for protocol invariants:
  certificate thresholds, slash-bundle shape, revocation activity, and the
  Shamir/Lagrange reconstruction contract.
- **Noir:** optional icing under `src/noir/post_binding/`. It provides a small,
  runnable ACIR/Nargo circuit for the anonymous-post binding relation and is
  documented in `src/docs/noir.md`.

Verified local commands:

```bash
cd src
scripts/local_submission_gate.py
python3 scripts/collect_localnet_evidence.py
python3 scripts/check_noir_icing.py --pretty
python3 scripts/make_submission_video.py
python3 scripts/demo_e2e.py
python3 -m unittest scripts/test_protocol.py
cargo build --workspace
cargo test --workspace
cargo run -p registry-sim
cd lean && lake build
```

GitHub Actions is not the acceptance gate for this repository because hosted
jobs are blocked before startup by account billing/spending limits. Use the
local submission gate and `src/docs/submission.md` for hackathon evidence.

Submission evidence:

- Local gate: `cd src && scripts/local_submission_gate.py`
- Local sequencer deploy/RISC0 evidence: `cd src && python3 scripts/collect_localnet_evidence.py`
- RISC0 proof performance: `cd src && python3 scripts/check_risc0_proof_performance.py --run-prover --fail-on-blocked`
- Optional Noir circuit check: `cd src && python3 scripts/check_noir_icing.py --pretty`
- Basecamp clean-shell artifact diagnostic: `cd src && python3 scripts/check_basecamp_inspector.py --pretty`
- Basecamp inspector verification, when an inspector-enabled app is available:
  `cd src && python3 scripts/check_basecamp_inspector.py --run-click-through --pretty`
- Narrated demo video: [submission/lp0016-demo.mp4](submission/lp0016-demo.mp4)
- Video generator: `cd src && python3 scripts/make_submission_video.py`
- Evidence JSON: `src/dist/submission/evidence.json`
- Localnet evidence JSON: `src/dist/submission/localnet_evidence.json`
- RISC0 proof performance JSON: `src/dist/submission/risc0_proof_performance.json`
- Localnet registry image ID: `dd914ffd8202da7c363d0aa7d9ad6222d1638b79f63a13f5dd24109896817e30`
- Program ID files: `src/registry/program_ids/`

Current blockers:

- CU measurements for `register_member` and `slash_member`: the registry guest
  deploys to the local sequencer, but the current scaffold/wallet path does not
  expose a custom deployed-program invoke command or CU report for those two
  instructions. `cd src && scripts/measure_cu.sh` records the narrowed blocker.
- Public LEZ devnet/testnet evidence, only if reviewers require separate public
  network endpoints: the official LEZ wallet quickstart documents a standalone
  local sequencer at `localhost:3040`, and this repo has local sequencer deploy
  evidence for that path. Separate public-network proof still needs
  `LOGOS_LEZ_DEVNET_URL`, `LOGOS_LEZ_TESTNET_URL`, and program IDs in
  `src/registry/program_ids/devnet.txt` and `src/registry/program_ids/testnet.txt`.
- Full Basecamp runtime click-through in a clean shell: artifact discovery is
  now durable and no longer relies on `/tmp`. `check_basecamp_inspector.py`
  searches `LOGOS_BASECAMP_CACHE`, `~/.cache/logos-basecamp`, and explicit
  `LOGOS_BASECAMP_APP` / `LOGOS_QT_MCP` / `LOGOS_DESIGN_SYSTEM_ROOT` values
  before legacy scratch paths. The public `logos-basecamp` v0.1.1 DMG and the
  current action-built app provide durable runtime binaries, but they do not
  expose the QML inspector endpoint used by `app/basecamp-forum/ui-tests.mjs`.
  Final click-through evidence therefore still needs an inspector-enabled
  Basecamp build and a successful
  `cd src && python3 scripts/check_basecamp_inspector.py --run-click-through --pretty`.

The local lifecycle covers forum creation, anonymous registration, anonymous
posting, N-of-M moderation, K-certificate slash, revocation, and retroactive
linking only for the slashed member. Basecamp packaging is included in the
local gate; clean-shell artifact discovery now works from durable cache/env
locations, while full Basecamp click-through needs an inspector-enabled
Basecamp runtime plus `logos-qt-mcp` and Logos design-system QML artifacts.

Lean 4 usage:

- Lean is used as a formal proof surface for the protocol invariants, not as
  application runtime code. The modules live under `src/lean/AnonymousForum/`
  and are verified by `cd src/lean && lake build`, which is part of the local
  verification story.
- `Basic.lean` defines the abstract forum model: forum parameters `K` and `N`,
  moderator membership, certificates, slash bundles, registry state, active
  commitments, and the revoke transition.
- `Invariants.lean` proves the core structural invariants: valid certificates
  meet the `N` threshold, every signer is a moderator, valid slash bundles carry
  exactly `K` valid certificates, and revoked commitments are no longer active.
- `Slash.lean` defines `VerifySlash` and proves `slash_sound`: a verified slash
  implies the commitment was registered, was not already revoked, and has a
  bundle with exactly `K` certificates. It also proves every certificate in a
  verified slash meets the `N` signer threshold.
- `Shamir.lean` defines the `ShamirSystem` proof contract for the
  Lagrange-reconstruction layer. The Rust implementation supplies the concrete
  field/interpolation behavior; Lean pins the downstream theorem shape via
  `lagrange_reconstructs_original_polynomial`. `ShamirTargets.lean` keeps a
  compatibility theorem name for this target.
- The Lean build is `sorry`-free. It is intentionally compact: Rust carries the
  production cryptography and execution, while Lean makes the threshold,
  slash, and revocation proof obligations explicit and mechanically checked.

Noir icing:

- Noir is added as an optional proof-circuit artifact under
  `src/noir/post_binding/`. It does not replace RISC0; it gives reviewers a
  compact ACIR/Nargo circuit for the anonymous-post binding shape. The dedicated
  write-up is `src/docs/noir.md`.
- The circuit keeps `member_secret` and `opening` private while constraining the
  public `registered_commitment`, `post_nullifier`, and `post_retro_tag` against
  the forum/slash domains and post nonce. This mirrors the core relation the app
  needs: a post is tied to a registered member secret without revealing that
  secret, and the public tags remain deterministic for verification/slash logic.
- The circuit intentionally uses small arithmetic constraints rather than the
  production hash/Merkle/threshold crypto. Those production pieces remain in the
  Rust/RISC0 path; Noir is the small, easy-to-review proof layer on top.
- Run it with `cd src/noir/post_binding && nargo test`. If Nargo is not
  installed, `cd src && python3 scripts/check_noir_icing.py --pretty` reports the
  exact install blocker and links the official Noir installation/test docs.
