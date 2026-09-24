# Procedure-match probe v1

This provider-free baseline calls the real `MemoryService::match_procedures` against an isolated,
deterministic Engram store. It was frozen after retrieval v5 rejected combined candidate bundles
and before any procedure-matcher production correction.

The 19-scenario matrix covers:

- exact verified matches in the main and moved Atlas worktrees;
- task discrimination between deploy and integration procedures;
- condition normalization, missing prerequisites, and mismatches;
- a verified failure-signature procedure;
- superseded procedure versions and lifecycle state;
- missing, tampered, expired, and incomplete verification evidence;
- unrelated and unscoped requests;
- explicit project values that conflict with the repository resolved from cwd.

All returned-key sets, abstention decisions, diagnostic fragments, receipt hashes, forbidden
procedures, result counts, and matcher latency are scored. Gates require perfect outcome, success,
abstention, diagnostics, evidence, and forbidden-result behavior, at most one returned procedure,
and p95 matcher latency no greater than 50 ms.

Frozen protocol SHA-256:
`f21be9e903f020da909d0b82c38dd293460a0d57b8e8782e5a13eaa7413a88f1`.

Run into a new private directory:

```bash
cargo run -p engram-eval -- probe-procedure-match \
  --protocol evals/procedure_match_probe_v1/protocol.json \
  --output /new/private/output
```

The CLI writes `procedure-report.json` before exiting nonzero on frozen gate failure. Scenario
mutations are confined to the isolated store and restored after each query. It uses no provider,
does not execute a stored procedure, and does not touch the live daemon, user database, adapter,
or agent configuration.
