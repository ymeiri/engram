# Brain Harness T334 Native Claude Preflight After T333

Date: 2026-06-07
Status: completed read-only preflight; native launch hard-stopped

## Research Question

After T333 repaired the generated Claude Code adapters, can Engram safely execute the native
Claude prompt-bearing, effective-hook, or host-label production gates?

## Hypotheses

| Hypothesis | Claim | Result |
| --- | --- | --- |
| Preferred | T333 removes generated-adapter drift, so fresh preflight can either execute or isolate the next blocker. | Supported. Adapter readiness is clean; process attribution is still blocked. |
| Null | Adapter repair did not change the executable gate state. | Rejected for adapter readiness, supported for native launch permission. |
| Simpler alternative | Leave the T333 non-claims as-is and defer all native gates. | Rejected because fresh preflight gives stronger current evidence. |
| Failure | Launch native Claude despite ambiguous attribution, execute stale T269/T270 scope, or overclaim production parity. | Avoided. |

## Preflight Evidence

Repository and PR state:

- Branch `yuval.meiri/memory-os-phase1` is synced with its upstream:
  `HEAD...@{u}` is `0 0`.
- `HEAD...origin/main` is `29 0`.
- The tracked worktree has no diff; only root `AGENTS.md` is untracked and user-owned.
- PR #3 is open and draft at head
  `819d9577cbceaa1e4bddb0c042043dd03ad1d738`.
- Hosted CI run `27098460967` still fails before workflow steps; all five jobs report failure
  from the external GitHub Actions billing/spending-limit gate, not source execution.

Claude runtime state:

- `which claude` resolves to `/Users/yuval.meiri/.local/bin/claude`.
- The symlink target is `/Users/yuval.meiri/.local/share/claude/versions/2.1.168`.
- `/Users/yuval.meiri/.local/bin/claude --version` returns `2.1.168 (Claude Code)`.
- Both the symlink and resolved target hash to
  `377f0ecedba8246bdabdf312ce8b7cc8ae1160997b26f5edca352a4a8d61dc78`.

Harness and daemon state:

- Installed Claude Code `harness status` reports `ready=true`.
- Installed Claude Code `harness doctor` reports `ready=true`.
- `engram harness install --harness claude-code --settings-target snippet-only --json` reports
  `planned=[]`.
- The daemon is running on port `8765`, PID `75180`, spawned by
  `/Users/yuval.meiri/.local/bin/engram`, spawn version `0.2.0-beta.1`.
- Obligations doctor returns no open obligations and no warnings.
- Canonical vault status is count-aligned at 2,539 generated files, zero user files, and
  2,539 expected generated files.

Known warnings remain bounded:

- `/Users/yuval.meiri/.claude/engram-settings-snippet.json` is user-owned.
- `settings.json` and `settings.local.json` still contain extra legacy Engram permissions.
- Engram Claude settings are split across `settings.json` and `settings.local.json`, so effective
  hook visibility still requires a native `/hooks` measurement.
- Lifecycle compliance remains a soft contract.

Telemetry state:

- `real_session_eval(project=engram, limit=50)` recorded 50 traces and 17 feedback records.
- Feedback coverage is about `0.32`, below the `0.50` confidence gate.
- The confidence gate remains failed only for feedback coverage. This continues to block M6
  write-apply confidence, not the read-only preflight.

Monitored hashes:

| Path | SHA-256 |
| --- | --- |
| `/Users/yuval.meiri/.claude/settings.json` | `06efbf2a5d84ba62a1fcba0854863579ae23aaabb270e8a7bce7a88368ecf549` |
| `/Users/yuval.meiri/.claude/settings.local.json` | `7395cb5bd9d6c6df7659673ddb4516ae5450a47f51b5d09cda80ff7c3a34d4f2` |
| `/Users/yuval.meiri/.claude/engram-settings-snippet.json` | `b677c1ed6b915e3186d433f25148d1f7f1e697b0ec0a793e5c3c742833733d60` |
| `/Users/yuval.meiri/.claude/commands/engram-memory-session.md` | `a5075190c01731c82be7b50eb219fe7e467812c3d210e083eec9405e1ff95259` |
| `/Users/yuval.meiri/.claude/commands/engram-end-session.md` | `63c932a02ebd40563be6b7aa90200653d04c8073df61a858a606a7a8dd6482fb` |
| `/Users/yuval.meiri/.claude/hooks/engram-stop-nudge.sh` | `3eabbfaf6921cedc5245c18450092747e0c8ba506bb4a47ca04d8b131b33633c` |

The adapter hashes match the T333 postflight state, and `engram-stop-nudge.sh` remains executable.

## Hard Stop

Fresh process inventory still shows already-running native Claude CLI sessions:

| PID | PGID | TTY | Command |
| --- | --- | --- | --- |
| `60453` | `60453` | `ttys001` | `claude` |
| `311` | `311` | `ttys005` | `claude --plugin-dir /Users/yuval.meiri/go/src/github.com/DataDog/claude-marketplace/ai-developer-workflows` |

Those processes make attribution ambiguous for any new prompt-bearing native Claude run. T312's
execution contract requires stopping before launch when existing native Claude or Claude-family
processes would make attribution ambiguous. Therefore T334 did not launch native Claude, send a
prompt, run `/hooks`, signal a process, use Claude Bridge, or mutate harness/settings/adapters.

## Gate Impact

- T312 prompt-bearing native Claude validation remains unexecuted. Its path/version/hash and
  harness assertions now match, but native process attribution does not.
- T269 effective-hook visibility remains unexecuted and is stale as-is for this runtime because
  its packet explicitly hard-stops when the Claude binary target/version differs from the
  `2.1.161` baseline unless a fresh exact drift approval names the drift.
- T270 live host-label proof remains unexecuted. It can piggyback only on an exact native Claude
  prompt-bearing packet that names the host-label scope and then proves the stored trace label.
- Hosted CI remains externally blocked by the GitHub Actions account gate.
- The supported local/Codex MVP beta scope is unchanged. Native Claude prompt-bearing proof,
  effective-hook visibility, and live host-label proof remain production-hardening gates.

## Interpretation

T334 narrows the production-hardening state. Generated Claude Code adapter drift is no longer a
blocker after T333; installed Claude Code harness status/doctor are now `ready=true`. The remaining
native-Claude blocker is not adapter readiness. It is clean attribution for a new native run, plus
runtime-specific successor handling for `/hooks` and host-label proof.

Do not claim production/GA Brain Harness completion from T334. The correct next choices are either:

- defer native Claude/effective-hook/host-label gates explicitly for the beta release, or
- rerun the same preflight only after the ambient native Claude sessions are gone and then execute
  a fresh exact packet that names the intended native prompt, `/hooks`, or host-label scope.
