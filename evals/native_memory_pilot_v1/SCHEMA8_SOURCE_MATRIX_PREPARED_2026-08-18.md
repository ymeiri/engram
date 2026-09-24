# Schema-8 source-backed matched/mismatch matrix completed — 2026-08-18

The forward-only successor is frozen at:

`/private/tmp/engram-native-memory-pilot-v1-source-matrix.5vNBK1/run-plan.json`

Run-plan SHA-256:

`45255c7aea02b20ce6859d221962581ebd09061a9286b6dd80f40a50fc0f3aa4`

The plan contains two source-grounded cases across native memory, lean Engram, and combined memory
on Codex and Claude Code: 12 fixed lanes total, one repetition per case/arm pair. The matched case
uses the moved Atlas checkout, freezes `tool.version = 3`, and requires successful first-attempt
cobalt execution. The mismatch case uses the legacy checkout, freezes `tool.version = 2`, and
requires evidence-backed abstention with zero learned or failed procedure attempts.

The corrected evaluator is
`/private/tmp/engram-schema8-postrun-check/debug/engram-eval`, SHA-256
`2be9232039bdc66b83edb7703424006e3aa124b857201b126c38ea8180079f75`. It contains the forward-only
Claude MCP result-shape fix discovered after schema 8: a missing optional `is_error` member is a
successful result, while explicit `is_error: true` remains rejected. The focused regression, all
103 evaluator tests, Clippy with warnings denied, and formatting pass.

Other frozen identities remain:

- Engram: `745a6c564227d26d3365e45ae87424ee3ea95c193d736a258b72ba1fab0e30f6`
- Codex: `6170ff5578170ee9b74ad92bfcff96e6186f41d02b60815a7c2b01ad424c754f`
- Claude Code: `bc466b6cde63edafc773f471a1fb98787fabb31f52240c8616ce7e1f587b212d`
- Effective six-tool schema:
  `7633baf9d364b74908c92a3b65f308080fc4925aab3dcbd75a50dd6961f2f179`
- Effective instructions:
  `2d0493af502412c6264e8d6c74f104fc39dcfa65874c80e4f1149a4050e55581`

Preparation, runtime attestation, and untouched lifecycle audit pass with `invalid=false`, zero
failures, and zero native-memory write attempts. The matched and mismatch source hashes are
`432c24060958c30296453fe2f2512aa4fdc240eb7e9b67182f939477e4248ae1` and
`206d57028c48a0e855e5d59b63d0c653be7b7f23b619aac4af37ed86df2ca4f3` respectively.

Claude prior accounted spend is 1,752,912 micro-USD. This plan allocates at most $1.00 additional
Claude spend and freezes a $3.00 cumulative ceiling. Its 12 Claude teaching/evaluation calls each
receive a $0.083 turn-boundary allocation. The user's standing provider authorization satisfies the
runner approval flag but does not weaken exact budget, hash, lifecycle, or outcome gates.

All six isolated Codex homes now pass Keychain-backed ChatGPT authentication with
`check-native-memory-pilot-auth --require-ready`. Teaching completed exactly once across all 12
lanes with provider exit 0, no recovery or replay, 12 passed teaching traces, and every required
Engram verification passed. The immutable teaching runner report is
`/private/tmp/engram-native-memory-pilot-v1-source-matrix.5vNBK1/runner-teaching.json`, SHA-256
`6696e6dd5901427f1349838feed411bf0750d93c4f24b21b8e4ae0c70555d594`. Claude teaching cost was
287,658 micro-USD. The latest Codex teaching call completed at
`2026-08-18T14:07:26+03:00`, so the frozen one-hour retention gate opens at
`2026-08-18T15:07:26+03:00`.

