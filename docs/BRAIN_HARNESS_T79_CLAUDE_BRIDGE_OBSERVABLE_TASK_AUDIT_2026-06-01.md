# Brain Harness T79 Claude Bridge Observable Task Audit

Status: Complete; harness inconclusive because Claude Bridge did not expose allowed Engram tools
Date: 2026-06-01
Scope: Cross-harness replication of the T78 controlled observable-task pattern through Claude Bridge

This slice is evidence-quality work only. It must not run M6 inspection, migration status,
prioritize, review apply, candidate decisions, deletion, lifecycle mutation, document indexing,
harness writes, schema/storage/index changes, ranking changes, public MCP changes, or `orient`
payload changes.

## Research Question

Can Claude Bridge / Claude Code produce transcript-visible, independently assessable non-`plan_work`
task outcomes using the same existing `orient`, `search`, and telemetry surfaces that T78 validated
in Codex?

This tests cross-harness reproducibility of the controlled observable-task evidence pattern. It does
not test broad organic trace coverage, ranking changes, or product readiness.

## Hypotheses

| Type | Hypothesis |
| --- | --- |
| Preferred | A single read-only Claude Bridge run can complete three pre-registered tasks whose trace IDs, returned memory IDs, and final answers are visible enough for Codex to classify them as `ASSESSABLE_TASK_OUTCOME`. |
| Null | Claude Bridge output or tool wiring is too lossy to reconstruct the evidence chain, making one or more tasks retrieval-only or unassessable. |
| Simpler alternative | Treat T78 Codex-only evidence as sufficient and move to an instrumentation design note. |
| Failure | T79 repeats T78's exact queries as path memorization, changes criteria after seeing Claude output, uses Bash/file writes, or treats any confidence-gate result as authorization for gated work. |

## Consultation Summary

AI Council recall surfaced prior eval guidance: cross-agent evidence should not be overclaimed as
causal proof, and passive telemetry coverage remains secondary to observable outcomes. AI Council
broadcast and Claude Bridge both supported a small Claude Bridge replication, with cautions:

- pre-register tasks before execution,
- avoid exact T78 query repetition,
- make "transcript-visible" operational,
- avoid Bash, file writes, and hidden state,
- treat the result as within-Claude-Bridge reproducibility, not proof of all Claude Code surfaces.

## Transcript-Visible Evidence Definition

For T79, transcript-visible evidence includes:

- Claude Bridge final output,
- tool response excerpts that Claude includes in its final output,
- trace IDs and memory IDs reported by Claude,
- Codex follow-up `telemetry(action="get_trace")` on reported trace IDs.

Transcript-visible evidence excludes:

- local file reads by Claude,
- Bash output,
- hidden bridge state,
- post-hoc task changes,
- any result that requires Codex to infer a Claude tool result not present in Claude's final output
  or the persisted trace.

## Pre-Registered Claude Tasks

Run one Claude Bridge call after this pre-registration is committed. The call must set
`write=false`, allow no Bash, and allow only these Engram tools:

- `mcp__engram__orient`
- `mcp__engram__search`

Claude must run exactly these tasks and then stop:

| Task | Intent | Required tool call | Why this is genuine current work | Success criterion |
| --- | --- | --- | --- | --- |
| T79-A | `verify_decision` | `orient(project="engram", cwd="/Users/yuval.meiri/projects/engram", intent="verify_decision", response_shape="lean", prompt="T79-A Claude Bridge parity: verify active current plan and hard gates after T78.")` | The active goal requires cross-harness validation that the hot-path entrypoint surfaces the current plan compactly. | Claude reports a trace ID, project `engram`, and active T78 current-plan memory `019e82aa-056f-7302-96e3-32bddedce792` in candidate/top guidance. |
| T79-B | `follow_user_preference` | `search(project="engram", cwd="/Users/yuval.meiri/projects/engram", intent="follow_user_preference", query="Engram software design philosophy Ousterhout evidence over confidence no unrequested features", limit=6)` | The user requires Ousterhout design and evidence discipline across Engram work. | Claude reports a trace ID and cites reviewed user preference memory `019e6924-256b-7093-b1c5-286ec4d02461` as a relevant returned result. |
| T79-C | `verify_decision` | Use one `orient` call and one `search` call to answer: "What is the next non-gated evidence step after T78, and which hard gates remain?" | The active goal requires a next-step/gate decision before any new work. | Claude's final answer references both the active T78 current-plan memory `019e82aa-056f-7302-96e3-32bddedce792` and at least one gate memory or gate rule, and explicitly says no gated work is authorized. |

