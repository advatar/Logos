# Threat model

## In scope

- malicious members attempting to post after revocation;
- moderators attempting to create certificates with fewer than N votes;
- slash submitters attempting to combine unrelated certificates;
- cross-forum replay of posts, votes, certificates, or slash bundles;
- storage or messaging outages handled by retry queues.

## Out of scope

- content-based deanonymization;
- moderator collusion at or above threshold;
- endpoint compromise;
- side channels during proof generation;
- denial-of-service against Logos network services.

## Security assumptions

- hash collision resistance;
- unforgeable moderator signatures;
- threshold encryption privacy and DLEQ proof soundness;
- RISC0 receipt soundness and zero-knowledge;
- correct LEZ registry execution.