The post-teaching lifecycle audit was `invalid=false` with zero failures and zero native-memory
write attempts. After the exact one-hour retention interval, the required plan, evaluator, Engram,
teaching, runtime, authentication, and lifecycle checks were repeated and remained pristine. All
six Codex activations then completed exactly once with provider exit 0 and no recovery or replay.
All 12 evaluations subsequently completed exactly once with provider exit 0 and no recovery or
replay. Never modify, repair, or replay these completed lanes or the predecessor schema-8 plan.

Native capture is a measured outcome, not an assumed prerequisite. Both Claude native-only lanes
produced one substantive auto-memory file; neither Claude combined lane produced an auto-memory
artifact. All four Codex native-bearing lanes produced substantive native-memory state. Every
Engram-bearing lane had a passed, exact trusted procedure verification before evaluation. The
artifacts remained sealed from evaluation except through their assigned fresh host lane.

## Frozen interpretation boundary

Experiment integrity, task acceptance, and incremental value are separate claims. A completed run
is admissible only when the provider-free audit reports `complete=true` and `invalid=false`, with no
integrity failure or native-memory write attempt. Individual lane acceptance is then read directly
from the frozen trace-grounded contract. The strict comparison may legitimately exit nonzero when
no Engram treatment Pareto-dominates its matched native control on both hosts; that is a valid
negative outcome, not a reason to replay a lane.

Even a portable positive signal remains descriptive because this plan has two cases and one
repetition. It may justify the separately frozen three-repetition matrix, but it cannot by itself
support a statistically powered product claim. A failed boundary must be diagnosed and fixed
forward before repetition rather than diluted with more provider calls.

“Lean Engram” in this frozen matrix means Engram without host-native memory; it does not mean a
minimal instruction payload. The four Claude Engram lanes each load a 4,645-byte (616-word)
instruction file, while the four Codex Engram lanes each load a 4,462-byte (592-word) skill. Host
token comparisons therefore include a material static-adapter floor in addition to MCP result
bytes. Post-run attribution must report both; a retrieval-only optimization cannot establish a lean
host boundary while this instruction cost remains.

## Completed lifecycle and immutable outputs

The completed lifecycle was:

- teaching: `2026-08-18T13:58:49+03:00` to `2026-08-18T14:07:28+03:00`, 519,613 ms;
- Codex activation: `2026-08-18T15:10:12+03:00` to `2026-08-18T15:11:19+03:00`, 66,820 ms;
- evaluation: `2026-08-18T15:13:12+03:00` to `2026-08-18T15:20:00+03:00`, 408,008 ms.

The immutable reports are:

- teaching runner: `6696e6dd5901427f1349838feed411bf0750d93c4f24b21b8e4ae0c70555d594`;
- activation runner: `ce750d7729add009fb9aa6e942a8a2338bf1579dc9c5e82e0ecff7d3207e6031`;
- evaluation runner: `7234a650ad9180318d26f7f4914861ea935fc37a793edb0f6c7f5188c7d6cd32`;
- completion audit: `3a7de964281b1d6b6f614171456a730947f900fbf9d9a67dbd433666b3198758`;
- comparison report: `7b53ff53541a6d6909c03ed2144cd7b3a79e69ecf673ffb161bbf00273a023b8`;
- strict comparison report: the same report hash, with the strict command exiting 1 because portable
  incremental value was not observed.

The final audit reports `complete=true`, `invalid=false`, 12 `evaluation_complete` lanes, no
integrity failure, and no native-memory shell write attempt. Claude teaching cost 287,658
micro-USD and evaluation cost 225,596 micro-USD. This plan therefore used 513,254 micro-USD and
brought cumulative accounted Claude spend to 2,266,166 micro-USD, below the frozen 3,000,000
micro-USD ceiling.

## Outcome

The valid preregistered result is **partial**, not portable. Codex lean Engram and Codex combined
memory each passed both cases and Pareto-dominated Codex native memory. Every Claude layer passed
zero of two full acceptance contracts. Claude native memory nevertheless produced the correct
task outcome in both cases, while Claude lean Engram and Claude combined memory each produced the
correct task outcome in only one case. The strict report therefore correctly rejects a portable
incremental-value claim.

