# Procedure-match probe v2

This source-attested replay inherits the exact 19 scenarios and quality gates from procedure-match
probe v1. It changes only the attested `MemoryService` source hash after adding a fail-closed guard
for material conflict between an explicit project and the repository resolved from `cwd`.

The replay format attests the immutable v1 protocol and its failed result. The runner rejects any
schema-v2 replay that changes more than the one declared source entry or uses another correction
policy. This keeps the post-correction run comparable to the baseline instead of tuning scenarios
or gates after seeing the failure.

Frozen replay protocol SHA-256:
`fe169a08310a9ef2c5c50e1aaf7d700c027a380530a8149e054bd0ee224f51df`.

Run into a new private directory:

```bash
cargo run -p engram-eval -- probe-procedure-match \
  --protocol evals/procedure_match_probe_v2/protocol.json \
  --output /new/private/output
```

The CLI writes `procedure-report.json` before exiting nonzero on any inherited gate failure. It
uses an isolated database and fixture, invokes no provider, executes no stored procedure, and does
not touch the live daemon, user database, adapter, or agent configuration.
