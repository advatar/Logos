Current placeholder audit after the latest build pass:

- The Rust/Python/Lean protocol layers are implemented and tested locally.
- The Basecamp app is no longer only a loose QML placeholder: it is an embeddable `ui_qml` module and can be packaged as an LGX with `src/scripts/package_basecamp.sh`.
- The LEZ/SPEL registry remains the main placeholder boundary: `src/registry/lp0016-registry` is a tested Rust boundary plus hand-written IDL, but there is not yet a deployable `src/methods/guest/src/bin/lp0016_registry.rs` LEZ guest.
- CU measurement remains blocked on that deployable LEZ guest and scaffold deploy/invoke reporting, not on missing LEZ binaries anymore.
- Full app-flow RISC0 receipt packaging is still open: `ZkReceipt::Risc0` exists, but local demos still use mock receipts except for feature-gated host checks.

Delivered a starter repository here:

[Download the ZIP](sandbox:/mnt/data/lp0016-anon-forum-starter.zip?_chatgptios_conversationID=6a0a0650-f5f4-8395-9b54-1237ffa02ba0&_chatgptios_messageID=e1d7c8cc-70b3-4956-b300-39c0abf5f0c2)
[Download the tarball](sandbox:/mnt/data/lp0016-anon-forum-starter.tar.gz?_chatgptios_conversationID=6a0a0650-f5f4-8395-9b54-1237ffa02ba0&_chatgptios_messageID=e1d7c8cc-70b3-4956-b300-39c0abf5f0c2)

This is **not yet a complete LP-0016 prize submission**. It is a concrete repo your developer can clone from and continue: it fixes the underspecified choices, includes working local code, and lays out the Rust/Lean/LEZ/RISC0/Basecamp integration boundaries.

What I included:

```text
lp0016-anon-forum/
  SPEC.md                         Concrete implementation decisions
  README.md                       Run instructions and next steps
  Cargo.toml                      Rust workspace
  rust-toolchain.toml             Rust 1.82.0 pin

  crates/
    protocol-core/                Pure protocol model: field, Shamir, certs, slash
    moderation-sdk/               Forum-agnostic SDK facade + storage trait
    registry-sim/                 Local registry simulation binary
    slash-verifier/               CLI placeholder

  scripts/
    lp0016_sim.py                 Dependency-free working protocol simulator
    demo_e2e.py                   Full local lifecycle demo
    test_protocol.py              Unit tests
    demo_e2e.sh
    measure_cu.sh

  lean/
    lakefile.lean
    AnonymousForum/
      Basic.lean                  Formal state definitions
      Invariants.lean             Basic no-sorry invariant proofs
      ShamirTargets.lean          Next proof targets

  registry/
    lez-program-stub/             LEZ/SPEL boundary stub

  zk/
    membership-guest/             RISC0 guest placeholder
    membership-host/              RISC0 host placeholder

  app/
    basecamp-forum/               Minimal Basecamp QML placeholder

  docs/
    protocol.md
    api.md
    threat-model.md
    performance.md
```

I verified the local simulator in this environment:

```bash
cd lp0016-anon-forum
python3 scripts/demo_e2e.py
python3 -m unittest scripts/test_protocol.py
```

The demo exercises:

```text
registration
anonymous post proof simulation
partial certificate rejection
N-of-M moderation certificate creation
K-strike slash
revocation
post rejection after revocation
retroactive linkability for only the slashed member
two independent forum instances with different K/N parameters
```

I could **not** verify the Rust or Lean builds inside this sandbox because `rustc`, `cargo`, `lean`, and `lake` are not installed here. The repo includes CI jobs for both, but your developer should run:

```bash
cargo test --workspace
cd lean && lake build
```

The repo is aligned with the prize shape: LP-0016 requires a forum-agnostic moderation library plus a Basecamp app, with off-chain posting/moderation and on-chain slash only at revocation time.  [oai_citation:0‡GitHub](https://github.com/logos-co/lambda-prize/blob/master/prizes/LP-0016.md) It also targets the LP-0016 success criteria around anonymous posting, N-of-M moderation, K-certificate slash, revocation, SDK APIs, SPEL IDL, LEZ testnet demo, CI, and RISC0 proof demo.  [oai_citation:1‡GitHub](https://github.com/logos-co/lambda-prize/blob/master/prizes/LP-0016.md)

I also reflected the current Logos stack constraints: LEZ uses stateless programs with persistent data passed through accounts, and supports public execution plus private execution verified through RISC0 proofs.  [oai_citation:2‡GitHub](https://github.com/logos-blockchain/logos-execution-zone) The repo therefore keeps the registry small and state-account oriented. I also included `logos-scaffold` assumptions because its current CLI exposes LEZ/SPEL/Basecamp flows such as `build idl`, `deploy`, `localnet`, and Basecamp launch/build commands.  [oai_citation:3‡GitHub](https://github.com/logos-co/scaffold) Basecamp is represented as a placeholder app because the public Basecamp module APIs are still evolving, while the current Basecamp repo documents LEZ wallet, Storage, Chat, and other module patterns rather than a stable LP-0016-specific template.  [oai_citation:4‡GitHub](https://github.com/logos-co/logos-basecamp)

The key caveat: the repo’s **working code is a local protocol simulator**, not production crypto. `SPEC.md` locks the intended production choices: Ristretto255 scalar field, Ed25519 moderator signatures, SHA-256 domain-separated transcripts, Ristretto threshold ElGamal, DLEQ share proofs, and RISC0 membership/post receipts. RISC Zero’s current docs describe the host/guest/receipt model and the `rzup install` workflow, so the RISC0 directories are prepared around that model.  [oai_citation:5‡dev.risczero.com](https://dev.risczero.com/api/zkvm/quickstart?utm_source=chatgpt.com)
