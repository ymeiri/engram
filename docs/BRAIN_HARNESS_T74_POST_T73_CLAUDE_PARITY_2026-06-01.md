# Brain Harness T74 Post-T73 Claude Parity

Status: Completed read-only cross-harness retrieval check
Date: 2026-06-01
Scope: Codex and Claude Code retrieval shape after T73 current-plan capture

This check did not edit files through Claude Code, run Bash through Claude Code, write memory
through Claude Code, archive, supersede, reject, review, scope-correct, or create any MemoryItem
through Claude Code. It did not run M6 inventory, review export, review apply, candidate decisions,
deletion, harness writes, schema/storage/index changes, public MCP changes, ranking changes,
document indexing, or `orient` payload changes.

## Research Question

After T73 current-plan capture, do Codex and Claude Code both surface T73 current-plan memory first
for the continuation prompt, while keeping stale repository-scoped current-plan guidance
lower-ranked and explicitly noisy?

## Hypotheses

| Type | Hypothesis |
| --- | --- |
| Preferred | Codex and Claude Code both return T73 current-plan memory first for lean `orient` and direct `search`; the stale repository-scoped target remains visible below it and should be rejected as guidance. |
| Null | Claude Code fails to expose Engram MCP, omits T73 current plan, or ranks stale repository-scoped guidance first. |
| Simpler alternative | Trust the Codex post-capture checks without a Claude parity run. |
| Failure | A read-only parity check is mistaken for lifecycle, ranking, harness-write, or `orient` approval. |

## Measurement

Codex first ran read-only post-capture checks:

- lean `orient` trace `019e8277-3c03-7f62-8bfe-cc6a79f48212`;
- direct `search` trace `019e8277-484c-7df1-a977-1e303a41d333`;
- `obligations(action="doctor")`, which returned `open=[]` and `warnings=[]`.

Claude Bridge then ran with `harness="personal"`, `write=false`, no Bash allowance, and only these
tools allowed:

- `mcp__engram__orient`;
- `mcp__engram__search`;
- `mcp__engram__obligations`.

## Results

Codex post-capture results:

- Lean `orient` trace `019e8277-3c03-7f62-8bfe-cc6a79f48212` returned T73 current-plan memory
  `019e8276-60ad-7110-9f10-d64a3249e93e` first and stale repository-scoped current-plan target
  `019e5e0a-86b4-73e3-aa9b-ca350e83e915` fifth.
- Direct `search` trace `019e8277-484c-7df1-a977-1e303a41d333` returned T73 current-plan memory
  first and stale repository-scoped target second.

Claude Code results through Claude Bridge:

- Lean `orient` trace `019e8278-671d-7d02-8a04-fe0a17d31de6` returned T73 current-plan memory
  `019e8276-60ad-7110-9f10-d64a3249e93e` first and stale repository-scoped target
  `019e5e0a-86b4-73e3-aa9b-ca350e83e915` fifth.
- Direct `search` trace `019e8278-6bd4-73f3-8973-8ea0d3ec24bc` returned T73 current-plan memory
  first and stale repository-scoped target second.
- Claude's `obligations(action="doctor")` reported two open obligations and two warnings:
  `design_context_reading` and `source_reading`.

Codex follow-through:

- Resolved Claude-created design-context obligation
  `019e8278-4e06-77c0-9b86-577ce009f24f` because Codex had already read AGENTS instructions and
  the governing Brain Harness docs before interpreting the result.
- Skipped Claude-created source-reading obligation
  `019e8278-4e06-77c0-9b86-576aa5cfa050` because this was a read-only retrieval validation with
  no source behavior claim or code change.

## Interpretation

The preferred hypothesis holds. T73 current-plan memory is first in both Codex and Claude Code for
the tested continuation prompt and direct search. The stale repository-scoped current-plan item
remains lower-ranked but noisy: it is fifth in lean `orient` and second in direct `search` for both
harnesses.

This strengthens cross-harness current-plan continuity for the post-T73 state. It also confirms the
T73/T52 caveat remains live across harnesses: stale repository-scoped current-plan guidance is still
active and visible enough that agents must reject it as guidance until the user explicitly chooses a
T52 lifecycle path.

The Claude Bridge obligation noise is expected for prompts that mention design/source cues. Future
Claude parity smokes should include a cleanup step: run Codex-side `obligations(action="doctor")`
after the bridge run, then resolve or skip synthetic obligations with concrete evidence.

## Completion Matrix Delta

The Cross-harness behavior row remains partially validated but stronger for the post-T73
current-plan path. The Memory quality / lifecycle row remains partially validated because parity
does not resolve the stale repository-scoped target or approve any lifecycle action.

No approval gates changed. T69, T70, T52, T47, M6 apply/deletion, schema/storage/index changes,
ranking changes, public MCP changes, document-index writes, harness writes, and `orient` expansion
remain separately gated.
