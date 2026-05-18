# Basecamp app

This directory contains a deterministic Basecamp-facing flow harness for LP-0016:

1. create forum;
2. register;
3. post;
4. moderate;
5. vote;
6. aggregate certificate;
7. review history;
8. submit slash;
9. reject post after revocation.

`core-module/` exposes the Rust C ABI bridge point and links through `moderation-sdk`. QML remains presentation-only and must not duplicate protocol logic.
