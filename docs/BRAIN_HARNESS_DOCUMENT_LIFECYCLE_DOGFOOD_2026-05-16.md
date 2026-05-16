# Brain Harness Document Lifecycle Dogfood

Date: 2026-05-16
Status: Live Codex dogfood passed for scoped document-disposition flow

## Question

Can the current Engram runtime support a low-friction document lifecycle loop for Codex:
detect a changed durable document, surface a scoped obligation, and require the agent to resolve or
explicitly skip it before final response?

## Setup

- Base repo commit: `ba06270` (`Scope obligation doctor reports`)
- Installed runtime: `/Users/yuval.meiri/.local/bin/engram`
- Installed binary hash after refresh:
  `423c97b6e8df6a431ad5cd72bd6fc1e388e83def5ec7d18d137d008ebf928174`
- Daemon restarted on port `8765`, PID `86914`
- Claude Code Engram hooks remained disabled; this run exercised Codex plus MCP.

## Runtime Smoke

After install and daemon restart, a scoped live MCP doctor call for:

```text
project=engram
cwd=/Users/yuval.meiri/projects/engram
```

returned:

```json
{ "open": [], "warnings": [] }
```

This confirms the newly installed daemon no longer surfaced unrelated open obligations for the
current Engram checkout.

## Clean Worktree Dogfood

To avoid contamination from the pre-existing untracked root `AGENTS.md`, the first live dogfood used
a clean detached worktree:

```text
/tmp/engram-doc-lifecycle-dogfood.G9awvy
```

The agent created:

```text
docs/document-lifecycle-dogfood-scratch.md
```

Dry-run detection returned exactly one candidate:

```text
Resolve document memory status for docs/document-lifecycle-dogfood-scratch.md
```

Durable detection wrote one obligation:

```text
019e3233-792a-75d0-ad9d-313b4216683f
```

Scoped doctor returned that one open obligation with the expected warning. The agent skipped it with
an explicit reason because the file was a throwaway scratch document and the durable result is this
committed report plus Engram memory. A follow-up scoped doctor call returned no open obligations.

The temporary worktree was removed after the obligation was resolved.

## Result

The narrow document lifecycle loop passed:

- document change detection worked,
- scoped doctor surfaced the relevant obligation,
- final-response cleanup was possible without unrelated-project noise,
- explicit skip was sufficient for a temporary document,
- the workflow remained low-friction for Codex.

## Caveats

- This was a single Codex dogfood, not cross-harness evidence.
- The main checkout still has a pre-existing untracked root `AGENTS.md`; it must stay out of commits.
- Claude Code hooks are still intentionally disabled, so this does not validate Claude terminal hook
  UX.
- This does not justify M6 write-apply, deletion, broad ranking changes, or hot-path expansion.

## Next Gate

The next evidence step should validate the same lifecycle on a non-temporary project document in the
main Engram checkout, resolving the obligation by recording/registering/indexing the document rather
than skipping it. If that stays low-friction, the lifecycle can become the default Codex document
follow-through practice before re-enabling Claude hooks.
