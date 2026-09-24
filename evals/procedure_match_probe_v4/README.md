# Procedure-match probe v4

This provider-free replay inherits the exact 19-scenario v3 procedure matrix and every original
quality gate through the attested replay chain. It updates the frozen procedure-service, domain
model, and deterministic fixture hashes to the current candidate. The service change makes a safe
no-result return bounded, read-only next actions for closing repository-local evidence; the domain
and fixture hashes attest the exact procedure boundary exercised by the replay.

The replay checks that the new diagnostic guidance does not change the matcher result set, weaken
scope, freshness, prerequisite, proof, or lifecycle abstention, return a forbidden procedure, or
exceed the inherited 50 ms p95 local matcher target. The unrelated no-result must also expose
exactly one bounded, checkout-local, read-only evidence-closure action that prohibits broadening,
inventing project identity, and executing candidate commands. The entire inherited matrix runs five
times, producing 95 latency samples for the unchanged 50 ms p95 gate. The probe does not test
whether a coding agent follows that action; that remains the purpose of the fresh native-memory
pilot.

Run into a new private directory:

```bash
cargo run -p engram-eval -- probe-procedure-match \
  --protocol evals/procedure_match_probe_v4/protocol.json \
  --output /new/private/output
```

The command invokes no provider, executes no stored procedure, and does not touch the live daemon,
user database, adapter, or agent configuration.
