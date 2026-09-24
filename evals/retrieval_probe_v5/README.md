# Retrieval probe v5: adversarial applicability boundary

This provider-free probe freezes 16 query texts and stronger judgments after the v4 candidate
algorithm and its result were already frozen. The replay path may substitute only these queries;
it must retain v4's lexical and dense limits, structured cues, lane budgets, source priority,
scope filtering, lifecycle rule, tag normalization, model snapshot, and corpus.

The holdout contains eight quality queries and eight no-result/wrong-scope queries. It targets:

- verified procedure discrimination rather than mere procedure-kind recall;
- handoff and rule cue confounders;
- generic tags extracted from opaque cross-project cues;
- untyped lifecycle successors across decisions and handoffs;
- semantic no-result burden;
- per-query candidate precision and explicitly forbidden candidates.

The frozen gates require at least 0.9 quality recall, 0.5 macro candidate precision, perfect
no-result abstention, mean/max bundle sizes no greater than 4/6, zero scope/stale/forbidden
results, and local query-embedding p95 no greater than 100 ms. Passing recall while failing
precision, abstention, or forbidden-result gates is a failed product boundary.

The initial preregistration at
`f734d83c0d92f9290cd4f50d896bbed0f86d93c71716b7f7bcdf46c5078322bb` was rejected before
fixture materialization because one query exactly duplicated pre-v4 data. It produced no
evaluation evidence. The corrected protocol hash is recorded below after validation.

Frozen valid protocol SHA-256:
`74ff526d221d3a3a45127752fdcf98a199cef532a0b884749d60b0bda443b503`.

Run it with the exact pre-cached snapshot into a new directory:

```bash
cargo run -p engram-eval -- probe-retrieval-v5 \
  --protocol evals/retrieval_probe_v5/protocol.json \
  --model-snapshot /Users/yuval.meiri/.engram/cache/fastembed/models--Qdrant--all-MiniLM-L6-v2-onnx/snapshots/5f1b8cd78bc4fb444dd171e59b18f3a3af89a079 \
  --output /new/private/output
```

The CLI exits nonzero when any frozen gate fails, after writing `adversarial-report.json`. The
runner uses no provider, performs no download, and does not touch the live Engram daemon, user
database, production retrieval, orientation, adapters, or agent configuration.
