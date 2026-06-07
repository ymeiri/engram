# Brain Harness T313 Claude Adapter Drift Repair Approval Packet - 2026-06-07

## Summary

T313 records the exact approval packet for repairing the current Claude Code generated-adapter
drift. It is docs-only. It does not run `harness install --write`, edit `~/.claude`, change
settings, launch native Claude, run `/hooks`, send process signals, or change repo/runtime state.

The next executable slice is T314, but only after explicit user approval for the exact write set
below. The preferred T314 execution path is:

```bash
/Users/yuval.meiri/.local/bin/engram harness install \
  --harness claude-code \
  --settings-target snippet-only \
  --write \
  --json
```

`--settings-target snippet-only` is part of the safety contract. It avoids mutating
`/Users/yuval.meiri/.claude/settings.json`; the default install target would plan a settings merge.

## Council Synthesis

A 2026-06-07 three-model AI Council follow-up reviewed the current drift, the default dry-run write
set, and the safer `snippet-only` dry-run. Two models recommended a docs-only approval packet before
any repair; one model favored immediate repair because the diff is fully characterized. All three
agreed that `--settings-target snippet-only` is the safer repair path if settings writes are not
separately approved.

The standing Engram rule is stricter than the dissenting recommendation: no adapter, hook, or
settings writes without explicit user approval. Therefore T313 stays docs-only and T314 must not
execute until the user approves the exact target paths and command.

## Current Evidence

Repository and PR baseline:

- Branch: `yuval.meiri/memory-os-phase1`
- Local head: `07610516030b21fda3599feb021280405be72946`
- Upstream head: `07610516030b21fda3599feb021280405be72946`
- Ahead/behind: `0 0`
- Worktree: clean except untracked user-owned `AGENTS.md`
- PR #3: draft, open, merge state `CLEAN`
- Exact-head CI: run `27088563682` passed Format, Docs, Check, Clippy, and Test

Harness status and doctor both report `ready=false`. The blocking generated-adapter drift is:

| Adapter | Path | Status |
| --- | --- | --- |
| `claude-memory-session-command` | `/Users/yuval.meiri/.claude/commands/engram-memory-session.md` | drifted |
| `claude-end-session-command` | `/Users/yuval.meiri/.claude/commands/engram-end-session.md` | drifted |
| `claude-stop-nudge-hook` | `/Users/yuval.meiri/.claude/hooks/engram-stop-nudge.sh` | drifted |

Other required generated Claude adapters are installed. The settings snippet remains user-owned, and
settings are split across `settings.json` and `settings.local.json` with extra legacy Engram
permissions. Those warnings are not part of the T314 adapter-only write set.

Source evidence in `engram-index/src/harness.rs`:

- `HarnessService::install` defaults to dry-run unless `write=true`.
- `install_with_options` writes missing or drifted generated adapters only when `options.write` is
  true.
- Claude Code install planning calls `merge_claude_settings` after adapter checks.
- `snippet-only` settings target records that no Claude settings file will be modified.

## Dry-Run Evidence

Default dry-run:

```bash
/Users/yuval.meiri/.local/bin/engram harness install --harness claude-code --json
```

plans four writes:

| Planned item | Path | Message |
| --- | --- | --- |
| `claude-memory-session-command` | `/Users/yuval.meiri/.claude/commands/engram-memory-session.md` | will update generated adapter |
| `claude-end-session-command` | `/Users/yuval.meiri/.claude/commands/engram-end-session.md` | will update generated adapter |
| `claude-stop-nudge-hook` | `/Users/yuval.meiri/.claude/hooks/engram-stop-nudge.sh` | will update generated adapter |
| `claude-settings-merge` | `/Users/yuval.meiri/.claude/settings.json` | will merge Engram MCP permissions and lifecycle hooks into settings.json |

Safer dry-run:

```bash
/Users/yuval.meiri/.local/bin/engram harness install \
  --harness claude-code \
  --settings-target snippet-only \
  --json
```

plans only the three generated-adapter updates and skips `claude-settings-merge` with:

```text
settings target is snippet-only; no Claude settings file will be modified
```

