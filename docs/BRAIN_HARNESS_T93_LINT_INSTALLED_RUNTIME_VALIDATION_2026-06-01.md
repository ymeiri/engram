# Brain Harness T93 Lint Installed Runtime Validation

Status: Completed installed-runtime validation

Scope: Validate that the committed T92 lint priority behavior is active in the live MCP daemon.

T93 does not change source behavior, archive memory, run `lint(action="apply_safe")`, inspect T69
files, run T70 document indexing, run M6, change retrieval ranking, expand `orient`, change public
MCP request fields, change schema/storage/index behavior, change document-index behavior, or write
harness adapters or hooks.

## Research Question

Does the live MCP lint path reflect the committed T92 ordering, where stale current-plan feedback
stays first and safe-action `superseded_item_still_active` findings appear before generic
`feedback_stale_active_memory` rows?

## Hypotheses

| Hypothesis | Prediction |
|---|---|
| Preferred | The live daemon is running an older binary; refreshing the installed binary and daemon makes the MCP lint report match the committed T92 source behavior. |
| Null | The live daemon is current, but live data does not produce safe-action superseded-active findings. |
| Simpler alternative | Keep T92 source-level validation only and document the live runtime gap. |
| Failure | Runtime refresh disrupts the daemon or is mistaken for lifecycle cleanup authority. |

## Measurement

Before refreshing the installed runtime:

- `cargo test -p engram-index lint_prioritizes_superseded_active_items_before_generic_feedback_noise`
  passed against the current source.
- Source inspection showed `engram-index/src/lint.rs` assigns priority `25` to safe-action
  `SupersededItemStillActive`, while generic `FeedbackStaleActiveMemory` remains priority `30`.
- The installed binary hash was
  `28c7ea10b61f1603b183b0c4d8d31e9d0b829ebba8eff8ad9a3dcaf26070457d`.
- The daemon was running on port `8765`, PID `4272`.
- Live MCP `lint(action="run", limit=20)` still showed generic `feedback_stale_active_memory`
  rows starting at rank 5.
- Live MCP `lint(action="run", limit=80)` still did not surface
  `superseded_item_still_active` before generic stale-feedback rows.

Refresh:

- Ran `cargo install --path engram-cli --force --root /Users/yuval.meiri/.local`.
- Stopped and restarted the daemon with `/Users/yuval.meiri/.local/bin/engram daemon stop` and
  `/Users/yuval.meiri/.local/bin/engram daemon start`.
- The refreshed installed binary hash is
  `e54aed9a4830cc53822100930d63541bf51d06b3f27c2844e6090bfe01f5379a`.
- The daemon restarted on port `8765`, PID `56865`.

After refresh:

- Live MCP `lint(action="run", limit=20)` returned `feedback_stale_current_plan` first.
- The three `feedback_wrong_scope_active_memory` rows followed.
- Safe-action `superseded_item_still_active` rows started at rank 5.
- Generic `feedback_stale_active_memory` rows no longer displaced safe-action superseded-active
  rows in the first 20.
- `applied_safe_actions` remained `0`.

## Completion Matrix Delta

| Area | State After T93 | Evidence | Remaining Risk |
|---|---|---|---|
| T92 installed-runtime behavior | Validated in live MCP daemon | Refreshed binary hash `e54aed9...`; MCP lint rank 5 starts `superseded_item_still_active` | Validation depends on current live data shape |
| Lifecycle cleanup | Still gated | `applied_safe_actions=0`; no archive command was run | T88 or a separate exact lifecycle approval remains required |
| Current-plan feedback | Preserved | MCP lint still returns stale current-plan feedback first | Stale repository-scoped current-plan memory remains a separate unresolved lifecycle decision |
| M6/document-index/harness | Unchanged and gated | No related tools or writes run | T69/T70/T88 exact approvals still required |

## Result

The preferred hypothesis held. T92 was correct in source and tests, but the live daemon had not yet
been refreshed. After installing the current Engram binary and restarting the daemon, the MCP lint
report matched the intended T92 ordering without applying any safe actions or crossing any gated
surface.
