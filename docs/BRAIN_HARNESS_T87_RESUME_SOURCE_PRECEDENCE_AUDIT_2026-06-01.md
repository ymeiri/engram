# Brain Harness T87 Resume Source Precedence Audit

Status: Pre-registered; execution pending
Date: 2026-06-01
Scope: Read-only resume-source validation plus rolling-handoff clarification if needed

This slice checks whether a future resume has a clear source order after T86. The Engram rolling
handoff is current, but the local markdown handoff at `~/notes/engram/handoff.md` may still contain
older open-source launch context. If that stale local note remains present, Codex may update only
the rolling handoff and repo docs to mark it as stale resume evidence.

This does not authorize T69 count-drift inspection, T70 document indexing, M6 review apply,
candidate decisions, deletion, lifecycle mutation, schema/storage/index changes, public MCP
changes, ranking changes, `orient` expansion, or harness adapter/hook writes.

## Research Question

After T86, do the available resume sources give a future agent an unambiguous current plan, or does
the stale local markdown handoff need to be explicitly demoted in Engram continuity records?

## Hypotheses

| Type | Hypothesis |
| --- | --- |
| Preferred | `orient`, direct Engram search, and `handoff(action="get")` surface current T86/T69/T70 context, while `~/notes/engram/handoff.md` is stale. A narrow rolling-handoff clarification prevents source-precedence confusion without product changes. |
| Null | The local markdown handoff is already current or absent, so no clarification is needed. |
| Simpler alternative | Rely on agents to notice the date/content mismatch manually during resume. |
| Failure | The audit edits the local note, changes resume tooling, mutates lifecycle state, or treats a generic approval as authorization for T69/T70/M6/harness work. |

## Measurement

Run only read-only checks:

1. `orient(project="engram", cwd="/Users/yuval.meiri/projects/engram", intent="plan_work", response_shape="lean")`.
2. Direct `search` for `handoff status next steps engram T86 T69 T70`.
3. `handoff(action="get", project="engram")`.
4. `git status --short` and recent commits.
5. Read only the first 220 lines of `/Users/yuval.meiri/notes/engram/handoff.md` if it exists.

Success criteria:

- Current Engram sources identify the latest plan and T69/T70 gates.
- The local markdown note is either absent or explicitly classified.
- If stale, the rolling handoff gets one clarification that the local note is old open-source
  launch context and must not override Engram `orient`, current-plan memory, or repo docs.
- Root `AGENTS.md` remains untouched and unstaged.

## Stop Conditions

Stop without handoff update if:

- the local markdown note is current and matches T86/T69/T70;
- source precedence is ambiguous after read-only checks;
- correcting the conflict would require editing files outside the repo, changing a skill, changing
  `orient`, changing search ranking, changing public MCP parameters, mutating lifecycle state,
  indexing documents, inspecting the T68 export snapshot, or writing harness hooks/settings.
