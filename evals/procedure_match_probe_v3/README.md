# Procedure-match probe v3

This replay inherits the exact v2 procedure matrix and every original v1 gate through an attested
replay chain. It changes only the attested `memory_ranker.rs` source hash after making structured
procedure prerequisites searchable alongside task, command, failure-signature, and verification
text.

The change is intended to let the active procedure reach applicability checks when a query names a
condition such as `tool.version`. Exact condition comparison remains in the matcher: a mismatched
version must still abstain, but it should now explain the mismatch. Superseded procedures remain
excluded from retrieval.

Frozen replay protocol SHA-256:
`bb1bfca815efba9dc7fd31ddd806f92b8cb4ad416a4f8c7aa8f8e3c17f99bf09`.

Run into a new private directory:

```bash
cargo run -p engram-eval -- probe-procedure-match \
  --protocol evals/procedure_match_probe_v3/protocol.json \
  --output /new/private/output
```

The CLI writes `procedure-report.json` before exiting nonzero on any inherited gate failure. It
uses an isolated database and fixture, invokes no provider, executes no stored procedure, and does
not touch the live daemon, user database, adapter, or agent configuration.