It still warns that the user-owned snippet was skipped and that settings were not modified.

## Drift Inventory

| Adapter | Current SHA-256 | Expected SHA-256 | Current bytes | Expected bytes | Mode |
| --- | --- | --- | ---: | ---: | --- |
| `claude-memory-session-command` | `6e12ba4416fe5d5a8b07d193e53db9e3bf2b6a70c5fa89f9a4e9257ed5eaaab4` | `a5075190c01731c82be7b50eb219fe7e467812c3d210e083eec9405e1ff95259` | 1853 | 1899 | `-rw-r--r--` |
| `claude-end-session-command` | `688af0b6ec43764f37635ab234d0dd3bb1c472f28db8c6f0fddc411182d889f0` | `63c932a02ebd40563be6b7aa90200653d04c8073df61a858a606a7a8dd6482fb` | 605 | 651 | `-rw-r--r--` |
| `claude-stop-nudge-hook` | `66ecbae5279f08a8e0d6ff52bd69e2e9b8b7dd4993c5753074196e03111d9f85` | `3eabbfaf6921cedc5245c18450092747e0c8ba506bb4a47ca04d8b131b33633c` | 933 | 977 | `-rwxr-xr-x` |

The expected diffs only scope obligation detection and doctor guidance with `project=...` and
`cwd=...`.

```diff
--- current:/Users/yuval.meiri/.claude/commands/engram-memory-session.md
+++ expected:claude-memory-session-command
@@ -17,8 +17,9 @@
 - When the current method, plan, or next action should survive resume, use
   `memory(action=capture_current_plan)` with compact content and file/tool/manual-review evidence.
 - Before final response, call `changes_since`; if relevant updates appeared, account for them.
-- Before final response, call `obligations(action=detect)` and `obligations(action=doctor)`;
-  resolve open obligations or report explicit skip reasons.
+- Before final response, call `obligations(action=detect, project=..., cwd=...)` and
+  `obligations(action=doctor, project=..., cwd=...)`; resolve open obligations or report
+  explicit skip reasons.
 - Before context compaction, context transition, or any expected loss of conversation state,
   update `handoff` and record/commit compact durable memory for future sessions.
 - At session end, compile a handoff and create a knowledge commit candidate.
```

```diff
--- current:/Users/yuval.meiri/.claude/commands/engram-end-session.md
+++ expected:claude-end-session-command
@@ -3,7 +3,8 @@

 Before ending:
 - Call `memory(action=changes_since)` from the latest cursor.
-- Call `obligations(action=detect)` and `obligations(action=doctor)`.
+- Call `obligations(action=detect, project=..., cwd=...)` and
+  `obligations(action=doctor, project=..., cwd=...)`.
 - Resolve open obligations or state explicit skip reasons in the handoff.
 - Update or compile `handoff` with completed work, open decisions, next actions, and risks.
 - If durable memory changed, prepare a `memory(action=commit)` candidate.
```

```diff
--- current:/Users/yuval.meiri/.claude/hooks/engram-stop-nudge.sh
+++ expected:claude-stop-nudge-hook
@@ -18,6 +18,6 @@
 cat <<'EOF'
 {
   "continue": true,
-  "systemMessage": "Engram final-response check: call memory(action=changes_since), obligations(action=detect), and obligations(action=doctor); submit telemetry(action=submit_feedback) with task_success, preference_adhered, repeated_context_questions, bad_memory_used, missing_context, used_memory_ids, rejected_memory_ids, stale_memory_ids, and wrong_scope_memory_ids for relevant trace_id values when those outcomes or attribution judgments can be made; resolve or explicitly skip open obligations, update handoff if context would be lost, then answer."
+  "systemMessage": "Engram final-response check: call memory(action=changes_since), obligations(action=detect, project=..., cwd=...), and obligations(action=doctor, project=..., cwd=...); submit telemetry(action=submit_feedback) with task_success, preference_adhered, repeated_context_questions, bad_memory_used, missing_context, used_memory_ids, rejected_memory_ids, stale_memory_ids, and wrong_scope_memory_ids for relevant trace_id values when those outcomes or attribution judgments can be made; resolve or explicitly skip open obligations, update handoff if context would be lost, then answer."
 }
 EOF
```