| Lane | Host/layer | Case | Full pass | Outcome correct | Procedure revalidated | Result |
| ---: | --- | --- | --- | --- | --- | --- |
| 1 | Claude both | mismatch | no | yes | no | safely abstained, but missed the required source inspection and usable match |
| 2 | Codex native | matched | no | no | yes | abstained despite an applicable learned procedure |
| 3 | Claude Engram | mismatch | no | yes | no | safely abstained immediately after orientation |
| 4 | Codex both | matched | yes | yes | yes | ran cobalt once and observed its success marker |
| 5 | Claude native | mismatch | no | yes | yes | safely abstained, but failed identity and source-evidence acceptance |
| 6 | Codex Engram | matched | yes | yes | yes | ran cobalt once and observed its success marker |
| 7 | Claude both | matched | no | no | no | stopped after orientation without procedure matching |
| 8 | Codex native | mismatch | no | yes | yes | safely abstained, but failed identity and source-evidence acceptance |
| 9 | Claude Engram | matched | no | no | no | stopped after orientation and abstained incorrectly |
| 10 | Codex both | mismatch | yes | yes | yes | proved the mismatch and safely abstained |
| 11 | Claude native | matched | no | yes | yes | ran cobalt once, but failed identity/evidence/context acceptance |
| 12 | Codex Engram | mismatch | yes | yes | yes | proved the mismatch and safely abstained |

For matched comparisons, each Codex Engram treatment improved full passes by 2, correct outcomes
by 1, identity by 2, first action by 2, evidence by 2, and successful execution by 1 relative to
Codex native memory, with no preregistered regression. Each Claude Engram treatment regressed
correct outcomes by 1, first action by 1, procedure revalidation by 2, and successful execution by
1 relative to Claude native memory. Both Claude treatments improved identity by 2, but neither
Pareto-dominated native memory.

## Trace-grounded failure attribution

The Claude regression is at the host-routing and adapter/orientation boundary, not in Engram's
procedure matcher: the same matcher passed every Codex treatment lane.

- In lane 1, Claude combined called `orient` without the task prompt. It then called
  `procedure_match` without the required `query`, received the exact `query required` error, and
  retried with the unrelated query `atlas worker queue setup initialization`. No procedure matched.
- In lane 3, Claude lean passed the prompt to `orient` but stopped when the returned project was
  unresolved, even though repository-local procedure matching remained authorized.
- In lanes 7 and 9, Claude combined and lean stopped after `orient`; neither called
  `procedure_match` for the matched task.
- The orientation recommendation says to confirm the project before project-scoped memory, but the
  generated Claude instructions bury the repository-local exception inside a 4,645-byte,
  616-word payload. The observed behavior treated project ambiguity as a global stop condition.

The next forward slice must make the learned-procedure route short and explicit: pass the exact
user prompt to `orient`; always supply a required `procedure_match.query` copied from the task;
and state near the top of the Claude adapter that unresolved project identity blocks project/task
memory but does not block repository-local procedure matching. This is a source and provider-free
test change first. It does not justify replaying this completed matrix or running the frozen
three-repetition plan before the boundary is repaired and discriminatingly re-tested.

## Forward provider-free boundary repair

The first forward repair changes source templates and tests only; it does not modify installed or
live Claude/Codex adapters, settings, hooks, or daemon state. Generated Claude memory, startup, and
resume guidance now puts the learned-procedure route ahead of project ambiguity, requires the full
user prompt on `orient`, requires `procedure_match.query` copied from the user's request, and says
explicitly that unresolved project identity blocks project/task memory but not repository-local
procedure matching. The compact agent MCP instructions, `orient` declaration, and `memory.query`
schema expose the same contract. When orientation resolves a repository with ambiguous projects,
its recommended actions now preserve that local procedure route.

