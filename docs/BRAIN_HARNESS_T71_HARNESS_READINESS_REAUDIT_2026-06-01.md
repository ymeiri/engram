# Brain Harness T71 Harness Readiness Re-Audit

Date: 2026-06-01
Status: Completed
Scope: Read-only harness readiness evidence refresh

This report updates the T46/T47 harness-readiness evidence without running any harness writes. No
adapter install, settings edit, hook registration, M6 action, lifecycle mutation, schema/storage or
index behavior change, public MCP change, ranking change, or `orient` payload change was performed.

## Boundary

T71 used only read-only `harness(action="doctor")` and `harness(action="status")` calls, plus
source/doc inspection. It did not run `harness(action="install")`, even in dry-run mode, because
T47 already contains the pending exact-write approval packet and no fresh write approval was given.

## Research Question

Has harness readiness drifted since T46/T47, and can the completion matrix be updated from
read-only evidence only?

## Hypotheses

| Type | Hypothesis |
| --- | --- |
| Preferred | Supported harnesses still report `ready=false`, with the same missing policy, missing Claude settings registrations, and generated-adapter drift recorded by T46/T47. |
| Null | All supported harnesses now report ready without adapter/settings/hook writes. |
| Simpler alternative | Rely on T46/T47 and make no evidence update. |
| Failure | The audit crosses into install, settings mutation, hook changes, migration, lifecycle, ranking, or `orient` changes. |

## Measurement

- Read-only `harness(action="doctor", project="engram")` for the default generic policy.
- Read-only `harness(action="status", harness=..., project="engram")` for `generic`,
  `claude_code`, `codex`, `gemini_cli`, and `cursor`.
- Source inspection of `engram-index/src/harness.rs` to verify how readiness is computed:
  required adapters must be installed, required MCP tools must not be missing when an observed tool
  list is supplied, and Claude Code also requires settings checks.

## Result

| Harness | Ready | Read-only finding |
| --- | --- | --- |
| `generic` | false | Required policy document is still missing at `/Users/yuval.meiri/.engram/harness-policy.md`. |
| `claude_code` | false | Required generated command and hook files are installed, but the optional settings snippet is user-owned, Claude settings still lack required `SessionStart:startup|resume|compact` and `SessionEnd` registrations, and settings still contain extra legacy Engram permission entries outside the current contract. |
| `codex` | false | Required `codex-memory-session-skill` and `codex-resume-session-skill` remain drifted; `project-agents-snippet` remains installed. |
| `gemini_cli` | false | Required `gemini-memory-session-command`, `gemini-resume-session-command`, and `gemini-global-context` remain drifted; `gemini-end-session-command` remains installed. |
| `cursor` | false | Required `cursor-memory-session-skill` and `cursor-resume-session-skill` remain drifted; `cursor-end-session-skill` remains installed. |

No status result reported missing MCP tools for this Codex session. The default generic
`harness(action="doctor")` call took anomalously long in this session before returning the generic
`ready=false` report; the per-harness `status` calls returned quickly. Treat that as a caveat for
the diagnostic surface, not as a hot-path `orient` finding.

## Source Check

`HarnessService::status` reads the generated adapter specs for the requested harness, checks each
target file, records missing/drifted/user-owned warnings, and sets `ready` only when every required
adapter is installed and no required observed MCP tool is missing. For Claude Code it additionally
runs `claude_settings_status` and forces `ready=false` if required settings entries are missing.
`HarnessService::doctor` extends `status` with a soft lifecycle warning; it does not install files.

## Completion Matrix Delta

The preferred hypothesis holds. T71 does not change the harness-readiness state: supported
harnesses still report `ready=false`, and the T47 repair packet remains the relevant pending write
gate. This refreshes the evidence date only. It does not authorize adapter installation, settings
edits, hook registration, M6 work, lifecycle mutation, schema/storage/index changes, public MCP
changes, ranking changes, or `orient` payload changes.