## T314 Approval Contract

T314 may proceed only if the user explicitly approves this exact action:

```text
Approve T314 to update only these generated Claude Code adapters:
/Users/yuval.meiri/.claude/commands/engram-memory-session.md
/Users/yuval.meiri/.claude/commands/engram-end-session.md
/Users/yuval.meiri/.claude/hooks/engram-stop-nudge.sh

using:
/Users/yuval.meiri/.local/bin/engram harness install --harness claude-code --settings-target snippet-only --write --json

Do not launch Claude, run /hooks, send process signals, mutate repository files,
mutate /Users/yuval.meiri/.claude/settings.json, adopt user-owned files, or modify
settings.local.json.
```

No broader "repair harness" or "make production ready" approval should be inferred from that text.
If any preflight differs, stop and request a renewed approval.

### Preflight

Before any T314 write:

1. Confirm branch, head, and upstream relationship:
   `git rev-parse HEAD`,
   `git rev-parse @{u}`,
   and `git rev-list --left-right --count HEAD...@{u}`.
2. Confirm `git status --short --branch` shows no repo changes except untracked user-owned
   `AGENTS.md`.
3. Confirm PR #3 is still open and not conflicted. If the head changed, record the new exact-head
   CI status before claiming CI coverage.
4. Confirm `harness status` and `harness doctor` still report only the three generated adapter
   drifts as required-readiness blockers.
5. Recompute the three current and expected SHA-256 hashes. They must match the T313 table unless
   the changed source/policy is explicitly reviewed and a new approval is requested.
6. Run the `snippet-only` dry-run and confirm the only planned writes are the three adapter paths.
   `claude-settings-merge` must be skipped, not planned as a settings file write.
7. Confirm the diff remains limited to scoped `obligations(action=detect, project=..., cwd=...)`
   and `obligations(action=doctor, project=..., cwd=...)` guidance.
8. Do not launch native Claude, run `/hooks`, send signals, attach to running Claude processes, or
   inspect/modify Claude runtime state beyond the read-only harness commands above.

### Postflight

After an approved T314 write:

1. The three adapter hashes must equal the expected hashes in this packet.
2. `/Users/yuval.meiri/.claude/hooks/engram-stop-nudge.sh` must remain executable.
3. `harness install --harness claude-code --settings-target snippet-only --json` must no longer
   plan the three adapter updates.
4. `harness status --harness claude-code --json` and
   `harness doctor --harness claude-code --json` should report `ready=true`. If they do not, stop
   and report the remaining blocker rather than expanding the write set.
5. `/Users/yuval.meiri/.claude/settings.json`,
   `/Users/yuval.meiri/.claude/settings.local.json`, and
   `/Users/yuval.meiri/.claude/engram-settings-snippet.json` must be unchanged.
6. The repo worktree must remain unchanged except for any separately intended result docs.
7. No native Claude execution, `/hooks` observation, process signaling, host-label proof, lifecycle
   mutation, M6 mutation, vault mutation, or ranking/orient change may be claimed.

## Beta Scope Impact

This packet does not block the supported local/Codex beta path. It does block calling the Claude
Code harness beta-ready until T314 repairs the generated adapter drift or the beta explicitly ships
with Claude Code marked as a known non-ready harness.

Initial beta can continue to drop:

- native Claude prompt-bearing execution proof;
- effective `/hooks` visibility proof;
- live Claude host-label proof;
- Gemini/Cursor parity;
- broad lifecycle cleanup and direct legacy deprecation/deletion;
- telemetry confidence gates and production operations hardening.

Initial beta should keep as hard gates:

- green source CI for the PR head being shipped;
- installed local/Codex runtime and daemon provenance checks;
- exact scope wording that separates local/Codex beta readiness from production/GA;
- no silent writes to adapters, hooks, settings, or user-owned files;
- if Claude Code is in beta scope, T314 adapter repair with the contract above.
