# Brain Harness T73 Stale Current-Plan Audit

Status: Completed read-only lifecycle evidence refresh; no lifecycle write authorized
Date: 2026-06-01
Scope: Repository-scoped stale current-plan target after T72

This audit did not archive, supersede, reject, review, scope-correct, or create any MemoryItem. It
did not run M6 inventory, review export, review apply, candidate decisions, deletion, harness
writes, schema/storage/index changes, public MCP changes, ranking changes, document indexing, or
`orient` payload changes.

## Research Question

After T72, does the stale repository-scoped current-plan target still require a T52-style user
decision, or has fresh read-only evidence changed the lifecycle state enough to reopen the plan?

## Hypotheses

| Type | Hypothesis |
| --- | --- |
| Preferred | The target remains active, remains the only repository-scoped current-plan item for this checkout, and lint reports a higher stale-feedback count with `safe_action=none`; T52 stays a user decision request rather than an executable packet. |
| Null | The target has already been archived, superseded, or replaced, so the T52 decision request no longer applies. |
| Simpler alternative | Rely on T52 and T72 current-plan memory without another audit. |
| Failure | The audit is misread as archive, replacement, scope-correction, ranking, or `orient` approval. |

## Measurement

Before editing docs, Codex used only read-only evidence:

- lean `orient` for the current task boundary;
- direct Engram searches for current plan, Brain Harness architecture, user design philosophy, and
  recent risks;
- `memory(action="get")` for target `019e5e0a-86b4-73e3-aa9b-ca350e83e915`;
- project-scoped and repository-scoped `memory(action="list", tags=["current-plan"])` calls;
- `lint(action="run", limit=20)` and `lint(action="apply_safe", write=false, limit=10)`;
- governing docs, especially T52, T72, `ORIENT_CONTRACT.md`, and the current completion matrix;
- `git status --short` and recent commit history.

## Fresh Read-Only Evidence

- Lean `orient` trace `019e8273-7bd6-72e1-983b-5b6988123b12` returned T72 current-plan
  MemoryItem `019e826e-e059-7e10-8ee3-facf9b470bfb` first. It also returned stale
  repository-scoped current-plan target `019e5e0a-86b4-73e3-aa9b-ca350e83e915` in the top five.
- Direct `search` trace `019e8273-868b-7370-8e3a-7fcfcb40a0e5` for the T73 next-step query
  returned T72 first and the stale repository-scoped target second.
- Direct risk search trace `019e8273-af03-75e1-94e8-fc5f019a88b5` returned T72 first, the active
  M6 gate second, and the stale repository-scoped target third.
- `memory(action="get")` confirmed target `019e5e0a-86b4-73e3-aa9b-ca350e83e915` is still
  `status=active`, `kind=decision`, tagged `current-plan`, and scoped to repository local path
  `/Users/yuval.meiri/projects/engram`.
- `memory(action="list", scope_type="repository",
  local_path="/Users/yuval.meiri/projects/engram", tags=["current-plan"],
  status_filter="active")` returned exactly that target.
- `memory(action="list", scope_type="project", project_name="engram", tags=["current-plan"],
  status_filter="active")` returned exactly T72 current plan
  `019e826e-e059-7e10-8ee3-facf9b470bfb`.
- `lint(action="run", limit=20)` reported
  `feedback-stale-current-plan:019e5e0a-86b4-73e3-aa9b-ca350e83e915` first, with 228 recent
  stale-feedback records and `safe_action=none`.
- `lint(action="apply_safe", write=false, limit=10)` repeated the same first finding and reported
  `applied_safe_actions=0`.
- Git status before doc edits was still only untracked root `AGENTS.md`, which is user-owned and
  out of scope.

## Interpretation

The preferred hypothesis holds. The stale target did not disappear, and no replacement appeared at
repository scope. T52 is still the right approval boundary, but its observed stale-feedback count is
now older evidence: the count increased from 142 in T52 to 228 in T73.

That count increase strengthens the review signal but does not change the safe action. The lint
finding still says `safe_action=none`, and `apply_safe` in dry-run mode still applies zero actions.
The scope gap also remains: archiving the target without a replacement would leave this checkout
with no active repository-scoped current-plan memory.

Direct retrieval is mostly healthy but still noisy. T72 project current-plan memory stays first for
the tested continuation prompt, yet the stale repository-scoped item appears second or third for
current-plan and risk searches. This supports continued rejection/feedback and explicit lifecycle
decisioning; it does not justify broad ranking churn or `orient` hot-path expansion.

## Completion Matrix Delta

The Memory quality / lifecycle row remains partially validated. T73 updates the stale current-plan
evidence date and count, but it does not resolve the lifecycle decision. The open decision is still:

- Option A: archive only;
- Option B: create a repository-scoped replacement, then archive;
- Option C: scope-correct or merge first.

No option is approved by this report. A future write still requires exact user approval for the
selected path, fresh pre-write checks, exact target IDs, exact reason text where applicable, and
stop conditions.

## Next Action

If the user wants to resolve this lifecycle issue, ask them to select T52 Option A, B, or C and
approve the exact write scope in a follow-up turn. Otherwise, continue only with non-gated
validation, evidence-quality work, cross-harness replication, or another concrete capture/lifecycle
gap surfaced by evidence.

T69, T70, T47, M6 apply/deletion, schema/storage/index changes, ranking changes, public MCP
changes, document-index writes, harness writes, and `orient` expansion remain separately gated.
