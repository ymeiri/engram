# Retrieval probe v4: structured candidate-set ablation

This provider-free diagnostic reuses the known retrieval-v3 failure corpus. It is not a held-out
generalization test. It asks one narrower question: can a bounded, explicitly labeled candidate
set recover the v3 lexical/dense misses when Engram also exposes high-signal structured metadata?

The separate lanes are production lexical Top-5, attested local dense Top-5, verified procedure /
current handoff / current rule kinds, exact normalized tags, and explicit lifecycle successors.
The final set deduplicates those lanes to at most 10 candidates using frozen presentation priority.
It never fuses scores, calls a candidate applicable, injects it into orientation, or executes it.

The primary gate is 100% set recall over the 12 known v3 quality queries. The diagnostic must also
recover at least one miss absent from the lexical/dense union, stay within the 10-candidate budget,
and return no out-of-scope, stale, or explicitly forbidden items. No-result candidate burden is
reported, not treated as automatic-application harm, because this experiment only models explicit
search candidates.

Entity aliases and graph traversal are deliberately deferred: the synthetic corpus does not carry
an authoritative entity-to-memory ownership edge, so adding graph behavior here would not test a
sound product contract.

Frozen protocol SHA-256:
`63937cc744195ae9d76121ec33f4ffd20e78ebfb1d9aaca1040267262dc0aaf4`.

Run the frozen protocol with the exact pre-cached snapshot into a new directory:

```bash
cargo run -p engram-eval -- probe-retrieval-v4 \
  --protocol evals/retrieval_probe_v4/protocol.json \
  --model-snapshot /Users/yuval.meiri/.engram/cache/fastembed/models--Qdrant--all-MiniLM-L6-v2-onnx/snapshots/5f1b8cd78bc4fb444dd171e59b18f3a3af89a079 \
  --output /new/private/output
```

The runner creates the output root with mode `0700` and the report with mode `0600`. It uses no
provider, performs no download, and does not touch the live Engram daemon or user database.
