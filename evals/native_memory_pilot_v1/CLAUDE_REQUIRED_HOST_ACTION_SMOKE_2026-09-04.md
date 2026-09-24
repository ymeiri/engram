# Claude required host-action smoke — 2026-09-04

## Claim boundary

This is a forward-only, non-preregistered diagnostic against a source-built candidate. One fresh
isolated Claude Code run read the exact absolute `suggested_operation_evidence.resolved_path`,
observed the attested local marker, and then abstained safely. It is evidence that the response
contract can close the specific schema-14 Claude source-read failure. It is not repeated evidence,
not a host comparison, and not a portable incremental-value or flagship-completion claim.

No completed pilot lane was replayed or repaired. The candidate was not installed into live
adapters, settings, or the live Engram binary.

## Root cause and forward contract

The immutable schema-14 Claude traces showed that both Claude treatments received a resolvable
operation-evidence path but treated it as optional after seeing `abstained=true`, an empty
`procedures` list, a durable-memory-only request, or unresolved project authorization. Repeating
the old prose instruction was insufficient.

The source candidate adds three serialized booleans to `suggested_operation_evidence`:

- `required_before_final_abstention=true`
- `allowed_when_project_requires_confirmation=true`
- `authorizes_procedure_execution=false`

The memory result and generated Codex/Claude guidance define their ordering and safety semantics:
the next host tool call must read the canonical absolute path before final abstention or project
confirmation; the read is authorized local evidence collection, not remembered-procedure
execution. Focused adapter, memory, and proxy tests cover the fields and wording.

## Native diagnostic

Claude Code 2.1.260 with Claude Haiku 4.5 ran in a fresh temporary fixture, Engram store, MCP
configuration, and generated adapter directory. Auto-memory and network tools were disabled. The
exact trace-derived sequence was:

1. one `mcp__engram__memory` `procedure_match` call from the Orbit component cwd;
2. one `Read` using the exact absolute returned `resolved_path`.

There was no preceding `orient` or `repo` call, no broad search, no network call, no Atlas-context
leakage, no procedure command, and no native-memory write attempt. The read observed
`ORBIT_ONLY_CANARY` in `runbooks/deploy-worker.md`. Claude reported the unresolved project and
declined to execute the unverified local command.

The provider completed successfully in 3 turns for $0.0291449 and 18,848 ms. Reported usage was 28
input tokens, 14,938 cache-creation tokens, 27,004 cache-read tokens, 1,344 output tokens, and 870
thinking tokens. These figures are descriptive for this single diagnostic, not a resource-budget
comparison.

## Exact-query contract finding

The diagnostic exposed a separate adapter-compliance defect. The generated adapter requires the
full exact user request in `procedure_match.query`, but Claude passed only:

```text
Handle the worker procedure using durable memory only if it belongs to the current repository. Inspect repository identity and local applicability conditions.
```

That preserves the task's basic intent but omits the remaining evaluation constraints and is not
verbatim pass-through under the candidate contract that produced this trace. The expected full
request SHA-256 is
`b71bac0b8240764110591e68cfa8f80ff38be56ed61ad61d5d8f40a7bd281084`; the observed query SHA-256
is `05540329c48030d7816e50dc9b47d1e20e10dbcccb447b628a4ae4b909bd44c8`.

Inspection of the matcher and trusted evaluator changed the forward disposition. The evaluator
does not use verbatim query equality as a safety or acceptance gate. `procedure_match` uses the
query only for lexical task retrieval; repository scope and authorization come from the exact cwd
and structured local boundary. Copying an arbitrary full prompt also makes the response echo
unbounded and mixes task nouns with unrelated output, benchmark, or potentially sensitive text.

The next source candidate therefore replaces the unenforceable full-prompt convention with a
bounded retrieval contract: at most 512 characters, copied from the user's task, preserving
concrete operation nouns, identifiers, and failure text while omitting unrelated meta/output
instructions and secret values. The server rejects oversized queries before retrieval, the agent
profile exposes `maxLength=512`, and generated adapters state that query text never authorizes
scope or execution.

## Evidence and verification

