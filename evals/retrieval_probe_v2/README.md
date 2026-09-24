# Retrieval probe v2

This frozen, provider-free probe tests the smallest retrieval changes suggested by v1:

- dense candidate generation with an exact, pre-cached local model snapshot;
- reciprocal-rank fusion with current lexical retrieval;
- deterministic expansion from a matched superseded record to its active successor;
- an explicit abstention gate, including exact handling of opaque canary-like cues.

The runner must verify the v1 protocol hash and every embedding-model file hash before doing
any setup. It loads the ONNX model from attested bytes and must fail if the snapshot is missing or
different; downloading or silently substituting a model is outside this protocol.

The JSON protocol is the preregistration. Its hash is recorded before the first execution and in
every report. Do not change thresholds in response to results; make any follow-up a new protocol.

Frozen protocol SHA-256:
`6b72ddbb3282659292d6c8de00415a18bb3fab6afe49602acef372d5ca0cccf9`.

Run it with an exact pre-cached snapshot into a new or empty directory:

```bash
cargo run -p engram-eval -- probe-retrieval-v2 \
  --protocol evals/retrieval_probe_v2/protocol.json \
  --model-snapshot /path/to/snapshots/5f1b8cd78bc4fb444dd171e59b18f3a3af89a079 \
  --output /tmp/engram-retrieval-probe-v2
```
