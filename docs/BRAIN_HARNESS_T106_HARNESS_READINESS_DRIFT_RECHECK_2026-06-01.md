# Brain Harness T106 Harness Readiness Drift Recheck

Date: 2026-06-01

## Status

T106 completed a read-only harness readiness drift recheck after T71 and T105. It found no
external readiness improvement. All checked harness targets remain `ready=false`, so cross-harness
behavior is still a risky/not-ready completion-matrix row.

No harness adapter install, settings edit, hook registration, generated-adapter write,
`adopt_user_owned`, M6 action, lifecycle archive, `lint(action="apply_safe")`, document indexing,
ranking change, `orient` expansion, public MCP change, schema/storage/index behavior change, or
document-index behavior change was run.

## Research Question

After T71 and T105, did any external harness configuration drift resolve readiness enough to change
the Brain Harness completion matrix or unlock the pending T47 harness repair packet?

## Hypotheses

- Preferred: readiness is unchanged. Generic, Claude Code, Codex, Gemini CLI, and Cursor still
  report `ready=false`; T47 remains the exact harness-write gate; no product or harness completion
  can be claimed.
- Null: one or more harnesses was externally repaired, producing a narrower follow-up slice.
- Simpler alternative: rely on the T71 audit and avoid repeating the read-only doctor.
- Failure: this report is misread as approval to install adapters, edit hooks/settings, or adopt
  user-owned harness files.

## Measurement

Read-only evidence collected on 2026-06-01:

- `harness(action="doctor", root="/Users/yuval.meiri")` returned `ready=false` for generic:
  missing `/Users/yuval.meiri/.engram/harness-policy.md`.
- Claude Code returned `ready=false`: required `SessionStart` and `SessionEnd` settings
  registrations are still missing. The generated command and hook adapters are installed, and the
  settings snippet is user-owned.
- Codex returned `ready=false`: `memory-session` and `resume-session` skills still drift from the
  generated adapters.
- Gemini CLI returned `ready=false`: `memory-session`, `resume-session`, and global context still
  drift from generated adapters.
- Cursor returned `ready=false`: `memory-session` and `resume-session` skills still drift from
  generated adapters.
- `lint(action="run", limit=10)` still surfaced stale current-plan feedback first:
  `feedback-stale-current-plan:019e5e0a-86b4-73e3-aa9b-ca350e83e915`, with 241 stale-feedback
  records and `safe_action=none`.
- `obligations(action="doctor")` was clean for the observed surface.
- Git status was clean except the user-owned untracked root `AGENTS.md`.

The Engram observation
`testing.harness-readiness-2026-06-01-t106-readonly`
(`019e839b-6920-7b22-8d43-d7728885ba63`) records the same read-only evidence.

## Completion Matrix Delta

| Area | T106 state | Evidence |
| --- | --- | --- |
| Cross-harness behavior | Risky/not ready | All five checked harness targets returned `ready=false`. |
| Harness repair | Blocked on exact approval | T47 remains the pending exact harness-write approval packet. |
| Memory quality/lifecycle | Partially validated | Lint still reports stale current-plan feedback, with no safe action. |
| Current-plan retrieval | Validated for observed continuation surface | T105 current-plan memory remains the top `orient` result. |
| M6 migration | Gated | T69 count-drift inspection and later apply/deletion gates remain pending. |
| Document visibility | Gated | T70 exact-file indexing approval remains pending and was not run. |

## Validation

This is a docs-only evidence slice. Validation is limited to:

- direct read-only tool evidence from harness doctor, lint, obligations, and git status;
- exact-source documentation updates in the Brain Harness architecture, research method, and Memory
  OS implementation plan;
- `git diff --check` before commit.

## Next Gate

The next product-moving gate remains exact T69 count-drift inspection approval:
`Approve T69: inspect index.md and 0012-skip-plan.md.`

If the user wants harness repair instead, use the existing T47 exact harness repair approval packet.
Generic approval does not authorize harness writes, lifecycle changes, indexing, or M6 migration.
