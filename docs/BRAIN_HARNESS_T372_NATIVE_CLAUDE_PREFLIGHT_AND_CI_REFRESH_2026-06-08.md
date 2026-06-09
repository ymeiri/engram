# Brain Harness T372 Native Claude Preflight And CI Refresh

Date: 2026-06-08
Status: completed read-only production-gate preflight and current hosted-CI refresh.

## Scope

T372 refreshes current production-gate evidence after T371 moved PR #3 to
`3cf0e3d453fe4f02a0e1019bcf79fe8779e72cde`
(`Record T371 exact-head validation`). It focuses on two questions:

- Is the current PR head still blocked only by hosted CI that fails before workflow steps?
- Can the native Claude prompt-bearing, effective-hook, and live host-label production proofs run
  without ambiguous attribution?

This slice is read-only for runtime and harness state. It does not launch native Claude, send
prompts, execute `/hooks`, signal or kill processes, mutate Claude settings, mutate hooks or
adapters, run `lint apply_safe`, run M6 write-apply, mark PR #3 ready, merge, tag, publish, delete
data, or change the supported beta scope.

## Current Branch And PR State

Current worktree state before this docs-only slice:

```text
## yuval.meiri/memory-os-phase1...origin/yuval.meiri/memory-os-phase1
?? AGENTS.md
HEAD = 3cf0e3d453fe4f02a0e1019bcf79fe8779e72cde
origin/yuval.meiri/memory-os-phase1 = 3cf0e3d453fe4f02a0e1019bcf79fe8779e72cde
HEAD...origin/yuval.meiri/memory-os-phase1 = 0 0
```

The untracked root `AGENTS.md` remains user-owned instruction context and was not staged.

PR #3 current state:

```text
headRefOid = 3cf0e3d453fe4f02a0e1019bcf79fe8779e72cde
isDraft = true
mergeable = MERGEABLE
mergeStateStatus = UNSTABLE
hosted run = 27142919365
hosted conclusion = failure
hosted jobs = Docs, Clippy, Test, Format, Check
hosted job steps = []
```

Hosted run `27142919365` targets the current T371 head and every hosted job still fails before any
workflow step executes. That keeps the hosted-CI signal non-diagnostic for Rust source, tests,
docs, and packaging.

## Installed Runtime And Harness Evidence

Installed Claude Code evidence:

```text
CLAUDE_BIN=/Users/yuval.meiri/.local/bin/claude
/Users/yuval.meiri/.local/bin/claude -> /Users/yuval.meiri/.local/share/claude/versions/2.1.168
claude --version = 2.1.168 (Claude Code)
sha256 = 377f0ecedba8246bdabdf312ce8b7cc8ae1160997b26f5edca352a4a8d61dc78
```

Installed Engram daemon evidence:

```text
Daemon status: running
Port: 8765
PID: 47577
Spawned by: /Users/yuval.meiri/.local/bin/engram
Spawn version: 0.2.0-beta.1
Current CLI: /Users/yuval.meiri/.local/bin/engram
```

`/Users/yuval.meiri/.local/bin/engram harness status --harness claude-code --json` and
`/Users/yuval.meiri/.local/bin/engram harness doctor --harness claude-code --json` both report
`ready=true`. Required generated command and hook adapters are installed, required Engram MCP
permissions and hook entries are present across `settings.json` and `settings.local.json`, and the
known warnings remain:

- the settings snippet is user-owned and will not be overwritten;
- extra historical Engram permissions exist outside the current Claude harness contract;
- settings are split across `settings.json` and `settings.local.json`, so effective hook
  configuration still needs Claude Code `/hooks` visibility proof;
- lifecycle compliance is still a soft contract that depends on agent behavior.

Memory OS state remains clean for this preflight:

```text
obligations doctor: open=[], warnings=[]
vault root: /Users/yuval.meiri/.engram/vault
vault initialized: true
vault total_file_count: 2686
vault generated_file_count: 2686
vault user_file_count: 0
vault expected_generated_file_count: 2686
```

## Native Claude Attribution Gate

Focused process inventory shows a live native Claude process tree:

```text
34797 18673 ttys004  Mon Jun  8 11:49:15 2026     claude
34808 34797 ttys004  Mon Jun  8 11:49:16 2026     peekaboo mcp serve
34809 34797 ttys004  Mon Jun  8 11:49:16 2026     /Users/yuval.meiri/.local/bin/engram serve
34811 34797 ttys004  Mon Jun  8 11:49:16 2026     node /Users/yuval.meiri/projects/ai-council/dist/index.js
34872 34797 ttys004  Mon Jun  8 11:49:16 2026     .../Python /Users/yuval.meiri/projects/markdown-to-doc/run_mcp_server.py
34881 34797 ttys004  Mon Jun  8 11:49:16 2026     .../Python /Users/yuval.meiri/mcp-servers/custom/nano-banana-mcp/main.py
35015 34797 ttys004  Mon Jun  8 11:49:17 2026     npm exec chrome-devtools-mcp@latest --isolated=true
35024 34797 ttys004  Mon Jun  8 11:49:18 2026     .../Python /Users/yuval.meiri/mcp-servers/custom/slides-mcp/main.py
35077 34797 ttys004  Mon Jun  8 11:49:19 2026     npm exec @playwright/mcp@latest
35282 34811 ttys004  Mon Jun  8 11:49:26 2026     engram serve
```

This preserves the T368 hard stop. T312 prompt-bearing proof, T335 `/hooks` effective-hook proof,
and T270 live host-label proof remain unproved because a new native Claude proof run would have
ambiguous attribution while this existing native `claude` session is live. Closing that gate still
requires either the process exiting naturally or explicit user approval naming the exact process or
session action, followed by a fresh read-only preflight.

## Validation

After this T372 evidence note and cross-references are present in the working tree, the T372
candidate is validated with:

- `./scripts/local-ci.sh`
- `./scripts/package-install-smoke.sh`

Those commands keep the current docs-only head aligned with the T371 exact-head validation
standard. They do not close hosted CI; hosted CI remains externally blocked until a hosted run
executes workflow steps and passes.

## Gate Impact

For the scoped local/Codex MVP beta, T372 keeps the ship path unchanged:

- release-owner acceptance of the exact-head local CI plus package/install fallback, or restored
  exact-head hosted CI green;
- then ready/merge/tag/publish mechanics for `v0.2.0-beta.1`.

For production/GA, T372 keeps the native Claude/effective-hook/live-host-label gate open. Generated
adapter readiness is present, but behavioral proof is still blocked by live-process attribution and
by the requirement to observe effective Claude Code `/hooks` output under an exact approved packet.
