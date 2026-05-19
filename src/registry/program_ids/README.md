# LP-0016 Registry Program IDs

`localnet.txt` records the RISC0 image ID produced by the local
`lp0016_registry` LEZ-framework guest build and submitted to the local
sequencer during the final local submission pass.

The official LEZ wallet quickstart currently documents the standalone local
sequencer at `localhost:3040` as the public developer path:

https://github.com/logos-co/logos-docs/blob/main/docs/apps/wallet/journeys/quickstart-for-the-logos-execution-zone-wallet.md

If reviewers require distinct public devnet/testnet networks, `devnet.txt` and
`testnet.txt` should contain the verified program IDs after deployment to those
networks.
