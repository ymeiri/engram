# T302 Phase 0 Merge And Native Claude Preflight

Date: 2026-06-07

## Research Question

Can the validated phase-0 beta candidate be resolved on `main`, and can phase 1 immediately close
the native Claude prompt-bearing gate?

## Decision

Phase 0 is merged. PR #2 was marked ready and merged into `main` as merge commit
`71fd746402c7d63f8b5aa758bc2011796819b5f6` after exact-head CI run `27077943994`
passed Check, Format, Docs, Clippy, and Test on head
`93bc2428a452edf9c19322e9a63b7b1c757b52f2`.

Phase 1 is now the active work branch: `yuval.meiri/memory-os-phase1`, created from
`origin/main`. The only untracked worktree file remains user-owned root `AGENTS.md`.

The native Claude prompt-bearing gate is still not safe to execute. Fresh process preflight still
shows live native Claude CLI sessions on `ttys001` and `ttys005`, which makes a new single native
PTY session's trace and startup attribution ambiguous under the T282 packet. No signal was sent,
no native Claude prompt was launched, and no user Claude process was killed.

## Evidence

- `gh pr ready 2` marked PR #2 ready.
- `gh pr merge 2 --merge` merged PR #2 with merge commit
  `71fd746402c7d63f8b5aa758bc2011796819b5f6`.
- Post-merge `gh pr view 2` returned `state=MERGED`, `isDraft=false`,
  `mergedAt=2026-06-07T01:15:03Z`, and the same successful exact-head CI rollup.
- Fresh `git fetch --prune origin` moved `origin/main` to
  `71fd746402c7d63f8b5aa758bc2011796819b5f6`.
- On the old phase-0 branch, `git merge-base --is-ancestor HEAD origin/main` exited `0` and
  `git rev-list --left-right --count HEAD...origin/main` returned `0 1`, showing the branch head
  is included in `main` behind only the merge commit.
- `git switch -c yuval.meiri/memory-os-phase1 origin/main` created the phase-1 branch from the
  merged base and set it to track `origin/main`.
- Canonical vault status after the latest pre-release compile was count-aligned at
  `2386` generated files, `0` user files, and `expected_generated_file_count=2386`.
- `obligations(action=doctor, project=engram, cwd=/Users/yuval.meiri/projects/engram)` returned
  no open obligations and no warnings before this docs slice.
- Native Claude process preflight showed:

```text
45186 ttys001  claude
  311 ttys005  claude --plugin-dir /Users/yuval.meiri/go/src/github.com/DataDog/claude-marketplace/ai-developer-workflows
```

## Non-Claims

- T302 does not prove native Claude prompt-bearing behavior.
- T302 does not prove effective-hook visibility.
- T302 does not prove live Claude host-label adoption.
- T302 does not complete broad lifecycle cleanup or direct legacy deletion/deprecation.
- T302 does not change code, ranking, `orient`, schema, storage, MCP behavior, or harness
  behavior.
- T302 does not delete the merged phase-0 branch.

## Next Action

Continue phase-1 production hardening from `yuval.meiri/memory-os-phase1`. The next safe native
Claude attempt requires a fresh T282 preflight with no attribution-confusing native Claude
processes, or a separately documented decision to defer that gate while closing another production
gap such as exact lifecycle cleanup or direct legacy deprecation evidence.
