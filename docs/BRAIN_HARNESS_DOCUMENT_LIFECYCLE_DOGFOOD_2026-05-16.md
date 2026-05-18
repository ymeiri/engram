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

## Main Checkout Follow-Up

Status: passed.

This report itself is now the non-temporary project document for the next lifecycle check. The
resolution was stronger than the scratch run:

- detect the changed report as a durable document,
- surface the report obligation through scoped doctor,
- register or index the report where supported by the current tools,
- record the result as durable Memory OS evidence,
- resolve the document obligation without using `skipped_with_reason`.

Detection wrote the report obligation:

```text
019e323a-233a-7a62-856a-b18bf3dd67ac
```

It also wrote a commit-preference obligation, which was resolved after checking the reviewed
project preference memory:

```text
019e03be-a9a5-7db2-848d-eb26ef78bcb5
```

The report was registered in the knowledge registry:

```text
019e323a-70d3-7fc0-a749-d32b20e6aea9
```

`docs(index)` also succeeded for this file:

```json
{ "documents_indexed": 1, "chunks_created": 1, "warnings": [] }
```

The follow-up was recorded as Memory OS evidence:

```text
019e323a-9a8c-79d1-8ad9-fd461dbeb257
```

The report obligation was resolved as `knowledge_registered`. A scoped doctor call for the main
checkout then returned:

```json
{ "open": [], "warnings": [] }
```

Because writing this section is itself another durable document edit, a final validation detect pass
created one more report obligation:

```text
019e323b-2d6d-7210-bdc3-2a7621607d50
```

That follow-up obligation was handled the same way: re-index the final report content and resolve it
as an indexed document. This confirms the behavior, but it also exposes a loop-avoidance design point
for future work: final-response obligation checks should avoid repeatedly creating new obligations
for the same already-addressed document update within one turn.

The pre-existing untracked root `AGENTS.md` remained untouched and out of the task commit.

## Content Idempotence Follow-Up

Status: passed.

This section is the live durable-document target for the content-idempotence fix in commit
`6cc08cf` (`Make document obligations content-idempotent`).

The first live report-content pass validated same-content suppression:

- installed binary hash:
  `5a554aa9411b17da559ec4bc558f0a25c28d847535a479e5d6918f236d9ebd07`,
- restarted daemon: port `8765`, PID `2817`,
- first report obligation:
  `019e39f7-726e-7d72-a394-8fc54898801e`,
- first report fingerprint:
  `af2a1c19199b0ed2d26ea5711a7b92dc1de39b05`,
- `docs(index)` result:
  `{ "documents_indexed": 1, "chunks_created": 1, "warnings": [] }`,
- same-content `obligations(detect, write=true, limit=1)` result after resolution:
  `written=0`, `skipped_existing=1`.

This final section edit is intentionally a second real content state. A fresh obligation for it is
correct behavior; after resolving that final-content obligation, a same-content detect should again
write no new obligation. The final-content obligation ID is recorded in Engram memory and the task
closeout instead of being embedded here, so recording the ID does not create a third document
content state.

The validated behavior is:

- detect this report as changed durable content,
- resolve the report obligation after indexing or recording it,
- run detection again without changing this content,
- verify no new report obligation is written for the already-resolved content,
- preserve the existing behavior that a later real content edit creates a fresh obligation.
