# Brain Harness T283 T282 Native Claude Preflight Ambient Process Deferral - 2026-06-06

## Scope

T283 executes the fresh read-only preflight for the T282 Claude `2.1.163` prompt-bearing native
Claude successor packet.

T283 does not launch native Claude, send prompts, run `/hooks`, edit hooks/settings/adapters, run
harness install, mutate lifecycle state, run M6, change ranking or `orient`, delete data, roll
back, send process signals, or touch user-owned files.

## Research Question

Can the T282 successor packet proceed to exactly one prompt-bearing native Claude PTY session under
the current state?

## Result

No. The preflight hard-stopped before launch because the process snapshot showed live native Claude
processes that make a new single-session transcript attribution ambiguous.

The T282 packet requires existing native Claude or Claude-family processes to be listed and says to
stop before launch if any process would make attribution ambiguous. T283 observed two live native
Claude processes before launch, so the prompt-bearing native Claude validation remains unexecuted.

## Preflight Evidence

| Assertion | Evidence | Result |
| --- | --- | --- |
| Branch/upstream | Branch `yuval.meiri/memory-os-phase0` tracks `origin/yuval.meiri/memory-os-phase0`; after `git fetch origin --prune`, `HEAD...@{u}` was `0 0`. | Pass |
| Main ancestry | `origin/main` remains an ancestor of `HEAD`; `HEAD...origin/main` was `396 0`. | Pass |
| Worktree | Tracked diff was empty; only root `AGENTS.md` was untracked and user-owned. | Pass |
| CLI path | `command -v claude` returned `/Users/yuval.meiri/.local/bin/claude`. | Pass |
| Symlink target | `/Users/yuval.meiri/.local/bin/claude -> /Users/yuval.meiri/.local/share/claude/versions/2.1.163`. | Pass |
| Version | `/Users/yuval.meiri/.local/bin/claude --version` returned `2.1.163 (Claude Code)`. | Pass |
| Target hash | Resolved target SHA-256 was `c7582e926e8fe459dbd9743f19ccb75500e3b455c722902d1aa587a74fb1fa7c`. | Pass |
| Harness status | `harness(action="status", harness="claude_code")` returned `ready=true` with the known user-owned snippet, split-settings, and extra legacy permission warnings. | Pass with known warnings |
| Harness doctor | `harness(action="doctor", harness="claude_code")` returned `ready=true` with the same warnings plus the soft lifecycle-compliance caveat. | Pass with known warnings |
| Daemon | `engram daemon status` reported running on port `8765`, PID `25189`. | Pass |
| Obligations | `obligations(action="doctor", project="engram")` returned `open=[]`, `warnings=[]`. | Pass |
| Telemetry window 20 | `real_session_eval(project="engram", limit=20)` returned `trace_count=20`, `feedback_trace_count=10`, `feedback_coverage=0.5`, `distinct_intent_count=2`, `task_failure_count=0`, and confidence gate failed because only two intents had feedback. | Recorded, not a launch blocker |
| Telemetry window 50 | `real_session_eval(project="engram", limit=50)` returned `trace_count=50`, `feedback_trace_count=30`, `feedback_coverage=0.6000000238418579`, `distinct_intent_count=5`, `task_failure_count=0`, and confidence gate passed. | Recorded |
| Monitored Claude config inventory | 43 files across user/project Claude commands, hooks, settings, and Engram snippets; aggregate SHA-256 over per-file hashes was `f3447ac2608c92ed4bc7d3986f9396e2395d162fcc3b1929057aafcdee034949`. | Baseline captured |
| Process attribution | Two live native `claude` processes were present before launch. | Hard stop |

## Process Snapshot

The preflight process snapshot showed:

```text
64035 61476 64035 ??      S  node /opt/homebrew/bin/codex-claude mcp
84269 61476 84269 ??      S  node /opt/homebrew/bin/codex-claude mcp
93703     1 93703 ??      S  /Applications/Visual Studio Code.app/Contents/MacOS/Code --goto /Users/yuval.meiri/Downloads/claude-code-partial-eval-command-recreation-prompt.md
45186 21753 45186 ttys001 S+ claude
  311 93883   311 ttys005 S+ claude --plugin-dir /Users/yuval.meiri/go/src/github.com/DataDog/claude-marketplace/ai-developer-workflows
```

The two `claude` processes on `ttys001` and `ttys005` are enough to make a new native PTY session's
startup, prompt, trace, and shutdown evidence ambiguous. T283 therefore stopped before launch and
sent no signal.

## Non-Claims

T283 does not prove prompt-bearing native Claude behavior, effective-hook visibility, live Claude
host-label adoption, clean EOF behavior, lifecycle cleanup, direct legacy deprecation, or broad
cross-harness parity.

T283 also does not invalidate the T282 successor packet. It only records that the packet was not
safe to execute under the current process state. A future run may retry the T282 execution contract
after fresh preflight shows no attribution-confusing native Claude processes.
