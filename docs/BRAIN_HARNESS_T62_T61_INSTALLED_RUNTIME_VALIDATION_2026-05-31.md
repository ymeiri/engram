# Brain Harness T62 T61 Installed Runtime Validation

Status: Completed with the known Claude handoff write caveat.
Date: 2026-05-31
Scope: Installed-runtime validation for the T61 continuation `should` gate fix

This validation did not run M6 review export, review apply, deletion, cleanup, lifecycle mutation,
schema/storage/index changes, public MCP changes, ranking changes, `orient` changes, or harness
adapter/hook changes.

It did refresh the installed Engram CLI from the current checkout and restart the local daemon so
the committed T61 code could be tested through the live MCP runtime.

## Research Question

After installing commit `abc13a789eba892994aafbd4dd01f1c134248c09`, does the live runtime surface
T61 first for the exact T60 continuation wording while preserving default-deny behavior for
explicit `migration_review_export` action prompts?

## Hypotheses

| Type | Hypothesis |
| --- | --- |
| Preferred | Installing T61 makes the exact continuation search and lean `orient` return the T61 current-plan memory first, while explicit `should we run migration_review_export` prompts remain gate-first. |
| Null | The deterministic fixture passes only in tests; live memory noise still pushes the current plan below older guidance. |
| Simpler alternative | Keep T61 as source-only evidence and wait for the next approved M6 step. |
| Failure | The installed runtime promotes current-plan guidance above M6 gate evidence for explicit review-export action prompts, or the runtime refresh causes unrelated data writes. |

## Baseline

Before install, the live exact continuation search trace `019e7e91-e4dc-73d0-827e-5bbfae88eed6`
returned the T61 current-plan memory second, behind the research-method rule. This was better than
the T60 rank-five miss but did not prove the stricter T61 fixture in live data.

The installed binary hash before refresh was:

```text
c8b1254ac71f53da80221a2a259014fca89e2e8e8ca1998a4f0128adce01e721
```

The daemon status also reported a stale/nonresponding daemon entry for port `8765`.

## Runtime Refresh

Installed the current checkout with:

```text
cargo install --path engram-cli --force --root /Users/yuval.meiri/.local
```

The refreshed installed binary hash is:

```text
25715d5c2334a423dfdf73d8fc3868037ffe9c1a180f8a3df9926c6727d1464f
```

The daemon was stopped and restarted. Final status reported port `8765`, PID `56374`, running.

## Codex Live Results

| Probe | Trace | Result |
| --- | --- | --- |
| Continuation `search` | `019e7f49-4837-7a91-ae45-218f0b440113` | T61 current-plan memory `019e7e8f-2743-7e13-955f-c406062c12a3` ranked first. Stale repository-scoped current-plan memory remained second and was treated as background evidence. |
| Explicit gate `search` | `019e7f49-4861-7d80-af4c-25671bc9e1f9` | Paused migration review gate `019dd35d-1a48-7103-b0e2-390225f8b418` ranked first; T61 ranked second; explicit M6 gate `019e7ce5-155d-7a10-85f5-00b9dcc69cd0` ranked third. |
| Continuation lean `orient` | `019e7f49-4918-77f2-9174-fe4d59647735` | T61 current-plan memory appeared first in Brain Loop top items. |

## Claude Code Results

Claude Bridge ran with `write=false` and only `mcp__engram__search` / `mcp__engram__orient`
allowed.

| Probe | Trace | Result |
| --- | --- | --- |
| Continuation `search` | `019e7f49-ecbf-7c43-990d-14e929ef89f1` | T61 current-plan memory ranked first. |
| Explicit gate `search` | `019e7f49-f477-70e3-942b-554f01086a0b` | Paused migration review gate ranked first; T61 ranked second; explicit M6 gate ranked third. |
| Continuation lean `orient` | `019e7f49-fabb-7ab1-924b-020831797cb9` | T61 current-plan memory appeared first in Brain Loop top items. |

## Write Caveat

The Claude no-write caveat repeated.

After the Claude probe, `memory(action="changes_since")` trace
`019e7f4a-6b3e-7d52-9521-1b484dafe357` reported two new active Claude Code rolling handoff
MemoryItems and zero Memory OS commits:

- `019e7f4a-47c9-79a1-9c8b-2f6960fbf0c2`
- `019e7f4a-47c9-79a1-9c8b-2f5860fe7324`

Both were duplicate session-end handoffs for Claude session
`6265b384-1221-4c89-8db5-dcafdb3cb6b3`. This validation does not authorize deleting,
archiving, suppressing, deduplicating, or changing handoff behavior, hooks, settings, or adapters.

## Verdict

T62 validates the installed T61 behavior in Codex and Claude Code:

- exact continuation search now returns T61 current-plan memory first;
- continuation lean `orient` returns T61 first in Brain Loop;
- explicit `should we run migration_review_export` prompts preserve default-deny gate-first
  behavior;
- no M6 review export/apply or candidate decision was run.

This is runtime validation for the T61 prompt class only. It does not prove broad ranking quality
and does not authorize M6 review export/apply, lifecycle writes, schema/storage/index changes,
public MCP changes, `orient` payload expansion, or harness adapter/hook changes.
