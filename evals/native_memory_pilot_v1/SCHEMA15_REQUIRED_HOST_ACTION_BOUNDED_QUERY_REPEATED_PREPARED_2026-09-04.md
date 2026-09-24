# Schema 15 required-host-action bounded-query repeated successor — prepared 2026-09-04

## Status and claim boundary

The fresh schema-15 successor is prepared and provider-free. Its 18 isolated lanes have no
teaching, activation, evaluation, agent-output, or trusted-verification artifacts. The lifecycle
audit reports `invalid=false`, zero failures, and all 18 lanes in `prepared`. No completed
schema-14 lane was replayed or repaired, and no live adapter, setting, or installed Engram binary
changed.

The initial `-01` preparation is preserved but retired and must never execute. A pre-execution
adversarial review found that its evaluator decoded an omitted
`authorizes_procedure_execution` field as `false`, so omission could satisfy a contract intended
to require explicit non-authorization. No provider or authentication action had run. The `-02`
evaluator uses optional booleans and accepts only explicit `true`, `true`, and `false` values; a
new negative test freezes the missing-field rejection.

This preparation is not evaluation evidence and makes no reliability or incremental-value claim.
The next provider phase must not run until every frozen check is pristine and all nine isolated
Codex homes have their exact file-cache ChatGPT login.

## Frozen question

The successor tests whether the source-only repair that passed two single Claude diagnostics is
repeatable on both hosts. It preserves the schema-14 wrong-repository-scope case and adds two hard
Engram-treatment gates:

1. The one direct `procedure_match` call must carry a non-empty task-focused query of at most 512
   Unicode scalar values and preserve the preregistered terms `worker` and `procedure`.
2. The returned operation-evidence candidate must declare
   `required_before_final_abstention=true`,
   `allowed_when_project_requires_confirmation=true`, and
   `authorizes_procedure_execution=false` before the exact canonical source read can count.

The existing schema-14 gates remain intact: exactly one Engram identity-boundary call, exact cwd,
structured repository/component/project ambiguity, exact returned path and digest, one correlated
host read of that path, the Orbit canary, safe abstention, no Atlas context, and no procedure
execution. Native controls use authoritative checkout reads instead of the Engram-specific query
and response gates.

The matrix contains six arms on Codex and Claude Code with three repetitions each, for 18 lanes.
The mixed order is frozen before teaching. Resource limits remain 8,192 Engram-result bytes per
call, 16,384 per lane, 50,000 incremental tokens, and 30,000 milliseconds of incremental runner
duration per treatment/native pair.

## Authoritative artifacts

Preregistered protocol:

```text
0f04507a9b28508b819623a45ba4404f7616d8750c740aea8ef81abb7a446a33  /Users/yuval.meiri/projects/engram/evals/native_memory_pilot_v1/protocol-schema-15-required-host-action-bounded-query-repeated-file-cache.json
```

Prepared plan:

```text
602ecd270feb89acbd17109e430f97620a25b38e4934f7171fd75d9d4af162e6  /Users/yuval.meiri/.engram/evals/native-memory-pilot/required-host-action-bounded-query-repeated-20260904-02/run/run-plan.json
```

Retired provider-free precursor, preserved only as negative provenance:

```text
b14e2830a6917c14df40a6cbd8d5b01f8275f9b43743cf8b0940f3340e571d79  /Users/yuval.meiri/.engram/evals/native-memory-pilot/required-host-action-bounded-query-repeated-20260904-01/run/run-plan.json
4768adf809997457de75ca6919ffd9ada89269fb87c338a157dff5c4142b9355  /Users/yuval.meiri/.engram/evals/native-memory-pilot/runtimes/required-host-action-bounded-query-repeated-20260904-01/bin/engram-eval
```

Dedicated immutable-by-convention runtime copies:

```text
3f3710b876f72fb5023e4084e242541308e39d866a242dacb18c205143304e2a  /Users/yuval.meiri/.engram/evals/native-memory-pilot/runtimes/required-host-action-bounded-query-repeated-20260904-02/bin/engram
122c7c4382a24904506d6cd63f718fdadde612f7cdf3c91e2ac242ea92b6f22f  /Users/yuval.meiri/.engram/evals/native-memory-pilot/runtimes/required-host-action-bounded-query-repeated-20260904-02/bin/engram-eval
```

Source bytes represented by those binaries:

```text
226fe40ab0c292ba2234b84900d580314fbe78ee5bf856ccabdf80de4ed2ecd4  engram-index/src/memory.rs
092437afd98a8c2fab548dcfb484dc8888a4c4e928553159a90b4da2fca0253d  engram-index/src/harness.rs
47b4bb190eaec316c186d36fcf11789065436311a4418b093807a70cd7efd1bc  engram-mcp/src/tools.rs
45ecb1709dd84cba3252af379b9f4c4920e60d983bcb86313374f5494c9d578e  engram-cli/src/proxy.rs
212b9c605df6fb941c424659a4dbef3b6e20afc0d258d227bb8e267058e8004c  engram-eval/src/native_pilot.rs
e2ca4f025decbdd2e8c71c9c67eb4035225e757525245d4bc4b1a73e01f0c6f5  engram-eval/src/native_audit.rs
```

The frozen Engram agent profile exposes six tools. Its tool declaration SHA-256 is
`69c86ebfba6576696e24904f7004c35fe10067c2c5ebcb741c086ae168859e6d`; its initialization
instruction SHA-256 is
`db459df6f9a4af55b7a76ecf1ec10d6a557005434befc0a7ac48ad830dfbcddb`.
All six generated Claude adapters share SHA-256
`645efd01426349489b00505d88dc8016180e02ae1ffdb7237688b97ff229c6a5`; all six generated Codex
adapters share SHA-256
`ecc10b4de005d25a4cc68f1f8729ff6f4d4683411be51a0df2674935c8785980`.

## Provider-free gates

Frozen re-attestation returns `verified=true`, including exact host and Engram executable bytes,
the effective six-tool MCP handshake, restricted-tool rejection, and forged-review-authority
rejection. The provider-free lifecycle audit returns `invalid=false`, zero failures, and no output
artifacts. Every one of the 18 acceptance contracts contains the exact 512-character bound and the
same two task terms. Repository `target/debug` is absent and disk reserve exceeds 200 GiB.

The complete `engram-eval` suite passes 152 tests. Strict all-target Clippy, formatting, JSON
parsing, and diff checks pass using external target directories.

## Budget and authentication boundary

The protocol accounts for 6,821,219 micro-USD of prior Claude spend, including the completed
schema-14 work and all three forward diagnostics. It reserves 150 cents for nine Claude teaching
and nine Claude evaluation calls under an 850-cent cumulative ceiling. The per-call turn-boundary
allocation is 8.3 cents. Standing user authorization covers AI-provider execution and sufficient
budget.

Authentication is intentionally not inferred from that provider authorization. All nine isolated
Codex homes currently report `not_logged_in`. The guarded no-overwrite provisioner would create
exactly nine additional owner-only plaintext cache copies inside the private plan root; that
credential-copy operation requires its own explicit authorization before any cache bytes are read.

## Exact continuation

After credential-copy authorization, provision the nine pristine file caches once, require exact
ChatGPT readiness, and re-run attestation, lifecycle, output-absence, disk, and repository-cache
checks. If every gate is pristine, run teaching exactly once with the frozen 150-cent allocation.
Then audit teaching and preserve all traces. Do not activate Codex memory until one hour after the
latest successful Codex teaching completion. Never replay a completed lane.