Claude's final response must include a compact table with:

- task ID,
- tool(s) used,
- trace ID(s),
- returned/used memory IDs,
- pass/fail/ambiguous,
- one-sentence evidence note.

## Classification Rubric

Codex will classify each task after the Claude Bridge run:

- `ASSESSABLE_TASK_OUTCOME`: expected behavior, tool trace, Claude answer, and outcome are visible
  from Claude output plus persisted trace.
- `ASSESSABLE_RETRIEVAL_ONLY`: retrieval can be judged, but Claude's downstream answer or memory
  use cannot be judged.
- `NO_OUTCOME_CONTEXT`: Claude output does not show what happened after retrieval.
- `MISSING_TRACE_ID`: Claude completed a task but did not report trace IDs, preventing trace join.
- `GATE_CONFLICT`: task execution or assessment required a forbidden operation.
- `HARNESS_INCONCLUSIVE`: Claude Bridge/tool access failed before task outcome could be assessed.

Only `ASSESSABLE_TASK_OUTCOME` traces may receive outcome feedback with `task_success`,
`preference_adhered`, `repeated_context_questions`, or `bad_memory_used`.

## Stop Rules

- Do not retry or add replacement tasks if Claude omits trace IDs or produces ambiguous output.
- Do not expand the task set or change criteria after Claude runs.
- If Claude uses Bash, writes files, calls unapproved tools, or asks for gated work, void the run.
- Submit feedback only after all three tasks are classified.
- If fewer than two tasks are `ASSESSABLE_TASK_OUTCOME`, submit no T79 outcome feedback.
- If feedback is submitted, run `telemetry(action="real_session_eval", project="engram",
  limit=50)` exactly once at the end.
- Treat any confidence-gate pass as diagnostic only. It does not authorize migration, lifecycle
  writes, harness writes, ranking changes, schema/storage/index changes, document-index actions,
  public MCP changes, or `orient` expansion.

## Execution Result

Pre-registration commit: `bdce09e` (`Pre-register T79 Claude Bridge audit`)

The post-commit Claude Bridge call used `harness="project"`, `write=false`, no Bash allowlist, and
only these allowed tools:

- `mcp__engram__orient`
- `mcp__engram__search`

Claude Bridge did not expose either tool to the Claude Code run. Claude returned:

| Task ID | Tool(s) attempted | Trace ID(s) | Returned/used memory IDs | Classification | Evidence note |
| --- | --- | --- | --- | --- | --- |
| T79-A | `mcp__engram__orient` | None | None | `HARNESS_INCONCLUSIVE` | The call failed with `No such tool available: mcp__engram__orient`, so the active T78 current-plan memory could not be observed through Claude Bridge. |
| T79-B | `mcp__engram__search` | None | None | `HARNESS_INCONCLUSIVE` | The call failed with `No such tool available: mcp__engram__search`, so the reviewed user-preference memory could not be observed through Claude Bridge. |
| T79-C | `mcp__engram__orient`, `mcp__engram__search` | None | None | `HARNESS_INCONCLUSIVE` | Both allowed tools were unavailable; Claude still stated that no gated work was authorized, but there is no Engram trace evidence for this task. |

Because there were zero `ASSESSABLE_TASK_OUTCOME` traces, T79 submitted no outcome feedback and did
not run `telemetry(action="real_session_eval")`. The stop rule also forbids retrying with alternate
tool names or replacement tasks inside this slice.

## Interpretation

T79 is evidence about Claude Bridge project-harness tool exposure, not evidence against Engram
`orient` or `search` behavior. The failure happened before any Engram retrieval call produced a
trace. The result therefore preserves the T78 Codex-only controlled outcome evidence and adds a
new cross-harness caveat: Claude Bridge parity audits must first prove that the target harness
actually exposes the requested Engram tools under the selected harness and allowlist.

No migration, lifecycle mutation, harness write, ranking change, schema/storage/index change,
document-index action, public MCP change, or `orient` expansion is authorized by this result.
