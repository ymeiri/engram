# Retrieval probe v1

This provider-free benchmark compares three deterministic retrieval arms over the same frozen
engineering-context corpus:

- `current_lexical`: the production `SearchService` memory path;
- `exact_overlap_resolved_scope`: a minimal token-overlap baseline with the repository remote
  already resolved;
- `bm25_resolved_scope`: a BM25 baseline with the same resolved scope.

The distinction is deliberate. It separates failures caused by text ranking from failures caused
by failing to carry stable repository identity into retrieval. The probe includes direct lexical,
semantic-paraphrase, repository-scope, stable-identifier, lifecycle-correction, cross-project
canary, and true no-result queries. Only active records are eligible. Any wrong-scope,
superseded, or false-positive abstention result violates a frozen gate.

Run it into a new or empty directory:

```bash
cargo run -p engram-eval -- probe-retrieval \
  --protocol evals/retrieval_probe_v1/protocol.json \
  --output /tmp/engram-retrieval-probe-v1
```

The command writes `retrieval-report.json`, including the protocol SHA-256, fixture revision,
ranked keys for every query, aggregate retrieval metrics, gate violations, and explicit
limitations. BM25 is a lexical baseline, not a semantic or graph result; this protocol cannot by
itself justify choosing BM25, dense, hybrid, graph, or agentic retrieval for production.
