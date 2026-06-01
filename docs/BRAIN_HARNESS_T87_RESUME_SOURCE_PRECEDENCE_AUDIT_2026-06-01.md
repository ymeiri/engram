# Brain Harness T87 Resume Source Precedence Audit

Status: Complete; rolling handoff clarified
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

## Execution Result

Pre-registration commit: `2810150` (`Pre-register T87 resume source precedence audit`)

The read-only probes found:

- lean `orient` trace `019e82f8-73d8-7423-9911-4c094dcbb5b1` returned current-plan memory
  `019e82f5-615d-7be1-975b-1bfcd92cd1d7` first;
- direct `search` trace `019e82f8-7580-7a11-a8b2-b08ed150d4e4` returned the same current-plan
  memory first and the then-current rolling handoff `019e82f3-53bc-7a83-9e39-cfdb29b06c44`
  second;
- the same search also surfaced older active handoff MemoryItems lower in the result set, which is
  useful conflict evidence but not a safe lifecycle-action authorization;
- `handoff(action="get", project="engram")` returned then-current handoff
  `019e82f3-53bc-7a83-9e39-cfdb29b06c44`;
- `git status --short` showed only untracked root `AGENTS.md`;
- `/Users/yuval.meiri/notes/engram/handoff.md` exists and starts with
  `# Handoff - engram - 2026-04-17`; it is an open-source launch handoff with no matches for
  `T86`, `T69`, `T70`, `Brain Harness`, `Memory OS`, or `2026-06-01`.

Codex updated only the rolling handoff. The new active handoff is:

```text
019e82f8-cada-7c31-b073-18ac41986b1e
```

It supersedes `019e82f3-53bc-7a83-9e39-cfdb29b06c44` and records source precedence:

- start with lean `orient` and direct Engram searches;
- use `handoff(action="get", project="engram")` as the active rolling handoff;
- treat older handoff search results as historical if they conflict with `handoff(get)` or the
  latest current-plan memory;
- treat `/Users/yuval.meiri/notes/engram/handoff.md` as stale 2026-04-17 open-source launch
  context unless the user explicitly refreshes it.

Validation re-read `handoff(action="get", project="engram")` and returned the new handoff id and
content. A post-update `lint(action="run", limit=20)` still reported stale-feedback and
wrong-scope review signals with `safe_action=none`; no lifecycle action was taken.

## Interpretation

The preferred hypothesis held. Current Engram sources are strong enough to resume safely, but the
local markdown handoff is stale and direct search can surface older handoff memories below the
current plan. The handoff clarification gives future agents an explicit precedence rule without
editing local notes, changing skills, mutating lifecycle state, or changing product behavior.

T69, T70, M6 apply/deletion, lifecycle mutation, schema/storage/index behavior changes, public MCP
changes, ranking changes, `orient` expansion, document indexing, and harness writes remain
separately gated.
