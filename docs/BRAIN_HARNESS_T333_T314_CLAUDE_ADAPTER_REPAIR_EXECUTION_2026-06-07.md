# Brain Harness T333 Executes T314 Claude Adapter Repair

Date: 2026-06-07
Status: completed exact generated-adapter repair

## Research Question

Can the prepared T314 Claude Code generated-adapter repair be executed without mutating Claude
settings, launching native Claude, running hooks, sending signals, or changing repository files?

## Hypotheses

| Hypothesis | Claim | Result |
| --- | --- | --- |
| Preferred | `snippet-only --write` updates exactly the three drifted generated adapters and leaves settings unchanged. | Supported. |
| Null | The dry-run or postflight no longer matches the T313 packet, so no write should occur. | Rejected by fresh preflight. |
| Simpler alternative | Record another docs-only approval packet instead of writing adapters. | Rejected because the active goal authorizes Engram-scoped adapter writes and fresh evidence matched the packet. |
| Failure | The write mutates settings, user-owned snippet, repo files, native Claude/runtime state, or a broader adapter set. | Avoided. |

## Preflight Evidence

Fresh branch and PR checks showed the repository was still on
`yuval.meiri/memory-os-phase1` at `308c31fc620b8bede181d174b022f1dfc1b22abe`, synced with
upstream (`0 0`), with only untracked user-owned root `AGENTS.md`.

The installed CLI preflight matched the T313 adapter inventory. Current generated-adapter hashes
matched the T313 current hashes, and rendered expected contents matched the T313 expected hashes:

| Adapter path | Current hash | Expected hash |
| --- | --- | --- |
| `/Users/yuval.meiri/.claude/commands/engram-memory-session.md` | `6e12ba4416fe5d5a8b07d193e53db9e3bf2b6a70c5fa89f9a4e9257ed5eaaab4` | `a5075190c01731c82be7b50eb219fe7e467812c3d210e083eec9405e1ff95259` |
| `/Users/yuval.meiri/.claude/commands/engram-end-session.md` | `688af0b6ec43764f37635ab234d0dd3bb1c472f28db8c6f0fddc411182d889f0` | `63c932a02ebd40563be6b7aa90200653d04c8073df61a858a606a7a8dd6482fb` |
| `/Users/yuval.meiri/.claude/hooks/engram-stop-nudge.sh` | `66ecbae5279f08a8e0d6ff52bd69e2e9b8b7dd4993c5753074196e03111d9f85` | `3eabbfaf6921cedc5245c18450092747e0c8ba506bb4a47ca04d8b131b33633c` |

The `snippet-only` dry-run planned exactly these three generated-adapter updates:

```bash
/Users/yuval.meiri/.local/bin/engram harness install \
  --harness claude-code \
  --settings-target snippet-only \
  --json
```

The same dry-run skipped `claude-settings-merge` with:

```text
settings target is snippet-only; no Claude settings file will be modified
```

The preflight diffs were limited to scoping obligation guidance:

```text
obligations(action=detect, project=..., cwd=...)
obligations(action=doctor, project=..., cwd=...)
```

## Execution

T333 executed the T314 command:

```bash
/Users/yuval.meiri/.local/bin/engram harness install \
  --harness claude-code \
  --settings-target snippet-only \
  --write \
  --json
```

The report wrote exactly:

- `/Users/yuval.meiri/.claude/commands/engram-memory-session.md`
- `/Users/yuval.meiri/.claude/commands/engram-end-session.md`
- `/Users/yuval.meiri/.claude/hooks/engram-stop-nudge.sh`

It skipped `claude-settings-merge`, the user-owned snippet, and already-installed generated
adapters. It did not launch native Claude, run `/hooks`, send signals, mutate repository files,
or modify Claude settings.

## Postflight Evidence

Post-write hashes match the expected T313 hashes:

| Adapter path | Post-write hash | Mode/bytes |
| --- | --- | --- |
| `/Users/yuval.meiri/.claude/commands/engram-memory-session.md` | `a5075190c01731c82be7b50eb219fe7e467812c3d210e083eec9405e1ff95259` | `-rw-r--r--`, 1899 |
| `/Users/yuval.meiri/.claude/commands/engram-end-session.md` | `63c932a02ebd40563be6b7aa90200653d04c8073df61a858a606a7a8dd6482fb` | `-rw-r--r--`, 651 |
| `/Users/yuval.meiri/.claude/hooks/engram-stop-nudge.sh` | `3eabbfaf6921cedc5245c18450092747e0c8ba506bb4a47ca04d8b131b33633c` | `-rwxr-xr-x`, 977 |

Settings-related hashes remained unchanged before and after the write:

| File | SHA-256 |
| --- | --- |
| `/Users/yuval.meiri/.claude/settings.json` | `06efbf2a5d84ba62a1fcba0854863579ae23aaabb270e8a7bce7a88368ecf549` |
| `/Users/yuval.meiri/.claude/settings.local.json` | `7395cb5bd9d6c6df7659673ddb4516ae5450a47f51b5d09cda80ff7c3a34d4f2` |
| `/Users/yuval.meiri/.claude/engram-settings-snippet.json` | `b677c1ed6b915e3186d433f25148d1f7f1e697b0ec0a793e5c3c742833733d60` |

Post-write `snippet-only` dry-run reports `planned=[]`. Installed CLI
`harness status --harness claude-code --json` and
`harness doctor --harness claude-code --json` both report `ready=true`.

Remaining warnings are not adapter drift: the snippet is user-owned, settings contain extra legacy
Engram permission entries, settings are split across `settings.json` and `settings.local.json`, and
lifecycle compliance remains a soft contract.

## Interpretation

T333 closes the generated-adapter drift blocker recorded by T313 and executes the prepared T314
contract. It does not prove prompt-bearing native Claude behavior, effective `/hooks` visibility,
live Claude host labels, multi-host parity, hosted CI, PR readiness, merge, tag, publish, lifecycle
cleanup, M6 mutation, or production/GA completion.
