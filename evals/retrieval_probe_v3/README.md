# Retrieval probe v3: selective application

This provider-free probe separates two product behaviors on query texts not used by retrieval v2:

1. scope-filtered dense Top-5 candidate discovery;
2. conservative automatic application of at most one result, otherwise explicit deferral.

The frozen selector was calibrated from the failed v2 abstention arm. It accepts only an exact
active scoped opaque cue, an explicit lifecycle successor reached after every query term matches
the inactive record, high absolute similarity, high top-rank margin, or lexical/dense agreement
with direct evidence-term overlap. Its primary gate is 100% application precision; coverage is a
secondary gate. A deferred candidate may remain available to explicit search but cannot enter an
automatic context packet.

No local reranker or entailment model was present when the protocol was frozen. The runner must
not download or substitute one.

Frozen protocol SHA-256:
`0ad8cb22c5ce16ee8f0985db66bd7622d472753a5c2c7488f1a3ef9441e2c27c`.
