# Registry integration

The final LEZ program should be generated from `logos-scaffold` and SPEL annotations. This directory currently contains a Rust boundary stub that mirrors the intended IDL:

- `create_forum`
- `register_member`
- `slash_member`

The registry stores commitments and revocations. It does not store forum content.