Provider-free regressions prove the generated Claude ordering, the ambiguous-project orientation
recommendation, and the compact six-tool MCP contract. The complete `engram-index` unit suite passes
271 tests with one model-download test ignored; all 57 `engram-cli` tests pass; affected-package
Clippy passes with warnings denied; and formatting passes. This evidence validates the source-level
repair only. A fresh Claude host-boundary run is still required before claiming the regression is
fixed in behavior.

That fresh six-lane Claude diagnostic subsequently completed validly under schema 11; see
`SCHEMA11_CLAUDE_ROUTING_REPAIR_PREPARED_2026-08-18.md`. It did not validate the repair. Claude
native memory again got both task outcomes right, while lean Engram and combined memory each got
only the mismatch outcome right and incorrectly abstained in the matched checkout. The repaired
project-ambiguity wording was understood, but the remaining mandatory route was keyed too narrowly
to the literal words “earlier, previous, remembered, or learned.” The frozen task asked to use
durable procedure memory without those words, so matched treatment lanes stopped after orientation;
one mismatch lane used `memory(list)` instead of `procedure_match`. The next forward repair must
make one bounded repository-local `procedure_match` mandatory for every actionable repository task,
with the exact user request, and explicitly reject `memory(list)` as a substitute. Neither completed
matrix may be replayed.

The unconditional successor is now complete; see
`SCHEMA11_CLAUDE_UNCONDITIONAL_ROUTE_FINAL_PREPARED_2026-08-18.md`. Its first attempt was
invalidated after one teaching call because insufficient persistent-store disk headroom prevented
Engram startup. The evaluator now rejects that condition before any provider process. A wholly
fresh sealed successor then completed validly: Claude lean Engram passed both the matched execution
and mismatch abstention contracts and Pareto-dominated Claude native memory across this two-case
slice. Combined memory passed only the mismatch case because its matched lane skipped Engram
entirely. This validates the source repair direction on the lean Claude boundary but still requires
fresh repetitions, a reliable combined path, and current-candidate Codex confirmation before a
portable claim.

## Supplementary host-boundary telemetry

Telemetry remains outside preregistered outcome scoring. Individual Engram results stayed below
the existing 8,192-byte reference ceiling: the maximum was 7,065 bytes. Aggregate Engram bytes per
treatment interaction ranged from 2,889 to 9,987 bytes.

| Lane | Input tokens | Output tokens | Reasoning tokens | Host ms | Runner ms | Engram calls | Total bytes | Max bytes |
| ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 1 | 71,415 | 2,175 | — | 22,632 | 29,191 | 3 | 5,099 | 2,910 |
| 2 | 173,453 | 1,657 | 862 | — | 44,046 | 0 | 0 | 0 |
| 3 | 26,496 | 1,600 | — | 14,847 | 22,515 | 1 | 2,889 | 2,889 |
| 4 | 155,995 | 963 | 370 | — | 43,198 | 2 | 9,987 | 7,065 |
| 5 | 85,418 | 2,991 | — | 30,200 | 32,356 | 0 | 0 | 0 |
| 6 | 145,241 | 1,171 | 516 | — | 46,874 | 2 | 9,924 | 7,023 |
| 7 | 33,487 | 1,067 | — | 11,444 | 18,438 | 1 | 2,931 | 2,931 |
| 8 | 127,600 | 1,261 | 536 | — | 34,752 | 0 | 0 | 0 |
| 9 | 26,080 | 1,085 | — | 10,835 | 18,367 | 1 | 2,910 | 2,910 |
| 10 | 119,985 | 701 | 231 | — | 39,011 | 2 | 5,959 | 3,058 |
| 11 | 72,113 | 2,482 | — | 23,896 | 25,065 | 0 | 0 | 0 |
| 12 | 111,143 | 866 | 343 | — | 44,017 | 2 | 5,924 | 3,044 |