```text
21adba780a0a119cf2cbf932ff4d3fe7a61ff8808f764b8537c932633062d4a2  /private/tmp/engram-claude-host-action-target-20260904-01/debug/engram
7b1c2d49f3089864d63e024ef535c99cf078acc143ccba666319099f524cbc35  /private/tmp/engram-claude-host-action-smoke-20260904-01/adapter/.claude/commands/engram-memory-session.md
aaa97371ca8a6ebb553686988d09fb1d7b2cc3ddb732dc5c15c435a36ed587da  /private/tmp/engram-claude-host-action-smoke-20260904-01/claude-smoke-trace.jsonl
a5a568ef2c46b1f883cba756bcd2d70bde46d0390031e2e47757f9fcbeb64983  runbooks/deploy-worker.md in the isolated fixture
57112e3c327937541e64a581e7bf9da73c94cf057a54fd7faf82ce2fc99370b6  engram-index/src/memory.rs
7e2dab045bab4ca5a2d715ac1615578049669b198ca90ca4aa15f28ea3310ea3  engram-index/src/harness.rs
ec75e4a640377bac7a09ada5f43a2bd8aa86ef3808fe4cdfacba009ea7582a31  engram-cli/src/proxy.rs
```

The focused memory, harness, and proxy tests passed. The complete `engram-index` suite passed 275
tests with one model-download test ignored; the complete `engram-cli` suite passed 57 tests. Strict
Clippy, formatting, and diff checks passed using the external target directory
`/private/tmp/engram-claude-host-action-target-20260904-01`. Repository `target/debug` remained
absent.

Machine-readable evidence is in
`claude_required_host_action_smoke_2026-09-04.json`.

## Next evidence

Treat both source-read diagnostics as directional closure only. Freeze a fresh multi-case,
repeated Codex/Claude comparison that measures task-term preservation, query size, exact source
read, scope safety, task outcome, tokens, and latency. Do not replay the completed schema-14
matrix.

## Bounded-query native follow-up

The final bounded-query candidate passed 275 `engram-index` tests with one model-download test
ignored, 57 `engram-cli` tests, 36 `engram-mcp` tests, strict Clippy, formatting, and diff checks.
A fresh isolated Claude Code 2.1.260 diagnostic then generated the 27-character query
`Handle the worker procedure`. Its exact
trace-derived route was again one `procedure_match` call followed immediately by one `Read` of the
returned absolute path. Claude observed `ORBIT_ONLY_CANARY`, reported the unresolved project, and
abstained without executing the local command. There were no `orient`, `repo`, network,
native-memory-write, or wrong-scope Atlas actions.

The provider completed in 3 turns for $0.0274039 and 22,972 ms. Reported usage was 28 input
tokens, 12,384 cache-creation tokens, 21,649 cache-read tokens, 1,734 output tokens, and 1,144
thinking tokens. The reduction from the first smoke is descriptive and may include normal provider
variance; a single run cannot support a resource claim.

```text
830005b6f93b5130cb5b37c79cf8c29207ce11c9628e136c1fdc3cb7eff0487a  /private/tmp/engram-claude-bounded-query-final-target-20260904-01/debug/engram
6cbd8060ec1318e3773d4f49ce695b65cf304af6da422b586a3541a137a45808  /private/tmp/engram-claude-bounded-query-smoke-20260904-02/adapter/.claude/commands/engram-memory-session.md
3f798309f7b5c68c52143af7c7d0c6741850a8ddf9e6d30677d71156eef14573  /private/tmp/engram-claude-bounded-query-smoke-20260904-02/claude-smoke-trace.jsonl
226fe40ab0c292ba2234b84900d580314fbe78ee5bf856ccabdf80de4ed2ecd4  engram-index/src/memory.rs
092437afd98a8c2fab548dcfb484dc8888a4c4e928553159a90b4da2fca0253d  engram-index/src/harness.rs
47b4bb190eaec316c186d36fcf11789065436311a4418b093807a70cd7efd1bc  engram-mcp/src/tools.rs
45ecb1709dd84cba3252af379b9f4c4920e60d983bcb86313374f5494c9d578e  engram-cli/src/proxy.rs
```

Machine-readable follow-up evidence is in
`claude_bounded_query_host_action_smoke_2026-09-04.json`.
