# Engram Memory OS: Design and Implementation Plan

Status: Implementation in progress
Date: 2026-04-26
Last Updated: 2026-06-04
Audience: Engram maintainers, AI coding-agent users, future contributors
Scope: Whether to extend Engram or build a new system; full design for a local-first AI memory operating system.

---

## Living Implementation Checklist

- [x] Memory OS ontology, writer provenance, lifecycle statuses, and knowledge commits.
- [x] Consolidated MCP `memory(action=commit)` surface; no separate `memory_commit` tool.
- [x] CLI `engram memory log`, `engram memory diff`, `engram memory changes-since`, `engram memory writer-stats`, and archive UX.
- [x] Generated Markdown vault with generated-file marker and Obsidian backlinks.
- [x] Orientation packet with repository/project resolution and deterministic relevance ordering.
- [x] Brain Loop v1 projection inside `orient`; bounded compiled context from already selected
      memory, with graph/obligations/lint kept out of the hot path.
- [x] Prompt-aware Brain Loop top-item calibration; buckets stay balanced, but a strongly
      prompt-matched reviewed decision can lead the bounded context.
- [x] MCP `orient` surfaces already-open agent obligations as a compact summary and recommended
      action without running obligation detection in the hot path.
- [x] `orient` filters stale git-status document obligations and untracked root instruction files
      from its open-obligation summary, while leaving stored obligation lifecycle explicit.
- [x] `orient` contract checkpoint documented and covered at the MCP boundary for review gating,
      prompt-specific ranking, obligation bounds, and stale-obligation suppression.
- [x] Real-session telemetry eval report: MCP `telemetry(action=real_session_eval)` summarizes
      persisted traces and agent feedback with coverage, per-intent signals, and a conservative
      migration confidence gate.
- [x] Telemetry memory-attribution coverage semantics: MCP `search` traces now populate
      `returned_memory_ids` for memory-layer results in addition to generic `returned_result_ids`,
      and `real_session_eval.memory_judgment_trace_coverage` uses a distinct trace denominator that
      remains valid for older search traces with memory-judgment feedback but no returned-memory
      field.
- [x] Live feedback coverage batch recorded: the pre-registered
      `live_feedback_coverage_2026_05_27` batch submitted feedback for all ten traces, lifted
      project-level `feedback_coverage` to `0.5227272510528564`, and exposed one design-preference
      retrieval failure plus migration-gate stale-evidence caveats without authorizing M6 work.
- [x] T04 design-preference retrieval repaired as a representation gap: the user-stated
      Ousterhout/deep-modules/no-unrequested-features preference is now an active reviewed
      user-scoped `MemoryItem`, and a deterministic unified-search fixture proves that an active
      design-philosophy preference is returned for the exact T04 query without ranking churn.
- [x] T06 lean-`orient` contract retrieval repaired as a representation gap: the lean response
      shape/hot-path contract is now an active reviewed project-scoped `MemoryItem`, and a
      deterministic unified-search fixture proves that active `orient` contract rules are returned
      for the exact T06 query without ranking churn.
- [x] T07 feedback-expectations retrieval repaired as a representation gap: the telemetry feedback
      contract and weak-signal caveat are now an active reviewed project-scoped `MemoryItem`, and a
      deterministic unified-search fixture proves that active feedback rules are returned for the
      exact T07 query without ranking churn.
- [x] T09 stale current-plan feedback lint specialized: active `decision`/`rule` MemoryItems tagged
      `current-plan` that receive stale feedback now report a specific read-only
      `feedback_stale_current_plan` lint finding with `safe_action=none`, so stale plan guidance is
      visible without automatic cleanup, ranking changes, or lifecycle mutation.
- [x] T10 old migration/export approval stale-feedback coverage clarified: old approval-shaped
      active MemoryItems that receive stale feedback remain covered by the generic read-only
      `feedback_stale_active_memory` lint finding with `safe_action=none`; no new M6 classifier,
      ranking change, lifecycle mutation, or migration action was introduced.
- [x] T11 startup feedback stabilization recorded: the post-T10 startup/orient and direct-search
      traces were scored, the current `real_session_eval(project=engram, limit=50)` report returned
      `feedback_coverage=0.5` (`22/44`) with `bad_memory_used_count=0`, the exact T07
      `review_memory` recheck passed, and stale migration-completion memory
      `019dd3fe-ec94-7122-af04-1f35b839387f` is visible through the generic read-only
      `feedback_stale_active_memory` lint finding with `safe_action=none`.
- [x] T12 gate-context current-plan ranking calibrated: direct unified `search` no longer treats bare
      query token `gate` as an approval-gate request when the prompt explicitly asks for
      current-plan/next-step context. Strong action or permission terms such as `should`, `proceed`,
      and `apply` still preserve migration-gate guidance above current-plan context. The focused
      fixture covers the observed `current plan next step ... M6 gate` wording without broad ranking
      weights, schema, lifecycle, migration, public MCP, or `orient` payload changes.
- [x] T13 installed-runtime validation for T12: after installing binary hash
      `62272400960eaaeb2fd7aa44aa13bf6f93abdbc81b5d11bc9106b0bcc82df29b`
      and restarting the daemon on port `8765`, PID `79904`, native MCP trace
      `019e6969-a674-7631-8ffa-b532b8638262` returned current-plan memory
      `019e6960-7ead-7001-9a4f-d8adce7c8264` first for the exact
      `current plan next step ... M6 gate` query. The same smoke exposed a separate live-data gap:
      explicit migration-apply/gate queries still ranked calibration or current-plan memory above M6
      gate context in traces `019e696a-0698-7e20-940a-b0ad23a29994` and
      `019e696a-2540-7172-a473-33f13538d54d`. That gap does not authorize M6 work.
- [x] T14 explicit migration-apply gate ranking calibrated: direct unified `search` now has a
      prompt-specific promotion for explicit migration/M6 apply-permission queries. Candidate gate
      items must carry migration/apply detail and actionable blocking language, and stronger stop
      signals outrank calibration notes, current-plan guidance, broad implementation history,
      reviewed dry-run batch summaries, and old approval history. After installing binary hash
      `fea91cc46549c138a425389394af9c4cdd9d8727eb39137f8afc179a976968eb`
      and restarting the daemon on port `8765`, PID `9969`, native MCP traces
      `019e698d-b766-7e71-a4da-a8c593f1b191` and
      `019e698d-b791-7d93-a0d6-542219e3eb6c` returned migration review gate memory
      `019dd35d-1a48-7103-b0e2-390225f8b418` first for explicit migration-apply prompts, while
      trace `019e698d-b7ae-7a13-b2c5-d58a9898deab` kept current-plan memory first for the
      current-plan/M6-gate context query. This is narrow ranking calibration, not M6
      authorization.
- [x] T15 Claude Code cross-harness smoke for T14: Claude Code `2.1.152`, using its own connected
      Engram MCP server and only `mcp__engram__search`, returned the paused migration gate memory
      first for explicit migration-apply traces `019e6993-d4da-70a1-b5eb-9185eeb23339` and
      `019e6993-d891-7ff3-93ef-4bd8ad14d9c7`, and returned current-plan memory
      `019e6992-e937-73e3-a165-a706d5f15a7d` first for contextual current-plan/M6 trace
      `019e6994-8ec9-7343-9198-9298867b9ceb`. This validates the shared MCP search behavior for
      the observed prompt class, not hooks, adapter writes, or broad ranking quality.
- [x] T17 read-only harness readiness re-audit: explicit `harness(action=doctor)` calls for
      `claude_code`, `codex`, `gemini_cli`, and `cursor` all returned `ready=false` with no writes.
      Claude Code's required generated adapter files are installed, but required `SessionStart` and
      `SessionEnd` settings hook registrations are missing and the optional settings snippet is
      user-owned. Codex, Gemini CLI, and Cursor still have required generated adapters drifted from
      current policy. This corrects the stale claim that Claude Code was fully ready; adapter and
      hook writes remain gated.
- [x] T18 read-only evidence gate re-audit: before scoring the T18 review-memory trace,
      `real_session_eval(project=engram, limit=50)` failed the confidence gate because feedback
      covered only two intents. After scoring T18 retrieval traces, the current report passes
      numerically again with `feedback_trace_count=32`, `feedback_coverage=0.7272727489471436`,
      `memory_judgment_coverage=1.0`, and `bad_memory_used_count=0`. This confirms the gate is
      sample-window sensitive and still weak agent-assessed evidence. Read-only
      `lint(action=apply_safe, write=false)` reported `applied_safe_actions=0`. The stale
      repository-scoped current-plan memory `019e5e0a-86b4-73e3-aa9b-ca350e83e915` now has
      repeated stale-feedback hits and still appears near the latest current plan in review-memory
      search. Archival/scope correction is a memory lifecycle write and needs explicit approval;
      document-index normalization also needs approval before implementation.
- [x] T19 trace-anchored real-session eval: `real_session_eval(limit=N)` now selects feedback by
      the sampled trace IDs instead of comparing independent newest trace and newest feedback
      windows. This fixes a coverage/confidence measurement distortion without changing public
      request parameters, output fields, formulas, confidence-gate constants, ranking, `orient`,
      migration, hooks, adapters, or schema/storage/index behavior. Validation passed with the
      focused regression test, full telemetry tests, Brain Harness eval tests,
      `cargo fmt --all --check`, and `cargo check -p engram-cli`.
- [x] T20 scoped real-session eval sampling: scoped reports now apply project, scenario, and arm
      filters before the trace limit, then fetch feedback for that scoped trace sample. This fixes
      eval starvation where newer out-of-scope traces could hide older in-scope traces. Public
      request parameters, output fields, formulas, ranking, `orient`, migration, lifecycle status,
      document-index behavior, hooks, adapters, schema/storage, and `list_feedback_scoped`
      semantics were not changed. Validation passed with the focused regression test, full
      telemetry tests, Brain Harness eval tests, `cargo fmt --all --check`,
      `cargo check -p engram-cli`, and `git diff --check`.
- [x] T21 installed-runtime validation for T19/T20: after installing binary hash
      `0192d24d945b7acb8bdfabe129c56d61a5abf0f7ce8223c854139677a93738ab`
      and restarting the daemon on port `8765`, PID `11922`, a controlled live MCP telemetry smoke
      validated both fixes together. Scoped report
      `real_session_eval(project=engram,
      scenario_id=t21_installed_runtime_eval_20260527_0192d24d,
      arm=memoryitem_orient, limit=2)` returned `trace_count=2`, `feedback_count=1`,
      `feedback_trace_count=1`, `feedback_coverage=0.5`, `task_success_count=1`, and
      `task_failure_count=0`, while newer out-of-scope traces and newer feedback on older in-scope
      traces were excluded. This is installed-runtime evidence only; it does not change ranking,
      `orient`, migration, lifecycle state, hooks, adapters, public MCP parameters, output fields,
      schema/storage, or `list_feedback_scoped`.
- [x] T22 Claude Code cross-harness smoke for T21: Claude Bridge still could not expose Engram MCP
      tools in the project harness and is treated as a bridge tool-exposure limitation. Native
      Claude Code `2.1.152` with `mcp__engram__telemetry` allowed reproduced the same read-only T21
      report through its own MCP connection: `trace_count=2`, `feedback_count=1`,
      `feedback_trace_count=1`, `feedback_coverage=0.5`, `task_success_count=1`, and
      `task_failure_count=0`, with newest in-scope trace IDs
      `019e69e4-6244-7123-a34e-d19e8c44341a` and
      `019e69e4-5582-79a1-8dc4-09411d58aca5`. Claude's explanation incorrectly inferred
      operation-level filtering; source inspection and the controlled fixture show the result is
      the newest two scoped traces under `limit=2`. This validates shared MCP report behavior for
      this read-only surface only, not hooks, adapters, ranking, migration, or broad product
      quality.
- [x] T26 obligation-noise suppression: agent-native obligation detection no longer treats a bare
      `schema` mention as a failed-tool recovery cue, and untracked root instruction files such as
      `AGENTS.md` are skipped before document-disposition candidates are created. Explicit
      failed-tool wording and ordinary durable document changes remain covered. Validation passed
      with `cargo test -p engram-index obligation::tests`,
      `cargo test -p engram-tests --test obligation_tests`, `cargo fmt --all --check`,
      `cargo check -p engram-cli`, and `git diff --check`.
- [x] T27 installed-runtime validation for T26: after installing binary hash
      `7d9256dc2ca9fcefaaa54bf620c15989fa20926c929d9e6beca27012b6afc9cf`
      and restarting the daemon on port `8765`, PID `50257`, live MCP obligation detection no
      longer returns `tool_failure_recovery` or `AGENTS.md` document-disposition candidates for the
      bare `schema` / `failure hypothesis` prompt that failed before install. Explicit failed-tool
      wording still returns `tool_failure_recovery`. This is installed-runtime evidence only; it
      does not change `orient`, ranking, migration, lifecycle status, hooks, adapters, public MCP
      parameters, telemetry formulas, schema, or storage.
- [x] T28 Claude Code cross-harness obligation smoke: Claude Bridge with `harness=personal`,
      `write=false`, and only `mcp__engram__obligations` allowed reproduced the T27 behavior.
      Claude Code returned only `source_reading` for the bare `schema` / `failure hypothesis` prompt,
      no `AGENTS.md` document-disposition candidate, and `tool_failure_recovery` for explicit
      failed-tool wording. Caveat: the requested validation calls were dry-run, but the Claude Code
      harness itself opened two prompt-derived obligations from the synthetic smoke prompt; Codex
      skipped them with explicit synthetic-smoke evidence. This validates the shared MCP obligations
      request shape only; it does not validate hooks, adapter/settings writes, ranking, migration,
      lifecycle status, public MCP changes, telemetry formulas, schema, storage, or `orient`.
- [x] T29 read-only completion gate audit: startup `orient` and direct searches returned the active
      current plan first for the current continuation prompt; `real_session_eval(project=engram,
      limit=50)` still passes numerically but is rolling-window sensitive and has
      `external_session_trace_count=0`; `obligations(action=doctor)` is clean; explicit
      `harness(action=doctor)` still reports `ready=false` for Claude Code, Codex, Gemini CLI, and
      Cursor. This audit records the current matrix; it does not authorize M6 work, lifecycle writes,
      hook/adapter writes, ranking changes, public MCP changes, telemetry formula changes,
      schema/storage changes, or `orient` payload expansion.
- [x] Brain Harness telemetry outcome dimensions: traces support free-form `scenario_id`/`arm`,
      feedback records task success, preference adherence, repeated context questions, and bad
      memory use, and the confidence gate requires behavioral outcome evidence.
- [x] Brain Harness research operating method documented: dogfood is one experimental instrument
      under explicit research questions, competing hypotheses, evidence levels, claim ledger, and
      decision gates.
- [x] Real `orient` and `search` operations can tag their automatic traces with free-form
      `scenario_id` and `arm`, so controlled eval comparisons use the same trace IDs agents
      receive during normal retrieval.
- [x] Generated agent harness adapters require the `telemetry` MCP tool and tell agents to keep
      `orient`/`search` trace IDs, then submit explicit outcome/gap feedback fields before final
      response when the result is assessable.
- [x] Brain Harness dogfood protocol documented with a read-only corpus preflight, labeled
      `scenario_id`/`arm` runs, feedback rubric, anti-overfit rules, and evidence-based next-step
      routing.
- [x] First labeled Brain Harness dogfood batch: run at least 4 scenarios across at least 2 arms,
      submit feedback for every trace, and route the next implementation step from
      `telemetry(action=real_session_eval)`.
- [x] Brain Harness claim ledger/RFC updated from the first matched dogfood batch: narrow durable
      preference-recall claim recorded, unsupported claims bounded, and
      `bounded_autonomous_followthrough_001` defined as the next evidence scenario.
- [x] Brain Harness bounded-autonomy protocol tightened after the first follow-through batch:
      implementation-bearing comparisons now require pre-selected non-self-referential work,
      isolated clean arm starts, and evaluator-recorded results.
- [x] Brain Harness BAF002 isolated follow-through scored: both arms passed a doc-only
      `ORIENT_CONTRACT.md` slice with no material `memoryitem_orient` advantage; next
      discriminating evidence should be code-bearing, not another doc-only task.
- [x] Brain Harness BAF003 code-bearing scenario pre-registered around scoped telemetry filtering
      for `scenario_id` and `arm`.
- [x] Brain Harness BAF003 run and scored: both arms passed the scoped telemetry-filtering code
      slice with no material `memoryitem_orient` advantage; the leaner implementation landed, but
      the result does not justify `orient` ranking, hot-path, migration, or legacy-simplification
      changes.
- [x] Brain Harness BAF004 pre-registered: next controlled code-bearing confidence scenario will
      make scoped `real_session_eval` reports self-describing with `applied_filters`, while testing
      whether `orient` preserves non-obvious commit-hygiene and safety-gate context.
- [x] Review-gated entity observation promotion through `memory(action=promote_observation)`,
      preserving the source observation as evidence instead of widening `orient`.
- [x] Low-friction current-plan capture through `memory(action=capture_current_plan)`: agents can
      write one compact active `decision` or `rule` with evidence and an automatic knowledge commit
      when the current method, plan, or next action should survive resume.
- [x] Review-gated migration and digest extraction flows; no automatic promotion from orphan, digest, or legacy data.
- [x] Agent harness layer: MCP `harness`, CLI `engram harness`, Claude Code adapter rendering, Codex skill rendering, Gemini CLI custom command/context rendering, Cursor Agent skill rendering, dry-run install by default.
- [x] Agent-native obligations layer: MCP `obligations`, CLI `engram obligations`, lifecycle detection for document disposition, source/design context reading, failed tool-call recovery, tests, handoffs, and commit preference checks.
- [x] Context-compaction lifecycle contract: harness policy and adapters tell agents to update handoffs and persist compact durable memory before expected context loss.
- [x] Lint/recalibration MVP: missing evidence, stale preference, duplicate entity candidate, orphan task/project scope, stale active session, superseded active item, vault marker/frontmatter, handoff missing next actions.
- [x] Archive metadata and archive-aware retrieval; normal orientation uses active memory only.
- [x] Derived graph traversal: MCP/CLI `graph around|path|subgraph|export`.
- [x] Multi-writer changes_since filters and relevance reasons for harness/model/surface/session-aware polling.
- [x] Rolling handoff: MCP/CLI `handoff get|update|compile`.
- [x] Session-following event vocabulary: prompts, plans, tool results, tests, preferences, rules, limitations, and handoff updates.
- [x] Session distillation dry-run candidate generation through `memory(action=distill_session)` and `engram memory distill-session`.
- [x] Lean `orient(response_shape="lean")` envelope validated in Codex and Claude Code smokes:
      trace/cursor/scope, Brain Loop top items, used memory candidate IDs, and obligation summary
      are available without the full raw packet payload. Native Claude Code CLI trace
      `019e68fe-6150-7ab3-9df7-8339e3766c76`, after installing binary hash
      `4f3bda71eb441d492ece4b1bb5983993be9cf47802fd10cdb3484f31f7e23f9c`, returned a compact
      inline lean packet for the current continuation prompt and included current-plan memory
      `019e68f9-31b1-7270-9095-4f0be5ffa94b` at position 2.
- [x] Mission-class `plan_work` prompts promote the latest current-plan memory into active
      decisions and used memory candidates, while `resume_session` remains the only intent with the
      Brain Loop pin and older-current-plan suppression.
- [x] Direct unified `search` continuation prompts prioritize scoped `current-plan` MemoryItems
      through query-gated ranking calibration, with migration gate prompts still preserving gate
      guidance above current-plan context. Native MCP smoke after installing binary hash
      `f5cb5816927b4e4a5b9cb92df560de47e201c2bccdcbfa05eeb25c9d35bcfb35` returned the active
      current-plan memory first in trace `019e68a5-ef05-7db0-8249-3722fcf78aea`; native Claude
      Code CLI then returned the same active current-plan memory first in trace
      `019e68ac-678e-7683-a241-08119fc6b03c`. A later native Claude Code CLI smoke against the
      currently installed binary returned current-plan memory `019e68f9-31b1-7270-9095-4f0be5ffa94b`
      first in direct `search` trace `019e68fe-6417-7590-8331-85ddf3dd4a86`.
- [x] Non-gated continuation wording in direct unified `search`: query classification no longer
      treats `non-gated` / `non gated` as an approval-gate request by substring alone. The
      deterministic fixture keeps the active current-plan item first for
      `move forward next non-gated Brain Harness implementation slice current plan`, while a
      mixed gate prompt with `should/proceed/migration apply` still preserves gate guidance above
      current-plan context. Live daemon smoke after installing binary hash
      `8859cacc921a243d5cd8dd3351f5f196c46d8074ecdc9933fa66e0ec490b1c7b`
      and restarting the daemon on port `8765`, PID `67660`, returned active current-plan memory
      `019e68c5-9b4f-7200-9dd0-3345b6163620` first in trace
      `019e68d4-05b7-79d3-8077-df6e2999482d`; a migration-apply gate query in trace
      `019e68d4-27b7-70e2-bdfe-5c879a97f0c8` kept migration/gate context above current-plan
      context.
- [x] Current-plan lifecycle predicate parity: `memory(action=capture_current_plan)` and `orient`
      current-plan post-prioritization now use the same operational meaning as direct search
      ranking: active `decision` or `rule` plus the `current-plan` tag. Non-guidance facts or
      limitations that carry `current-plan` remain normal active memory and are not automatically
      superseded by current-plan capture. The focused regression
      `capture_current_plan_does_not_supersede_non_guidance_current_plan_tags` covers the bug
      exposed during the `90c20f6` post-commit memory capture.
- [x] MCP `memory(action=list,tags=...)` tag filtering: the existing `tags` request field is now
      honored by the `list` action, requiring all requested tags before applying `limit`. This is
      an evidence-quality sampling fix only; it does not change `orient`, ranking, migration,
      schemas, or memory lifecycle status. Live daemon smoke after installing binary hash
      `4f3bda71eb441d492ece4b1bb5983993be9cf47802fd10cdb3484f31f7e23f9c` and restarting the
      daemon on port `8765`, PID `93788`, returned three active `current-plan` tagged items, all
      `decision` kind, so no non-guidance current-plan lint rule was added in this slice.
- [x] MCP `memory(action=list,scope_type=...)` explicit scope filtering: the list action now honors
      an explicit scope filter before applying `limit`, preventing wrong-project active guidance
      from contaminating evidence-quality sampling. The live failure was
      `memory(action=list, scope_type=project, project_name=engram, tags=[current-plan])`
      returning a `voice-layer` current-plan item; after installing binary hash
      `0d4581c1cffdd17af0d4d8f0911812a05a2c3ce3f9ff8766d455e043ed73a211` and restarting the
      daemon on port `8765`, PID `36805`, the same call returned only the Engram project current
      plan `019e6997-96d0-76a0-ac67-c7655df0958f`. This does not change `orient`, ranking,
      migration, schema, hooks, adapters, or memory lifecycle status. Native Claude Code `2.1.152`
      then reproduced the same scoped list behavior through its own Engram MCP connection after the
      T16 current-plan capture, returning only `019e69af-011f-7450-9f8c-1ff067f0f183` with scope
      `project / engram`.
- [x] Stale Memory OS guidance cleanup after the current-plan fix: the old mission-class
      PlanWork limitation and BAF007/BAF008 sealed implementation targets are superseded by
      active resolved/accepted outcome memory.
- [x] Full validation and live daemon smoke from installed binary.
- [x] Lint actionability follow-up: duplicate entity findings are bounded at the service and MCP
      boundary, and item-scoped missing-evidence/handoff warnings name the affected item.
      Live daemon smoke after installing commit `7d56006` with binary hash
      `cc8e30db22be3f106454c21c334441c113f168ca6290c4554168e708ebfecb49`
      confirmed MCP `lint(action=run, limit=80)` returns handoff warnings naming handoff titles
      and missing-evidence warnings naming item title/kind.
- [x] Telemetry-backed memory quality lint: read-only `lint` now reports active MemoryItems that
      recent agent feedback flagged as stale or wrong-scope. Findings are informational,
      deduplicated per active memory item, ignore non-active or dangling IDs, and intentionally
      have no safe automatic action. Live daemon smoke after installing binary hash
      `1d4d5134cc9d89e977e635d439b2008ddfaf459e91b4d00df0997fb39ab78934` and restarting the
      daemon on port `8765`, PID `54666`, confirmed `lint(action=run, limit=80)` includes
      `feedback_stale_active_memory` and `feedback_wrong_scope_active_memory` findings with
      `safe_action=none`.
- [x] T33 Claude Code lint-ordering parity smoke: Claude Bridge with `harness=personal`,
      `write=false`, and only `mcp__engram__lint` plus `mcp__engram__obligations` allowed
      reproduced the T32 live result through Claude Code's Engram MCP path. `lint(action=run,
      limit=10)` returned `feedback_stale_current_plan` for
      `019e5e0a-86b4-73e3-aa9b-ca350e83e915` first with `safe_action=none`; the synthetic
      design-context obligation created by the validation prompt was resolved from the already-read
      startup docs.
- [x] T34 governing-doc sync and rolling eval audit: `docs/BRAIN_HARNESS_ARCHITECTURE.md` and
      `docs/BRAIN_HARNESS_RESEARCH_METHOD.md` now record the T30/T31 documentation audit, T32/T33
      lint-ordering evidence, and the T34 live-state sample. The T34 sample keeps current-plan
      retrieval usable and obligations clean, but after scoring the T34 startup traces
      `real_session_eval(project=engram, limit=50)` still fails the conservative confidence gate
      because feedback spans only two intents. This is documentation/evidence alignment only; no
      source behavior, ranking, lifecycle, migration, hook, adapter, schema/storage, public MCP,
      telemetry formula, or `orient` payload change was introduced.
- [x] T35 pre-registered evidence-quality audit: fixed read-only `review_memory`,
      `verify_decision`, and `prepare_handoff` cases were committed before running traces. The
      M6 gate check passed, stale-plan memory review was noisy but usable, and lean
      `orient(intent=prepare_handoff)` failed because it omitted explicit M6/harness-write gates
      and returned stale repository-scoped current-plan guidance without a caveat. The rolling
      confidence gate passed numerically afterward (`feedback_trace_count=48`,
      `feedback_coverage=0.9599999785423279`, `bad_memory_used_count=0`,
      `task_failure_count=1` after startup feedback scoring), but this is not product proof and
      does not authorize `orient`, ranking, lifecycle, migration, hook, adapter, schema/storage,
      public MCP, or telemetry formula changes.
- [x] T38 prepare-handoff `orient` repair: after explicit user approval, `PrepareHandoff` now
      presents one latest applicable current-plan item across matching project/repository scopes,
      pins that current plan in Brain Loop, and keeps stale current-plan guidance out of lean
      handoff candidate IDs without lifecycle mutation or payload expansion. Focused service and
      MCP lean fixtures reproduce the T35 shape with M6 and harness-write gates.
- [x] T39 installed prepare-handoff gate validation: after installing binary
      `d9db0ee830ef261c582e31f0c327f8198d4b6d1f556f11820bcec27fc64dfe42`,
      `PrepareHandoff` treats the phrase `approval gate` as explicit gate intent, promotes active
      approval-gate MemoryItems over generic gate/calibration chatter, and live Codex/Claude Code
      traces now surface the latest current plan plus M6 and harness-write approval gates. This did
      not expand the payload, synthesize gates, mutate lifecycle state, authorize M6 work, or write
      harness adapters/hooks.
- [x] T40 partial completion audit: fixed checks were pre-registered in commit `0322566`, then
      run only against approved/read-only surfaces. Codex and native Claude Code
      `prepare_handoff` traces returned the same latest current-plan and M6/harness gate IDs;
      `plan_work` returned the latest current plan first with stale current-plan memory lower;
      the explicit M6 negative-control search returned gate/blocked context ahead of old
      approval-shaped records; all four harnesses still reported `ready=false`; lint kept stale
      current-plan feedback visible with `safe_action=none`; obligations were clean after audit
      cleanup. The mixed non-gated search `current plan next non-gated Brain Harness feedback
      confidence M6 gate` returned latest current-plan first but did not surface the active M6 gate
      in top memory results, so this is partial evidence, not completion or M6 authorization.
- [x] T41 mixed-query fixture validation: after T40 current-plan capture, live search recheck
      returned the latest current plan first and the active M6 gate in top memory results for the
      exact T40-04 mixed query. The deterministic fixture
      `test_memory_search_t40_mixed_query_surfaces_current_plan_and_m6_gate` now seeds
      live-shaped current-plan, stale repository-plan, non-gated calibration, and M6 gate records,
      and verifies current-plan-first plus M6-gate top-five behavior while preserving explicit
      M6 write/apply/deletion gate-first behavior. No production ranking code, lifecycle state,
      public MCP surface, `orient` payload, migration flow, schema/storage/index behavior, or
      harness adapter/hook behavior changed.
- [x] T42 Claude Code parity for T41 mixed-query behavior stopped before the Claude Code scoreable
      run. The pre-run Codex baseline returned latest current-plan memory first, but active M6 gate
      memory `019e7ce5-155d-7a10-85f5-00b9dcc69cd0` was absent from the top eight results and only
      appeared at rank 17 in a diagnostic `limit=20` search for the exact mixed query. The
      negative-control query still preserved gate/blocked context above current-plan guidance and
      did not imply M6 approval. This records a live-data retrieval gap; it does not authorize
      ranking, lifecycle, migration, `orient`, schema/storage/index, public MCP, or harness
      adapter/hook changes.
- [x] T43 prompt-specific mixed-query gate repair: direct `search` now keeps latest current-plan
      guidance first while surfacing already-ranked active M6 gate context for the exact mixed
      current-plan/M6-gate prompt class. The repair is search-only, preserves pure continuation and
      explicit M6 apply/gate controls, and did not change broad ranking weights, lifecycle state,
      migration state, `orient`, schema/storage/index, public MCP, or harness adapter/hook behavior.
- [x] T44 Claude Code parity for T43 direct-search repair: Claude Code reproduced the T43 installed
      direct-search behavior for the exact mixed current-plan/M6-gate prompt class, explicit M6
      negative control, and pure continuation control. This was read-only validation only; it did
      not authorize implementation, migration, lifecycle, `orient`, schema/storage/index, public
      MCP, ranking, or harness adapter/hook changes.
- [x] T45 M6 read-only scope proposal packet: prepared
      `docs/BRAIN_HARNESS_T45_M6_READ_ONLY_SCOPE_PROPOSAL_2026-05-31.md` from existing evidence
      only. It requests approval for exactly one bounded inventory-only
      `memory(action="migration_inventory", ...)` call and a Markdown report. It does not authorize
      review export, apply, deletion, lifecycle mutation, schema/storage/index changes, public MCP
      changes, ranking or `orient` changes, or harness adapter/hook changes.
- [x] T46 harness readiness re-audit: read-only `harness(action="doctor")` and
      `harness(action="status")` checks confirmed `ready=false` for the generic policy,
      Claude Code, Codex, Gemini CLI, and Cursor. Generic policy is missing, Claude Code is missing
      required `SessionStart` and `SessionEnd` settings registrations, and Codex/Gemini/Cursor
      still have required generated adapter drift. This is evidence only; adapter, settings, and
      hook writes remain approval-gated.
- [x] T47 harness repair approval packet: prepared
      `docs/BRAIN_HARNESS_T47_HARNESS_REPAIR_APPROVAL_PACKET_2026-05-31.md` from read-only
      `harness(action="install", write=false, ...)` dry-runs, source inspection, AI Council, and
      Claude Bridge critique. It requests approval for exactly five local harness repair
      `write=true` calls, each preceded by a matching dry-run. It does not authorize the writes,
      M6 work, user-owned adoption, `settings.json` edits, hook rewrites, schema/storage/index
      changes, public MCP changes, ranking or `orient` changes, or memory lifecycle mutation.
- [x] T48 stale current-plan lifecycle approval packet: prepared
      `docs/BRAIN_HARNESS_T48_STALE_CURRENT_PLAN_LIFECYCLE_APPROVAL_PACKET_2026-05-31.md` from
      read-only `orient`, `search`, `memory(action="get")`, scoped current-plan list, and
      `lint(action="run", write=false)` evidence. It requests approval for exactly one
      `memory(action="archive")` call on stale repository-scoped current-plan MemoryItem
      `019e5e0a-86b4-73e3-aa9b-ca350e83e915`, contingent on fresh matching read-only evidence. It
      does not authorize lifecycle writes by itself, other memory mutations, M6 work, harness
      writes, schema/storage/index changes, public MCP changes, ranking or `orient` changes.
- [x] T51 T48 archive packet drift report: documented that the T48 approval packet is no longer
      executable as written after T49/T50 current-plan supersession. Fresh read-only evidence shows
      T50 is now the active project current plan, the stale repository-scoped target remains active,
      and lint reports 139 stale-feedback records with `safe_action=none`. This did not create a
      refreshed approval packet or run lifecycle writes.
- [x] T52 stale current-plan resolution request: prepared
      `docs/BRAIN_HARNESS_T52_STALE_CURRENT_PLAN_RESOLUTION_REQUEST_2026-05-31.md` from fresh
      read-only get/list/lint/orient/search evidence, source inspection, AI Council, and Claude
      Bridge critique. It reframes the stale repository-scoped current-plan target
      `019e5e0a-86b4-73e3-aa9b-ca350e83e915` as a user decision between archive-only,
      replacement-then-archive, or scope-correction/merge. It does not authorize lifecycle writes,
      create a replacement, run M6, write harness adapters/settings/hooks, change
      schema/storage/index state, change public MCP behavior, change ranking, or expand `orient`.
- [x] T53 post-T52 Claude parity smoke: documented
      `docs/BRAIN_HARNESS_T53_T52_CLAUDE_PARITY_2026-05-31.md` from read-only Codex and Claude
      Bridge retrieval traces. Claude Code returned T52 current-plan memory
      `019e7d5d-c450-7171-9fdb-8d1a5e745b0b` first in lean `orient` and direct `search`, while
      stale repo current-plan target `019e5e0a-86b4-73e3-aa9b-ca350e83e915` remained visible only as
      pending-decision evidence. It did not run lifecycle, M6, harness, schema/storage/index, public
      MCP, ranking, or `orient` changes.
- [x] T54 rolling telemetry audit: documented
      `docs/BRAIN_HARNESS_T54_ROLLING_TELEMETRY_AUDIT_2026-05-31.md` from read-only telemetry and
      feedback queries after T53. The current `real_session_eval(project=engram, limit=50)` window
      passed numerically with `feedback_trace_count=31`, `feedback_coverage=0.6200000047683716`,
      `memory_judgment_coverage=1.0`, `bad_memory_used_count=0`, and
      `confidence_gate.passed=true`, but still showed one task failure, 25 stale-memory judgments,
      six missing-context reports, and partial external-session labeling. It is evidence-quality
      calibration only, not product completion or approval for M6, lifecycle, harness,
      schema/storage/index, public MCP, ranking, or `orient` changes.
- [x] T55 post-T54 Claude parity smoke: documented
      `docs/BRAIN_HARNESS_T55_T54_CLAUDE_PARITY_2026-05-31.md` from read-only Codex and Claude
      Bridge retrieval traces. Claude Code returned T54 current-plan memory first in lean
      `orient` and direct `search`; the first project-harness bridge attempt exposed only file
      tools and was unmeasured. This did not run lifecycle, M6, harness, schema/storage/index,
      public MCP, ranking, or `orient` changes.
- [x] T56 post-T55 feedback telemetry audit: documented
      `docs/BRAIN_HARNESS_T56_POST_T55_FEEDBACK_TELEMETRY_AUDIT_2026-05-31.md` from read-only
      telemetry after T55 scoring. Feedback coverage improved to `33/50` and external-session
      trace labeling improved to `23/50`, but one task failure remained and stale-memory judgments
      increased to `31`. It is evidence-quality calibration only, not completion or approval for
      M6, lifecycle, harness, schema/storage/index, public MCP, ranking, or `orient` changes.
- [x] T57 post-T56 Claude parity and search-visibility smoke: documented
      `docs/BRAIN_HARNESS_T57_T56_CLAUDE_PARITY_AND_SEARCH_VISIBILITY_2026-05-31.md` from
      read-only Codex and Claude Bridge retrieval traces. Claude Code returned T56 first in lean
      `orient` and exact continuation `search`, while a broader implementation-plan query kept
      T56 rank 2 behind historical calibration. This validates narrow continuity only and does not
      authorize broad ranking, lifecycle, M6, harness, schema/storage/index, public MCP, or
      `orient` changes.
- [x] T58 approved inventory-only M6 scoping run: after explicit user approval, Codex ran exactly
      one read-only `memory(action="migration_inventory", ...)` call with the T45 parameters and
      recorded `docs/BRAIN_HARNESS_T58_T45_M6_INVENTORY_REPORT_2026-05-31.md`. The run scanned
      115 sources, returned 11 candidates, was not truncated, and wrote no Memory OS records. It
      did not authorize review export, apply, deletion, lifecycle mutation, schema/storage/index,
      public MCP, ranking, `orient`, or harness changes.
- [x] T59 M6 review-export approval packet: prepared
      `docs/BRAIN_HARNESS_T59_M6_REVIEW_EXPORT_SCOPE_PROPOSAL_2026-05-31.md` from the T58
      inventory, prior AI Council recall, and Claude Bridge critique. It requests approval for
      exactly one `memory(action="migration_review_export", ...)` call with the T58
      `exclude_reviewed_path`, a path-existence preflight, and strict stop conditions. It does not
      run the export or authorize apply, candidate decisions, deletion, lifecycle mutation,
      schema/storage/index, public MCP, ranking, `orient`, or harness changes.
- [x] T60 T59 gate retrieval parity: documented
      `docs/BRAIN_HARNESS_T60_T59_GATE_RETRIEVAL_PARITY_2026-05-31.md` from predeclared Codex and
      Claude Code retrieval probes. Both harnesses surfaced T59 and preserved default-deny for
      `migration_review_export`, but continuation search remained noisy and the clean no-write
      condition failed because Claude Bridge `write=false` still triggered existing Claude Code
      session-end rolling handoff MemoryItem writes. This does not authorize review export, apply,
      deletion, lifecycle cleanup, schema/storage/index, public MCP, ranking, `orient`, or harness
      adapter/hook changes.
- [x] T61 continuation `should` gate false-positive fixed: direct unified `search` no longer
      treats `what should happen next` as approval-gate intent. The narrow fixture for the exact
      T60 continuation wording now promotes the active current-plan item first while preserving
      gate-first behavior for explicit `should we run migration_review_export` prompts. This is a
      prompt-class ranking repair only; it does not authorize review export, apply, lifecycle
      mutation, schema/storage/index changes, public MCP changes, `orient` expansion, or harness
      adapter/hook changes.
- [x] T62 installed-runtime validation for T61: after installing binary hash
      `25715d5c2334a423dfdf73d8fc3868037ffe9c1a180f8a3df9926c6727d1464f`
      and restarting the daemon on port `8765`, Codex and Claude Code both returned T61 first for
      the exact T60 continuation search and lean `orient`, while explicit
      `should we run migration_review_export` prompts kept migration gate evidence first. Claude
      Bridge `write=false` again wrote duplicate rolling handoffs, so no-write parity remains
      unproven and no handoff cleanup or hook change is authorized.
- [x] T63 scoped feedback drill-down fix: `telemetry(action="list_feedback", project/scenario/arm,
      limit=N)` now applies scope before the limit and fetches feedback for matching trace IDs,
      matching the scoped `real_session_eval` sampling model. Regression coverage proves newer
      out-of-scope telemetry no longer hides an in-scope feedback row under a small limit; installed
      binary `fd7287ef6186d77532c20486034f95729b89e00c043e6ef94aa870bc873846da`
      reproduced the behavior in a live MCP smoke. This is telemetry drill-down hygiene only.
- [x] T64 post-T63 continuity and T59 visibility audit: read-only Codex probes validated that T63
      current-plan memory surfaces first for lean `orient`, broad current-plan search, and exact
      continuation search. Explicit `migration_review_export` probes preserved default-deny gate
      context but did not surface the T59 packet itself in top memory results, while stale older
      migration/export records remained visible. The T59 document remains the source of truth; no
      new gate MemoryItem, ranking change, `orient` change, migration action, or lifecycle write was
      created.
- [x] T65 T59 document-index visibility approval packet: prepared
      `docs/BRAIN_HARNESS_T65_T59_DOCUMENT_INDEX_VISIBILITY_APPROVAL_PACKET_2026-06-01.md`
      from read-only search traces, source inspection, AI Council, and Claude Bridge critique. It
      requests approval for a bounded document-index visibility repair on exactly the T58, T59, and
      T64 evidence docs. It does not run indexing, create MemoryItems, run M6 review export/apply,
      mutate lifecycle state, change schema/storage/index behavior, change public MCP behavior,
      change ranking, expand `orient`, or write harness adapters/hooks.
- [x] T66 T65 exact-index preflight: documented
      `docs/BRAIN_HARNESS_T66_T65_EXACT_INDEX_PREFLIGHT_2026-06-01.md` from source and help-text
      inspection only. The existing MCP document handler can target a single file path, while
      directory paths remain out of scope because they call directory indexing and the default
      pipeline is recursive. No indexing, planning against target files, M6 action, lifecycle
      mutation, schema/storage/index behavior change, public MCP change, ranking change, `orient`
      change, or harness write was run.
- [x] T67 approved T65 exact-file document indexing: after explicit user approval, Codex indexed
      exactly the T58, T59, and T64 report files through MCP `docs(action="index", path=...)`,
      producing 11, 9, and 8 chunks respectively. Validation found T59 rank 1 for exact title and
      filename-stem document searches and top-five for relative-path search, while absolute-path
      semantic search remained weak. No M6 review export/apply, MemoryItem creation, lifecycle
      mutation, schema/storage/index behavior change, public MCP change, ranking change, `orient`
      change, or harness write was run.
- [x] T68 approved T59 review-export-only run: after explicit user approval, Codex ran exactly one
      `memory(action="migration_review_export", ...)` call with the T59 parameters. It wrote
      `/Users/yuval.meiri/.engram/reviews/2026-05-31-t58-m6-review-export`, returned 12 candidates
      rather than the expected 11 because one `skip` candidate appeared, and stopped on the T59
      count-drift guardrail. No review apply, candidate decision, lifecycle mutation,
      schema/storage/index behavior change, public MCP change, ranking change, `orient` change, or
      harness write was run.
- [x] T69 T68 count-drift decision packet: prepared
      `docs/BRAIN_HARNESS_T69_T68_COUNT_DRIFT_DECISION_PACKET_2026-06-01.md` after AI Council and
      Claude Bridge critique. The packet says the ambiguous `i approve` reply is not scoped
      authorization and asks for an exact reply naming T69 plus `index.md` and
      `candidates/0012-skip-plan.md` before reading those two written export files. No M6
      inspection, review apply, candidate decision, lifecycle mutation, schema/storage/index
      behavior change, public MCP change, ranking change, `orient` change, or harness write was
      run.
- [x] T70 T69 document visibility audit and index packet: prepared
      `docs/BRAIN_HARNESS_T70_T69_DOCUMENT_VISIBILITY_AUDIT_AND_INDEX_PACKET_2026-06-01.md` after
      read-only document-search evidence showed T69 and T68 absent from top document results and
      stale pre-T68 T59 chunks still visible. Source inspection found exact-file reindexing reuses
      the existing source identity and replaces chunks for that source. The packet asks for approval
      to index exactly T59, T68, and T69; it does not run indexing, inspect M6 review-export files,
      create MemoryItems, mutate lifecycle state, change schema/storage/index behavior, change
      public MCP behavior, change ranking, expand `orient`, or write harness adapters/hooks.
- [x] T70 approved exact-file document indexing: executed
      `docs/BRAIN_HARNESS_T70_EXACT_FILE_INDEX_RESULT_2026-06-02.md` after exact user approval.
      Codex indexed exactly the T59, T68, and T69 report files, producing 9, 8, and 9 chunks
      respectively with no warnings. T68 and T69 became visible for exact-title probes, while T59
      remained recoverable by filename-stem and scoped approval phrasing but still did not appear in
      the tested top five for its exact title after reindexing. No M6 candidate inspection,
      status/prioritize/apply/rerun, candidate decision, deletion, lifecycle mutation, ranking,
      `orient`, public MCP/schema/storage/index behavior change, document-index behavior change, or
      harness write was run.
- [x] T71 harness readiness re-audit: read-only `harness(action="doctor")` /
      `harness(action="status")` checks reconfirmed `ready=false` for generic, Claude Code, Codex,
      Gemini CLI, and Cursor. Generic policy is still missing; Claude Code still lacks required
      `SessionStart` and `SessionEnd` settings registrations; Codex, Gemini CLI, and Cursor retain
      generated-adapter drift. No install, settings edit, hook registration, M6 action, lifecycle
      mutation, schema/storage/index behavior change, public MCP change, ranking change, `orient`
      change, or harness write was run.
- [x] T72 rolling telemetry audit: documented
      `docs/BRAIN_HARNESS_T72_ROLLING_TELEMETRY_AUDIT_2026-06-01.md` from read-only telemetry
      after T71 feedback/current-plan capture. The sampled project report passed numerically with
      `feedback_trace_count=32`, `feedback_coverage=0.6399999856948853`,
      `memory_judgment_coverage=1.0`, `task_failure_count=0`, and `bad_memory_used_count=0`, but
      it still showed stale/wrong-scope judgments, only three sampled intents, and
      external-session labels on `11/50` traces. It is evidence-quality calibration only, not
      completion or approval for M6, lifecycle, harness, schema/storage/index, public MCP, ranking,
      document-index, or `orient` changes.
- [x] T73 stale repository current-plan audit: documented
      `docs/BRAIN_HARNESS_T73_STALE_CURRENT_PLAN_AUDIT_2026-06-01.md` from read-only `orient`,
      direct search, scoped memory list/get, and lint evidence after T72. T72 current-plan memory
      remains first for the tested continuation prompt, but stale repository-scoped current-plan
      target `019e5e0a-86b4-73e3-aa9b-ca350e83e915` still appears near the top, remains the only
      active repository-scoped current-plan item for `/Users/yuval.meiri/projects/engram`, and
      now has 228 recent stale-feedback records with `safe_action=none`. This refreshes T52
      evidence only; it does not approve archive, replacement, scope correction, M6, lifecycle,
      harness, schema/storage/index, public MCP, ranking, document-index, or `orient` changes.
- [x] T74 post-T73 Claude parity: documented
      `docs/BRAIN_HARNESS_T74_POST_T73_CLAUDE_PARITY_2026-06-01.md` from read-only Codex and
      Claude Bridge retrieval checks. Codex traces `019e8277-3c03-7f62-8bfe-cc6a79f48212` and
      `019e8277-484c-7df1-a977-1e303a41d333`, plus Claude traces
      `019e8278-671d-7d02-8a04-fe0a17d31de6` and
      `019e8278-6bd4-73f3-8973-8ea0d3ec24bc`, all returned T73 current-plan memory first for the
      tested continuation/search path. The stale repository-scoped target remained fifth in lean
      `orient` and second in direct `search` in both harnesses. Claude's synthetic
      design/source-reading obligations were resolved or skipped by Codex after the run. This is
      cross-harness evidence only, not approval for lifecycle, M6, harness, schema/storage/index,
      public MCP, ranking, document-index, or `orient` changes.
- [x] T75 post-T74 telemetry audit: documented
      `docs/BRAIN_HARNESS_T75_POST_T74_TELEMETRY_AUDIT_2026-06-01.md` from read-only telemetry
      after T74 feedback/current-plan capture. The sampled project report had zero task failures,
      zero bad-memory-used records, zero wrong-scope judgments, and improved external-session
      labeling (`36/50` traces), but the confidence gate failed because only one intent had
      feedback. Feedback coverage was `27/50` (`0.5400000214576721`) and stale-memory judgments
      remained. This is evidence-quality calibration only, not completion or approval for M6,
      lifecycle, harness, schema/storage/index, public MCP, ranking, document-index, or `orient`
      changes.
- [x] T77 organic non-plan scoring audit: documented
      `docs/BRAIN_HARNESS_T77_ORGANIC_NON_PLAN_SCORING_PREREG_2026-06-01.md` from two
      pre-registered intent-filtered trace windows. The audit found 30 older-unseen retrieval-only
      assessable traces across `follow_user_preference` and `verify_decision`, but zero
      task-outcome assessable traces, so it submitted no feedback and did not run a final
      `real_session_eval`. This is negative evidence about historical organic outcome scoring, not
      completion or approval for M6, lifecycle, harness, schema/storage/index, public MCP, ranking,
      document-index, or `orient` changes.
- [x] T78 controlled observable task audit: documented
      `docs/BRAIN_HARNESS_T78_CONTROLLED_OBSERVABLE_TASK_AUDIT_2026-06-01.md` from four
      pre-registered genuine current-work tasks. The audit produced four
      `ASSESSABLE_TASK_OUTCOME` non-`plan_work` traces, submitted feedback for all four, and ran
      one diagnostic `real_session_eval` with `feedback_coverage=0.6000000238418579`,
      `task_failure_count=0`, `bad_memory_used_count=0`, and `confidence_gate.passed=true`. This
      validates prospective controlled-task outcome scoring only, not completion or approval for
      M6, lifecycle, harness, schema/storage/index, public MCP, ranking, document-index, or
      `orient` changes.
- [x] T79 Claude Bridge observable-task audit: documented
      `docs/BRAIN_HARNESS_T79_CLAUDE_BRIDGE_OBSERVABLE_TASK_AUDIT_2026-06-01.md` from a committed
      pre-registration and one read-only Claude Bridge project-harness run. Claude Bridge reported
      both allowed Engram tools, `mcp__engram__orient` and `mcp__engram__search`, as unavailable.
      All three tasks were classified `HARNESS_INCONCLUSIVE`, no Engram trace IDs were produced,
      and no feedback or diagnostic `real_session_eval` was submitted. This is a harness
      tool-exposure caveat, not completion or approval for M6, lifecycle, harness,
      schema/storage/index, public MCP, ranking, document-index, or `orient` changes.
- [x] T80 outcome-link decision packet: documented
      `docs/BRAIN_HARNESS_T80_OUTCOME_LINK_DECISION_PACKET_2026-06-01.md` after read-only source
      inspection, AI Council broadcast, and Claude Bridge critique. The packet keeps
      real-session telemetry unchanged, classifies `AgentFeedback` task outcome fields as weak
      self-report unless paired with transcript-visible or independent evidence, and defines a
      future controlled-outcome link contract. This is not completion or approval for schema,
      storage, public MCP, harness, migration, lifecycle, ranking, document-index, or `orient`
      changes.
- [x] T81 feedback outcome-pointer proxy audit: documented
      `docs/BRAIN_HARNESS_T81_FEEDBACK_OUTCOME_POINTER_PROXY_AUDIT_2026-06-01.md` from the latest
      20 project feedback rows. Every sampled row had a note and positive task outcome fields, but
      zero had non-empty `missing_context`, only the four T78 rows had explicit
      `ASSESSABLE_TASK_OUTCOME` labels, and no row had a structured transcript/commit/test/user
      review or controlled-outcome artifact pointer. This is not completion or approval for
      schema, storage, public MCP, harness, migration, lifecycle, ranking, document-index, or
      `orient` changes.
- [x] T82 controlled outcome artifact pilot: documented
      `docs/BRAIN_HARNESS_T82_CONTROLLED_OUTCOME_ARTIFACT_PILOT_2026-06-01.md` as a doc-only
      immutable snapshot. The pilot links four T78 assessable traces and one weak T79 startup
      feedback row to trace IDs, feedback IDs, durable evidence refs, evidence strength, T80
      classes, confounds, and pending reviewer agreement. It validates a possible artifact shape
      but not schema, storage, public MCP, harness, migration, lifecycle, ranking, document-index,
      or `orient` changes.
- [x] T83 T82 second-reader review: pre-registered and documented
      `docs/BRAIN_HARNESS_T83_T82_SECOND_READER_REVIEW_2026-06-01.md`. A read-only Claude Bridge
      review agreed with all five T82 classes, kept T82-5 as `SELF_REPORTED_OUTCOME`, and said the
      artifact shape is reviewable enough for a future approval packet. It also flagged T82-4's
      staging-discipline subclaim as relying on an authored doc summary rather than raw preserved
      git-status/staging evidence. This is not completion or approval for schema, storage, public
      MCP, harness, migration, lifecycle, ranking, document-index, or `orient` changes.
- [x] T84 raw terminal evidence rule: documented
      `docs/BRAIN_HARNESS_T84_RAW_TERMINAL_EVIDENCE_RULE_2026-06-01.md` and the research method.
      T84 explicitly chose not to run a standalone `git status` output pilot because it would not
      retroactively strengthen T82-4. Instead, future controlled artifact rows whose outcome
      depends on terminal state must either preserve scoped raw output with interpretation and
      limitations, or keep that subclaim indirect. This is not completion or approval for schema,
      storage, public MCP, harness, migration, lifecycle, ranking, document-index, or `orient`
      changes.
- [x] T85 Claude Bridge project-harness tool-exposure recheck: documented
      `docs/BRAIN_HARNESS_T85_CLAUDE_BRIDGE_PROJECT_TOOL_EXPOSURE_RECHECK_2026-06-01.md` from one
      pre-registered `write=false`, no-Bash Claude Bridge project-harness run allowing only
      `mcp__engram__orient` and `mcp__engram__search`. Both tools returned
      `No such tool available`, so the classification is `TOOLS_UNAVAILABLE`; no Engram traces
      existed and no telemetry feedback was submitted. This closes that exact recheck line until
      bridge or harness configuration changes and does not approve schema, storage, public MCP,
      harness, migration, lifecycle, ranking, document-index, or `orient` changes.
- [x] T86 rolling handoff freshness audit: documented
      `docs/BRAIN_HARNESS_T86_ROLLING_HANDOFF_FRESHNESS_AUDIT_2026-06-01.md` and refreshed the
      rolling handoff from low-information Claude Code handoff
      `019e82ec-b571-7830-b8f2-661da91585e7` to current handoff
      `019e82f3-53bc-7a83-9e39-cfdb29b06c44`. The new handoff records T85, current-plan memory
      `019e82ee-dd81-7ba0-8f97-1933965f6d8e`, and exact T69/T70 approval phrases. This is
      continuity repair only; it does not approve schema, storage, public MCP, harness, migration,
      lifecycle, ranking, document-index, or `orient` changes.
- [x] T87 resume source precedence audit: documented
      `docs/BRAIN_HARNESS_T87_RESUME_SOURCE_PRECEDENCE_AUDIT_2026-06-01.md` and clarified the
      rolling handoff from `019e82f3-53bc-7a83-9e39-cfdb29b06c44` to
      `019e82f8-cada-7c31-b073-18ac41986b1e`. T87 found the local
      `/Users/yuval.meiri/notes/engram/handoff.md` is a stale 2026-04-17 open-source launch
      handoff, while current Engram `orient`, direct search, and `handoff(get)` surface T86/T69/T70
      context. This is source-precedence continuity repair only; it does not approve schema,
      storage, public MCP, harness, migration, lifecycle, ranking, document-index, or `orient`
      changes.
- [x] T88 stale handoff lifecycle approval packet: documented
      `docs/BRAIN_HARNESS_T88_STALE_HANDOFF_LIFECYCLE_APPROVAL_PACKET_2026-06-01.md` for exactly one
      superseded rolling handoff target. This is an approval packet only; no archive or lifecycle
      write was run.
- [x] T89 changes-since MCP cursor ergonomics: documented
      `docs/BRAIN_HARNESS_T89_CHANGES_SINCE_CURSOR_ERGONOMICS_2026-06-01.md`; the MCP error now
      tells agents to pass `memory_cursor.timestamp` and optionally `memory_cursor.commit_id`
      without changing cursor semantics or public request fields.
- [x] T90 changes-since CLI cursor ergonomics: documented
      `docs/BRAIN_HARNESS_T90_CLI_CHANGES_SINCE_CURSOR_ERGONOMICS_2026-06-01.md`; CLI help and
      invalid timestamp errors now point at `memory_cursor.timestamp` while keeping the same
      `--timestamp` / `--commit-id` behavior.
- [x] T91 rolling handoff T90 freshness repair: documented
      `docs/BRAIN_HARNESS_T91_ROLLING_HANDOFF_T90_FRESHNESS_REPAIR_2026-06-01.md` and refreshed
      the active rolling handoff from T87/T86 context to T90 context. This is continuity
      maintenance only; no T69/T70/T88, M6, lifecycle archive, ranking, `orient`, public MCP,
      schema/storage/index, document-index, or harness behavior changed.
- [x] T92 lint superseded-active visibility: documented
      `docs/BRAIN_HARNESS_T92_LINT_SUPERSEDED_VISIBILITY_2026-06-01.md` and adjusted private lint
      ordering so safe-action superseded-active findings surface before generic stale-feedback
      noise while stale current-plan feedback stays first. No lifecycle write, M6, document-index,
      retrieval ranking, `orient`, public MCP, schema/storage/index, or harness behavior changed.
- [x] T93 lint installed-runtime validation: documented
      `docs/BRAIN_HARNESS_T93_LINT_INSTALLED_RUNTIME_VALIDATION_2026-06-01.md`; after source-level
      T92 validation but stale live MCP lint output, Codex installed the current Engram binary and
      restarted the daemon. Live MCP `lint(action="run", limit=20)` then returned stale
      current-plan feedback first, wrong-scope feedback next, and safe-action
      `superseded_item_still_active` rows before generic stale-feedback rows. No lifecycle write,
      `apply_safe`, M6, document-index, retrieval ranking, `orient`, public MCP,
      schema/storage/index, or harness behavior changed.
- [x] T121 approved T69 count-drift inspection: documented
      `docs/BRAIN_HARNESS_T121_T69_COUNT_DRIFT_INSPECTION_RESULT_2026-06-02.md` after exact user
      approval to inspect only `index.md` and `candidates/0012-skip-plan.md` from the written T68
      review-export snapshot. The drift is explained by one generated `skip` candidate from a
      `session_event` plan source; the review-actionable queue remains 9 review plus 2 quarantine
      candidates. No candidate decision, review apply, deletion, lifecycle mutation,
      schema/storage/index behavior change, public MCP change, ranking change, `orient` change,
      document indexing, or harness write was run.
- [x] T122 M6 candidate-review approval packet: documented
      `docs/BRAIN_HARNESS_T122_M6_CANDIDATE_REVIEW_APPROVAL_PACKET_2026-06-02.md` after AI Council
      and Claude Bridge critique. The packet asks for an exact T123 approval to read only candidate
      files 0001-0004 from the written T68 review-export snapshot. It does not read candidate
      files, run migration status/prioritize/apply/rerun, index documents, mutate lifecycle state,
      make candidate decisions, change schema/storage/index or public MCP behavior, change
      ranking, expand `orient`, or write harness adapters/hooks.
- [x] T123 approved M6 candidate 0001-0004 inspection: documented
      `docs/BRAIN_HARNESS_T123_M6_CANDIDATE_0001_0004_INSPECTION_RESULT_2026-06-02.md` after exact
      user approval. Codex read only candidate files 0001-0004 from the written T68 snapshot. All
      four are project-observation `review` candidates; candidate 0004 carries a later-context risk
      because its 2026-05-24 Claude Code readiness wording conflicts with later readiness audits.
      No quarantine files, status/prioritize/apply/rerun, candidate decisions, active memory write,
      deletion, lifecycle mutation, document indexing, ranking, `orient`, public
      MCP/schema/storage/index behavior change, document-index behavior change, or harness write
      was run.
- [x] T124 approved M6 candidate 0005-0009 inspection: documented
      `docs/BRAIN_HARNESS_T124_M6_CANDIDATE_0005_0009_INSPECTION_RESULT_2026-06-02.md` after exact
      user approval. Codex read only candidate files 0005-0009 from the written T68 snapshot. All
      five are project-observation `review` candidates; candidates 0005, 0008, and 0009 carry
      later-context or supersession risks, and candidate 0006 is harness-write-adjacent. No
      quarantine files, status/prioritize/apply/rerun, candidate decisions, active memory write,
      deletion, lifecycle mutation, document indexing, ranking, `orient`, public
      MCP/schema/storage/index behavior change, document-index behavior change, or harness write
      was run.
- [x] T125 approved M6 quarantine candidate inspection: documented
      `docs/BRAIN_HARNESS_T169_T125_QUARANTINE_INSPECTION_REPORT_2026-06-03.md` after exact user
      approval. Codex read only quarantine candidate files 0010-0011 from the written T68 snapshot.
      Both candidates are entity-scoped `quarantine` candidates tied to `review-all-system`, with
      scope-confirmation gaps preserved. No review files, status/prioritize/apply/rerun, candidate
      decisions, active memory write, deletion, lifecycle mutation, document indexing, ranking,
      `orient`, public MCP/schema/storage/index behavior change, document-index behavior change, or
      harness write was run.
- [x] T126 harness readiness recheck: documented
      `docs/BRAIN_HARNESS_T126_HARNESS_READINESS_RECHECK_2026-06-02.md` from read-only
      `harness(action="doctor")` and per-harness `harness(action="status")` evidence. Generic,
      Claude Code, Codex, Gemini CLI, and Cursor still report `ready=false`. This refreshes the
      evidence date only; T47 remains the harness-write approval packet. No install, adapter write,
      settings edit, hook registration, user-owned file adoption, M6 action, lifecycle mutation,
      document indexing, ranking, `orient`, public MCP/schema/storage/index behavior change,
      document-index behavior change, or harness write was run.
- [x] T127 post-T126 startup continuity audit: documented
      `docs/BRAIN_HARNESS_T127_POST_T126_STARTUP_CONTINUITY_AUDIT_2026-06-02.md` from read-only
      `orient`, direct search, `memory(list)`, `handoff(get)`, `docs(search)`, `lint(run)`,
      `telemetry(real_session_eval)`, and `obligations(doctor)` evidence. Lean startup retrieval
      and continuation search recovered the T126 current plan first, and scoped current-plan
      listing returned exactly one active Engram project current-plan item. Exact T125 phrasing
      remains noisy because older active handoffs can rank above the current plan, the stale
      repository-scoped current-plan item still appears in lower-ranked broad searches, the T126
      report is not top-five visible through document search, and the rolling telemetry confidence
      gate fails at 38% feedback coverage. No T125 quarantine file, status/prioritize/apply/rerun,
      candidate decision, candidate active-memory write, deletion, lifecycle mutation, document
      indexing, ranking, `orient`, public MCP/schema/storage/index behavior change, document-index
      behavior change, or harness write was run.
- [x] T128 Claude Code post-T127 parity check: documented
      `docs/BRAIN_HARNESS_T128_CLAUDE_POST_T127_PARITY_CHECK_2026-06-02.md` after a read-only
      Claude Bridge run. Claude Code recovered T127 current-plan memory first in lean `orient` and
      broad continuation search, and scoped current-plan listing returned exactly one active
      Engram project current-plan item. Exact T125 search remained noisy: the current plan ranked
      fifth behind handoffs. Handoff continuity failed because Claude Code session-end automation
      wrote stub handoffs despite bridge `write=false`, superseding the rich T127 handoff. Codex
      skipped two prompt-generated Claude obligations as inapplicable. T128 does not authorize hook
      changes, harness repair, lifecycle mutation, document indexing, candidate inspection,
      ranking, `orient`, public MCP/schema/storage/index behavior changes, document-index behavior
      changes, or M6 status/prioritize/apply/rerun.
- [ ] Migration completion run: explicit read-only inventory scope approval, inventory,
      review export, prioritize/dedupe, human review, dry-run apply, explicit write-apply approval,
      knowledge commit, vault compile, lint run.

## Current Completion Matrix

Matrix reconciliation note, 2026-06-04: T214 supersedes older cross-harness readiness wording
below. Current read-only `harness(action="doctor")` evidence reports `ready=true` for generic,
Claude Code, Codex, Gemini CLI, and Cursor. Generated local adapter readiness is validated, while
behavioral caveats remain: lifecycle compliance is soft, Claude Code settings are split and retain
extra legacy permissions, `/hooks` effective-hook visibility did not produce a usable report in
T179, prompt-bearing native Claude behavior is unproved, external-session joinability depends on
real caller/host labels, and stale active handoffs remain until a future non-dry-run handoff update
or exact lifecycle cleanup.

Matrix freshness note, 2026-05-31: T43 and T44 extend the current-plan / next-step retrieval
evidence beyond the older continuation and explicit migration-apply prompt classes. The exact
mixed current-plan/M6 direct-search prompt class is now validated in Codex and Claude Code, while
T45 records only a pending approval packet for one inventory-only M6 scoping run. T46 reconfirmed
from read-only harness checks that generic, Claude Code, Codex, Gemini CLI, and Cursor readiness
all remain false. T47 records only a pending approval packet for exact local harness repair writes.
T48 records only a pending approval packet for one stale current-plan archive. M6 inventory, review
export, apply, deletion, lifecycle mutation, schema/storage/index changes, public MCP changes,
ranking or `orient` changes, and harness adapter/hook/settings changes remain unapproved.
T49 records a read-only pending-approval retrieval audit: direct `search` surfaces the active M6,
harness-write, and T48 lifecycle gates for an explicit approval-gates prompt, while lean `orient`
only exposes the full queue indirectly through the latest current-plan memory.
T50 records a read-only Claude Code parity smoke after T49 current-plan capture: Claude Code
surfaced the latest T49 plan first, harness-write gate second, and M6 gate third in lean `orient`,
while pending-approval direct `search` returned M6 first and harness-write second.
T51 records a read-only drift report for the T48 archive packet: T48 remains a useful stale-memory
proposal, but it is no longer executable as written because the active project current-plan memory
is T50 rather than the T47 item named by T48. No refreshed packet or lifecycle write was created.
T52 records a refreshed read-only resolution request rather than an archive-only approval packet:
fresh evidence shows T51 is now the active project current plan, the stale repository-scoped target
is still the only active repository-scoped current-plan item, and lint reports 142 stale-feedback
records with `safe_action=none`. No lifecycle write, replacement, M6, harness, schema/storage/index,
public MCP, ranking, or `orient` change was created.
T53 records read-only Claude Code parity for the post-T52 continuation prompt: T52 surfaced first in
Claude lean `orient` and direct `search`, while the stale repository-scoped target remained visible
as pending-decision evidence only. No gated work was run.
T54 records a read-only rolling telemetry audit after T53 feedback: the sampled project report
still passes numerically and has no bad-memory-used evidence, but coverage, stale-memory judgments,
one task failure, missing-context reports, and sparse external-session labels keep the evidence row
partial rather than complete. No gated work was run.
T55 records read-only Claude Code parity for the post-T54 continuation prompt: the first
project-harness bridge attempt exposed only file tools and was unmeasured, while the personal-harness
rerun with only Engram read tools returned T54 first in lean `orient` and direct `search`. Synthetic
obligations were resolved or skipped with evidence. No gated work was run.
T56 records a read-only telemetry audit after T55 feedback scoring: feedback coverage and
external-session labeling improved, but stale-memory judgments increased and one task failure
remains. The numerical confidence gate is still weak evidence only. No gated work was run.
T57 records read-only Claude Code parity for the post-T56 continuation prompt and a broader
implementation-plan search visibility check: T56 surfaced first in Claude lean `orient` and exact
continuation `search`, and stayed rank 2 behind older calibration history for the broader query.
No gated work was run.
T58 records the explicitly approved inventory-only M6 scoping run: 115 sources scanned, 11
candidates returned, no truncation, no writes, and no migration decisions. Review export remains a
separate approval gate.
T59 records a pending review-export approval packet with exact `migration_review_export`
parameters, the T58 `exclude_reviewed_path`, a path-existence preflight, and stop conditions for
count drift or unexpected write/apply behavior. No export has been run.
T60 records cross-harness retrieval parity for the T59 gate with caveats: Codex and Claude Code
both surfaced T59 and default-deny evidence, but broad continuation search still ranked historical
items above T59 and Claude Bridge `write=false` still wrote rolling handoff MemoryItems through
existing Claude Code session-end behavior.
T61 records a narrow direct-search fix for the T60 `what should happen next` false-positive: bare
`should` no longer triggers gate mode, while modal action prompts such as `should we run
migration_review_export` still preserve gate-first behavior. The Claude Bridge critique retry also
repeated the known rolling-handoff write confound. No gated work was run.
T62 installs the T61 commit into the live runtime and validates the same boundary in Codex and
Claude Code: exact continuation search and lean `orient` return T61 first, while explicit
`should we run migration_review_export` prompts keep migration gate evidence first. Claude Bridge
`write=false` again wrote duplicate rolling handoffs, so clean no-write parity remains unproven.
No gated work was run.
T63 fixes scoped feedback drill-down sampling so `list_feedback` with project/scenario/arm filters
matches the scoped trace-first model used by `real_session_eval`; newer out-of-scope telemetry no
longer hides in-scope feedback under small limits. No gated work was run.
T64 validates that T63 current-plan guidance now appears first in Codex continuation probes, but
records a T59 packet visibility gap: explicit `migration_review_export` prompts preserve
default-deny gate context yet do not surface the T59 approval packet itself in top memory results.
The T59 document remains authoritative and review export is still unapproved. No gated work was run.
T65 records a pending approval packet for the safest useful follow-up to that gap: a bounded
document-index visibility repair for exactly the T58, T59, and T64 evidence docs. The packet asks
for approval only. No indexing, M6 action, MemoryItem creation, lifecycle mutation, schema/storage
or document-index behavior change, public MCP change, ranking change, `orient` change, or harness
write has been run.
T66 confirms from source that T65 can be executed as three exact file-path MCP
`docs(action="index", path=...)` calls if approved. It also records that directory paths are too
broad for T65 because directory indexing uses recursive default behavior. T66 did not run indexing
or planning against target files.
T67 executes the approved T65 scope as exactly three MCP file-path index calls for the T58, T59,
and T64 reports. T59 now appears rank 1 for exact title and filename-stem document searches, and
explicit `migration_review_export` prompts return T59/T64 document evidence while preserving active
M6 gate context. Absolute-path semantic search remains weak, so repo docs remain the source of
truth before M6 decisions. T67 did not run review export/apply, create a T59 MemoryItem, mutate
lifecycle state, change schema/storage/index behavior, change public MCP behavior, change ranking,
expand `orient`, or write harness adapters/hooks.
T68 executes the approved T59 review-export-only call and stops on its count guardrail. The review
workspace was written to `/Users/yuval.meiri/.engram/reviews/2026-05-31-t58-m6-review-export`,
but the fresh inventory returned 12 candidates rather than the expected 11: 9 review, 2
quarantine, and 1 skip. The tool also reported no Memory OS records were written. No review apply,
candidate decision, lifecycle mutation, schema/storage/index behavior change, public MCP change,
ranking change, `orient` change, or harness write followed.
T69 converts the post-T68 ambiguity into a decision packet. AI Council and Claude Bridge agreed
that `i approve` is not sufficient authorization for a count-drift action. The packet asks for a
reply naming T69 and exactly two files from the written export snapshot, `index.md` and
`candidates/0012-skip-plan.md`. T121 executes that exact read-only approval and explains the drift:
the twelfth candidate is a generated `skip` candidate from a `session_event` plan source, so the
review-actionable queue remains 9 review plus 2 quarantine candidates. M6 apply, candidate
decisions, deletion, lifecycle mutation, rerun/prioritize, document indexing, and behavior changes
remain gated.
T70 records a follow-up document visibility gap: T68 and T69 are not yet visible through top
document-search results, and T59 still has stale pre-export chunks indexed. Source inspection shows
exact-file `docs(action="index")` reuses the existing source identity and replaces chunks for that
source. T70 asks for explicit approval to index exactly T59, T68, and T69; it does not grant T69
inspection approval or any M6 migration action. After exact user approval on 2026-06-02, Codex
executed exactly those three file-path index calls. T68 and T69 now surface for exact-title probes,
while T59 remains visible by filename-stem and scoped approval phrasing but not by the tested exact
title query.
T122 prepares the next M6 candidate-review gate after T121. AI Council and Claude Bridge supported
a docs-only packet, with Claude cautioning against meta-planning inflation and all-11 batching. The
packet therefore recommends T123 as a small read-only inspection of candidate files 0001-0004 from
the written T68 snapshot only, leaving remaining review candidates, quarantine candidates,
status/prioritize, apply, T70 indexing, lifecycle mutation, ranking, `orient`, public MCP,
schema/storage/index, document-index behavior, and harness writes behind separate exact gates.
T123 executes that first-batch read-only inspection. Candidates 0001-0004 are all
project-observation `review` candidates from May 24 dogfood/Claude Code validation work. No
candidate decisions were made. Candidate 0004's Claude Code `ready=true` wording must be treated as
time-bound or stale because later readiness audits report `ready=false`.
T124 executes the second review-candidate read-only inspection. Candidates 0005-0009 are all
project-observation `review` candidates. Candidate 0005 has the same historical `ready=true` risk
and an obligation-list bug later narrowed by scope-fix work; candidates 0008 and 0009 contain older
next-step guidance likely narrowed by later current-plan/retrieval work; candidate 0006 is
harness-write-adjacent and must not authorize harness writes.
T126 rechecks harness readiness without writes after T124. Generic, Claude Code, Codex, Gemini CLI,
and Cursor all still report `ready=false`: generic policy is missing, Claude Code lacks required
`SessionStart` and `SessionEnd` settings registrations, and Codex/Gemini/Cursor generated adapters
remain drifted. T126 does not execute the T47 harness repair packet or authorize any adapter,
settings, hook, lifecycle, M6, ranking, `orient`, public MCP, schema/storage/index, or
document-index behavior change.
T127 audits startup continuity after T126 without behavior changes. Lean `orient` and direct
continuation search recover T126 current-plan memory
`019e877e-dc11-7b90-b5ec-7bca7720a9f4` first, scoped current-plan listing returns exactly one
active Engram project current-plan item, and `handoff(get)` returns the T126 handoff. Exact T125
phrasing is still noisy because older active handoffs can outrank the current plan, broad searches
still surface stale repository-scoped current-plan memory
`019e5e0a-86b4-73e3-aa9b-ca350e83e915` lower down, the T126 report is not top-five visible
through document search, lint still reports stale/wrong-scope feedback with no safe lifecycle
action, and rolling telemetry fails the confidence gate at 38% feedback coverage.
T128 replicates the post-T127 startup retrieval in Claude Code through Claude Bridge. Claude Code
returns the active T127 current plan first in lean `orient` and broad continuation search, but exact
T125 search remains noisy and ranks the current plan fifth behind handoffs. More importantly,
Claude Code session-end automation writes stub handoffs despite bridge `write=false`, superseding
the rich T127 handoff and leaving current-plan memory as the only reliable carrier of T125/T47 gate
context until Codex restores the handoff. This is a cross-harness continuity failure, not a
retrieval pass.
T71 reconfirms from read-only harness status checks that generic, Claude Code, Codex, Gemini CLI,
and Cursor still report `ready=false`. This updates the harness evidence date only; the T47 exact
harness-write approval packet remains pending and unexecuted.
T72 records a read-only rolling telemetry audit after T71 feedback/current-plan capture: the sampled
project report now has zero task failures and zero bad-memory-used records while still showing
partial coverage, stale/wrong-scope judgments, narrow intent diversity, and weak external-session
joinability. This updates evidence quality only; no approval gate changed.
T73 refreshes the stale repository-scoped current-plan evidence after T72: the current project plan
still ranks first for the tested continuation prompt, while target
`019e5e0a-86b4-73e3-aa9b-ca350e83e915` remains the sole active repository-scoped current-plan item
and lint now reports 228 stale-feedback records with `safe_action=none`. This keeps T52 as a user
decision request rather than an executable lifecycle packet.
T74 replicates the post-T73 current-plan shape through Claude Code: T73 current-plan memory ranks
first in Codex and Claude Code lean `orient` and direct `search`, while the stale repository-scoped
target remains lower-ranked noise. Claude Bridge again showed synthetic obligation noise for
design/source cues, which Codex resolved or skipped after the run.
T75 records the post-T74 telemetry state: zero task failures and zero bad-memory-used records
remain, and external-session labeling improved to `36/50`, but the confidence gate fails because
only `plan_work` has feedback in the sampled window. Evidence-loop completion therefore remains
unproven.

| Area | Status | Current evidence | Remaining risk or gate |
| --- | --- | --- | --- |
| Memory OS substrate and MCP/CLI surfaces | Implemented | Checklist above; MCP/CLI surfaces listed below | None currently blocking the hot path. |
| `orient` hot path | Implemented and validated | Lean response shape, Brain Loop projection, current-plan continuity, prepare-handoff current-plan/gate fixtures, open-obligation summary, prompt-specific ranking tests, installed Codex trace `019e7ce5-4d19-7060-aa12-ab0f6d9b5695`, native Claude Code trace `019e7ce5-b4e4-7830-94a4-48f87ebf56b2`, T49 trace `019e7d43-c1d2-78b0-b815-a398f924a765` showing pending approvals are recoverable indirectly through the latest current-plan memory, and T50 Claude trace `019e7d48-6e97-7513-96af-f49d5a61bfc5` surfacing T49 current plan plus both active approval gates in lean top items | No hard byte/token budget yet; keep payload expansion gated. `prepare_handoff` is still compact orientation, not a generated handoff or approval-audit tool; explicit gates must exist as active MemoryItems. T49 shows explicit pending-approval audit prompts are stronger through direct `search` than lean `orient`, while T50 shows the post-T49 continuation prompt is now healthy in Claude Code too. Do not treat `orient` as a broad approval-audit surface without a separately approved prompt-class slice. |
| Current-plan / next-step retrieval | Validated for continuation prompts and narrow explicit migration-apply prompts | Commit `0b4e35b`; commit `94930ad`; `orient_mission_prompt_diagnostic_distinguishes_intent_from_ranking`; direct `search` fixtures `test_memory_search_prioritizes_current_plan_for_next_step_query`, `test_memory_search_prefers_project_current_plan_over_repository_plan`, `test_memory_search_keeps_gate_guidance_above_current_plan`, `test_memory_search_treats_non_gated_next_slice_as_current_plan`, `test_memory_search_t60_what_should_happen_next_promotes_current_plan`, and `test_memory_search_promotes_live_like_migration_gate_over_calibration_noise`; native Codex MCP trace `019e68a5-ef05-7db0-8249-3722fcf78aea` and native Claude Code CLI trace `019e68ac-678e-7683-a241-08119fc6b03c` both returned active current-plan memory first after installed binary `f5cb5816927b4e4a5b9cb92df560de47e201c2bccdcbfa05eeb25c9d35bcfb35`; live installed trace `019e68d4-05b7-79d3-8077-df6e2999482d` returned current-plan memory first for the `non-gated` continuation prompt after installed binary `8859cacc921a243d5cd8dd3351f5f196c46d8074ecdc9933fa66e0ec490b1c7b`; native Claude Code CLI trace `019e68fe-6417-7590-8331-85ddf3dd4a86` returned current-plan memory `019e68f9-31b1-7270-9095-4f0be5ffa94b` first after installed binary `4f3bda71eb441d492ece4b1bb5983993be9cf47802fd10cdb3484f31f7e23f9c`; active current-plan memory is intentionally looked up from Engram at task start because each slice supersedes the previous plan; T11 exact `review_memory` feedback-contract recheck trace `019e694c-57fd-7702-9824-ccb7932a92f6` returned the active telemetry-feedback rule first; T12 fixture coverage now keeps current-plan first for explicit `current plan next step ... M6 gate` context; T13 native MCP trace `019e6969-a674-7631-8ffa-b532b8638262` confirmed the exact T12 prompt after installed binary `62272400960eaaeb2fd7aa44aa13bf6f93abdbc81b5d11bc9106b0bcc82df29b`; T14 native MCP traces `019e698d-b766-7e71-a4da-a8c593f1b191` and `019e698d-b791-7d93-a0d6-542219e3eb6c` returned migration gate memory `019dd35d-1a48-7103-b0e2-390225f8b418` first for explicit migration-apply prompts after installed binary `fea91cc46549c138a425389394af9c4cdd9d8727eb39137f8afc179a976968eb`, while regression trace `019e698d-b7ae-7a13-b2c5-d58a9898deab` kept current-plan memory first for the T12 prompt; T62 installed binary `25715d5c2334a423dfdf73d8fc3868037ffe9c1a180f8a3df9926c6727d1464f` and Codex trace `019e7f49-4837-7a91-ae45-218f0b440113` returned T61 current-plan memory first for the exact T60 continuation search; Claude trace `019e7f49-ecbf-7c43-990d-14e929ef89f1` reproduced the same order; T64 Codex traces `019e7f67-041c-7552-8e34-54a156a86644`, `019e7f67-2d0d-7922-8de1-f598545c2e2d`, and `019e7f67-be5d-7d52-ab29-556c256cc502` returned T63 current-plan memory first for lean `orient`, broad current-plan search, and exact continuation search | Broad risk/implementation-plan searches can still surface stale historical memory below current guidance, including old current-plan and migration-completion records. T61/T62 fix and validate one `what should happen next` prompt class only; they are not proof of broad ranking quality or authorization for M6 work. T64 exposed a separate exact T59 packet visibility gap for explicit review-export prompts; use repo docs as authority before M6 decisions. |
| Cross-harness behavior | Partially validated | Codex and Claude Code lean-orient smokes; BAF008 real Claude Code treatment with clean controls; native Claude Code CLI direct-search smoke trace `019e68ac-678e-7683-a241-08119fc6b03c` returned current-plan memory `019e689c-b188-70e2-acfc-2d00f956bd24` first; native Claude Code CLI traces `019e68fe-6150-7ab3-9df7-8339e3766c76` and `019e68fe-6417-7590-8331-85ddf3dd4a86` confirmed lean `orient` stayed compact inline and direct `search` returned the latest current plan first after the latest installed binary; T15 Claude Code MCP traces `019e6993-d4da-70a1-b5eb-9185eeb23339`, `019e6993-d891-7ff3-93ef-4bd8ad14d9c7`, and `019e6994-8ec9-7343-9198-9298867b9ceb` validated the T14 explicit-gate/current-plan boundary in Claude Code; T16 native Claude Code memory-list smoke returned only the Engram project-scoped current-plan item for explicit scoped list filtering; T17 read-only explicit `harness doctor` re-audit showed all four supported harnesses `ready=false`; T28 Claude Bridge with `harness=personal` and only `mcp__engram__obligations` allowed reproduced the T27 obligation request shape; T29 `harness(action=doctor)` still reports `ready=false` for Claude Code, Codex, Gemini CLI, and Cursor; T33 Claude Bridge with `harness=personal`, `write=false`, and only `mcp__engram__lint` plus `mcp__engram__obligations` allowed reproduced the T32 lint ordering, with `feedback_stale_current_plan` first for `019e5e0a-86b4-73e3-aa9b-ca350e83e915` and `safe_action=none`; T39 native Claude Code trace `019e7ce5-b4e4-7830-94a4-48f87ebf56b2` matched Codex on prepare-handoff gate IDs after installed binary `d9db0ee830ef261c582e31f0c327f8198d4b6d1f556f11820bcec27fc64dfe42`; T46 read-only harness doctor/status checks returned `ready=false` for generic, Claude Code, Codex, Gemini CLI, and Cursor; T47 records an approval packet for exact dry-run-derived harness repair writes without executing them; T50 Claude trace `019e7d48-6e97-7513-96af-f49d5a61bfc5` surfaced T49 current plan, harness-write gate, and M6 gate in lean orient, and trace `019e7d48-905b-75c2-9d5b-e9cb657024c9` returned M6 then harness-write for direct pending-approval search; T53 Claude trace `019e7d60-64af-76d3-948f-5dd6068aa3d8` surfaced T52 current plan first in lean orient and trace `019e7d60-67e9-71d0-a421-f3364d4a5131` returned T52 first in direct search; T55 Claude trace `019e7d6d-460f-7ae1-bae0-1a662ace3e5d` surfaced T54 current plan first in lean orient and trace `019e7d6d-4648-71e2-9cd7-c702a5b9cd48` returned T54 first in direct search; T57 Claude trace `019e7d76-62e1-7b73-901e-5f839bcec551` surfaced T56 current plan first in lean orient, trace `019e7d76-67ee-7a72-9bb1-e41a5489d7fe` returned T56 first in exact continuation search, and trace `019e7d76-6d71-7d50-ab3d-1fcd32453cbd` kept T56 rank 2 behind historical calibration for the broader implementation-plan query | Broad harness reliability still depends on hook/config status and future real sessions. Generic policy is missing, Claude Code still lacks required `SessionStart` and `SessionEnd` settings registrations, and Codex, Gemini CLI, and Cursor still have required generated adapter drift. Claude Bridge project harness exposed only file-read tools in T55, while the personal harness exposed Engram MCP tools; the T28/T33/T50/T53/T55/T57 smokes showed synthetic prompts can create startup obligations. The T57 broader query shows old calibration history can outrank current plan while T56 remains top-three; this caveat does not justify broad ranking churn by itself. The T47 packet is not write approval; adapter, settings, or hook writes remain gated until the user explicitly approves its exact scope. |
| Evidence and feedback loop | Partially validated | Trace IDs, scenario/arm telemetry, feedback attribution warnings, real-session eval report, BAF007/BAF008 accepted outcome memory; installed binary `5b989d898ff033505c584c27d483ea9b3b433e679cc5bbf16befb59c48d1325c` live smoke fixed `memory_judgment_trace_coverage` from the impossible `1.78` to `0.94`, and search trace `019e6911-2f5b-7e02-a6d4-1c8b3b24b17e` recorded memory result IDs in both returned ID fields; the pre-registered `live_feedback_coverage_2026_05_27` batch recorded feedback for all ten traces and raised project-level `feedback_coverage` to `23/44` (`0.5227272510528564`); T04 follow-up trace `019e6924-3c0b-7031-a54a-3cdee7bf2647` returned the new reviewed software-design preference MemoryItem `019e6924-256b-7093-b1c5-286ec4d02461` first for the exact query; T06 follow-up trace `019e6931-d088-7493-a0d7-7795485ac944` returned the new reviewed lean-`orient` contract rule MemoryItem `019e6931-bd2d-7281-b9f6-952eaa2a20e4` first for the exact query; T07 follow-up trace `019e692b-827d-7c11-93ee-94e30d6198b6` returned the new reviewed telemetry-feedback rule MemoryItem `019e692b-635e-7d80-9f2f-8796abc95234` first for the exact query; after T11 startup/search traces were scored, `real_session_eval(project=engram, limit=50)` reported `feedback_trace_count=22`, `trace_count=44`, `feedback_coverage=0.5`, `memory_judgment_coverage=0.9545`, and `bad_memory_used_count=0`; T12 before trace `019e6954-4bf3-7432-9122-057cb9ab5b9b` was scored as a current-plan ranking miss caused by contextual `M6 gate` wording; after T12 trace scoring, the same report returned `feedback_trace_count=27`, `trace_count=44`, `feedback_coverage=0.6136363744735718`, `memory_judgment_coverage=0.9629629850387573`, and `bad_memory_used_count=0`; after T14 scoring, the same report returned `feedback_trace_count=35`, `trace_count=44`, `feedback_coverage=0.7954545617103577`, `memory_judgment_coverage=0.9714285731315613`, and `bad_memory_used_count=0`; T18 before-feedback re-audit showed `confidence_gate.passed=false` because feedback covered only two intents; after scoring T18 retrieval traces, the current report returned `feedback_trace_count=32`, `feedback_coverage=0.7272727489471436`, `memory_judgment_coverage=1.0`, `bad_memory_used_count=0`, and `confidence_gate.passed=true`; T19 changed real-session eval to anchor feedback selection to sampled trace IDs and added regression coverage for independent-window distortion; T20 changed scoped real-session eval to apply project/scenario/arm filters before trace limits; T24 external-session audit showed the current project eval still passes numerically (`trace_count=50`, `feedback_trace_count=38`, `confidence_gate.passed=true`) but joinability is sparse (`external_session_trace_count=5/50`, latest 12 traces had null labels) while source/tests already cover caller-supplied pass-through and feedback inheritance; T29 read-only audit returned `trace_count=50`, `feedback_trace_count=37`, `feedback_coverage=0.7400000095367432`, `memory_judgment_coverage=1.0`, `bad_memory_used_count=0`, `confidence_gate.passed=true`, and `external_session_trace_count=0`; T35 fixed-case audit returned a numerical gate pass (`feedback_trace_count=42`, `feedback_coverage=0.8399999737739563`, `distinct_intent_count=5`, `bad_memory_used_count=0`) while also recording one task failure for lean `prepare_handoff` orientation; T54 rolling audit returned `trace_count=50`, `feedback_trace_count=31`, `feedback_coverage=0.6200000047683716`, `memory_judgment_coverage=1.0`, `bad_memory_used_count=0`, `task_failure_count=1`, `stale_memory_count=25`, and `external_session_trace_count=13`; T56 post-feedback audit returned `trace_count=50`, `feedback_trace_count=33`, `feedback_coverage=0.6600000262260437`, `memory_judgment_coverage=1.0`, `bad_memory_used_count=0`, `task_failure_count=1`, `stale_memory_count=31`, `missing_context_count=5`, and `external_session_trace_count=23`; T63 aligns scoped `list_feedback` drill-down with scoped `real_session_eval` by filtering traces before limiting feedback, with installed live smoke feedback `019e7f63-654d-72b1-bf10-44ce210fa206` proving newer out-of-scope telemetry no longer hides the newest in-scope row under `limit=1`; T72 rolling audit returned `trace_count=50`, `feedback_trace_count=32`, `feedback_coverage=0.6399999856948853`, `memory_judgment_coverage=1.0`, `bad_memory_used_count=0`, `task_failure_count=0`, `stale_memory_count=28`, `wrong_scope_memory_count=1`, and `external_session_trace_count=11` | Agent feedback remains weak evidence unless checked against transcript, tests, or user review; the numerical confidence gate still requires user approval for migration decisions and should not override fixed-case failures. T10/T11/T12/T14/T18/T35/T54/T56/T72 showed old migration/export, migration-completion, calibration, implementation-history, reviewed-batch, stale repository-scoped current-plan memories, missing handoff gate context, stale-memory feedback, task-failure residue or rolling-window replacement, wrong-scope feedback, and sparse labels can surface as stale/noisy evidence, and the T04/T06/T07/T19/T20/T63 repairs only cover their target guidance, eval-window correctness, and scoped-feedback drill-down behavior rather than all legacy observations, doc-only contracts, or live daemon behavior. `external_session_id` is still mostly a caller/harness adoption and host-availability gap; T29/T35/T54/T56/T72 show sampled traces can have sparse or missing labels, and Engram should not auto-fill them from unrelated transport metadata without an approved host-session contract. |
| Memory quality / lifecycle | Partially validated | Review-gated promotion, archive-aware retrieval, stale-target cleanup, document lifecycle dogfood, Codex adapter follow-through, bounded/actionable lint reports, telemetry-backed stale/wrong-scope active-memory lint, specialized `feedback_stale_current_plan` lint for stale current-plan guidance, generic `feedback_stale_active_memory` coverage for stale old migration/export approval-shaped records, current-plan lifecycle predicate parity for active `decision`/`rule` guidance, and MCP tag/scope-filtered memory listing for evidence-quality sampling; T11 `lint(action=run, limit=80)` reported stale migration-completion memory `019dd3fe-ec94-7122-af04-1f35b839387f` as `feedback_stale_active_memory` after four stale-feedback records; T16 live smoke showed explicit `memory(action=list, scope_type=project, project_name=engram, tags=[current-plan])` returns only the Engram project current-plan item after binary `0d4581c1cffdd17af0d4d8f0911812a05a2c3ce3f9ff8766d455e043ed73a211`; T32 lint ordering makes feedback-stale current-plan and wrong-scope feedback findings appear before duplicate-entity, unresolved-obligation, and archive-safe lifecycle noise under normal limits, with live installed `lint(action=run, limit=10)` returning `feedback_stale_current_plan` for `019e5e0a-86b4-73e3-aa9b-ca350e83e915` first after binary `62db1e301ef7913ad685caa39d96ce0c479fc160fff3e8002df66401f619fce9`; T33 reproduced that same first finding through Claude Code's MCP path; T48 records a pending approval packet for exactly one archive action on stale repository-scoped current-plan MemoryItem `019e5e0a-86b4-73e3-aa9b-ca350e83e915`, after read-only lint reported 129 recent stale-feedback records and `safe_action=none`; T51 records that this T48 packet drifted after T49/T50, with T50 now active and fresh lint reporting 139 stale-feedback records with `safe_action=none`; T52 records a refreshed resolution request after fresh lint reported 142 stale-feedback records, but does not include an executable archive payload because the target is also the only active repository-scoped current-plan item; T73 refreshes the same target after T72 and finds it still active, still sole repository-scoped current-plan guidance, still near the top for tested continuation/risk searches, and now carrying 228 recent stale-feedback records with `safe_action=none` | Duplicate entity, handoff, missing-evidence, unresolved-obligation, archive-safe, and feedback-flagged active-memory lint findings still require human/agent review; no automatic lifecycle action is safe without approval. The T48 packet is not lifecycle approval, T51 is not a refreshed packet, T52 is a user decision request rather than approval for archive/replacement/scope-correction, and T73 does not change that decision boundary. Old migration/export approval-shaped records and old migration-completion records are not automatically classified or invalidated; feedback only marks them for review unless a current user-approved M6 scope exists. |
| Migration from legacy layers | Review export produced a stopped review workspace; apply still gated | Review-gated migration/digest flows exist; one accidental broad read-only inventory returned `3934` sources and `3641` candidates with no writes; the live feedback batch found no current M6 authorization and rejected older migration/export approvals as stale for this gate; T14 explicit migration-apply prompts now retrieve the paused migration review gate first; T58 completed an explicitly approved inventory-only scope with 115 sources scanned, 11 candidates returned, no truncation, and no writes; T59 prepared the exact review-export approval packet later executed as T68; T64 explicit review-export probes preserved default-deny gate context but did not surface the T59 packet itself in top memory results; T65 prepared a pending bounded document-index visibility approval packet for T58/T59/T64 docs; T66 source-only preflight confirmed exact-file MCP indexing is executable if approved; T67 executed the approved exact-file indexing and improved T59 document-search visibility for title, filename-stem, and explicit review-export probes; T68 executed the approved exact review-export call but stopped because the export returned 12 candidates, including one `skip`, instead of the expected 11; T121 completed the exact T69 read-only inspection and explained the drift as one generated `skip` candidate from a `session_event` plan source, leaving 9 review plus 2 quarantine candidates as the review-actionable queue; T123 inspected candidate files 0001-0004; T124 inspected candidate files 0005-0009; T169 inspected quarantine candidate files 0010-0011; T209 validated the generated 12-file snapshot and read-only status path, with all 12 files still undecided and `ready_to_apply=false`; T210 defined the next candidate-disposition gate; T211/T212 indexed the latest M6 gate reports | Do not run review apply, prioritize, candidate decisions, deletion, lifecycle mutation, schema/storage/index behavior change, ranking, public MCP changes, `orient` expansion, or harness writes without separate explicit approval. T123/T124/T169 are read-only inspection evidence only and do not approve candidate decisions or M6 execution. T210 requires explicit human-provided dispositions for 0001-0011 plus explicit 0012 handling, or an intentional 0001-0012 all-snapshot scope. T70/T211/T212 indexing was document visibility only and does not replace M6 candidate review/apply gates. T59 exact-title document search remains noisy after reindexing, so read repo docs before M6 decisions. Do not write-apply/delete/simplify without reviewed candidates, dry-run report, rollback plan, and explicit approval. |

T35/T38 matrix note: after the T35 fixed-case audit and subsequent startup-trace feedback scoring,
`real_session_eval(project=engram, limit=50)` returned `feedback_trace_count=48`,
`feedback_coverage=0.9599999785423279`, `distinct_intent_count=5`,
`bad_memory_used_count=0`, `task_failure_count=1`, and `confidence_gate.passed=true`. T38 repairs
that fixed-case `prepare_handoff` failure with deterministic service and MCP lean fixtures, but the
aggregate gate pass still is not product completion or migration approval.

T39 matrix note: installed-runtime validation found that the T38 binary fixed stale current-plan
suppression but live data/ranking still omitted gate context. The repair was phrase-local to
`approval gate` and captured two existing rules as active MemoryItems:
`019e7cde-b517-77d0-aaac-c8638811d4e8` for harness writes and
`019e7ce5-155d-7a10-85f5-00b9dcc69cd0` for M6. After current-plan capture, Codex trace
`019e7ceb-fda5-79b1-a997-725a9914840e` returned new current-plan memory
`019e7ceb-d8bb-73f0-960c-85b667b872de` first with both gates still present. This is
validation/capture work, not M6 or harness-write approval.

T40 matrix note: the partial completion audit in
`docs/BRAIN_HARNESS_T40_PARTIAL_COMPLETION_AUDIT_2026-05-31.md` produced four successful
scoreable retrieval/safety checks and one partial mixed-query check. Scenario eval for
`t40_partial_completion_audit_20260531` recorded five scored traces, four task successes, one task
failure for T40-04, and `bad_memory_used_count=0`; the scenario confidence gate failed only because
the fixed batch is below minimum trace/feedback thresholds. Project rolling eval after T40 feedback
still passed numerically with `feedback_coverage=0.70`, `memory_judgment_coverage=1.0`, and
`bad_memory_used_count=0`, but this remains weak evidence. The matrix delta is: current-plan and
handoff continuity are stronger across Codex and native Claude Code; stale current-plan lint
visibility remains healthy; M6 and harness readiness remain explicit approval gates; the mixed
non-gated search caveat is a retrieval-quality issue, not approval for migration, lifecycle,
ranking, hook/adapter, schema/storage, public MCP, or `orient` payload work.

T75 matrix note: after T74 feedback and current-plan capture,
`real_session_eval(project=engram, limit=50)` returned `feedback_trace_count=27`,
`feedback_coverage=0.5400000214576721`, `memory_judgment_coverage=1.0`,
`task_failure_count=0`, `bad_memory_used_count=0`, `wrong_scope_memory_count=0`, and
`external_session_trace_count=36`, but `confidence_gate.passed=false` because only one intent had
feedback. Treat this as useful evidence-loop telemetry, not completion or approval for migration,
lifecycle writes, hook/adapter writes, schema/storage changes, public MCP changes, `orient`
expansion, document indexing, or broad ranking changes.

T76 matrix note: the pre-registered organic non-plan feedback audit stopped before trace scoring
because the telemetry inspection surface could not list traces by intent. Source inspection showed
that `TelemetryRequest.intent`, persisted `intent_key`, and `idx_trace_intent` already existed, but
`list_traces` forwarded only project, scenario, and arm filters. The narrow implementation slice
wires the existing intent field through `telemetry(action="list_traces")`, canonicalizes aliases
through `BrainHarnessIntent::parse`, and adds focused MCP coverage. This is instrumentation hygiene
only. Post-commit live validation installed the T76 build into `/Users/yuval.meiri/.local/bin/engram`,
restarted the global daemon on port 8765, and confirmed `list_traces` returns only the requested
`follow_user_preference` or `verify_decision` traces. The validation opened trace bodies, so those
viewed traces must not be reused as blind organic scoring evidence. T76 does not submit new
non-plan feedback, prove confidence-gate readiness, authorize migration, lifecycle writes, harness
writes, ranking changes, schema/storage/index changes, document-index actions, `orient` expansion,
or new public MCP request parameters.

T77 matrix note: after T76, the pre-registered fixed-window audit ran
`list_traces(project=engram, intent=follow_user_preference, limit=20)` and
`list_traces(project=engram, intent=verify_decision, limit=20)` exactly once each. It found 14 and
16 older-unseen retrieval-only assessable traces respectively, but zero
`ASSESSABLE_TASK_OUTCOME` traces for either intent, so it submitted no scoring feedback and did not
run a final `real_session_eval`. The evidence-loop gap is now clearer: existing historical organic
non-plan trace bodies are insufficient for honest outcome scoring without richer outcome links or
controlled non-synthetic tasks. This does not authorize migration, lifecycle writes, harness
writes, ranking changes, schema/storage/index changes, public MCP changes, document indexing, or
`orient` expansion.

T78 matrix note: the controlled observable-task audit in
`docs/BRAIN_HARNESS_T78_CONTROLLED_OBSERVABLE_TASK_AUDIT_2026-06-01.md` used the T77 result to
test the next non-gated evidence path. Four genuine current-work tasks were pre-registered and
committed before execution, then run exactly as specified through existing `orient` and `search`
surfaces. All four became `ASSESSABLE_TASK_OUTCOME` traces and received feedback. The single
post-feedback `real_session_eval(project=engram, limit=50)` returned `feedback_trace_count=30`,
`feedback_coverage=0.6000000238418579`, `outcome_trace_count=30`, `task_failure_count=0`,
`bad_memory_used_count=0`, `wrong_scope_memory_count=0`, and `confidence_gate.passed=true`.
Because the tasks were prospectively selected for transcript-visible outcomes, this is evidence
that controlled non-synthetic tasks can produce honest non-plan feedback today; it is not evidence
that historical organic traces are broadly outcome-assessable and does not authorize migration,
lifecycle writes, harness writes, ranking changes, schema/storage/index changes, public MCP
changes, document indexing, or `orient` expansion.

T79 matrix note: the Claude Bridge replication attempt in
`docs/BRAIN_HARNESS_T79_CLAUDE_BRIDGE_OBSERVABLE_TASK_AUDIT_2026-06-01.md` was pre-registered in
commit `bdce09e` and then run once with `harness="project"`, `write=false`, no Bash, and only
`mcp__engram__orient` plus `mcp__engram__search` allowed. Claude Bridge reported both tools as
unavailable. The run produced zero Engram trace IDs, zero `ASSESSABLE_TASK_OUTCOME` tasks, no
feedback, and no final confidence report. This leaves T78's Codex controlled-task evidence intact
but keeps cross-harness observable outcome validation partial until a harness with exposed Engram
tools repeats the pattern.

T80 matrix note: the outcome-link decision packet in
`docs/BRAIN_HARNESS_T80_OUTCOME_LINK_DECISION_PACKET_2026-06-01.md` answers the T77/T78 evidence
gap without changing source behavior. Source inspection found that `AgentFeedback` already has
task outcome fields but no judgment source or evidence pointer, while controlled eval outcomes in
`brain_harness_eval.rs` reject using-agent judgment. T80 therefore classifies ordinary
real-session outcomes as weak self-report unless transcript-visible or independently judged
evidence exists, and defers any schema/API/storage work until a read-only proxy audit or pilot
justifies it.

T81 matrix note: the feedback outcome-pointer proxy audit in
`docs/BRAIN_HARNESS_T81_FEEDBACK_OUTCOME_POINTER_PROXY_AUDIT_2026-06-01.md` sampled the latest 20
project feedback rows. The sample has good note hygiene and universal positive outcome fields, but
no non-empty `missing_context`, no structured durable outcome artifact pointers, and only four
explicit outcome-assessability labels, all from the controlled T78 batch. This is evidence against
immediate `AgentFeedback`/`TelemetryRequest` schema work and supports either a larger read-only
audit or a controlled document-artifact pilot as the next non-gated step.

T82 matrix note: the controlled outcome artifact pilot in
`docs/BRAIN_HARNESS_T82_CONTROLLED_OUTCOME_ARTIFACT_PILOT_2026-06-01.md` tests that second path as
a doc-only snapshot. The five-row artifact uses trace ID, feedback ID, durable evidence refs,
evidence strength, T80 class, confounds, and reviewer agreement to preserve four T78
transcript-visible outcomes and refuse over-linking one positive T79 startup self-report. This
validates a reviewable artifact shape, but independent reviewer agreement remains pending and no
schema/API/storage/public MCP/harness/ranking/lifecycle/migration/document-index/`orient` change is
approved.

T83 matrix note: the second-reader review in
`docs/BRAIN_HARNESS_T83_T82_SECOND_READER_REVIEW_2026-06-01.md` was pre-registered, then run once
through Claude Bridge with `harness="isolated"`, `write=false`, no Bash, and no tool allowlist.
Claude agreed with all five T82 classes and explicitly preserved T82-5 as
`SELF_REPORTED_OUTCOME`. The review strengthens the artifact shape but adds an evidence-quality
requirement: future controlled rows that depend on git status, staged diff, test output, or other
terminal state should preserve raw durable output or keep that subclaim indirect. This still does
not approve schema/API/storage/public MCP/harness/ranking/lifecycle/migration/document-index/
`orient` changes.

T84 matrix note: the raw terminal evidence rule in
`docs/BRAIN_HARNESS_T84_RAW_TERMINAL_EVIDENCE_RULE_2026-06-01.md` codifies the T83 caveat without
running a disconnected standalone `git status` pilot. The rule improves future artifact quality:
if a controlled row genuinely depends on terminal state, pre-register the exact command/scope and
preserve short raw output with interpretation and limitations; otherwise keep the subclaim indirect.
Copied output remains author-captured evidence, not independent proof. This does not approve
schema/API/storage/public MCP/harness/ranking/lifecycle/migration/document-index/`orient` changes.

T85 matrix note: the Claude Bridge project-harness tool-exposure recheck in
`docs/BRAIN_HARNESS_T85_CLAUDE_BRIDGE_PROJECT_TOOL_EXPOSURE_RECHECK_2026-06-01.md` repeated the
T79 caveat under a single pre-registered `write=false`, no-Bash call with only
`mcp__engram__orient` and `mcp__engram__search` allowed. Claude Bridge reported
`No such tool available` for both tools and produced no Engram trace IDs. Treat this as stable
project-harness exposure evidence only; do not recheck the same line until bridge or harness
configuration changes, and do not infer anything about native Claude Code MCP behavior or Engram
retrieval quality.

T86 matrix note: the rolling handoff freshness audit in
`docs/BRAIN_HARNESS_T86_ROLLING_HANDOFF_FRESHNESS_AUDIT_2026-06-01.md` found the active handoff
was a low-information Claude Code session-end note that did not carry T85/T69/T70 context. Codex
refreshed only the rolling handoff to `019e82f3-53bc-7a83-9e39-cfdb29b06c44`, preserving exact
approval phrases and the default-deny boundaries. Treat this as continuity repair only; it does
not approve migration, lifecycle writes, document indexing, schema/storage/index changes, public
MCP changes, ranking changes, harness writes, or `orient` expansion.

T87 matrix note: the resume source precedence audit in
`docs/BRAIN_HARNESS_T87_RESUME_SOURCE_PRECEDENCE_AUDIT_2026-06-01.md` confirmed that current
Engram sources recover the active plan and handoff, while `/Users/yuval.meiri/notes/engram/handoff.md`
is stale open-source launch context from 2026-04-17. Direct search can still surface older handoff
MemoryItems lower in the result set, so the rolling handoff now explicitly says to use
`handoff(get)` and the latest current-plan memory as current resume sources. Treat this as
continuity repair only; it does not approve lifecycle writes, migration, document indexing,
schema/storage/index changes, public MCP changes, ranking changes, harness writes, or `orient`
expansion.

T88 matrix note: the stale handoff lifecycle packet in
`docs/BRAIN_HARNESS_T88_STALE_HANDOFF_LIFECYCLE_APPROVAL_PACKET_2026-06-01.md` freezes one exact
archive target, `019e82f3-53bc-7a83-9e39-cfdb29b06c44`, because the active handoff
`019e82f8-cada-7c31-b073-18ac41986b1e` supersedes it and direct searches still return both at
equal score. This is an approval packet only. No archive was run, no broad stale-handoff sweep was
authorized, and older active handoff noise remains a separate audit problem.

T89 matrix note: the changes-since cursor ergonomics slice in
`docs/BRAIN_HARNESS_T89_CHANGES_SINCE_CURSOR_ERGONOMICS_2026-06-01.md` keeps the existing
timestamp-based cursor semantics but improves the commit-id-only MCP error. Agents now get an
actionable instruction to pass `memory_cursor.timestamp` and optionally `memory_cursor.commit_id`
from `orient` or `memory(action="cursor")`. This is a continuity papercut fix only: no public MCP
request parameters, ranking, `orient` payload, migration state, lifecycle state, document-index
state, schema/storage/index behavior, or harness hooks/adapters changed.

T90 matrix note: the CLI changes-since cursor ergonomics slice in
`docs/BRAIN_HARNESS_T90_CLI_CHANGES_SINCE_CURSOR_ERGONOMICS_2026-06-01.md` applies the same cursor
guidance to `engram memory changes-since`. CLI help now points `--timestamp` at
`memory_cursor.timestamp`, `--commit-id` is documented as optional cursor context, and invalid
timestamp errors name the cursor field. This is a CLI continuity papercut fix only: no CLI flag
shape, public MCP request parameters, ranking, `orient` payload, migration state, lifecycle state,
document-index state, schema/storage/index behavior, or harness hooks/adapters changed.

T91 matrix note: the rolling handoff freshness repair in
`docs/BRAIN_HARNESS_T91_ROLLING_HANDOFF_T90_FRESHNESS_REPAIR_2026-06-01.md` found live resume
drift after T90: lean `orient` and direct `search` recovered T90 current-plan memory first, while
`handoff(get)` still described T87/T86 as the latest state. Codex refreshed only the rolling
handoff to `019e8316-ebd1-7220-b18e-f0d33110131a`, superseding
`019e82f8-cada-7c31-b073-18ac41986b1e`. This improves continuity only; it does not archive old
handoffs, inspect T69 files, run T70 indexing, run M6, mutate lifecycle state, change ranking,
expand `orient`, change public MCP/schema/storage/index behavior, change document-index behavior,
or write harness adapters/hooks.

T92 matrix note: the lint superseded-active visibility slice in
`docs/BRAIN_HARNESS_T92_LINT_SUPERSEDED_VISIBILITY_2026-06-01.md` responded to post-T91 evidence
that direct search still shows old active handoffs while `lint(action="run", limit=20)` is crowded
by generic stale-feedback rows. The private lint priority now keeps stale current-plan feedback
first, then surfaces safe-action `superseded_item_still_active` findings before generic
`feedback_stale_active_memory`. This is read-only report visibility only; no lifecycle archive,
`apply_safe`, M6, document-index, retrieval ranking, `orient`, public MCP, schema/storage/index, or
harness behavior changed.

T93 matrix note: the T92 source behavior is now validated in the installed live MCP runtime.
Before runtime refresh, MCP lint still returned generic stale-feedback rows ahead of
superseded-active rows despite the passing source regression. After installing binary hash
`e54aed9a4830cc53822100930d63541bf51d06b3f27c2844e6090bfe01f5379a` and restarting the daemon on
port `8765`, live MCP `lint(action="run", limit=20)` returned stale current-plan feedback first,
wrong-scope feedback next, and safe-action `superseded_item_still_active` rows starting at rank 5.
This validates installed report ordering only. It does not authorize `lint(action="apply_safe")`,
archive old handoffs, inspect T69 files, run T70 indexing, run M6, mutate lifecycle state, change
ranking, expand `orient`, change public MCP/schema/storage/index behavior, change document-index
behavior, or write harness adapters/hooks.

T94 matrix note: the rolling handoff freshness repair in
`docs/BRAIN_HARNESS_T94_ROLLING_HANDOFF_T93_FRESHNESS_REPAIR_2026-06-01.md` found live resume
drift after T93: lean `orient`, direct `search`, docs, git, and `changes_since` recovered T93,
while `handoff(get)` still described T90 as the latest implementation slice. Codex refreshed only
the rolling handoff to `019e8352-a610-7f92-859f-f9d74b026ba7`, superseding
`019e8316-ebd1-7220-b18e-f0d33110131a`. This improves continuity only; it does not archive old
handoffs, inspect T69 files, run T70 indexing, run M6, mutate lifecycle state, change ranking,
expand `orient`, change public MCP/schema/storage/index behavior, change document-index behavior,
or write harness adapters/hooks.

T95 matrix note: the stale T91 handoff lifecycle packet in
`docs/BRAIN_HARNESS_T95_STALE_HANDOFF_T91_LIFECYCLE_APPROVAL_PACKET_2026-06-01.md` freezes one
exact archive target, `019e8316-ebd1-7220-b18e-f0d33110131a`, because active T94 handoff
`019e8352-a610-7f92-859f-f9d74b026ba7` supersedes it and direct searches still return both at
equal score. This is an approval packet only. No archive was run, no broad stale-handoff sweep was
authorized, and T88 remains a separate exact approval packet for
`019e82f3-53bc-7a83-9e39-cfdb29b06c44`.

T96 matrix note: the rolling handoff freshness repair in
`docs/BRAIN_HARNESS_T96_ROLLING_HANDOFF_T95_FRESHNESS_REPAIR_2026-06-01.md` found live resume
drift after T95: lean `orient`, direct `search`, docs, git, and `changes_since` recovered T95,
while `handoff(get)` still described T93/T94 as the latest implementation context. Codex refreshed
only the rolling handoff to `019e835e-81c2-7562-897a-e42c0fe8dc08`, superseding
`019e8352-a610-7f92-859f-f9d74b026ba7`. This improves continuity only; it does not archive old
handoffs, inspect T69 files, run T70 indexing, run M6, mutate lifecycle state, change ranking,
expand `orient`, change public MCP/schema/storage/index behavior, change document-index behavior,
or write harness adapters/hooks.

T97 matrix note: the stale T94 handoff lifecycle packet in
`docs/BRAIN_HARNESS_T97_STALE_HANDOFF_T94_LIFECYCLE_APPROVAL_PACKET_2026-06-01.md` freezes one
exact archive target, `019e8352-a610-7f92-859f-f9d74b026ba7`, because active T96 handoff
`019e835e-81c2-7562-897a-e42c0fe8dc08` supersedes it and direct searches still return both at
equal score. This is an approval packet only. No archive was run, no broad stale-handoff sweep was
authorized, and T88/T95 remain separate exact approval packets for
`019e82f3-53bc-7a83-9e39-cfdb29b06c44` and
`019e8316-ebd1-7220-b18e-f0d33110131a`.

T98 matrix note: the rolling handoff freshness repair in
`docs/BRAIN_HARNESS_T98_ROLLING_HANDOFF_T97_FRESHNESS_REPAIR_2026-06-01.md` found live resume
drift after T97: lean `orient`, direct `search`, docs, git, and `changes_since` recovered T97,
while `handoff(get)` still described T95/T96 as the latest implementation context. Codex refreshed
only the rolling handoff to `019e836a-435a-75e1-8702-ced8eabe85cc`, superseding
`019e835e-81c2-7562-897a-e42c0fe8dc08`. This improves continuity only; it does not archive old
handoffs, inspect T69 files, run T70 indexing, run M6, mutate lifecycle state, change ranking,
expand `orient`, change public MCP/schema/storage/index behavior, change document-index behavior,
or write harness adapters/hooks.

T99 matrix note: the stale T96 handoff lifecycle packet in
`docs/BRAIN_HARNESS_T99_STALE_HANDOFF_T96_LIFECYCLE_APPROVAL_PACKET_2026-06-01.md` freezes one
exact archive target, `019e835e-81c2-7562-897a-e42c0fe8dc08`, because active T98 handoff
`019e836a-435a-75e1-8702-ced8eabe85cc` supersedes it and direct search still returns both active
handoffs at the same score. This is an approval packet only. No archive, `lint(action="apply_safe")`,
broad stale-handoff sweep, migration inspection/apply/deletion, document indexing, lifecycle
mutation, ranking, `orient`, public MCP, schema/storage/index, or harness write was run. T88, T95,
and T97 remain separate exact approval packets for `019e82f3-53bc-7a83-9e39-cfdb29b06c44`,
`019e8316-ebd1-7220-b18e-f0d33110131a`, and
`019e8352-a610-7f92-859f-f9d74b026ba7`.

T100 matrix note: the rolling handoff freshness repair in
`docs/BRAIN_HARNESS_T100_ROLLING_HANDOFF_T99_FRESHNESS_REPAIR_2026-06-01.md` found live resume
drift after T99: lean `orient`, direct `search`, docs, git, and `changes_since` recovered T99,
while `handoff(get)` still described T97/T98 as the latest implementation context. Codex refreshed
only the rolling handoff to `019e8378-b2f0-7260-a887-4abdf6c0e4e2`, superseding
`019e836a-435a-75e1-8702-ced8eabe85cc`. This improves continuity only; it does not archive old
handoffs, inspect T69 files, run T70 indexing, run M6, mutate lifecycle state, change ranking,
expand `orient`, change public MCP/schema/storage/index behavior, change document-index behavior,
or write harness adapters/hooks.

T101 matrix note: the stale T98 handoff lifecycle packet in
`docs/BRAIN_HARNESS_T101_STALE_HANDOFF_T98_LIFECYCLE_APPROVAL_PACKET_2026-06-01.md` freezes one
exact archive target, `019e836a-435a-75e1-8702-ced8eabe85cc`, because active T100 handoff
`019e8378-b2f0-7260-a887-4abdf6c0e4e2` supersedes it and direct search still returns both active
handoffs at the same score. This is an approval packet only. No archive, `lint(action="apply_safe")`,
broad stale-handoff sweep, migration inspection/apply/deletion, document indexing, lifecycle
mutation, ranking, `orient`, public MCP, schema/storage/index, or harness write was run. T88, T95,
T97, and T99 remain separate exact approval packets for
`019e82f3-53bc-7a83-9e39-cfdb29b06c44`,
`019e8316-ebd1-7220-b18e-f0d33110131a`,
`019e8352-a610-7f92-859f-f9d74b026ba7`, and
`019e835e-81c2-7562-897a-e42c0fe8dc08`.

T102 matrix note: the rolling handoff freshness repair in
`docs/BRAIN_HARNESS_T102_ROLLING_HANDOFF_T101_FRESHNESS_REPAIR_2026-06-01.md` found live resume
drift after T101: lean `orient`, direct `search`, docs, git, and `changes_since` recovered T101,
while `handoff(get)` still described T99/T100 as the latest implementation context. Codex refreshed
only the rolling handoff to `019e8381-5e35-78d2-b4f9-7ef949fc6e6b`, superseding
`019e8378-b2f0-7260-a887-4abdf6c0e4e2`. This improves continuity only; it does not archive old
handoffs, inspect T69 files, run T70 indexing, run M6, mutate lifecycle state, change ranking,
expand `orient`, change public MCP/schema/storage/index behavior, change document-index behavior,
or write harness adapters/hooks.

T103 matrix note: the stale T100 handoff lifecycle packet in
`docs/BRAIN_HARNESS_T103_STALE_HANDOFF_T100_LIFECYCLE_APPROVAL_PACKET_2026-06-01.md` freezes one
exact archive target, `019e8378-b2f0-7260-a887-4abdf6c0e4e2`, because active T102 handoff
`019e8381-5e35-78d2-b4f9-7ef949fc6e6b` supersedes it and direct search still returns both active
handoffs near the top of relevant results. This is an approval packet only. No archive,
`lint(action="apply_safe")`, broad stale-handoff sweep, migration inspection/apply/deletion,
document indexing, lifecycle mutation, ranking, `orient`, public MCP, schema/storage/index, or
harness write was run. T88, T95, T97, T99, and T101 remain separate exact approval packets for
`019e82f3-53bc-7a83-9e39-cfdb29b06c44`,
`019e8316-ebd1-7220-b18e-f0d33110131a`,
`019e8352-a610-7f92-859f-f9d74b026ba7`,
`019e835e-81c2-7562-897a-e42c0fe8dc08`, and
`019e836a-435a-75e1-8702-ced8eabe85cc`.

T104 matrix note: the rolling handoff freshness repair in
`docs/BRAIN_HARNESS_T104_ROLLING_HANDOFF_T103_CLAUDE_BRIDGE_FRESHNESS_REPAIR_2026-06-01.md`
found live resume drift after the T103 Claude Bridge critique: `handoff(get)` returned
low-information Claude Code session-end handoff `019e8388-2744-79d3-b91a-61bde6da34d5`, while
lean `orient`, direct `search`, docs, git, and current-plan memory recovered T103. Codex refreshed
only the rolling handoff to `019e838b-6b25-7011-8b4b-b4cc61dc450f`, superseding
`019e8388-2744-79d3-b91a-61bde6da34d5`. This improves continuity only; it does not archive old
handoffs, inspect T69 files, run T70 indexing, run M6, mutate lifecycle state, change ranking,
expand `orient`, change public MCP/schema/storage/index behavior, change document-index behavior,
or write harness adapters/hooks.

T41 matrix note: the T40-04 mixed-query caveat is now covered by deterministic fixture evidence,
not a production ranking change. Live recheck after the T40 current-plan capture returned current
plan first and active M6 gate context in top memory results, and
`test_memory_search_t40_mixed_query_surfaces_current_plan_and_m6_gate` preserves that invariant
with live-shaped stale/noisy records. This strengthens the current-plan/next-step retrieval row
for one exact mixed prompt class only. It does not prove broad ranking quality or approve M6
inventory/export/apply/deletion, lifecycle cleanup, hook/adapter writes, schema/storage changes,
public MCP changes, or `orient` payload changes.

T42 matrix note: the planned Claude Code parity check was stopped before the scoreable Claude run
because the pre-run Codex baseline failed. Trace `019e7d08-d297-71b3-b8dd-495078383ce9` returned
current-plan memory first but omitted active M6 gate memory from the top eight results for the exact
mixed query, and diagnostic trace `019e7d09-d6ae-7a83-a9c7-b835c25b9df4` placed the active M6 gate
at rank 17. This re-opens the live installed mixed-query retrieval gap while preserving the T41
deterministic fixture as regression evidence. The next non-gated work should be a new
prompt-specific retrieval slice for the exact mixed prompt class, not a Claude parity run against a
failing baseline and not M6, lifecycle, schema/storage/index, public MCP, `orient`, broad ranking,
or harness-write work.

T43 matrix note: the T42 live installed mixed-query gap is repaired for direct `search` only.
`test_memory_search_promotes_m6_gate_context_below_current_plan_for_mixed_query` covers live-shaped
noise, stale current-plan guidance, active M6 gate memory, pure continuation control, and explicit
M6 apply/gate control. After installing binary
`c8b1254ac71f53da80221a2a259014fca89e2e8e8ca1998a4f0128adce01e721` and restarting the daemon,
trace `019e7d1c-b20a-7c52-b8af-e6d82439988c` returned current-plan memory
`019e7d0b-3425-7c00-a395-a69c14cf2a47` first and active M6 gate memory
`019e7ce5-155d-7a10-85f5-00b9dcc69cd0` second for the exact mixed query. Negative-control trace
`019e7d1c-c100-7721-82ba-8061330aff8f` preserved gate/blocked context above current-plan guidance,
pure continuation trace `019e7d1e-29ad-7540-bcfc-d28131851091` did not promote M6 gate context, and
lean `orient` sanity trace `019e7d1e-2a48-7d63-a49d-a7da22bfa68f` stayed compact. This closes the
immediate T42 search-ranking gap but does not prove broad ranking quality, Claude Code parity for
this repaired prompt class, or M6/lifecycle/schema/public-MCP/`orient`/harness readiness.

T44 matrix note: Claude Code parity for the T43 direct-search prompt class passed without code or
configuration changes. Claude trace `019e7d21-cec2-7c60-b570-40bb6b79574e` returned current-plan
memory `019e7d20-d54d-7d61-99ac-f6ed805848c9` first and active M6 gate memory
`019e7ce5-155d-7a10-85f5-00b9dcc69cd0` second for the exact mixed query. Negative-control trace
`019e7d21-d4c6-7eb0-80a7-244042f513b0` preserved gate/blocked context above current-plan guidance,
and pure continuation trace `019e7d21-da4e-7e72-9e40-35153ba73628` returned current-plan first with
active M6 gate absent from the top eight. This closes the immediate Claude parity question for the
T43 prompt class only; it does not prove broad ranking quality or authorize M6, lifecycle, public
MCP, `orient`, broad ranking, or harness adapter/hook work.

T45 matrix note: the next M6 step is now expressed as an explicit approval packet rather than an
implicit request. The packet asks only for one bounded inventory-only
`memory(action="migration_inventory", project_name="engram", limit=200, include_entity_observations=true, include_session_history=true, include_work_observations=true, exclude_reviewed_path="/Users/yuval.meiri/.engram/reviews/2026-04-28-memory-os-completion")`
call and a Markdown report. Review export, apply, deletion, lifecycle mutation, schema/storage/index
changes, public MCP changes, ranking or `orient` changes, and harness adapter/hook changes remain
out of scope until separately approved.

T46 matrix note: the harness readiness re-audit was read-only and reconfirmed `ready=false` across
the generic, Claude Code, Codex, Gemini CLI, and Cursor harness surfaces. The generic policy
document is missing; Claude Code has generated adapter files installed but lacks required
`SessionStart` and `SessionEnd` settings registrations; Codex, Gemini CLI, and Cursor retain
required generated-adapter drift. This updates the evidence date only. It does not authorize
adapter installation, settings edits, hook registration, M6 inventory/export/apply/deletion,
lifecycle mutation, schema/storage/index changes, public MCP changes, ranking changes, or `orient`
payload changes.

T47 matrix note: the next harness-readiness step is now expressed as an explicit approval packet
rather than an implicit repair request. The packet asks only for five dry-run-derived
`harness(action="install", write=true, ...)` calls against exact local paths, each preceded by a
fresh matching `write=false` dry-run, with Claude Code restricted to
`settings_target="settings.local.json"` and `adopt_user_owned=false`. It does not authorize the
writes by itself, user-owned adoption, `settings.json` edits, snippet edits, root `AGENTS.md` edits,
hook rewrites, globs, M6 inventory/export/apply/deletion, lifecycle mutation,
schema/storage/index changes, public MCP changes, ranking changes, or `orient` payload changes.

T48 matrix note: the stale repository-scoped current-plan memory
`019e5e0a-86b4-73e3-aa9b-ca350e83e915` still appears below the latest project-scoped T47 current
plan in `orient` and search, while read-only lint now reports 129 recent stale-feedback records for
that item with `safe_action=none`. The T48 packet asks only for one exact
`memory(action="archive", ...)` call on that item, contingent on fresh matching read-only
get/list/lint/orient evidence. It does not authorize that archive by itself, any other lifecycle
write, M6 inventory/export/apply/deletion, harness writes, schema/storage/index changes, public MCP
changes, ranking changes, or `orient` payload changes.

T49 matrix note: the pending approval retrieval audit in
`docs/BRAIN_HARNESS_T49_PENDING_APPROVAL_RETRIEVAL_AUDIT_2026-05-31.md` is partial. Direct
`search` trace `019e7d43-c140-72f1-86c4-6b32096c6095` returned the active M6 gate, harness-write
gate, and T48 lifecycle approval packet as the top three memory results for an explicit approval
gates prompt, and trace `019e7d43-c096-70b3-897c-5a5d205817a7` recovered the same queue for an
explicit T45/T47/T48 query. Lean `orient` trace `019e7d43-c1d2-78b0-b815-a398f924a765` returned
the T48 current-plan memory first but did not individually surface M6 and harness-write gate
memories. This is read-only evidence only; it does not authorize M6, lifecycle writes, harness
writes, ranking changes, `orient` payload expansion, schema/storage/index changes, or public MCP
changes.

T50 matrix note: the Claude Code pending-approval parity smoke in
`docs/BRAIN_HARNESS_T50_CLAUDE_PENDING_APPROVAL_PARITY_2026-05-31.md` passed for the post-T49
continuation prompt class. Claude Bridge ran with only read-only Engram `orient`, `search`, and
`obligations` tools allowed. Claude reported lean `orient` trace
`019e7d48-6e97-7513-96af-f49d5a61bfc5` with T49 current plan first, harness-write gate second,
M6 gate third, and stale repository-scoped current-plan memory fifth; direct `search` trace
`019e7d48-905b-75c2-9d5b-e9cb657024c9` returned M6 first, harness-write second, and T49 current
plan third. The smoke created two synthetic obligations, resolved/skipped with evidence, and a
final doctor returned clean. This is read-only parity evidence only; it does not authorize M6,
lifecycle writes, harness writes, ranking changes, `orient` payload expansion,
schema/storage/index changes, or public MCP changes.

T51 matrix note: the T48 stale current-plan archive packet has drifted. Fresh read-only evidence
in `docs/BRAIN_HARNESS_T51_T48_ARCHIVE_PACKET_DRIFT_2026-05-31.md` shows project-scoped active
current-plan memory is now T50 `019e7d4b-f526-7141-809d-035a7003a2ed`, while the old
repository-scoped target `019e5e0a-86b4-73e3-aa9b-ca350e83e915` remains active and lint reports
139 stale-feedback records with `safe_action=none`. Because T48 hard-coded the T47 project plan
and a 129-record archive reason, the T48 payload must not be executed as written. T51 does not
create a refreshed approval packet, archive the target, mutate lifecycle state, run M6, write
harness adapters/settings/hooks, change schema/storage/index state, change public MCP behavior,
change ranking, or expand `orient`.

T52 matrix note: the stale repository-scoped current-plan target now has a refreshed resolution
request in `docs/BRAIN_HARNESS_T52_STALE_CURRENT_PLAN_RESOLUTION_REQUEST_2026-05-31.md`. Fresh
read-only evidence shows T51 `019e7d55-b103-70b3-a023-6398e96d6430` is the active project-scoped
current plan, the target `019e5e0a-86b4-73e3-aa9b-ca350e83e915` is still the only active
repository-scoped current-plan item for `/Users/yuval.meiri/projects/engram`, and lint reports 142
stale-feedback records with `safe_action=none`. AI Council and Claude Bridge critique found that
archive-only approval would hide a real scope-gap decision, so T52 asks the user to choose
archive-only, replacement-then-archive, or scope-correction/merge. It does not authorize lifecycle
writes, create a replacement, run M6, write harness adapters/settings/hooks, change
schema/storage/index state, change public MCP behavior, change ranking, or expand `orient`.

T53 matrix note: the post-T52 Claude parity smoke in
`docs/BRAIN_HARNESS_T53_T52_CLAUDE_PARITY_2026-05-31.md` passed for the current continuation
prompt class. Codex baseline trace `019e7d5f-3fb4-7430-ab79-320a0e938156` and direct search trace
`019e7d5f-a55f-7e61-9d71-286093777d46` returned T52 current-plan memory
`019e7d5d-c450-7171-9fdb-8d1a5e745b0b` first. Claude Bridge, with only read-only Engram
`orient`, `search`, and `obligations` tools allowed, returned the same T52-first shape in lean
`orient` trace `019e7d60-64af-76d3-948f-5dd6068aa3d8` and direct `search` trace
`019e7d60-67e9-71d0-a421-f3364d4a5131`. The stale repo current-plan target remained visible below
T52 and was treated as pending-decision evidence only. This does not authorize lifecycle writes,
replacement memory, M6, harness writes, schema/storage/index changes, public MCP changes, ranking
changes, or `orient` expansion.

T54 matrix note: the rolling telemetry audit in
`docs/BRAIN_HARNESS_T54_ROLLING_TELEMETRY_AUDIT_2026-05-31.md` keeps the evidence loop partially
validated rather than complete. The sampled project report passed numerically with
`trace_count=50`, `feedback_trace_count=31`, `feedback_coverage=0.6200000047683716`,
`memory_judgment_coverage=1.0`, `bad_memory_used_count=0`, and `confidence_gate.passed=true`.
However, it also showed one task failure, `stale_memory_count=25`, `missing_context_count=6`, and
only `external_session_trace_count=13`. This is weak rolling evidence and does not authorize M6,
lifecycle writes, replacement memory, harness writes, schema/storage/index changes, public MCP
changes, ranking changes, or `orient` expansion.

T55 matrix note: the post-T54 Claude parity smoke in
`docs/BRAIN_HARNESS_T55_T54_CLAUDE_PARITY_2026-05-31.md` passed for the current continuation
prompt class after rerunning through the Claude Bridge personal harness. Codex baseline orient trace
`019e7d6b-ad40-7002-a777-4f5c6fdc0923` and direct search trace
`019e7d6b-f000-7323-99c0-5010649f2dc1` returned T54 current-plan memory
`019e7d68-a1b5-74c1-beb6-3a27d8495b93` first. The project-harness Claude attempt was unmeasured
because only file tools were available. The personal-harness Claude run, with only read-only Engram
`orient`, `search`, and `obligations` tools allowed, returned T54 first in lean `orient` trace
`019e7d6d-460f-7ae1-bae0-1a662ace3e5d` and direct `search` trace
`019e7d6d-4648-71e2-9cd7-c702a5b9cd48`. Stale current-plan and migration records remained visible
below T54 and were treated as historical evidence only. This does not authorize lifecycle writes,
replacement memory, M6, harness writes, schema/storage/index changes, public MCP changes, ranking
changes, or `orient` expansion.

T56 matrix note: the post-T55 feedback telemetry audit in
`docs/BRAIN_HARNESS_T56_POST_T55_FEEDBACK_TELEMETRY_AUDIT_2026-05-31.md` keeps the evidence loop
partially validated. The sampled project report still passed numerically with `trace_count=50`,
`feedback_trace_count=33`, `feedback_coverage=0.6600000262260437`,
`memory_judgment_coverage=1.0`, `bad_memory_used_count=0`, and `confidence_gate.passed=true`.
Compared with T54, feedback coverage improved from `31/50` to `33/50` and
`external_session_trace_count` improved from `13` to `23`, but one task failure remained and
`stale_memory_count` increased from `25` to `31`. This is weak rolling evidence and does not
authorize M6, lifecycle writes, replacement memory, harness writes, schema/storage/index changes,
public MCP changes, ranking changes, or `orient` expansion.

T57 matrix note: the post-T56 Claude parity and search-visibility smoke in
`docs/BRAIN_HARNESS_T57_T56_CLAUDE_PARITY_AND_SEARCH_VISIBILITY_2026-05-31.md` passed for the
current continuation prompt class. Codex baseline orient trace
`019e7d75-8458-7df1-9581-996188f71d27` and exact continuation search trace
`019e7d75-aff0-72a1-8dc9-9a473ff4da89` returned T56 current-plan memory
`019e7d74-51e9-7103-92b7-67104e6e22e9` first. Claude Bridge, with only read-only Engram
`orient`, `search`, and `obligations` tools allowed, returned T56 first in lean `orient` trace
`019e7d76-62e1-7b73-901e-5f839bcec551` and exact continuation `search` trace
`019e7d76-67ee-7a72-9bb1-e41a5489d7fe`. The broader implementation-plan query returned older
non-gated calibration first and T56 second in both Codex trace
`019e7d75-b03f-7da0-922f-7a4c878ce3b7` and Claude trace
`019e7d76-6d71-7d50-ab3d-1fcd32453cbd`; that is a documented caveat, not evidence for broad
ranking changes. This does not authorize lifecycle writes, replacement memory, M6, harness writes,
schema/storage/index changes, public MCP changes, ranking changes, or `orient` expansion.

T58 matrix note: the approved inventory-only M6 scoping run in
`docs/BRAIN_HARNESS_T58_T45_M6_INVENTORY_REPORT_2026-05-31.md` completed without writes. It
scanned 115 sources, returned 11 candidates, skipped 55 already migrated sources and 49
already-decided review-workspace candidates, and found 9 review plus 2 quarantine dispositions.
This is bounded inventory evidence only; it does not authorize review export, apply, deletion,
lifecycle mutation, schema/storage/index changes, public MCP changes, ranking changes, `orient`
expansion, or harness adapter/hook changes.

T59 matrix note: `docs/BRAIN_HARNESS_T59_M6_REVIEW_EXPORT_SCOPE_PROPOSAL_2026-05-31.md` prepared
the approval question for exactly one `memory(action="migration_review_export", ...)` call using
the T58 `exclude_reviewed_path` and a fixed review path. It recorded path-existence and count-drift
stop conditions and required a new pause before any candidate decisions or write apply. The exact
scope was later approved and executed as T68, where the count guard stopped the run after 12
candidates appeared.

T60 matrix note: the T59/default-deny retrieval parity report in
`docs/BRAIN_HARNESS_T60_T59_GATE_RETRIEVAL_PARITY_2026-05-31.md` is a retrieval pass with a write
confound. Codex and Claude Code both surfaced T59 and preserved default-deny for
`migration_review_export`; no prompt claimed review export was approved. Continuation `search`
remained noisy, with historical research/calibration items above T59, and broader search still
returned stale repository current-plan evidence near T59. The intended no-write condition failed
because Claude Bridge `write=false` triggered existing Claude Code session-end rolling handoff
MemoryItem writes. This does not authorize deleting or cleaning those handoffs, changing hooks or
adapters, running M6 review export/apply, lifecycle writes, schema/storage/index changes, public
MCP changes, ranking changes, or `orient` expansion.

T23 matrix audit, 2026-05-27:

- Research question: after T21/T22, does the completion matrix still identify the next real Brain OS
  blocker and the latest evidence without overstating completion?
- Hypotheses: preferred: the matrix remains directionally right but needs a current audit delta;
  null: T21/T22 change no matrix conclusions; simpler alternative: rely on the checklist only;
  failure: the audit implies M6 or lifecycle approval that the user has not granted.
- Measurement: `orient(response_shape=lean, intent=plan_work)` trace
  `019e69ec-679d-7df0-985a-6f159a3165fc` returned current-plan memory
  `019e69eb-4f19-7c90-b930-96036b4e23cb` first and stale repository-scoped current-plan memory
  `019e5e0a-86b4-73e3-aa9b-ca350e83e915` fifth. Direct search trace
  `019e69ec-f013-7370-8757-af7091bd65e2` returned the same current plan first and the stale
  repository-scoped plan second. `memory(action=list, scope_type=project, project_name=engram,
  tags=[current-plan])` returned only the current project-scoped plan, so scoped memory-list
  sampling is clean even though broader orientation/search can still surface stale repository
  guidance as noisy context. `real_session_eval(project=engram, limit=50)` returned
  `trace_count=50`, `feedback_trace_count=40`, `feedback_coverage=0.800000011920929`,
  `distinct_intent_count=4`, `bad_memory_used_count=0`, and `confidence_gate.passed=true`.
- Matrix delta: current-plan retrieval is validated for the latest startup prompt, but stale
  repository-scoped plan guidance still appears as lower-ranked noise and remains a lifecycle
  review issue, not an approved archival/scope-correction action. Cross-harness behavior now also
  includes T22 native Claude Code telemetry-report parity for the T21 scenario; Claude Bridge tool
  exposure is still limited. Evidence-loop plumbing is stronger after T19/T20/T21/T22 and the
  current project-level gate is numerically passing, but the report still requires user approval for
  migration decisions and remains weak agent-assessed evidence unless corroborated. M6 remains the
  only known high-risk completion blocker in this matrix and is approval-gated before even another
  read-only inventory/review-export pass.

T24 external-session audit, 2026-05-27:

- Research question: is sparse `external_session_id` coverage a core telemetry implementation gap,
  a harness-guidance/adoption gap, or a host-session availability limit?
- Measurement: live `real_session_eval(project=engram, limit=50)` returned `trace_count=50`,
  `feedback_trace_count=38`, `feedback_coverage=0.7599999904632568`,
  `external_session_trace_count=5`, `unspecified_external_session_trace_count=45`,
  `external_session_feedback_count=5`, and `confidence_gate.passed=true`. The latest 12 project
  traces all had `external_session_id=null`.
- Source/test check: `BrainHarnessTrace` and `AgentFeedback` store the optional label; `orient`,
  `search`, `changes_since`, and `telemetry` pass it through; feedback inherits the trace label when
  omitted; focused tests cover service and MCP pass-through plus eval counts.
- Matrix delta: this is a transcript-joinability weakness, not evidence that core telemetry storage
  or eval plumbing is missing. Future hook/adapter/host integration work to provide stable labels
  remains separately approval-gated.

T25 rolling evidence-window audit, 2026-05-27:

- Research question: after T24 feedback scoring and the next T25 startup traces, does the rolling
  `real_session_eval(project=engram, limit=50)` report still support the completion-matrix evidence
  claims without overstating confidence?
- Measurement: after T24 trace feedback, the project report reached `feedback_trace_count=44/50`,
  `feedback_coverage=0.88`, `external_session_trace_count=5/50`, and
  `bad_memory_used_count=0`. After the T25 startup added fresh unscored orient/search traces, the
  read-only report generated at `2026-05-27T15:06:04Z` returned `trace_count=50`,
  `feedback_trace_count=38`, `feedback_coverage=0.7599999904632568`,
  `memory_judgment_coverage=1.0`, `bad_memory_used_count=0`, `confidence_gate.passed=true`,
  `external_session_trace_count=5`, and `unspecified_external_session_trace_count=45`. After those
  T25 startup traces were scored, the report generated at `2026-05-27T15:10:44Z` returned
  `feedback_trace_count=44`, `feedback_coverage=0.8799999952316284`,
  `bad_memory_used_count=0`, `confidence_gate.passed=true`, and
  `external_session_trace_count=5`.
- Matrix delta: the confidence gate remains a useful rolling operational signal, but the latest
  sample proves it is window-sensitive rather than a durable completion proof; scoring the fresh
  T25 traces restored coverage to the same `44/50` level seen after T24 scoring. Current-plan
  retrieval still returned the active current-plan memory first for T25 startup prompts, while
  stale repository-scoped current-plan guidance remains lower-ranked lifecycle noise. This does not
  authorize M6 inventory/export/apply/deletion, lifecycle writes, hook/adapter writes, telemetry
  formula changes, or `orient` payload expansion.

T26 obligation-noise suppression, 2026-05-27:

- Research question: can obligation detection reduce false follow-through pressure from safety-gate
  wording and local instruction files without weakening explicit failed-tool or document-disposition
  detection?
- Measurement: source inspection found bare `schema` in `detect_prompt_obligations` triggered
  `tool_failure_recovery`, and `detect_document_obligations` created candidates for untracked root
  instruction files before later filtering/skipping. The patch routes tool-failure matching through
  a narrower helper and skips untracked root instruction files before document candidates are built.
  Validation passed with `cargo test -p engram-index obligation::tests` (`6` passed),
  `cargo test -p engram-tests --test obligation_tests` (`10` passed),
  `cargo fmt --all --check`, `cargo check -p engram-cli`, and `git diff --check`.
- Matrix delta: memory quality/follow-through is slightly stronger because false obligation pressure
  is lower while explicit failed-tool and document-disposition coverage remains tested. This does
  not change `orient`, ranking, migration, lifecycle status, hook/adapter behavior, telemetry
  semantics, schema, or storage.

T27 installed-runtime validation for T26, 2026-05-27:

- Research question: after installing the T26 code and restarting the daemon, does live MCP
  obligation detection apply the same noise suppression that passed source and MCP-boundary tests?
- Measurement: before install, daemon PID `11922` with binary hash
  `0192d24d945b7acb8bdfabe129c56d61a5abf0f7ce8223c854139677a93738ab` returned
  `document_disposition` for `AGENTS.md`, `source_reading`, and `tool_failure_recovery` for prompt
  `Failure hypothesis: avoid schema changes unless evidence justifies them.` After
  `cargo install --path engram-cli --force --root /Users/yuval.meiri/.local` and daemon restart,
  binary hash `7d9256dc2ca9fcefaaa54bf620c15989fa20926c929d9e6beca27012b6afc9cf` on port `8765`,
  PID `50257`, returned only `source_reading` for the same prompt. A separate live MCP dry-run for
  `A tool call failed because of wrong parameters.` still returned `tool_failure_recovery`.
- Matrix delta: the T26 obligation signal-quality fix is now validated in the installed live runtime,
  closing the immediate binary-drift caveat for this slice. This does not authorize M6
  inventory/export/apply/deletion, lifecycle writes, ranking changes, hook/adapter writes, public MCP
  changes, telemetry formula changes, schema/storage changes, or `orient` payload expansion.

T28 Claude Code cross-harness obligation smoke, 2026-05-27:

- Research question: does native Claude Code, through the Claude Bridge personal harness and Engram
  MCP obligations tool, observe the same T27 installed-runtime obligation behavior as Codex?
- Measurement: AI Council recall found no directly relevant prior decision. Claude Bridge ran a
  foreground task with `harness=personal`, `write=false`, and only `mcp__engram__obligations`
  allowed for the requested calls. For prompt
  `Failure hypothesis: avoid schema changes unless evidence justifies them.`, Claude Code returned
  only `source_reading` id `019e6a13-8410-7be3-8c5a-38614d40e9d1`, with no
  `tool_failure_recovery` and no `AGENTS.md` document-disposition candidate. For prompt
  `A tool call failed because of wrong parameters.`, Claude Code returned `tool_failure_recovery`
  id `019e6a13-890d-7781-9e1e-cc50c5793c75`. Both requested calls reported dry-run results with no
  writes, no skipped existing obligations, and no warnings. Follow-up Codex
  `obligations(action=doctor)` found two prompt-derived open obligations written by the Claude Code
  harness itself, `019e6a13-6a3f-7d12-92b6-a89cc7d91b37` and
  `019e6a13-6a3f-7d12-92b6-a884a00d46c9`; Codex skipped both as synthetic-smoke artifacts with
  explicit evidence.
- Matrix delta: cross-harness behavior is slightly stronger for the shared MCP obligations surface.
  This does not change the T17 harness-readiness finding: hooks/settings/adapters are still not
  broadly ready, and adapter or hook writes remain gated. The smoke also exposes a harness caveat:
  prompts containing obligation trigger phrases can create startup obligations even when the requested
  validation calls are dry-run, so future smokes should run doctor and close synthetic artifacts. It
  does not authorize M6 work, lifecycle writes, ranking changes, public MCP changes, telemetry
  formula changes, schema/storage changes, or `orient` payload expansion.

T29 read-only completion gate audit, 2026-05-27:

- Research question: after T27/T28, does the completion matrix still identify a concrete non-gated
  implementation slice, or is the remaining Brain OS definition of done blocked by approval-gated
  migration and harness-configuration work?
- Hypotheses: preferred: the matrix is current enough to show meaningful progress while preserving
  the remaining approval gates; null: T27/T28 do not change any completion status; simpler
  alternative: rely on the prior T23/T24 matrix without another audit; failure: the audit implies
  approval for migration, lifecycle mutation, hook writes, or hot-path changes that the user has not
  granted.
- Measurement: T29 startup `orient` trace `019e6a18-38dc-7b01-9f0f-802c995e4830` returned active
  current-plan memory `019e6a16-e428-7ee0-9959-78af745a72ae` first. Direct startup searches for
  current plan, architecture, implementation plan, user philosophy, and risks also surfaced the
  active current plan and relevant gate/caveat memory. `git status --short` showed only the
  user-owned untracked root `AGENTS.md`; the daemon was still running on port `8765`, PID `50257`,
  with installed binary hash `7d9256dc2ca9fcefaaa54bf620c15989fa20926c929d9e6beca27012b6afc9cf`.
  `real_session_eval(project=engram, limit=50)` at `2026-05-27T15:42:45Z` returned
  `trace_count=50`, `feedback_trace_count=37`, `feedback_coverage=0.7400000095367432`,
  `memory_judgment_coverage=1.0`, `bad_memory_used_count=0`, `confidence_gate.passed=true`, and
  `external_session_trace_count=0`. `obligations(action=doctor)` returned no open obligations or
  warnings. Explicit `harness(action=doctor)` returned `ready=false` for `claude_code`, `codex`,
  `gemini_cli`, and `cursor`: Claude Code still lacks required `SessionStart` and `SessionEnd`
  settings registrations, while Codex, Gemini CLI, and Cursor still have required generated adapter
  drift.
- Matrix delta: current-plan/next-step retrieval remains validated for the current continuation
  prompt class, and obligation signal quality is now validated in both Codex and Claude Code for the
  observed request shape. The evidence loop remains partially validated because rolling feedback
  coverage is sample-window sensitive and the latest sampled traces have no external session labels.
  Cross-harness behavior remains partial because shared MCP request shapes work, but harness
  readiness is still false. The remaining high-risk completion gate is still M6 migration, and even
  another read-only inventory/review-export pass requires explicit user-approved scope. Adapter or
  hook writes also remain explicitly gated.

T30/T31 documentation and live-state audit, 2026-05-27:

- Research question: after the T30 architecture/research-method doc sync commits and T31 startup
  retrieval, does the completion matrix change, or do the same approval gates still define the next
  major product work?
- Hypotheses: preferred: the matrix remains stable, with better synchronized docs and fresher
  evidence for the same gates; null: T30/T31 add no new completion evidence; simpler alternative:
  rely on the T29 audit only; failure: the audit implies approval for migration inventory, lifecycle
  mutation, hook/adapter writes, ranking changes, schema/storage changes, or `orient` expansion.
- Measurement: commits `cb39282` and `42ed92c` synced
  `docs/BRAIN_HARNESS_ARCHITECTURE.md` and `docs/BRAIN_HARNESS_RESEARCH_METHOD.md` through T29
  evidence without runtime behavior changes. T31 startup lean `orient` trace
  `019e6a25-a81f-7a00-807f-4b5c30c91432` returned current-plan memory
  `019e6a24-99c8-7043-89a5-b363ca755460` first, while stale repository current-plan memory
  `019e5e0a-86b4-73e3-aa9b-ca350e83e915` still appeared as lower-ranked noise. Direct T31
  searches again surfaced historical no-evidence architecture/council memories and the stale
  migration-completion memory below current guidance. `git status --short` showed only the
  user-owned untracked root `AGENTS.md`; daemon PID `50257` was still serving installed binary hash
  `7d9256dc2ca9fcefaaa54bf620c15989fa20926c929d9e6beca27012b6afc9cf`. Read-only
  `harness(action=doctor)` still returned `ready=false` for Claude Code, Codex, Gemini CLI, and
  Cursor. Read-only `lint(action=run, limit=80)` still had no safe automatic action and reported
  `feedback_stale_current_plan` for `019e5e0a-86b4-73e3-aa9b-ca350e83e915` with 79 recent stale
  feedback records. Before scoring the new T31 traces, `real_session_eval(project=engram, limit=50)`
  returned `trace_count=50`, `feedback_trace_count=38`, `feedback_coverage=0.7599999904632568`,
  `memory_judgment_coverage=1.0`, `bad_memory_used_count=0`, `confidence_gate.passed=true`, and
  `external_session_trace_count=0`. After scoring the T31 startup traces, the same report returned
  `feedback_trace_count=44`, `feedback_coverage=0.8799999952316284`, `bad_memory_used_count=0`,
  `confidence_gate.passed=true`, and `external_session_trace_count=0`.
- Matrix delta: T30 improved documentation synchronization, and T31 reconfirmed current-plan
  retrieval for the current continuation prompt. No completion status changes: evidence-loop
  coverage remains sample-window sensitive, external-session joinability is still absent in the
  latest sampled window, stale historical memories remain review signals with `safe_action=none`,
  all supported harnesses remain not ready, M6 remains approval-gated before even read-only
  inventory/review-export, and adapter/hook writes remain separately gated.

T32 lint evidence-prioritization slice, 2026-05-27:

- Research question: can lint report ordering make review-critical feedback signals visible under
  normal limits without changing schemas, request parameters, lifecycle state, ranking, migration,
  hooks, adapters, public MCP surface, or the `orient` payload?
- Hypotheses: preferred: a deterministic private priority sort can surface stale current-plan and
  wrong-scope feedback before duplicate-entity, unresolved-obligation, and archive-safe lifecycle
  noise; null: agents already scan enough of the report for order not to matter; simpler
  alternative: document the lint-noise caveat only; failure: the ordering implies lifecycle cleanup
  authority or changes finding generation.
- Measurement: source inspection found `LintService::run` sorted findings lexicographically by ID
  before applying `limit`, so `duplicate_entity_candidate`, unresolved obligations, and
  archive-safe superseded items could hide `feedback_stale_current_plan` under small limits.
  Focused validation passed with `cargo test -p engram-index lint` and
  `cargo test -p engram-tests --test lint_tests`; the new tests assert that a stale current-plan
  feedback finding wins `limit=1` even with duplicate-entity, unresolved-obligation, and
  archive-safe distractors. Standard checks passed: `cargo fmt --all --check`,
  `cargo check -p engram-cli`, and `git diff --check`. Attempting a direct read-only CLI smoke
  against the live DB with `cargo run -p engram-cli -- lint run --limit 10 --json` failed because
  the daemon held the RocksDB lock, so live validation used the installed daemon instead. After
  installing binary
  `62db1e301ef7913ad685caa39d96ce0c479fc160fff3e8002df66401f619fce9` and restarting the daemon on
  port `8765`, PID `85531`, MCP `lint(action=run, limit=10)` returned
  `feedback_stale_current_plan` for stale current-plan memory
  `019e5e0a-86b4-73e3-aa9b-ca350e83e915` first. After scoring the T32 retrieval traces,
  `real_session_eval(project=engram, limit=50)` returned `feedback_trace_count=49`,
  `feedback_coverage=0.9800000190734863`, `bad_memory_used_count=0`,
  `confidence_gate.passed=true`, and `external_session_trace_count=0`.
- Matrix delta: memory-quality review is easier because the highest-signal feedback finding is no
  longer buried behind broad lint noise. This does not change `safe_action`, archive/apply
  behavior, lifecycle status, migration authority, ranking, telemetry formulas, hooks, adapters,
  schema/storage, public MCP request shape, or `orient`.

T33 Claude Code lint-ordering parity smoke, 2026-05-27:

- Research question: can Claude Code, through its own Engram MCP path, observe the same T32
  `lint(action=run, limit=10)` priority ordering that Codex sees?
- Hypotheses: preferred: Claude Code returns the same first lint finding as Codex; null: Codex-only
  validation is enough for this report-ordering surface; simpler alternative: document T32 without
  another parity smoke; failure: Claude Bridge tool exposure or synthetic prompt obligations make
  the result unusable.
- Measurement: AI Council prior-decision recall found no directly matching T32 lint-ordering
  consultation. Claude Bridge ran a read-only personal-harness task with `write=false`, allowing
  only `mcp__engram__lint` and `mcp__engram__obligations`. Claude Code reported that
  `lint(action=run, limit=10)` returned ten findings with `applied_safe_actions=0`; the first
  finding was `feedback_stale_current_plan` with id
  `feedback-stale-current-plan:019e5e0a-86b4-73e3-aa9b-ca350e83e915`, item id
  `019e5e0a-86b4-73e3-aa9b-ca350e83e915`, title `Current-plan guidance has stale feedback`, and
  `safe_action=none`. A follow-up Claude Code obligations doctor call reported one open
  synthetic `design_context_reading` obligation created by the validation prompt; Codex resolved
  it as `design_context_read` using the T33 startup docs already read before choosing the slice.
  A subsequent Codex `obligations(action=doctor, project=engram, cwd=...)` returned no open
  obligations or warnings. After scoring the T33 startup retrieval traces,
  `real_session_eval(project=engram, limit=50)` returned `feedback_trace_count=47`,
  `feedback_coverage=0.9399999976158142`, `bad_memory_used_count=0`,
  `confidence_gate.passed=true`, and `external_session_trace_count=0`.
- Matrix delta: cross-harness evidence is slightly stronger for the shared MCP `lint` surface and
  the T32 ordering result is no longer Codex-only. This does not validate broad harness readiness,
  hooks/settings/adapters, lifecycle cleanup, migration authority, ranking, telemetry formulas,
  schema/storage, public MCP request shape, or `orient`. Synthetic validation prompts can still
  create startup obligations, so future cross-harness smokes must run doctor and close artifacts.

T34 governing-doc sync and rolling eval audit, 2026-05-27:

- Research question: after T33, do the governing architecture/research-method docs and the central
  completion matrix still reflect the latest evidence without implying gated M6 or harness writes?
- Hypotheses: preferred: synchronize docs through T33/T34 evidence and preserve the same approval
  gates; null: T33 adds no doc synchronization need; simpler alternative: rely on the T33 report
  only; failure: the update implies migration, lifecycle, hook/adapter, ranking, schema/storage,
  public MCP, telemetry formula, or `orient` payload authority that the user has not granted.
- Measurement: T34 startup lean `orient` trace `019e6a51-23a7-72b1-b9b8-2e37fbcbc812`
  returned the current plan `019e6a50-1cb8-7623-97f6-9a7446fd7abc` first; direct current-plan
  search trace `019e6a51-3881-7002-b359-7ce125029735` also returned it first, with stale
  repository-scoped current-plan memory `019e5e0a-86b4-73e3-aa9b-ca350e83e915` still lower as
  review noise. Live `lint(action=run, limit=10)` returned `feedback_stale_current_plan` for that
  stale current-plan item first with 87 recent stale-feedback records and `safe_action=none`.
  `obligations(action=doctor)` returned `open=[]` and `warnings=[]`. Explicit harness doctor calls
  still returned `ready=false` for Claude Code, Codex, Gemini CLI, and Cursor. The installed daemon
  was running on port `8765`, PID `85531`, binary hash
  `62db1e301ef7913ad685caa39d96ce0c479fc160fff3e8002df66401f619fce9`.
  After scoring the T34 startup traces, `real_session_eval(project=engram, limit=50)` at
  `2026-05-27T16:49:00Z` returned `trace_count=50`, `feedback_trace_count=47`,
  `feedback_coverage=0.9399999976158142`,
  `memory_judgment_coverage=1.0`, `bad_memory_used_count=0`, `external_session_trace_count=0`,
  and `confidence_gate.passed=false` because feedback covered only two intents.
- Matrix delta: current-plan retrieval and lint visibility remain usable for the observed surfaces,
  but the evidence loop is again explicitly partial because the rolling confidence gate currently
  fails on intent diversity. This does not call for ranking or hot-path changes; it reinforces that
  M6 remains blocked without explicit user-approved scope and stronger evidence. The non-gated T34
  implementation was documentation synchronization plus document indexing and obligation cleanup.

Current MCP/CLI Memory OS surface:

- MCP: `orient`, `memory` including `capture_current_plan`, `harness`, `obligations`, `lint`, `graph`, `handoff`, `vault`, `digest`, `repo`.
- CLI: `engram orient`, `engram memory`, `engram harness`, `engram obligations`, `engram lint`, `engram graph`, `engram handoff`, `engram vault`, `engram digest`, `engram repo`.

Migration safety rule:

No orphan, digest, or legacy Engram data is automatically promoted to active memory. Even read-only
M6 inventory/review-export requires an explicit user-approved scope. Promotion requires review
decisions, dry-run apply, explicit approved write apply, and a knowledge commit.

---

## 1. Executive Summary

Engram should be extended, not replaced from scratch.

The current Engram codebase already has valuable foundations: a Rust workspace, MCP server, CLI, SurrealDB-backed persistence, session history, entity knowledge, document indexing, tool intelligence, coordination, knowledge management, and work management. Rebuilding those from zero would mostly recreate solved infrastructure.

However, the current product shape is too tool-centric. It behaves like a memory database that agents may use if prompted. The desired product is a memory environment that agents naturally orient through, update while working, lint for correctness, and expose as a human-readable Obsidian-compatible Markdown library.

The recommended path is a staged extension:

```text
Keep:
  current Engram MCP, CLI, storage, sessions, entities, work management

Add:
  Memory OS layer
  Markdown vault
  knowledge commits
  ontology and graph traversal
  lint and recalibration
  automatic orientation
  live session distillation

Avoid:
  a ground-up rewrite before the desired behavior is proven
```

The strategic shift is:

```text
Old framing:
  Persistent memory for AI coding agents.

New framing:
  Git-like, source-grounded, Obsidian-readable memory for AI coding agents.
```

In the new model:

- Markdown is the durable, human-editable memory surface.
- SurrealDB is the index, graph, and query engine.
- MCP is the agent interface.
- The CLI is the human/operator interface.
- Lint keeps memory current, sourced, non-contradictory, and scoped.
- Knowledge commits record how project understanding changes over time.

---

## 2. Why This Needs To Change

### 2.1 Current Daily Workflow Pain

Observed user pain:

1. The user has to remind the agent every few prompts to use Engram.
2. Session start/end commands are too manual.
3. Project and subproject identity gets confused, especially when working in notes folders.
4. The entity graph exists, but it does not drive behavior enough.
5. RAG/embeddings retrieve snippets, but daily coding needs a durable narrative.
6. Preferences and working agreements are discovered but not reliably retained.
7. Old knowledge is not archived, superseded, or recalibrated in a clear way.

The result is that Engram can store memory, but it does not yet behave like a persistent brain.

### 2.2 The Product Gap

Current shape:

```text
Agent works
  -> user reminds agent to log something
  -> agent may call MCP
  -> fact goes into database
  -> future retrieval depends on search/tool use
```

Desired shape:

```text
User prompts agent
  -> agent automatically orients through Engram
  -> Engram identifies project/task/scope
  -> Engram returns current context pack
  -> work happens
  -> Engram follows session events
  -> session is distilled into knowledge commits
  -> Markdown vault and graph are updated
  -> lint checks contradictions, staleness, scope, and provenance
```

### 2.3 Why RAG Is Not Enough

Modern models have very large context windows. This changes the value of embeddings.

Embeddings are still useful for discovery:

- Find candidate memories.
- Search old sessions.
- Search long documents.
- Retrieve similar events.

But embeddings should not be the main user experience.

Daily coding requires compiled context:

- What is this project?
- What subproject am I in?
- What decisions are active?
- What rules apply?
- What does the user prefer?
- What changed since the last related work?
- Which old facts are stale or superseded?

Compiled Markdown pages and knowledge commits answer these better than raw RAG snippets.

---

## 3. Extend Engram Or Build New?

### 3.1 Decision

Recommendation: extend Engram.

Build the new Memory OS as a new subsystem inside Engram, with new crates/modules and clear interfaces. Do not rewrite the existing system before proving the new behavior.

### 3.2 Decision Matrix

| Option | Benefits | Costs | Verdict |
|---|---|---|---|
| Extend Engram in-place | Reuses MCP, CLI, SurrealDB, sessions, entities, tests | Must manage legacy concepts and migration | Recommended |
| Build new system from scratch | Clean design, no legacy constraints | Rebuilds storage, MCP, CLI, tests, daemon, project model | Not worth it now |
| Fork Engram into a new product | Clean branding and architecture | Splits attention and compatibility | Consider later only if Memory OS outgrows current product |
| Add only an Obsidian export | Fastest visible improvement | Does not solve graph, lint, orientation, or lifecycle | Useful but insufficient |
| Add only more MCP tools | Easy incremental work | Keeps user reminding agent to use tools | Insufficient |

### 3.3 Why Extend

Engram already has:

- Rust workspace structure.
- Domain types.
- SurrealDB repository layer.
- MCP server.
- CLI.
- Session history.
- Work management.
- Knowledge registry.
- Entity relationships.
- Document indexing.
- Tests for most layers.

The desired system needs all of these. The missing parts are higher-level memory lifecycle, not the base infrastructure.

### 3.4 How To Avoid Legacy Drag

Add a new boundary:

```text
engram-memory
  memory ontology
  knowledge commits
  vault compiler
  orientation service
  lint service
  distillation service
```

This can live initially inside `engram-index` and `engram-core`, but the design should keep it separable.

Suggested final crate shape:

```text
engram-core
  existing domain types
  new memory ontology types

engram-store
  existing repos
  new memory/vault/lint repos

engram-index
  existing services
  new MemoryService, OrientationService, VaultService, LintService

engram-mcp
  existing tools
  new orient, memory, lint, vault, graph tools

engram-cli
  existing commands
  new orient, memory, vault, lint, graph commands
```

---

## 4. Target Product

### 4.1 One-Sentence Product Definition

Engram Memory OS is a local-first, source-grounded, Obsidian-readable memory system that orients AI coding agents, tracks how project knowledge evolves, and lints memory for freshness, scope, contradictions, and provenance.

### 4.2 Core Principles

1. The agent should orient automatically.
2. Markdown is the durable human-readable memory.
3. Raw evidence is immutable.
4. Compiled memory is editable and linted.
5. Every important fact needs provenance.
6. Memory is typed and scoped.
7. Old knowledge is archived, not lost.
8. Preferences need lifecycle and recalibration.
9. Graph relationships must drive behavior.
10. Search should be hybrid: keyword, vector, graph, time, and scope.

### 4.3 What This Is Not

This is not just:

- A vector database.
- A chat transcript store.
- A generic note-taking app.
- A hidden agent memory blob.
- A replacement for Git.
- A replacement for Obsidian.

It is a bridge between:

- raw evidence,
- compiled project memory,
- graph structure,
- human-readable notes,
- and agent runtime context.

---

## 5. System Architecture

### 5.1 High-Level Architecture

```text
User prompt
    |
    v
Agent calls orient
    |
    v
Orientation Service
    |        |          |          |
    |        |          |          |
    v        v          v          v
Project   Graph      Vault      Knowledge
Identity  Index      Pages      Commits
    |        |          |          |
    +--------+----------+----------+
             |
             v
       Context Pack
             |
             v
      Agent does work
             |
             v
      Live Event Stream
             |
             v
      Session Distiller
             |
             v
      Knowledge Commit
             |
      +------+-------+
      |              |
      v              v
   Vault Update    Graph Update
      |              |
      +------+-------+
             |
             v
          Lint
```

### 5.2 Storage Layers

```text
Raw evidence layer
  append-only events, session logs, command results, file refs, external refs

Compiled Markdown layer
  Obsidian-compatible vault pages

Graph/index layer
  entities, relationships, scopes, timestamps, embeddings, provenance

Knowledge commit layer
  Git-like memory diffs between project knowledge states

Runtime context layer
  generated context packs for agents
```

### 5.3 Relationship To Existing Engram

Existing Engram layers map naturally:

| Existing Layer | Memory OS Use |
|---|---|
| Entity Knowledge | Becomes graph backbone |
| Session History | Raw evidence and episodic memory |
| Document Knowledge | Source ingestion and retrieval |
| Tool Intelligence | Procedural/tool preference memory |
| Session Coordination | Live agent awareness |
| Knowledge Management | Vault/source registry foundation |
| Work Management | Project/task/subproject scope |

---

## 6. Tech Stack

### 6.1 Recommended Stack

| Concern | Technology |
|---|---|
| Core language | Rust |
| Persistence | SurrealDB embedded RocksDB initially |
| Graph/index | SurrealDB tables now; optional native graph traversal later |
| Markdown vault | Plain Markdown files, Obsidian-compatible links |
| MCP interface | Existing `rmcp` stack |
| CLI | Existing `clap` CLI |
| Embeddings | Existing `fastembed` path |
| Reranking | Start with deterministic scoring; add local/LLM reranking later |
| Lint rules | Rust rule engine with optional LLM-assisted checks |
| Session event capture | MCP calls plus optional shell/git/file watcher hooks |
| Frontend | Obsidian first; local web UI later |

### 6.2 Why Obsidian First

Building a polished graph browser is expensive. Obsidian gives immediate value:

- readable Markdown,
- backlinks,
- graph view,
- manual edits,
- local-first workflow,
- no new UI stack,
- easy user trust.

Engram should compile and index the vault; Obsidian should browse it.

### 6.3 Future UI

After the vault model is stable, add a local dashboard:

```text
engram dashboard
  project overview
  active tasks
  graph explorer
  lint findings
  stale memory
  recent knowledge commits
```

Do not build this before the memory model is proven.

---

## 7. Memory Ontology

### 7.1 Why Ontology Matters

Without ontology, memory becomes a bag of notes.

The system needs to distinguish:

- a user preference,
- a project rule,
- a decision,
- a limitation,
- an open question,
- a task,
- a session event,
- and a source document.

These need different fields, lifecycles, retrieval rules, and lint checks.

### 7.2 Core Node Types

```text
User
Agent
Workspace
Project
Subproject
Task
Session
Entity
Decision
Rule
Procedure
Preference
Limitation
OpenQuestion
Evidence
SourceDocument
KnowledgeCommit
Handoff
Tool
Command
TestRun
CodeFile
ExternalReference
```

### 7.3 Core Edge Types

```text
parent_of
subproject_of
contains
touches
modifies
creates
depends_on
uses
owned_by
documented_by
implements
decided_by
derived_from
evidenced_by
supersedes
superseded_by
contradicts
resolves
blocks
applies_to
triggered_by
mentioned_in
archived_from
loaded_by
```

### 7.4 Memory Types

Use three memory classes:

```text
Semantic memory
  facts, preferences, rules, decisions, limitations

Episodic memory
  session events, tool calls, failures, user corrections, commands

Procedural memory
  how the agent should act, project workflows, preferred tool usage
```

Examples:

```text
User dislikes AI-branded commit messages
  -> Semantic Preference

The Google Workspace MCP returned 401 invalid_token
  -> Episodic Event / Limitation Evidence

When OAuth is needed, use browser login with user assistance
  -> Procedural Rule
```

### 7.5 Required Fields By Type

#### Preference

```text
id
statement
scope
applies_to
evidence_refs
confidence
sensitivity
created_at
last_used_at
usage_count
review_after
status
```

#### Decision

```text
id
title
decision
rationale
alternatives_rejected
scope
evidence_refs
supersedes
status
created_at
updated_at
```

#### Rule

```text
id
statement
scope
trigger
strength
evidence_refs
status
review_after
```

#### Limitation

```text
id
statement
cause
workaround
scope
evidence_refs
status
recheck_after
```

#### OpenQuestion

```text
id
question
scope
owner
evidence_refs
created_at
status
resolved_by
```

#### KnowledgeCommit

```text
id
parent_id
project_id
scope_id
session_id
title
summary
added_memory_ids
changed_memory_ids
superseded_memory_ids
archived_memory_ids
vault_files_changed
graph_changes
evidence_refs
created_at
author_agent
lint_status
```

---

## 8. Project And Subproject Model

### 8.1 Problem

The current system can confuse a notes folder, a repo, a high-level project, and a sub-feature.

This is common:

```text
~/projects/engram
~/notes/engram
~/notes/ai-memory
~/notes/engram/memory-os
```

These should not automatically become unrelated projects.

### 8.2 Target Hierarchy

```text
Workspace
  Project
    Initiative
      Subproject
        Task
          Session
```

Example:

```text
Workspace: local-yuval
Project: Engram
Initiative: Memory OS
Subproject: Obsidian Vault Compiler
Task: Design knowledge commit schema
Session: 2026-04-26 planning
```

### 8.3 Project Identity Signals

Engram should infer project identity from:

```text
git root
git remote
current branch
current working directory
AGENTS.md
CLAUDE.md
package/Cargo metadata
existing Engram project records
vault path
recent sessions
user prompt
active task
```

### 8.4 Orientation Should Return Ambiguity

If identity is ambiguous, Engram should not silently create a new project.

Example response:

```json
{
  "status": "ambiguous",
  "candidates": [
    {
      "kind": "project",
      "name": "Engram",
      "confidence": 0.84,
      "reason": "cwd is inside repo with matching git remote"
    },
    {
      "kind": "subproject",
      "name": "Memory OS",
      "parent": "Engram",
      "confidence": 0.68,
      "reason": "prompt mentions memory architecture and recent notes path"
    }
  ],
  "recommended_action": "attach_to_existing_subproject"
}
```

The agent can then ask:

```text
This looks related to Engram / Memory OS. Should I attach this work there?
```

### 8.5 Rules For New Project Creation

Only create a new project when:

1. No existing project matches with confidence above threshold.
2. The user explicitly indicates a new standalone project.
3. Git remote and workspace path are new.
4. No parent project is semantically close.

Otherwise create or attach a subproject/task.

---

## 9. Repository Topology And Monorepo Awareness

### 9.1 Problem

Real projects are not always one Git repository.

Example:

```text
Project: Debug with AI
  Repo: co-gen backend
  Repo: WebUI
  Repo: dd-source
    Type: monorepo
    Relevant internal areas:
      path/to/debug-ai-service
      path/to/shared-platform-code
      path/to/frontend-or-infra-surface
```

Engram needs to remember:

- original Git remote,
- local checkout path,
- worktree paths,
- monorepo internal components,
- which subproject touches which repo/component,
- which agent/session/task modified or investigated which area,
- and whether two local folders are checkouts of the same remote repo.

Without this, a notes folder, repo folder, monorepo subdirectory, and project can be mistaken for separate projects.

### 9.2 Repository Ontology

Add repository-specific node types:

```text
GitProvider
GitRepository
LocalCheckout
GitWorktree
MonorepoComponent
Branch
Commit
PullRequest
CodeOwner
BuildTarget
Service
Package
```

Add repository-specific edge types:

```text
hosted_on
checkout_of
worktree_of
contains_component
component_of
implements
owned_by
builds
deploys
touches
depends_on
uses
modified_by
investigated_by
source_of_truth_for
```

### 9.3 GitRepository Schema

```rust
pub struct GitRepository {
    pub id: Id,
    pub name: String,
    pub provider: GitProvider,
    pub remote_url: String,
    pub normalized_remote: String,
    pub default_branch: Option<String>,
    pub repo_kind: RepoKind,
    pub owner: Option<String>,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
}

pub enum RepoKind {
    SingleProject,
    Monorepo,
    Unknown,
}
```

### 9.4 LocalCheckout Schema

```rust
pub struct LocalCheckout {
    pub id: Id,
    pub repo_id: Id,
    pub path: PathBuf,
    pub git_root: PathBuf,
    pub current_branch: Option<String>,
    pub current_head: Option<String>,
    pub is_worktree: bool,
    pub last_seen_at: OffsetDateTime,
}
```

### 9.5 MonorepoComponent Schema

```rust
pub struct MonorepoComponent {
    pub id: Id,
    pub repo_id: Id,
    pub name: String,
    pub root_path: PathBuf,
    pub component_type: ComponentType,
    pub owners: Vec<String>,
    pub build_targets: Vec<String>,
    pub tags: Vec<String>,
}
```

### 9.6 Example Graph

```text
Project: Debug with AI
  --uses_repo--> GitRepository: co-gen-backend
  --uses_repo--> GitRepository: webui
  --uses_repo--> GitRepository: dd-source

GitRepository: dd-source
  --contains_component--> MonorepoComponent: debug-ai-service
  --contains_component--> MonorepoComponent: shared-llm-platform

LocalCheckout: /Users/yuval/src/dd-source
  --checkout_of--> GitRepository: dd-source

Subproject: Debug with AI / Investigation UX
  --touches--> GitRepository: webui
  --touches--> MonorepoComponent: debug-ai-service
```

### 9.7 Repo Detection

When orienting, Engram should collect:

```bash
git rev-parse --show-toplevel
git remote -v
git branch --show-current
git rev-parse HEAD
git worktree list
```

Then normalize remotes:

```text
git@github.com:DataDog/dd-source.git
https://github.com/DataDog/dd-source
ssh://git@github.com/DataDog/dd-source.git

all become:
github.com/DataDog/dd-source
```

### 9.8 Orientation Behavior

If cwd is inside a monorepo, Engram should not only identify the repo. It should identify the relevant component.

Example:

```json
{
  "project": "Debug with AI",
  "repo": "dd-source",
  "repo_kind": "monorepo",
  "component": "debug-ai-service",
  "local_checkout": "/Users/yuval/src/dd-source",
  "confidence": 0.86
}
```

### 9.9 Migration Of Existing Repo Knowledge

Migration should cluster existing memory by:

```text
git remote
local path
repo name mentions
project name mentions
file path prefixes
branch names
PR URLs
```

Any migrated memory with unclear repo identity should be quarantined:

```text
80_Archive/Needs-Review/Repo-Identity.md
```

---

## 10. Markdown Vault Design

### 10.1 Vault Location

Default:

```text
~/.engram/vault/
```

Configurable:

```toml
[vault]
path = "/Users/yuval.meiri/notes/engram"
format = "obsidian"
```

### 10.2 Folder Structure

```text
engram-vault/
  00_Inbox/
    Untriaged.md

  10_Projects/
    Engram/
      Overview.md
      Context-Pack.md
      Decisions.md
      Rules.md
      Limitations.md
      Open-Questions.md
      Handoffs.md
      Entities.md
      Features/
        Memory-OS.md
        Multi-Session.md
        Google-Workspace-MCP.md

  20_Areas/
    AI-Coding/
      Overview.md
      Agent-Memory.md

  30_Procedures/
    Git.md
    Code-Review.md
    Testing.md
    Browser-Login.md

  40_Preferences/
    User-Profile.md
    Communication.md
    Coding-Style.md
    Tool-Use.md

  50_Knowledge/
    Concepts/
    Systems/
    Patterns/

  80_Archive/
    Projects/
    Sessions/
    Superseded-Decisions/

  90_Evidence/
    Sessions/
    Commands/
    External-Docs/
    Commits/

  95_Knowledge_Commits/
    2026/
      2026-04-26-engram-memory-os.md

  99_System/
    Ontology.md
    Lint-Rules.md
    Vault-Index.md
```

### 10.3 Markdown Page Frontmatter

Every generated page should include YAML frontmatter.

Example:

```yaml
---
engram_id: mem_01
type: preference
scope: global
status: active
confidence: high
created_at: 2026-04-26T10:00:00Z
updated_at: 2026-04-26T10:00:00Z
review_after: 2026-07-26
evidence:
  - session:2026-04-26-memory-os
tags:
  - engram/preference
  - user/communication
---
```

### 10.4 User Preference Page Example

```markdown
# Communication Preferences

## Active Preferences

### Avoid AI-branded commit messages

The user does not want commit messages or summaries that reference the AI
agent's name as the author or brand.

- Scope: global coding work
- Confidence: high
- Evidence: [[2026-04-26-memory-os-session]]
- Last used: 2026-04-26
- Review after: 2026-07-26

### Be explicit about blockers

When a requested action cannot be completed, state the blocker directly and
do not imply success.

- Scope: global
- Confidence: high
- Evidence: Google Docs publishing blocker during Engram analysis
```

### 10.5 Project Context Pack Example

```markdown
# Context Pack: Engram / Memory OS

Generated: 2026-04-26T12:00:00Z

## Current Focus

Design Engram Memory OS: an Obsidian-compatible, source-grounded memory layer
with knowledge commits, graph traversal, linting, and automatic orientation.

## Active Decisions

- Extend Engram rather than rebuild from scratch.
- Markdown vault is durable memory.
- SurrealDB is the graph/index layer.
- MCP is the agent interface.
- Lint is mandatory for memory quality.

## Relevant Preferences

- Do not require the user to remind the agent to use Engram.
- Prefer direct, implementation-oriented plans.
- Record blockers explicitly.

## Known Limitations

- Current Engram graph supports single-hop entity relationships but not
  graph browsing or multi-hop traversal.
- Current multi-session tests have known failures.

## Open Questions

- Should the vault live under `~/.engram/vault` or user-selected notes folder?
- Should generated pages be overwritten directly or updated through patch blocks?
```

### 10.6 Generated Vs User-Edited Sections

Pages should support safe update markers:

```markdown
# Rules

<!-- engram:generated:start rules-active -->
Generated content here.
<!-- engram:generated:end rules-active -->

## User Notes

Human edits here. Engram must not overwrite this section.
```

This allows the user to edit Obsidian pages without Engram destroying manual notes.

---

## 11. Knowledge Commits

### 11.1 Concept

Knowledge commits are Git-like commits for memory.

They answer:

```text
What did we learn during this session?
What changed in the project understanding?
What was superseded?
What evidence supports the change?
Which vault pages changed?
Which graph edges changed?
```

### 11.2 Difference From Session Logs

Session logs are raw evidence:

```text
At 10:31, command ran.
At 10:35, user corrected preference.
At 10:42, agent found internal docs.
```

Knowledge commits are distilled diffs:

```text
Added preference:
  User wants Engram orientation to be automatic, not manual.

Changed decision:
  "resume session" should become internal orientation behavior.

Added limitation:
  Google Workspace MCP requires OAuth reset when stale token appears.
```

### 11.3 Knowledge Commit Markdown

```markdown
# Knowledge Commit: Engram Memory OS Direction

Date: 2026-04-26
Project: [[Engram]]
Scope: [[Memory OS]]
Parent: [[2026-04-25-engram-project-analysis]]

## Summary

Reframed Engram from an MCP memory database into a local-first memory OS
with an Obsidian-readable vault, knowledge commits, graph ontology, and lint.

## Added

- Preference: Agent should orient automatically through Engram.
- Decision: Extend Engram rather than rebuild from scratch.
- Architecture: Markdown vault is durable memory; DB is index/graph.

## Changed

- Session resume is no longer a user-facing primitive; it becomes orient.
- RAG is discovery support, not the primary memory surface.

## Superseded

- "Manual start/resume/end is enough" is superseded by live orientation
  and session-following brain.

## Evidence

- [[session-2026-04-26-memory-os-discussion]]
- [[engram-project-book-2026-04-25]]

## Vault Files Changed

- [[10_Projects/Engram/Features/Memory-OS]]
- [[40_Preferences/Communication]]
- [[99_System/Ontology]]
```

### 11.4 Storage Model

Store in DB and vault.

DB:

```text
knowledge_commit
knowledge_commit_change
knowledge_commit_evidence
```

Vault:

```text
95_Knowledge_Commits/YYYY/YYYY-MM-DD-title.md
```

### 11.5 Diff Types

```text
added_fact
changed_fact
superseded_fact
archived_fact
added_decision
changed_rule
added_preference
updated_preference
resolved_question
added_question
graph_edge_added
graph_edge_removed
vault_page_changed
```

---

## 12. Graph Structure

### 12.1 Current Engram Graph

Current Engram has:

- entities as nodes,
- relationships as directed edges,
- relationship types such as `depends_on`, `uses`, `owned_by`, `documents`, and `related_to`,
- incoming/outgoing single-hop retrieval.

Current limitation:

- no graph browser,
- no multi-hop traversal MCP tool,
- no graph lint,
- project/task entity connections are separate from the entity graph,
- relationships do not yet drive orientation enough.

### 12.2 Target Graph

Graph should become the memory backbone.

```text
               +----------------+
               | User           |
               +-------+--------+
                       |
                       | has_preference
                       v
               +-------+--------+
               | Preference     |
               +-------+--------+
                       |
                       | applies_to
                       v
+------------+  parent_of   +------------+  contains  +------------+
| Workspace  +-------------> | Project    +----------> | Subproject |
+------------+              +-----+------+            +-----+------+
                                  |                         |
                                  | has_decision            | has_task
                                  v                         v
                            +-----+------+            +-----+------+
                            | Decision   |            | Task       |
                            +-----+------+            +-----+------+
                                  |                         |
                                  | evidenced_by            | touches
                                  v                         v
                            +-----+------+            +-----+------+
                            | Evidence   |            | Entity     |
                            +------------+            +------------+
```

### 12.3 Graph Node Tables

Initial tables:

```text
memory_node
memory_edge
memory_fact
memory_preference
memory_decision
memory_rule
memory_limitation
memory_question
memory_evidence
knowledge_commit
vault_page
```

Alternative:

Use typed tables for each node. This gives stronger validation but more migrations.

Recommendation:

Start with typed domain tables for high-value concepts and a general `memory_edge` table for relationships.

### 12.4 Memory Edge Schema

```rust
pub struct MemoryEdge {
    pub id: Id,
    pub source_id: Id,
    pub source_type: MemoryNodeType,
    pub target_id: Id,
    pub target_type: MemoryNodeType,
    pub relation: MemoryRelation,
    pub scope_id: Option<Id>,
    pub evidence_refs: Vec<Id>,
    pub confidence: Confidence,
    pub status: MemoryStatus,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
}
```

### 12.5 Graph Compilation

Graph compilation is the process of turning raw evidence and Markdown into nodes and edges.

Inputs:

```text
session events
user corrections
tool results
files touched
docs indexed
vault pages
git commits
manual notes
```

Compiler stages:

```text
1. Parse evidence
2. Extract candidate memories
3. Classify by ontology type
4. Resolve entities
5. Assign scope
6. Create or update nodes
7. Create or update edges
8. Write knowledge commit
9. Update vault pages
10. Run lint
```

### 12.6 Entity Resolution

Entity resolution should prevent duplicates.

Signals:

```text
exact name
aliases
git remote
file path
URL
semantic similarity
parent project
entity type
recent usage
```

Example:

```text
"Google Workspace MCP"
"datadog-google-workspace"
"GWS MCP"

Same canonical entity:
  Entity: Google Workspace MCP Server
```

### 12.7 Project/Subproject Edges

Example:

```text
Engram --contains--> Memory OS
Memory OS --contains--> Obsidian Vault Compiler
Memory OS --has_decision--> Extend Engram Rather Than Rebuild
Obsidian Vault Compiler --modifies--> Vault Pages
User --has_preference--> Avoid Manual Resume Ritual
Preference --applies_to--> AI Coding Agents
```

### 12.8 Supersession Edges

Example:

```text
Decision: Use SurrealDB server mode
  --superseded_by-->
Decision: Use Engram HTTP daemon + stdio proxy
```

This lets retrieval avoid stale active context while preserving history.

### 12.9 Graph Traversal Examples

Find current context for task:

```text
Task
  -> parent subproject
  -> parent project
  -> active decisions
  -> active rules
  -> limitations
  -> relevant preferences
  -> recent knowledge commits
```

Find what a file is connected to:

```text
CodeFile
  <- touches - Task
  <- modifies - KnowledgeCommit
  <- applies_to - Decision
```

Find stale docs:

```text
SourceDocument
  -> documents Entity
  <- modified_by CodeCommit
  -> older_than active implementation evidence
```

---

## 13. Retrieval Architecture

### 13.1 Retrieval Should Be Hybrid

Use multiple retrieval paths:

```text
scope retrieval
keyword search
vector search
graph traversal
recency search
knowledge commit diff search
vault page lookup
```

### 13.2 Orientation Retrieval

`orient` should not simply search the prompt.

It should build context in layers:

```text
1. Identify workspace/project/task
2. Load project context pack
3. Load active subproject page
4. Load active decisions/rules/limitations
5. Load relevant preferences
6. Load recent knowledge commits
7. Search for prompt-specific facts
8. Rerank and trim
9. Return context pack and citations
```

### 13.3 Candidate Generation

Candidate sources:

```text
exact project match
active task
recent sessions
vault page frontmatter
graph neighbors
keyword matches
embedding matches
open questions
lint warnings
known limitations
```

### 13.4 Reranking

Initial deterministic score:

```text
score =
  semantic_similarity * 0.25
  + keyword_score * 0.15
  + scope_match * 0.25
  + graph_distance_score * 0.15
  + recency_score * 0.10
  + confidence_score * 0.05
  + active_status_score * 0.05
```

Hard filters:

```text
exclude superseded unless specifically asking history
exclude archived unless needed
exclude sensitive memories unless allowed by task type
exclude low-confidence preferences from hard rules
```

### 13.5 Reranking With LLM

Optional second-stage rerank:

```text
Input:
  user prompt
  project identity
  candidate memories with source and scope

Output:
  include
  exclude
  reason
  priority
```

This should be optional because it adds latency and model dependency.

### 13.6 Context Pack Format

MCP `orient` response:

```json
{
  "project": "Engram",
  "scope": "Engram / Memory OS",
  "confidence": 0.91,
  "context_pack_path": "/Users/.../Context-Pack.md",
  "summary": "...",
  "active_decisions": [],
  "active_rules": [],
  "preferences": [],
  "limitations": [],
  "open_questions": [],
  "recent_knowledge_commits": [],
  "recommended_actions": [],
  "ambiguities": []
}
```

---

## 14. Session-Following Brain

### 14.1 Remove Manual Resume As A User Ritual

Users should not need to say "resume session."

Every meaningful prompt should trigger:

```text
orient(cwd, prompt, agent, recent_files)
```

The agent should ask Engram:

```text
Where am I?
What do we know?
What changed recently?
What rules apply?
What should I avoid?
```

### 14.2 Session Lifecycle

Keep sessions internally, but change user-facing behavior.

```text
Implicit start:
  first orient call creates or attaches active session

During work:
  event stream accumulates evidence

Periodic distill:
  every N tool events, major user correction, or task boundary

Explicit end optional:
  writes final handoff and knowledge commit

Idle close:
  stale sessions are closed automatically with partial handoff
```

### 14.3 Event Capture

Event types:

```text
user_prompt
assistant_plan
tool_call
tool_result
file_read
file_write
command_run
test_result
error
decision_candidate
preference_candidate
rule_candidate
limitation_candidate
handoff_update
```

### 14.4 What To Record

Do not save everything as durable memory.

Record raw events cheaply. Promote only useful items.

Promotion triggers:

```text
user says "remember", "do not", "I prefer", "we decided"
agent changes approach due to discovered limitation
test failure reveals project constraint
external doc resolves a blocker
manual correction to style/workflow
new project/subproject identity inferred
decision affects future work
tool behavior is reusable
```

### 14.5 Rolling Handoff

Instead of handoff only at the end, maintain a rolling handoff.

```markdown
# Active Handoff: Engram / Memory OS

## Current State

Designing Memory OS architecture and implementation plan.

## Decisions So Far

- Extend Engram rather than rebuild.
- Add Markdown vault as durable memory.
- Add knowledge commits for memory diffs.

## Next Actions

- Implement orientation service.
- Implement vault compiler.
- Implement lint MVP.

## Open Questions

- Vault default path.
- Whether generated sections should be patch-only or full page rewrites.
```

---

## 15. Multi-Writer Provenance And Session-Time Memory Awareness

### 15.1 Problem

Different models and harnesses may write to the same memory system.

Examples:

```text
Agent A: Claude Code explores the backend failure.
Agent B: Codex analyzes the frontend state.
Agent C: ChatGPT reviews the design plan.
```

They may be:

- using different harnesses,
- using different models,
- working from different directories,
- or writing to the same memory surface.

Memory OS must know who wrote what and when. It does not need to become a collaboration platform in v1.

### 15.2 First Principle

Separate memory provenance from agent collaboration.

The required v1 behavior is:

```text
Claude Code wrote/explored these facts.
Codex wrote/explored these facts.
ChatGPT contributed these analysis notes.
This memory was user-stated.
This memory was tool-observed.
This memory was agent-inferred.
```

The system should not try to coordinate agents like a team chat or task delegation platform yet.

### 15.3 Writer Provenance

Every memory item, event, evidence record, and knowledge commit should record writer provenance.

```rust
pub struct WriterProvenance {
    pub actor_type: ActorType,
    pub harness: Option<String>,
    pub model: Option<String>,
    pub surface: Option<String>,
    pub session_id: Option<Id>,
    pub workspace_id: Option<Id>,
    pub origin: ClaimOrigin,
    pub created_at: OffsetDateTime,
}
```

Actor types:

```text
user
agent
tool
system
migration
```

Harness examples:

```text
claude-code
codex
chatgpt
cursor
unknown
```

Origin examples:

```text
user_stated
user_corrected_agent
agent_inferred
tool_observed
code_observed
document_observed
migrated
lint_generated
```

This is enough to distinguish:

```text
Claude Code found this in dd-source.
Codex generated this frontend implementation note.
ChatGPT summarized this design tradeoff.
The user explicitly corrected this preference.
```

### 15.4 Trust And Retrieval Implications

Writer provenance should influence retrieval and lint.

Trust order:

```text
highest:
  user_stated
  user_corrected_agent
  tool_observed
  code_observed

medium:
  document_observed
  agent_distilled_with_evidence

lower:
  agent_inferred_without_evidence
  migrated_unsourced
```

Retrieval should show source attribution:

```text
Preference: Avoid AI-branded commit messages
Origin: user_corrected_agent
Written by: Claude Code / unknown model
Scope: global coding work
Confidence: high
```

### 15.5 Session-Time Memory Change Awareness

The important multi-writer problem is not collaboration. It is awareness.

After an agent receives its initial context pack, another agent may write a relevant memory item during the same session.

Engram should support:

```text
memory changes since orientation
memory changes relevant to current project/scope
memory changes written by other harnesses
memory changes that supersede loaded context
memory changes that contradict loaded context
```

### 15.6 Memory Cursor

Every `orient` response should include a memory cursor.

```json
{
  "context_pack_path": "...",
  "memory_cursor": "kc_2026_04_26_00042"
}
```

During a session, the agent can ask:

```json
{
  "action": "changes_since",
  "cursor": "kc_2026_04_26_00042",
  "project": "Engram",
  "scope": "Memory OS"
}
```

Response:

```json
{
  "changes": [
    {
      "type": "new_limitation",
      "title": "Claude Code Google Workspace MCP has stale OAuth token",
      "writer": {
        "harness": "claude-code",
        "model": "claude-opus-4.7"
      },
      "relevance": 0.81,
      "reason": "Current session discusses Google Workspace publishing"
    }
  ],
  "new_cursor": "kc_2026_04_26_00044"
}
```

### 15.7 Notification Model

Start with polling, not push.

Agent behavior:

```text
1. orient at task start
2. receive memory_cursor
3. before major decisions, call memory changes_since cursor
4. before final answer/handoff, call changes_since cursor
5. if relevant changes exist, incorporate or mention them
```

Do not require real-time streaming in v1.

### 15.8 Relevance Scoring For New Memory

A new memory item is relevant to an active session if it matches:

```text
same project
same subproject
same repo/component
same task
same file path
same entity
same decision/rule/limitation
semantic similarity to current prompt
supersedes an item already loaded in context pack
contradicts an item already loaded in context pack
```

### 15.9 Why Not Full A2A First?

Google's Agent2Agent protocol is an open protocol for communication and interoperability between opaque agentic applications. It is valuable when agents are independent applications, often remote, potentially built with different frameworks, and need formal contracts, discovery, authentication, streaming, and long-running task collaboration.

References:

```text
A2A project: https://github.com/a2aproject/A2A
A2A specification: https://a2a-protocol.org/latest/specification/
Google ADK A2A guidance: https://adk.dev/a2a/intro/
```

That is not the first problem for Engram.

For Engram v1, the first problem is:

```text
How do agents know which memory was written by which harness/model,
and whether memory changed after they oriented?
```

### 15.10 When A2A Becomes Worth It

Add A2A or A2A-inspired support when:

```text
agents run as independent long-lived services
agents are maintained by different teams
agents run across machines
agents use different frameworks/languages
formal capability discovery is needed
delegation crosses trust boundaries
network authentication/authorization matters
```

Do not use A2A for:

```text
two local Claude/Codex sessions sharing one Engram daemon
simple helper agents inside one harness
basic memory provenance
checking memory changes since orientation
```

### 15.11 Future Full A2A Bridge

If needed later, Engram can expose an A2A-compatible memory agent:

```text
Engram Memory Agent
  capabilities:
    orient
    retrieve_context
    get_memory_changes_since
    query_graph
    create_knowledge_commit
```

This lets external agents use Engram through A2A while local agents continue using MCP.

Recommended order:

```text
1. Add writer provenance.
2. Add memory_cursor and changes_since.
3. Add relevance scoring for newly written memory.
4. Add A2A-compatible facade only if independent remote agents need it.
```

---

## 16. Lint And Recalibration

### 16.1 Purpose

Lint prevents memory rot.

It should find:

- contradictions,
- stale facts,
- missing evidence,
- orphan nodes,
- duplicate entities,
- over-broad preferences,
- unused old preferences,
- unresolved questions,
- project hierarchy mistakes,
- docs that drift from code,
- old active sessions.

### 16.2 Lint Finding Schema

```rust
pub struct LintFinding {
    pub id: Id,
    pub severity: Severity,
    pub finding_type: LintFindingType,
    pub scope_id: Option<Id>,
    pub subject_id: Option<Id>,
    pub title: String,
    pub detail: String,
    pub evidence_refs: Vec<Id>,
    pub suggested_fix: Option<String>,
    pub auto_fixable: bool,
    pub status: LintStatus,
    pub created_at: OffsetDateTime,
}
```

### 16.3 Lint Rules MVP

```text
missing_evidence
stale_preference
contradictory_fact
superseded_loaded_as_active
orphan_entity
duplicate_entity_candidate
weak_related_to_overuse
subproject_without_parent
project_without_context_pack
task_without_entity_links
active_session_stale
handoff_missing_next_actions
vault_page_missing_frontmatter
generated_section_modified
```

### 16.4 Recalibration

Some memories should be periodically reviewed.

Example:

```text
Preference:
  "User allows browser login when OAuth is needed"

Review:
  after 90 days
  or after 5 relevant tasks where it was not used
```

Agent prompt:

```text
You previously allowed browser login for OAuth flows. I have not used that
preference recently. Should I keep it as a global preference, narrow it to
Google Workspace auth, or retire it?
```

### 16.5 Auto-Fix Policy

Safe auto-fixes:

- close stale active sessions,
- regenerate context packs,
- update `last_used_at`,
- add missing backlinks when evidence is unambiguous,
- move old closed projects to archive,
- merge exact duplicate aliases.

Human-reviewed fixes:

- factual contradictions,
- preference changes,
- decision supersession,
- project hierarchy changes,
- deletion of memories.

---

## 17. Archival Memory

### 17.1 Archiving Is Not Deleting

Archived memory remains searchable but not loaded by default.

Archive candidates:

```text
closed projects
superseded decisions
old sessions
inactive subprojects
outdated context packs
resolved limitations
stale preferences
```

### 17.2 Archive Metadata

```text
archived_at
archive_reason
archive_scope
retrieval_policy
summary
retained_facts
evidence_refs
```

### 17.3 Archive Page Example

```markdown
# Archived Project: Google Workspace MCP Import Experiment

Status: archived
Archived: 2026-04-26

## Retained Facts

- Remote Google Workspace MCP supports Docs writes.
- Claude Code stale OAuth cache can cause `401 invalid_token`.
- Recommended reset: clear `~/.mcp-auth`, remove old config, re-add server.

## Why Archived

The immediate publishing task is blocked pending user re-authentication.

## Reopen If

- User authenticates the remote MCP.
- Codex exposes write-capable Google Docs tools.
```

---

## 18. MCP API Design

### 18.1 New Tools

Add tools:

```text
orient
repo
memory
memory_commit
vault
graph
lint
handoff
```

### 18.2 `orient`

Purpose:

Return context for the current user request.

Request:

```json
{
  "cwd": "/Users/yuval.meiri/projects/engram",
  "prompt": "write implementation plan",
  "agent": "codex",
  "include_recent_commits": true
}
```

Response:

```json
{
  "project": "Engram",
  "scope": "Engram / Memory OS",
  "context_pack": "...",
  "brain_loop": {
    "compiled_context": "...",
    "top_items": [],
    "degraded": false
  },
  "active_decisions": [],
  "active_rules": [],
  "preferences": [],
  "limitations": [],
  "review_needed": [],
  "memory_cursor": {
    "timestamp": "...",
    "commit_id": "..."
  },
  "ambiguities": []
}
```

### 18.3 `repo`

Actions:

```text
detect
register
list
get
connect_project
connect_component
local_checkouts
monorepo_components
```

Purpose:

Track Git remotes, local checkouts, worktrees, and monorepo components so project orientation is not confused by folder layout.

Request:

```json
{
  "action": "detect",
  "cwd": "/Users/yuval/src/dd-source"
}
```

Response:

```json
{
  "repo": "github.com/DataDog/dd-source",
  "repo_kind": "monorepo",
  "local_checkout": "/Users/yuval/src/dd-source",
  "branch": "feature/debug-ai",
  "head": "abc123",
  "components": [
    {
      "name": "debug-ai-service",
      "root_path": "path/to/debug-ai-service"
    }
  ]
}
```

### 18.4 `memory`

Actions:

```text
add
get
list
search
update
archive
supersede
link
unlink
recalibrate
changes_since
```

### 18.5 `memory_commit`

Actions:

```text
create
get
list
diff
log
rollback_compiled_state
```

Rollback should not erase raw evidence. It should create a new commit that reverts compiled memory.

### 18.6 `vault`

Actions:

```text
init
compile
sync
status
page
context_pack
validate
```

### 18.7 `graph`

Actions:

```text
neighbors
traverse
path
subgraph
connect
disconnect
stats
lint
export
```

Example:

```json
{
  "action": "traverse",
  "start": "Engram",
  "depth": 2,
  "relations": ["contains", "has_decision", "has_rule", "has_limitation"]
}
```

### 18.8 `lint`

Actions:

```text
run
list
get
resolve
ignore
stats
```

### 18.9 `handoff`

Actions:

```text
get
update
write
finalize
```

---

## 19. CLI Design

### 19.1 Commands

```bash
engram orient
engram orient --prompt "fix multi-session tests"

engram repo detect
engram repo register --remote git@github.com:DataDog/dd-source.git --kind monorepo
engram repo component add dd-source debug-ai-service --path path/to/debug-ai-service
engram repo connect-project "Debug with AI" dd-source

engram vault init
engram vault compile
engram vault status
engram vault page "Engram/Memory OS"

engram memory add preference ...
engram memory search "commit message style"
engram memory archive --project old-project

engram graph around "Google Workspace MCP" --depth 2
engram graph path "User Preference" "Git Procedure"
engram graph lint

engram memory changes-since <cursor> --project Engram
engram memory writer-stats --project Engram

engram lint run --project Engram
engram lint list --severity high
engram lint resolve <finding-id>

engram commit-memory --session current
engram memory-log --project Engram
engram memory-diff <commit-a> <commit-b>
```

### 19.2 Human Workflow

```bash
cd ~/projects/engram
engram orient
engram graph around Engram --depth 2
engram lint run --project Engram
engram vault compile
```

### 19.3 Agent Workflow

At start of meaningful task:

```text
call orient
read context pack
continue work
```

During work:

```text
record raw events
record candidate durable memories
update rolling handoff
```

At boundary:

```text
distill
create knowledge commit
compile vault
run lint
```

---

## 20. Implementation Plan

### Phase 0: Stabilize Existing Base

Goal:

Make current Engram reliable enough to build on.

Tasks:

1. Fix `cargo clippy --all-targets -- -D warnings`.
2. Fix multi-session daemon/proxy tests.
3. Update stale multi-session docs.
4. Add current MCP tool reference.
5. Add smoke test for active Engram MCP capability surface.

Exit criteria:

```text
cargo fmt --all --check passes
cargo clippy --all-targets -- -D warnings passes
cargo test --all-targets --no-fail-fast passes except intentional ignored embedding tests
```

### Phase 1: Memory Ontology MVP

Goal:

Add typed memory objects without changing the whole product.

Tasks:

1. Add core types:
   - `MemoryItem`
   - `MemoryType`
   - `MemoryScope`
   - `MemoryStatus`
   - `EvidenceRef`
   - `Preference`
   - `Decision`
   - `Rule`
   - `Limitation`
   - `OpenQuestion`
2. Add `WriterProvenance` and `ClaimOrigin`.
3. Add store repos.
4. Add service methods.
5. Add MCP `memory` tool.
6. Add CLI `memory` commands.
7. Add tests.

Exit criteria:

The system can store and retrieve typed preferences, decisions, rules, limitations, and questions with scope, evidence, and writer provenance.

### Phase 2: Vault MVP

Goal:

Create Obsidian-readable Markdown pages from memory.

Tasks:

1. Add vault config.
2. Add vault initializer.
3. Add page templates.
4. Add generated section markers.
5. Add project overview page compiler.
6. Add preference/rule/decision pages.
7. Add context pack compiler.
8. Add vault status command.

Exit criteria:

Running:

```bash
engram vault init
engram vault compile --project Engram
```

creates a readable Markdown vault with project pages and context pack.

### Phase 3: Orientation

Goal:

Remove manual resume as the main user ritual.

Tasks:

1. Add `OrientationService`.
2. Implement project identity detection.
3. Implement repository detection from Git root, remote, branch, worktree, and HEAD.
4. Add repository topology records for Git repositories, local checkouts, and monorepo components.
5. Connect projects/subprojects/tasks to repositories and monorepo components.
6. Implement ambiguity response.
7. Implement context pack retrieval.
8. Add MCP `orient` and `repo`.
9. Add CLI `orient` and `repo`.
10. Add agent instructions for automatic orientation.

Exit criteria:

Given cwd and prompt, Engram returns project, scope, relevant memory, relevant Git repo/local checkout/component, and context pack path.

### Phase 4: Knowledge Commits

Goal:

Track memory evolution like Git.

Tasks:

1. Add `KnowledgeCommit` domain type.
2. Add commit changes table.
3. Add memory diff builder.
4. Add vault commit page writer.
5. Add memory cursor generation.
6. Add `changes_since` query for session-time memory awareness.
7. Add CLI `memory-log`, `memory-diff`, and `memory changes-since`.
8. Add MCP `memory_commit` and `memory(action=changes_since)`.

Exit criteria:

After a session distillation, Engram can show what knowledge changed since the previous commit. After orientation, an agent can query what relevant memory changed since its initial context pack.

### Phase 5: Live Session Distillation

Goal:

Follow the session and promote useful events.

Tasks:

1. Add raw event stream.
2. Add event capture MCP endpoint.
3. Add rolling handoff file.
4. Add candidate memory extractor.
5. Add human confirmation mode for high-impact memories.
6. Add periodic distillation.

Exit criteria:

Engram can produce a handoff and candidate knowledge commit from session events without relying on manual logging every few prompts.

### Phase 6: Graph Upgrade

Goal:

Make the graph behaviorally useful.

Tasks:

1. Add general `memory_edge`.
2. Link projects, subprojects, tasks, entities, decisions, rules, and evidence.
3. Add graph traversal service.
4. Add MCP `graph`.
5. Add CLI `graph`.
6. Add Obsidian graph-friendly backlinks.
7. Add graph export.

Exit criteria:

Engram can answer:

```text
What decisions apply to this subproject?
What preferences apply to this task?
What limitations are connected to this tool?
What changed because of this knowledge commit?
```

### Phase 7: Multi-Writer Awareness

Goal:

Let sessions notice relevant memory written by other harnesses/models after their initial orientation.

Tasks:

1. Add writer/harness/model filters to memory queries.
2. Add `changes_since` relevance scoring.
3. Add checks for memory that supersedes or contradicts context loaded at orientation.
4. Add agent guidance to poll changes before major decisions and final responses.
5. Add CLI `memory writer-stats`.
6. Add tests for cross-harness memory writes and changes-since behavior.

Exit criteria:

If Claude Code writes a relevant memory item while Codex is in-session, Codex can discover it with `changes_since` and decide whether to incorporate it.

### Phase 8: Lint And Recalibration

Goal:

Keep memory healthy.

Tasks:

1. Add lint finding types.
2. Add lint runner.
3. Add core lint rules.
4. Add recalibration scheduling.
5. Add safe auto-fixes.
6. Add MCP/CLI lint APIs.
7. Add vault lint report page.

Exit criteria:

Engram can report stale preferences, missing evidence, duplicate entities, orphan subprojects, contradictions, and stale sessions.

### Phase 9: Archival Memory

Goal:

Keep old memory accessible but out of the default context.

Tasks:

1. Add archive status and archive reason.
2. Add archive policies.
3. Add archive pages.
4. Add retrieval rules for archived memory.
5. Add archive search.

Exit criteria:

Closed projects and superseded decisions move out of default context while remaining discoverable.

### Phase 10: Retrieval Reranking

Goal:

Improve context quality.

Tasks:

1. Implement deterministic scoring.
2. Add scope-aware ranking.
3. Add graph distance scoring.
4. Add freshness/supersession filtering.
5. Add optional LLM rerank hook.
6. Add retrieval evaluation tests.

Exit criteria:

Orientation returns fewer irrelevant memories and avoids stale/superseded items by default.

---

## 21. Example End-To-End Flow

### 21.1 User Prompt

```text
Let's design the Engram graph model for the memory OS.
```

### 21.2 Agent Calls Orient

```json
{
  "cwd": "/Users/yuval.meiri/projects/engram",
  "prompt": "Let's design the Engram graph model for the memory OS.",
  "agent": "codex"
}
```

### 21.3 Engram Returns Context

```json
{
  "project": "Engram",
  "scope": "Memory OS",
  "active_decisions": [
    "Extend Engram rather than rebuild",
    "Markdown vault is durable memory",
    "Graph should drive orientation and lint"
  ],
  "preferences": [
    "User wants comprehensive implementation plans",
    "User dislikes having to remind agent to use Engram"
  ],
  "limitations": [
    "Current graph is single-hop and not browsable"
  ],
  "recent_knowledge_commits": [
    "2026-04-26-memory-os-direction"
  ],
  "context_pack_path": "~/.engram/vault/10_Projects/Engram/Context-Pack.md"
}
```

### 21.4 Work Happens

Raw events:

```text
user_prompt
file_read docs/MEMORY_OS_IMPLEMENTATION_PLAN.md
decision_candidate "Graph should use typed nodes plus memory_edge"
preference_candidate "User wants ambiguity called out explicitly"
```

### 21.5 Distillation

Generated memory:

```text
Decision:
  Use typed domain tables plus general memory_edge table.

Preference:
  User wants uncertainty/ambiguity surfaced before implementation.

OpenQuestion:
  Should vault default to ~/.engram/vault or user's existing notes folder?
```

### 21.6 Knowledge Commit

```text
Knowledge commit:
  Added graph schema decision.
  Added vault path open question.
  Updated Memory OS implementation page.
```

### 21.7 Vault Update

Files updated:

```text
10_Projects/Engram/Features/Memory-OS.md
10_Projects/Engram/Decisions.md
40_Preferences/Communication.md
95_Knowledge_Commits/2026/2026-04-26-graph-schema.md
```

---

## 22. Migration Strategy

### 22.1 Existing Data

Existing Engram data should be migrated into the new model.

Mapping:

```text
Entity -> Entity node
Relationship -> memory_edge or entity relationship
Session -> Session + Evidence
Event -> Evidence event
Observation -> MemoryItem candidate
Project -> Project node
Task -> Task node
PR -> ExternalReference / WorkArtifact
KnowledgeDoc -> SourceDocument
ToolUsage -> Episodic event + Tool memory
Git remote/local path references -> GitRepository + LocalCheckout
Monorepo path references -> MonorepoComponent candidates
```

### 22.2 Observation Migration

Existing observations are mixed-type. Migrate conservatively.

Example:

```text
key starts with decisions.*
  -> Decision candidate

key starts with patterns.*
  -> Rule or Procedure candidate

key starts with gotchas.*
  -> Limitation candidate

key starts with status.*
  -> Project status memory
```

Mark migrated items:

```text
status: active
confidence: medium
source: migrated_observation
needs_review: true
```

### 22.3 Repository Migration

Existing Engram memory may mention projects by repo name, local folder, product name, branch, or monorepo path. Migration must normalize these before promoting memories.

Migration steps:

```text
1. Scan all project paths, session cwd values, file refs, PR URLs, and observations.
2. Extract candidate Git remotes and local checkout paths.
3. Normalize remote URLs.
4. Cluster checkouts that point to the same remote.
5. Detect monorepos by size, known names, repeated path prefixes, or user-provided config.
6. Create GitRepository and LocalCheckout records.
7. Create MonorepoComponent candidates from recurring path prefixes.
8. Ask user to review ambiguous repo/project mappings.
9. Create a repository migration knowledge commit.
```

Example review batch:

```text
Project: Debug with AI

Candidate repos:
  - co-gen-backend: remote unknown, local path /Users/yuval/...
  - webui: remote unknown, local path /Users/yuval/...
  - dd-source: github.com/DataDog/dd-source, monorepo

Candidate dd-source components:
  - debug-ai-service
  - shared-llm-platform

Questions:
  1. Which local paths are canonical?
  2. Which dd-source components belong to this project?
  3. Should old mentions of "Debug with AI backend" map to co-gen-backend?
```

### 22.4 Digest Source Migration

Scheduled daily digests under `~/notes` are high-value but sensitive migration
sources. They should not be bulk-imported directly into active memory.

Digest migration steps:

```text
1. Inventory digest-like files by path and metadata only.
2. Export a generated digest source review batch.
3. Review each source as accept, source_only, quarantine, or reject.
4. Read only accepted sources into generated candidate-memory review pages.
5. Review each candidate memory as accept, quarantine, or reject.
6. Apply accepted candidates into Memory OS with writer provenance and source evidence.
7. Create a knowledge commit for written digest-derived memories.
8. Skip duplicates using stable digest extraction candidate tags.
```

Current command shape:

```bash
engram digest review-export ~/notes /tmp/engram-digest-review
engram digest source-index /tmp/engram-digest-review
engram digest source-index /tmp/engram-digest-review --write
engram digest extraction-plan /tmp/engram-digest-review /tmp/engram-digest-extraction
engram memory digest-extraction-apply /tmp/engram-digest-extraction
engram memory digest-extraction-apply /tmp/engram-digest-extraction --write
```

Safety rules:

```text
- Inventory and source-review export do not read digest contents.
- Source indexing reads only `source_only` decisions and defaults to dry-run.
- Extraction reads only `accept` decisions.
- quarantine, reject, and pending source decisions are not read.
- Extraction creates review pages only; it does not write active memory.
- Source indexing writes document evidence only; it does not write active memory.
- Candidate memory apply defaults to dry-run.
- Accepted candidate pages must include memory_kind, scope_type, scope_name when required, and source evidence.
- Applied memories are marked active because the candidate itself passed human review.
```

### 22.5 Vault Bootstrap

Bootstrap steps:

```bash
engram vault init
engram memory migrate-observations --project Engram
engram vault compile --project Engram
engram lint run --project Engram
```

---

## 23. Risks And Tradeoffs

### 23.1 Risk: Memory Becomes Too Intrusive

If the agent asks too many recalibration questions, users will disable it.

Mitigation:

- batch recalibration questions,
- ask only for high-impact memories,
- silently downgrade low-value memories,
- expose weekly memory review.

### 23.2 Risk: Generated Markdown Conflicts With Human Edits

Mitigation:

- generated section markers,
- patch-based updates,
- human-only sections,
- vault lint,
- backups before write.

### 23.3 Risk: Sensitive User Preferences Are Overused

Mitigation:

- sensitivity field,
- task-type filters,
- never quote sensitive emotional context unless needed,
- distill into operational guidance.

### 23.4 Risk: Graph Complexity Bloats Implementation

Mitigation:

- start with typed nodes and generic edges,
- implement single useful traversal first,
- use Obsidian for visualization before building UI.

### 23.5 Risk: LLM Distillation Hallucinates Memory

Mitigation:

- require evidence refs,
- mark confidence,
- lint missing provenance,
- human confirmation for high-impact changes.

### 23.6 Risk: Extending Engram Is Slower Than A New Prototype

Mitigation:

- prototype Memory OS behind feature flags,
- write vault compiler first,
- do not refactor old tools until new flow proves value.

### 23.7 Risk: Repository Topology Is Wrong

If Engram maps a local checkout or monorepo component to the wrong project, agents will load incorrect context and write memory to the wrong place.

Mitigation:

- normalize Git remotes,
- track local checkouts separately from canonical repositories,
- quarantine ambiguous repo mappings,
- require user review for monorepo component assignment,
- include repo/component identity in every orientation response.

### 23.8 Risk: Multi-Writer Awareness Becomes Accidental Collaboration

There is a temptation to turn writer provenance into a full coordination or delegation system. That would add complexity before the memory system needs it.

Mitigation:

- keep v1 focused on writer provenance,
- add memory cursor and changes-since polling,
- avoid work claims, agent chat, and A2A until explicitly needed,
- make source attribution visible in retrieval results.

---

## 24. Ambiguities And Questions

These need user/product decisions.

1. Vault location:
   - default `~/.engram/vault`,
   - existing `~/notes`,
   - per-project vaults,
   - or configurable all of the above?

2. Generated page ownership:
   - Can Engram rewrite whole pages?
   - Or only generated marked sections?

3. Agent autonomy:
   - Should agents always call `orient` automatically?
   - Should this be enforced by AGENTS.md/CLAUDE.md instructions?

4. Privacy:
   - Which user preferences are allowed in global memory?
   - Should sensitive memories require explicit confirmation?

5. Distillation:
   - Should distillation require an LLM call?
   - Should Engram support local-only deterministic extraction as fallback?

6. Project hierarchy:
   - Should users manually define parent/subproject relationships?
   - Or should Engram infer and ask only when ambiguous?

7. Archival:
   - When should a project be archived?
   - Who decides: user, inactivity rule, or agent proposal?

8. Git integration:
   - Should knowledge commits be real Git commits in the vault repo?
   - Or separate Engram DB records plus Markdown files?

9. Multi-writer behavior:
   - How often should agents poll for memory changes since orientation?
   - Should changes written by other harnesses be shown by default or only if highly relevant?
   - What level of writer/model detail should be visible in normal context packs?

10. UI:
    - Is Obsidian enough for v1?
    - When is a local graph dashboard worth building?

11. Repository topology:
    - Should users manually register multi-repo projects?
    - How should Engram discover monorepo components?
    - What is the canonical local checkout when multiple worktrees exist?

12. Future coordination:
    - Is coordination beyond memory provenance needed at all?
    - At what point would work claims, agent messages, or an A2A-compatible facade become worth building?

---

## 25. Recommended MVP

The highest-value MVP is not the entire system.

Build this first:

```text
1. Vault init/compile
2. Typed preferences, decisions, rules, limitations
3. Context pack generation
4. orient tool
5. repo detection and project/repo/component mapping
6. knowledge commit page
7. writer provenance and memory changes-since cursor
8. lint for missing evidence, stale preferences, duplicate entities
```

MVP user experience:

```text
User prompts Codex/Claude in a repo.
Agent calls orient.
Engram returns project context pack.
Agent works.
Engram distills key memories at end or checkpoint.
Vault updates readable Markdown.
Lint flags memory issues.
```

This directly addresses the user's lived pain:

- no manual reminder to use Engram,
- project/subproject context is clearer,
- preferences are retained,
- memory is readable in Obsidian,
- old knowledge can be archived,
- graph starts to affect retrieval.

---

## 26. Final Recommendation

Do not build a new memory system from scratch yet.

Extend Engram into Memory OS through a new subsystem:

```text
Engram Core remains:
  storage, MCP, CLI, sessions, entities, work, documents

Memory OS adds:
  orientation, ontology, vault, graph traversal, knowledge commits, lint,
  archival memory, repository topology, writer provenance, session-time memory
  change awareness, and recalibration
```

The core product should become:

```text
Engram is Git for AI-agent knowledge:
it orients agents automatically, records how understanding changes,
maintains an Obsidian-readable memory library, and lints memory for drift.
```

This direction uses the best ideas from:

- LLM Wiki: immutable sources, compiled Markdown, lint,
- Letta/MemGPT: core versus archival memory,
- Mem0: scoped memory and reranking,
- Graphiti: temporal facts and supersession,
- Cognee: ontology and entity hygiene,
- LangMem: semantic, episodic, and procedural memory classes.

The decisive improvement is not better search. It is better memory lifecycle.

The second decisive improvement is better identity: Engram must know which project, subproject, repository, local checkout, monorepo component, agent, harness, and model produced or needs a piece of knowledge.

T105 matrix note: the post-T104 completion audit in
`docs/BRAIN_HARNESS_T105_POST_T104_COMPLETION_MATRIX_AUDIT_2026-06-01.md` rebuilt the current
matrix from live `orient`, direct search, `changes_since`, `handoff(get)`, obligations, repo docs,
the stale local markdown handoff, and git status. Current-plan retrieval and the active handoff are
healthy for the observed continuation surface, obligations are clean, and the worktree is clean
except untracked user-owned root `AGENTS.md`. M6 remains gated at T69 count-drift inspection, T70
document indexing remains exact-approval gated, lifecycle archive packets remain exact-ID gated,
and harness readiness remains risky/not ready. No M6 inspection/apply/deletion, document indexing,
lifecycle mutation, ranking, `orient`, public MCP, schema/storage/index, document-index behavior,
or harness write was run.

T106 matrix note: the harness readiness drift recheck in
`docs/BRAIN_HARNESS_T106_HARNESS_READINESS_DRIFT_RECHECK_2026-06-01.md` reconfirmed `ready=false`
for generic, Claude Code, Codex, Gemini CLI, and Cursor after T71/T105. Generic policy is still
missing; Claude Code still lacks required `SessionStart` and `SessionEnd` settings registrations;
Codex, Gemini CLI, and Cursor still have generated-adapter drift; and lint still reports stale
current-plan feedback with `safe_action=none`. Cross-harness behavior therefore remains
risky/not ready, T47 remains the exact harness-write gate, and no adapter install, settings edit,
hook registration, `adopt_user_owned`, M6 action, document indexing, lifecycle mutation, ranking,
`orient`, public MCP, schema/storage/index, or document-index behavior change was run.

T107 matrix note: the broad next-step direct-search calibration in
`docs/BRAIN_HARNESS_T107_BROAD_NEXT_STEP_SEARCH_CALIBRATION_2026-06-01.md` adds only exact
`what should happen next` / `what should we do next` / `what do we do next` current-plan guidance
phrases. Deterministic search fixtures and installed Codex MCP runtime trace
`019e8443-2d4d-7f30-969b-b7b235324ad5` now return active T106 current-plan memory first for
`what should happen next Engram Brain Harness`, while explicit M6/apply traces still return
migration gate context above current-plan guidance. Cross-harness parity is not validated for T107
because Claude Bridge timed out and native Claude Code could not connect through the configured
auth/network path. This is narrow search calibration only, not broad ranking proof or approval for
M6, lifecycle cleanup, document indexing, harness writes, `orient` expansion, public MCP changes,
schema/storage/index behavior changes, or document-index behavior changes.

T108 matrix note: `docs/BRAIN_HARNESS_T108_STALE_CURRENT_PLAN_EVIDENCE_SNAPSHOT_2026-06-01.md`
is a read-only stale current-plan evidence snapshot for exact target
`019e5e0a-86b4-73e3-aa9b-ca350e83e915`. Fresh T108 evidence shows T107 project current-plan memory
`019e844f-b038-7f50-b2fc-635771b15a06` ranks first in tested `orient` and direct `search`, while
the repository-scoped target remains active, current-plan tagged, second in tested current-plan
searches, and linted as `feedback_stale_current_plan` with 237 recent stale-feedback records plus
`safe_action=none`. AI Council supported freezing the exact target; Claude Bridge warned against
proxy-approval framing, so the artifact is a default-deny evidence snapshot. No lifecycle archive,
scope correction, `lint(action="apply_safe")`, M6 action, document indexing, harness write, ranking
change, `orient` expansion, public MCP change, schema/storage/index behavior change, or
document-index behavior change was run.

T109 matrix note: `docs/BRAIN_HARNESS_T109_TELEMETRY_CONFIDENCE_AUDIT_2026-06-01.md` is a
docs-only telemetry confidence audit. Source inspection confirms `real_session_eval` reports over a
bounded sampled trace set and trace-linked feedback, with `intent` as secondary metadata and
outcome fields required for stronger evidence. The latest report still fails the confidence gate
after ordinary startup feedback scoring: `feedback_coverage=0.36000001430511475`,
`external_session_trace_count=0`, `task_failure_count=1`, and `bad_memory_used_count=0`. AI
Council was willing to allow isolated calibration traces, but Claude Bridge warned they risk metric
gaming; T109 chooses the conservative path and creates no calibration traces. No telemetry code,
M6 action, lifecycle mutation, document indexing, harness write, ranking change, `orient`
expansion, public MCP change, schema/storage/index behavior change, or document-index behavior
change was run.

T110 matrix note: `docs/BRAIN_HARNESS_T110_TELEMETRY_WINDOW_REGRESSION_2026-06-01.md` adds an
executable telemetry-window regression,
`real_session_eval_default_sample_can_mask_recent_window_failure`. The test constructs a recent
50-trace sparse-feedback window that fails the confidence gate and a larger default sample that
passes because older feedback-rich traces dominate. This preserves T109's default-vs-recent
measurement caveat without changing telemetry behavior. AI Council split on changing the default
immediately, and Claude Bridge timed out, so T110 does not change `DEFAULT_REAL_SESSION_EVAL_LIMIT`,
confidence formulas, public MCP request parameters, M6 state, lifecycle state, document indexing,
harness writes, ranking, `orient`, schema/storage/index behavior, or document-index behavior.

T112 matrix note: `docs/BRAIN_HARNESS_T112_RECOMMENDATION_SURFACE_AUDIT_2026-06-01.md` is a
docs-only audit of the existing `RealSessionEvalReport.recommendations` surface after the T111
eval-design disagreement. Source and repo search show the field is documented as operator-facing
follow-up text, is serialized through the existing telemetry report, and has no repo-local
control-flow consumer; current tests assert only targeted substrings. Because the field is still
observable MCP output and external agents may read it as guidance, T112 does not add a
recommendation string or resolve T111. No telemetry behavior, public MCP request/response shape,
confidence formula, M6 state, lifecycle state, document indexing, harness write, ranking,
`orient`, schema/storage/index behavior, or document-index behavior changed.

T113 matrix note:
`docs/BRAIN_HARNESS_T113_POST_T112_STARTUP_RETRIEVAL_VALIDATION_2026-06-01.md` records a
read-only post-T112 startup validation. Fresh Codex lean `orient` and direct searches returned the
T112 current-plan memory first for continuation/current-plan probes, recovered the reviewed
software-design preference in preference-oriented `orient`, and surfaced T111 gate context without
treating generic `i approve` as authorization. Obligations were clean; lint still reported stale
current-plan target `019e5e0a-86b4-73e3-aa9b-ca350e83e915` with `safe_action=none`; git remained
clean except user-owned untracked root `AGENTS.md`. No telemetry behavior, public MCP
request/response shape, confidence formula, M6 state, lifecycle state, document indexing, harness
write, ranking, `orient`, schema/storage/index behavior, or document-index behavior changed.

T114 matrix note:
`docs/BRAIN_HARNESS_T114_CURRENT_PLAN_NOISE_FIXTURE_2026-06-01.md` records a test-only regression
fixture for the post-T113 direct-search noise shape. The new
`test_memory_search_t114_current_plan_outranks_stale_and_wrong_scope_noise` seeds a latest
project-scoped current-plan MemoryItem, a stale repository-scoped current-plan distractor, and a
Claude-Code-authored `Claude Code user-stated instruction` rule, then asserts only that the latest
project plan ranks first and both noisy items rank below it. This preserves the observed behavior
without changing ranking, lifecycle state, telemetry behavior, public MCP request/response shape,
M6 state, document indexing, harness writes, `orient`, schema/storage/index behavior, or
document-index behavior.

T115 matrix note:
`docs/BRAIN_HARNESS_T115_POST_T114_DOCUMENT_VISIBILITY_AUDIT_2026-06-01.md` records a read-only
document-search visibility audit after T114. Existing document search still recovers T59 from the
earlier T67 exact-file indexing, but tested T68, T69, T70, T113, and T114 title/gate queries did
not return those packets in the top five. `docs(action="stats")` reported `source_count=76`,
`chunk_count=4114`, `searchable_chunk_count=2102`, and `orphan_chunk_count=2012`. This keeps
repo-file reads, `orient`, direct memory search, and `handoff(get)` authoritative for recent
startup evidence until an exact indexing scope is approved. T115 did not run document indexing,
cleanup, reindex, orphan recovery, M6 inspection/apply, lifecycle mutation, ranking changes,
`orient` changes, public MCP changes, schema/storage/index behavior changes, document-index
behavior changes, or harness writes.

T116 matrix note:
`docs/BRAIN_HARNESS_T116_APPROVAL_SCOPE_AUDIT_2026-06-01.md` records a docs-only approval-scope
audit after a stale completed T65 approval phrase and generic approval appeared in the thread.
Current live evidence shows T65 was already completed by T67, T70 remains pending with the exact
phrase `Approve T70: index exact files T59, T68, and T69.`, obligations are clean, lint still
reports stale/wrong-scope memory with no safe current-plan lifecycle action, document search still
misses T70/T114 in the tested top five, and `real_session_eval(project=engram, limit=50)` fails the
confidence gate with `feedback_coverage=0.47999998927116394` and only two intents with feedback.
T116 does not run document indexing, M6 inspection/export/apply, lifecycle mutation, ranking
changes, `orient` changes, public MCP changes, schema/storage/index behavior changes,
document-index behavior changes, or harness writes.

T117 matrix note:
`docs/BRAIN_HARNESS_T117_T116_CLAUDE_PARITY_AUDIT_2026-06-01.md` records a docs-only
cross-harness read-path audit after T116. Fresh Codex startup evidence returned the T116 current
plan first in `orient` and direct current-plan search. Claude Bridge project-harness probing still
found no Engram MCP tools, but personal-harness probing returned the T116 current plan first in
both `orient` and direct search. Exact T70 phrase retrieval remains noisy: direct search still
ranked older handoffs above the T116 current plan, and document search returned older T64/T59/T58
material rather than T70/T116. The latest `real_session_eval(project=engram, limit=50)` still fails
the confidence gate because only two intents have feedback. T117 does not run document indexing,
M6 inspection/export/apply, lifecycle mutation, ranking changes, `orient` changes, public MCP
changes, schema/storage/index behavior changes, document-index behavior changes, or harness writes.

T118 matrix note:
`docs/BRAIN_HARNESS_T118_EXACT_APPROVAL_COMMAND_RANKING_2026-06-01.md` records a narrow direct
memory-search ranking calibration for exact approval-command prompts such as
`Approve T70: index exact files T59, T68, and T69.` The new path is search-only, requires a scoped
`Approve T<number>:` command, and promotes only matching active Decision/Rule MemoryItems tagged
`current-plan`; it does not treat retrieval as execution approval. Deterministic ranker and
memory-search fixtures passed, including existing migration-gate regressions. T118 does not change
`orient`, run document indexing, inspect T69 files, run M6 inventory/export/apply, mutate lifecycle
state, change public MCP parameters or response shape, change schema/storage/index behavior, change
document-index behavior, or write harness adapters/hooks.

T119 matrix note:
`docs/BRAIN_HARNESS_T119_EXACT_APPROVAL_COMMAND_RUNTIME_GAP_2026-06-01.md` records a docs-only
runtime-gap audit after T118. Live exact T70 search still ranked old T110/T109 handoffs above
current-plan before and after repairing active current-plan memory to include the literal
`Approve T70: index exact files T59, T68, and T69.` phrase. Lean `orient` did recover the repaired
T119 current-plan first, but direct exact search did not, which indicates the active in-thread MCP
runtime has not picked up the T118 ranker code. `cargo install --path engram-cli` installed the
current binary to `/Users/yuval.meiri/.cargo/bin/engram`, but starting a fresh HTTP server against
the global store failed because the existing Engram process holds the RocksDB lock. No daemon
restart, `kill`, document indexing, T69 inspection, M6 action, lifecycle mutation, ranking change,
`orient` change, public MCP/schema/storage/index behavior change, document-index behavior change,
or harness write was run.

T120 matrix note:
`docs/BRAIN_HARNESS_T120_RUNTIME_REFRESH_VALIDATION_2026-06-02.md` records the approved runtime
refresh that closed the immediate T119 stale-runtime caveat. `/Users/yuval.meiri/.local/bin/engram`
was replaced from current source and now matches `/Users/yuval.meiri/.cargo/bin/engram` at SHA-256
`ff7e2994cf5f49ba0d7d276cf9e2e71acb587d9947e6695832cb4e085ef5a726`; the old HTTP daemon PID
`1236` was stopped cleanly and the refreshed daemon started as PID `85557` on port `8765`. Active
MCP trace `019e8724-de63-7003-8d57-db2a05a53525` now returns current-plan memory
`019e8506-1b1e-7da0-9a21-96f098765a43` first for
`Approve T70: index exact files T59, T68, and T69.`, while migration controls
`019e8725-7fdf-76f1-8ae0-8a73419760c5` and `019e8725-8016-7bb1-aff4-9da9c827384d` still return
default-deny M6 gate evidence first. After T120 feedback scoring,
`real_session_eval(project=engram, limit=50)` passed numerically with coverage `0.54`, three
intents with feedback, and no bad memory used, but still reports `requires_user_approval=true` and
remains weak rolling evidence. T120 does not run T70 indexing, inspect T69 files, run M6, mutate
lifecycle state, change ranking logic, expand `orient`, change public MCP or schema/storage/index
behavior, change document-index behavior, or write harness adapters/hooks.

T121 matrix note:
`docs/BRAIN_HARNESS_T121_T69_COUNT_DRIFT_INSPECTION_RESULT_2026-06-02.md` records the approved
read-only execution of the T69 inspection packet. Only the written export snapshot's `index.md`
and `candidates/0012-skip-plan.md` were read. The count drift from T68 is explained by one
generated `skip` candidate from a `session_event` plan source: the export index reports 116
sources, 12 candidates, and dispositions `review: 9`, `quarantine: 2`, `skip: 1`, while candidate
12 is a low-confidence `session_insight` skip for a resumed-session plan note. This closes the T69
inspection gate, but it does not run T70 indexing, make candidate decisions, run M6 apply, mutate
lifecycle state, rerun/prioritize export, change ranking logic, expand `orient`, change public MCP
or schema/storage/index behavior, change document-index behavior, or write harness adapters/hooks.

T122 matrix note:
`docs/BRAIN_HARNESS_T122_M6_CANDIDATE_REVIEW_APPROVAL_PACKET_2026-06-02.md` records the next
docs-only M6 approval packet after T121. AI Council and Claude Bridge both supported preparing an
approval packet rather than stopping for T70, but Claude warned against all-11 batching and
meta-planning inflation. T122 therefore asks for exact T123 approval to read only candidate files
0001-0004 from the written T68 snapshot. It does not read candidate files, run migration
status/prioritize/apply/rerun, make candidate decisions, index documents, mutate lifecycle state,
change ranking logic, expand `orient`, change public MCP or schema/storage/index behavior, change
document-index behavior, or write harness adapters/hooks.

T70 execution note:
`docs/BRAIN_HARNESS_T70_EXACT_FILE_INDEX_RESULT_2026-06-02.md` records the exact approved T70
indexing run. The three approved file paths were indexed and returned chunk counts T59=9, T68=8,
T69=9 with no warnings. Validation improved T68 and T69 exact-title retrieval, but T59 remained
noisy for the exact-title probe after reindexing and is better recovered by filename-stem or scoped
approval phrasing. This indexing run does not authorize M6 candidate inspection, candidate
decisions, status/prioritize/apply/rerun, deletion, lifecycle mutation, ranking, `orient`, public
MCP/schema/storage/index behavior changes, document-index behavior changes, or harness writes.

T123 matrix note:
`docs/BRAIN_HARNESS_T123_M6_CANDIDATE_0001_0004_INSPECTION_RESULT_2026-06-02.md` records the exact
approved read-only inspection of candidate files 0001-0004 from the written T68 review-export
snapshot. All four candidates are `project_observation` sources with `disposition: review` and
project-scoped migrated proposed memory. Candidate 0004's 2026-05-24 Claude Code `ready=true`
claim conflicts with later readiness audits and must be treated as time-bound or stale before any
acceptance decision. T123 does not make candidate decisions, inspect quarantine files, run
status/prioritize/apply/rerun, write active memory, delete data, mutate lifecycle state, index
documents, change ranking logic, expand `orient`, change public MCP or schema/storage/index
behavior, change document-index behavior, or write harness adapters/hooks.

T124 matrix note:
`docs/BRAIN_HARNESS_T124_M6_CANDIDATE_0005_0009_INSPECTION_RESULT_2026-06-02.md` records the exact
approved read-only inspection of candidate files 0005-0009 from the written T68 review-export
snapshot. All five candidates are `project_observation` sources with `disposition: review` and
project-scoped migrated proposed memory. T124 completes read-only inspection of the 9 review
candidates, but it does not inspect the 2 quarantine candidates and does not make candidate
decisions. Candidate 0005 contains historical `ready=true` and obligation-list leak claims later
narrowed by subsequent evidence; candidates 0008 and 0009 contain older next-step guidance likely
narrowed by later work; candidate 0006 is harness-write-adjacent. T124 does not run
status/prioritize/apply/rerun, write active memory, delete data, mutate lifecycle state, index
documents, change ranking logic, expand `orient`, change public MCP or schema/storage/index
behavior, change document-index behavior, or write harness adapters/hooks.

T126 matrix note:
`docs/BRAIN_HARNESS_T126_HARNESS_READINESS_RECHECK_2026-06-02.md` records a read-only harness
readiness recheck after T124. Generic, Claude Code, Codex, Gemini CLI, and Cursor all still report
`ready=false`. Generic policy is missing; Claude Code still lacks required `SessionStart` and
`SessionEnd` settings registrations; Codex, Gemini CLI, and Cursor still have generated-adapter
drift. T126 updates evidence only and does not run T47 harness repair, inspect T125 quarantine
candidates, run status/prioritize/apply/rerun, write active memory, delete data, mutate lifecycle
state, index documents, change ranking logic, expand `orient`, change public MCP or
schema/storage/index behavior, change document-index behavior, or write harness adapters/hooks.

T127 matrix note:
`docs/BRAIN_HARNESS_T127_POST_T126_STARTUP_CONTINUITY_AUDIT_2026-06-02.md` records a read-only
startup continuity audit after T126. Lean `orient` and direct continuation search recovered T126
current-plan memory first, `memory(list)` returned exactly one active Engram project current-plan
item, and `handoff(get)` returned the T126 handoff. Exact T125 wording still surfaces older active
handoffs above the current plan, broad next-step search still includes stale repository-scoped
current-plan memory lower down, the T126 report is not top-five visible through `docs(search)`, and
rolling telemetry fails the confidence gate at 38% feedback coverage. T127 does not inspect T125
quarantine candidates, run status/prioritize/apply/rerun, make candidate decisions, write active
candidate memory, delete data, mutate lifecycle state, index documents, change ranking logic,
expand `orient`, change public MCP or schema/storage/index behavior, change document-index
behavior, or write harness adapters/hooks.

T128 matrix note:
`docs/BRAIN_HARNESS_T128_CLAUDE_POST_T127_PARITY_CHECK_2026-06-02.md` records a read-only Claude
Code parity check after T127. Claude Code recovered T127 current-plan memory first in lean
`orient` and broad continuation search, but exact T125 search remained noisy and ranked current
plan fifth behind handoffs. Handoff continuity failed: Claude Code session-end automation wrote
stub handoffs despite bridge `write=false`, superseding the rich T127 handoff and dropping T125/T47
gate detail from the canonical handoff. T128 does not authorize hook changes, harness repair,
lifecycle mutation, document indexing, candidate inspection, ranking, `orient`, public
MCP/schema/storage/index behavior changes, document-index behavior changes, or M6
status/prioritize/apply/rerun.

T129 matrix note:
`docs/BRAIN_HARNESS_T129_CLAUDE_SESSION_END_HANDOFF_ROOT_CAUSE_2026-06-02.md` records a docs-only
root-cause packet for the T128 handoff-continuity failure. Source inspection shows the daemon writes
Claude `SessionEnd` handoffs only when `write_policy=durable`, while the generated command-style
Claude session-end hook defaults a missing hook-input `write_policy` to `durable` before calling
`harness(action="hook_event")`. Because rolling handoff updates supersede the previous active
handoff, this can overwrite a rich Codex handoff with a low-information Claude session-end stub
even when the surrounding bridge task was launched with `write=false`. Live Claude harness doctor
still reports `ready=false`: generated files are installed, but required `SessionStart` and
`SessionEnd` settings registrations are missing. AI Council consensus recommended a docs-only
packet as the smallest safe slice and treating the likely hook-template fix as an exact approval
gate. T129 does not change code, installed hooks, settings, adapters, lifecycle state, migration
state, document indexing, candidate files, ranking, `orient`, public MCP parameters, or
schema/storage/index behavior.

T132 matrix note:
`docs/BRAIN_HARNESS_T132_POST_T129_STARTUP_GATE_AUDIT_2026-06-02.md` records a read-only
post-T129 startup and gate audit. Lean `orient` recovered the T129 current-plan memory first, exact
T130 approval search returned the current plan first and T129 handoff second, scoped current-plan
listing returned exactly one active project current-plan item, and `handoff(get)` returned the T129
handoff. Broad direct current-plan searches remain handoff-noisy, the T129 report is not top-five
visible through `docs(search)`, Claude Code and Codex harnesses still report `ready=false`, lint
still reports stale/wrong-scope feedback for repository-scoped current-plan memory
`019e5e0a-86b4-73e3-aa9b-ca350e83e915`, and telemetry passes numerically with
`feedback_coverage=0.5` while still requiring user approval. T132 does not authorize T130, T131,
T125, T47, migration status/prioritize/apply/rerun, lifecycle mutation, document indexing,
ranking changes, `orient` expansion, public MCP/schema/storage/index behavior changes,
document-index behavior changes, hook/settings/adapter writes, harness install, or user-owned file
adoption.

T130 matrix note:
`docs/BRAIN_HARNESS_T130_CLAUDE_SESSION_END_HOOK_DEFAULT_2026-06-02.md` records the approved narrow
repair for the T129 root cause. The generated command-style Claude `SessionEnd` hook now defaults
missing hook-input `write_policy` to `nudge` instead of `durable`; daemon handling still writes
handoffs only for explicit `write_policy=durable`. Focused validation covered missing-policy
no-write behavior, explicit durable handoff writes, rendered adapter output, tempdir-installed
generated hook output, MCP `render_adapter` output, all `engram-index` harness unit tests, the MCP
harness integration file, formatting, `cargo check -p engram-cli`, and whitespace checks. T130 did
not edit installed user hooks/settings, run harness install, change public MCP parameters,
schema/storage/index behavior, ranking, `orient`, migration, lifecycle state, document-index
behavior, or user-owned files. Claude Code harness readiness remains separately gated because real
settings/adapters were not installed or repaired.

T133 matrix note:
`docs/BRAIN_HARNESS_T133_POST_T130_LIVE_RUNTIME_GAP_AUDIT_2026-06-02.md` records a read-only
source-vs-live audit after T130. Source and committed tests are T130-correct:
`engram-index/src/harness.rs` defaults missing hook-input `write_policy` to `nudge`. The running MCP
runtime is not yet T130-correct: live `harness(render_adapter)` still renders
`.write_policy // "durable"`, and the installed generated Claude hook at
`/Users/yuval.meiri/.claude/hooks/engram-session-end.sh` still defaults to `durable`. Claude Code,
Codex, Gemini CLI, and Cursor doctors still report `ready=false`, lint still reports stale active
memory pressure, and telemetry remains weak despite a numerical pass. T133 does not install a
binary, restart a daemon, edit installed hooks/settings, run `harness install`, change public MCP
parameters, schema/storage/index behavior, ranking, `orient`, migration, lifecycle state,
document-index behavior, or user-owned files. The next recommended gate is binary refresh plus
daemon restart plus read-only live validation only; installed hook/settings repair remains a
separate exact harness-write approval.

T134 matrix note:
`docs/BRAIN_HARNESS_T134_T133A_LIVE_RUNTIME_REFRESH_APPROVAL_PACKET_2026-06-02.md` prepares the
exact T133A approval wording and validation scope. It records the same live evidence after startup:
source is still `nudge`, live MCP render and installed Claude hook are still `durable`, Claude Code
readiness is still false, and lint still reports stale active memory pressure. T134 does not install
a binary, restart a daemon, edit installed hooks/settings, run `harness install`, use
`adopt_user_owned`, change public MCP parameters, schema/storage/index behavior, ranking, `orient`,
migration, lifecycle state, document-index behavior, or inspect M6 quarantine candidates. If the
user approves T133A exactly, the next slice is binary install, daemon restart, and read-only live
render/doctor validation only; hook/settings repair remains a separate exact approval gate.

T133A matrix note:
`docs/BRAIN_HARNESS_T133A_LIVE_RUNTIME_REFRESH_VALIDATION_2026-06-02.md` records the approved
runtime refresh. The current repo binary was installed to `/Users/yuval.meiri/.local/bin/engram`,
the Engram daemon restarted on port 8765, and live `harness(render_adapter, claude_code,
claude-session-end-hook)` now renders `.write_policy // "nudge"`. This validates the T130 repair in
the running MCP runtime. It does not repair installed hooks/settings: the installed Claude
`SessionEnd` hook still renders `.write_policy // "durable"` and Claude Code doctor reports it as
drifted. Claude Code, Codex, Gemini CLI, and Cursor readiness remain false. No installed
hooks/settings, `harness install`, `adopt_user_owned`, public MCP parameters, schema/storage/index
behavior, ranking, `orient`, migration, lifecycle state, document-index behavior, or M6 quarantine
candidates were changed or inspected.

T135 matrix note:
`docs/BRAIN_HARNESS_T135_REFRESHED_HARNESS_REPAIR_APPROVAL_PACKET_2026-06-02.md` refreshes the
stale T47 harness repair packet using fresh post-T133A read-only status and dry-run evidence. The
key delta is that Claude Code now plans an update to
`/Users/yuval.meiri/.claude/hooks/engram-session-end.sh`; T47 listed that hook as skipped/already
installed, so it should not be reused as approval. T135 remains docs-only and requests exact
approval for five one-at-a-time harness install writes after matching fresh dry-runs, all with
`adopt_user_owned=false` and Claude Code `settings_target=settings.local.json`. It does not
authorize harness writes, user-owned adoption, `settings.json` edits, unlisted hook/command edits,
M6 action, lifecycle mutation, ranking, `orient`, public MCP/schema/storage/index changes, or
document-index behavior changes until the user explicitly approves the T135 wording.

T136 matrix note:
`docs/BRAIN_HARNESS_T136_STALE_ACTIVE_HANDOFF_NOISE_AUDIT_2026-06-02.md` records a read-only
audit of active rolling handoff noise. The current plan is not ambiguous: lean `orient`,
`handoff(get)`, and scoped current-plan listing recover the T135 gate. The noise is real anyway:
MCP listing returned 50 active project-scoped rolling handoffs at the requested limit, including a
T135->T133A->T134->T133 chain and low-information Claude session-end stubs. Source inspection
explains the shape: `handoff(update)` adds a supersedes edge to the previous handoff but saves only
the new handoff, while `capture_current_plan` explicitly marks older current-plan guidance
`superseded`. T136 made no lifecycle, ranking, `orient`, schema/storage/index, document-index,
M6, or harness/settings change. Lifecycle cleanup or handoff semantics repair remains exact-gated.

T137 matrix note:
`docs/BRAIN_HARNESS_T137_HARNESS_READINESS_RECHECK_2026-06-02.md` records a read-only installed
harness readiness recheck after T136. Lean `orient` and direct search recover the active T136
current-plan memory first. Live `harness(status)` and `harness(doctor)` still return
`ready=false` for generic, Codex, Gemini CLI, Cursor, and Claude Code. The failures match the T135
gate shape: missing generic policy adapter; drifted Codex, Gemini CLI, and Cursor generated
adapters; drifted Claude `SessionEnd` hook; missing Claude SessionStart/SessionEnd settings
registrations; and user-owned Claude settings snippet left untouched. T137 did not run
`harness(install)` dry-runs because T135 requires those immediately before approved writes. It made
no harness/settings writes, daemon/binary changes, lifecycle, ranking, `orient`, schema/storage/
index, document-index, M6, or public MCP changes. T135 remains the next product-moving exact gate.

T138 matrix note:
`docs/BRAIN_HARNESS_T138_CRITICAL_VALIDATION_BASELINE_2026-06-02.md` records a non-destructive
validation baseline after T137. The first `cargo clippy --all-targets -- -D warnings` run exposed
one existing `engram-cli/src/main.rs` `items_after_test_module` failure. T138 fixed only that
mechanical ordering issue by moving the existing CLI timestamp test module to the end of the file;
the test body and assertions did not change. On the final tree, `cargo fmt --all --check`, focused
harness/orient/obligation/telemetry/lint/Brain Harness/search tests, `cargo check -p engram-cli`,
`cargo clippy --all-targets -- -D warnings`, `cargo test --all-targets`, and `git diff --check`
passed. T138 does not install or repair harnesses, change lifecycle state, run M6, inspect
quarantine candidates, change ranking or `orient`, change public MCP/schema/storage/index, or
change document-index behavior. T135 remains the next product-moving exact gate.

T139 matrix note:
`docs/BRAIN_HARNESS_T139_STALE_CURRENT_PLAN_LIFECYCLE_APPROVAL_PACKET_2026-06-02.md` records a
docs-only, default-deny approval packet for exactly one future lifecycle write: archiving stale
repository-scoped current-plan MemoryItem `019e5e0a-86b4-73e3-aa9b-ca350e83e915`. Fresh read-only
evidence showed T138 current-plan memory `019e885c-abde-7811-9314-7654bb6667a9` remains first in
lean `orient` and direct current-plan search while the stale target remains active retrieval noise;
lint reported 207 stale-feedback records and 14 wrong-scope records for the target with
`safe_action=none`; direct graph depth 1 showed only evidence, repository scope, capture, and
supersedes edges, with no MemoryItem directly depending on the target. T139 does not authorize the
archive itself, `lint apply_safe`, any other lifecycle mutation, handoff semantic change, M6,
quarantine inspection, harness writes, ranking/`orient`, public MCP, schema/storage/index, or
document-index behavior changes. Any future archive requires exact user approval and fresh matching
pre-write evidence with no intervening writes.

T140 matrix note:
`docs/BRAIN_HARNESS_T140_APPROVAL_GATE_CONTEXT_SEARCH_FIX_2026-06-02.md` records a narrow
direct-search ranking repair after live trace `019e8866-0e96-7b73-a107-e4a756684bf0` returned old
active rolling handoffs above the T139 current-plan memory for a continuation query that mentioned
T135/T139 approval gates as context. The fix refines `approval gate` classification so continuation
queries still promote the latest current-plan MemoryItem, while explicit permission/action and
handoff-summary prompts remain gate-mode. Focused validation passed with ranker unit tests, the
full `search_tests` integration suite, `cargo fmt --all --check`, `cargo check -p engram-cli`, and
`git diff --check`. T140 does not install a binary, restart the daemon, validate installed runtime,
change `orient`, mutate lifecycle state, alter handoff semantics, run M6, inspect quarantine
candidates, write harness files, or change public MCP/schema/storage/index/document-index behavior.

T141 matrix note:
`docs/BRAIN_HARNESS_T141_T140_RUNTIME_REFRESH_APPROVAL_PACKET_2026-06-02.md` records a docs-only
approval packet for the next T140 validation step. Fresh read-only startup evidence after T140
showed lean `orient` trace `019e8874-f501-7863-b67e-2c6e7cca890f` returning T140 current-plan
memory first, while live direct search trace `019e8875-01bb-7763-a9d9-86c10830e3fc` still ranked
active rolling handoff `019e8872-cb39-74b0-9594-e052aeb6d993` above current-plan guidance for a
T140 continuation prompt. The packet asks for exact approval before running only the known
runtime-refresh sequence (`cargo install --path engram-cli --force --root /Users/yuval.meiri/.local`,
`engram daemon stop`, `engram daemon start`) plus read-only live validation of the T140 query
class. T141 does not run the refresh and does not authorize harness install, hooks/settings/
adapters, `adopt_user_owned`, lifecycle mutation, T139 archive, M6/migration/quarantine, `orient`,
ranking source, public MCP, schema/storage/index, document-index behavior, shell profile, PATH,
auth, or service configuration changes.

T142 matrix note:
`docs/BRAIN_HARNESS_T142_POST_T141_SOURCE_VALIDATION_BASELINE_2026-06-02.md` records a source-only
validation baseline after T140/T141. `cargo fmt --all --check`, focused T140 ranker tests, focused
`search_tests`, `cargo check -p engram-cli`, `cargo clippy --all-targets -- -D warnings`,
`cargo test --all-targets`, and `git diff --check` passed. T142 does not prove installed runtime
parity, installed harness readiness, lifecycle cleanup, or migration completion. Because T133A is
already committed and rerunning its install/restart wording at current `HEAD` would also deploy
the later T140 ranking source change, T142 did not rerun T133A. The next runtime-moving step remains
exact T141 approval for binary install, daemon restart, and read-only live validation of the T140
continuation/current-plan approval-gate-context query class.

T143 matrix note:
`docs/BRAIN_HARNESS_T143_CURRENT_HANDOFF_SEARCH_FIXTURE_2026-06-02.md` records a source-level
regression fixture for the fresh post-T142 handoff-vs-current-plan shape. Live read-only evidence
showed lean `orient` trace `019e8884-e4d7-7cb2-bbe2-39b26935c3ce` returning T142 current-plan
memory first, while direct search trace `019e8884-fedb-73d0-802e-bed68d71f4f3` returned the fresh
rolling handoff first for a T140/T141 approval-gate-context continuation query. The new integration
fixture proves current source already ranks the active project `decision` tagged `current-plan`
above a fresher rolling handoff for that query while keeping the handoff retrievable. Validation
passed with the focused T143 test, ranker unit tests, full `search_tests`, `cargo fmt --all
--check`, `cargo check -p engram-cli`, and `git diff --check`. No ranker source changed; T141
runtime refresh remains the next exact approval gate.

T144 matrix note:
`docs/BRAIN_HARNESS_T144_T143_RUNTIME_REFRESH_APPROVAL_PACKET_2026-06-02.md` records a refreshed,
docs-only/default-deny runtime-refresh approval packet that supersedes stale T141. Current HEAD is
`ab2f5e25b78f1224a7dbc4d5615c143f286a750b`, while the installed binary hash remains
`837ef2cabf08f1481ff66d44911387cf3e5d1941f86a41431780dde48bdef724` and daemon PID remains
`23341`. Fresh live `orient` trace `019e8889-f44b-7d32-8363-b0105366eb8a` returned T143
current-plan memory first, but direct live traces `019e888a-116a-7923-940c-cc5668240877` and
`019e888a-125f-7273-93a9-ea1a21bc34d4` still returned active rolling handoffs above current-plan
guidance. T144 asks for exact approval before running only `cargo install --path engram-cli
--force --root /Users/yuval.meiri/.local`, `engram daemon stop`, `engram daemon start`, and
read-only live validation of the listed T140/T143 query shapes. It does not run the refresh or
authorize harness install, hooks/settings/adapters, `adopt_user_owned`, lifecycle mutation, T139
archive, M6/migration/quarantine, `orient`, ranking source, public MCP, schema/storage/index,
document-index behavior, shell profile/PATH/auth/service configuration, rollback, force-kill,
deletion, or old-binary reinstall commands.

T145 matrix note:
`docs/BRAIN_HARNESS_T145_BINARY_SOURCE_RUNTIME_REFRESH_APPROVAL_PACKET_2026-06-02.md` records a
refreshed, docs-only/default-deny runtime-refresh approval packet that supersedes stale T141 and
stale T144. T144 became self-stale because it stopped on any HEAD drift and then its own docs-only
commit moved HEAD from source baseline `ab2f5e25b78f1224a7dbc4d5615c143f286a750b` to
`7baf1365ff72ad3007082be0763a28d5918b0b3f`. Read-only diff evidence showed only docs changes and
no `Cargo.toml`, `Cargo.lock`, or `engram-*` drift; the installed binary hash and daemon PID
remain `837ef2cabf08f1481ff66d44911387cf3e5d1941f86a41431780dde48bdef724` and `23341`.
T145 replaces the full-HEAD invariant with a deny-by-default binary-source invariant: after exact
approval, the first checks must prove no committed, staged, or unstaged binary-relevant drift from
`ab2f5e25` before any install command runs. It does not run the refresh or authorize stale
T141/T144 execution, harness install, hooks/settings/adapters, user-owned edits, lifecycle
mutation, T139 archive, M6/migration/quarantine, `orient`, ranking source, public MCP,
schema/storage/index, document-index behavior, shell profile/PATH/auth/service configuration,
rollback, force-kill, deletion, or old-binary reinstall commands.

T145 execution matrix note:
`docs/BRAIN_HARNESS_T145_RUNTIME_REFRESH_VALIDATION_RESULT_2026-06-02.md` records the approved
runtime refresh and partial validation failure. The required binary-source prechecks passed, the
installed `.local/bin/engram` hash changed from
`837ef2cabf08f1481ff66d44911387cf3e5d1941f86a41431780dde48bdef724` to
`3d801be9dcae4b26bd03b27cadd0d4449cc32322e7d0cb3bcff0b0ac58b6686b`, and the daemon restarted
from PID `23341` to PID `10768`. The three listed direct T140/T143 live search queries all returned
active T145 current-plan memory first, above rolling handoff noise. Exact no-prompt lean `orient`
trace `019e89b6-6fa0-71f2-977a-f9046eaabbdf` did not return current-plan guidance, so T145 does
not pass overall. The next narrow work is a read-only root-cause and fixture proposal for this
`orient` miss; any implementation changing `orient`, ranking-source behavior, public MCP,
schema/storage/index, lifecycle state, harness files, M6, or document-index behavior remains
separately approval-gated.

T146 matrix note:
`docs/BRAIN_HARNESS_T146_NO_PROMPT_PLAN_WORK_ORIENT_APPROVAL_PACKET_2026-06-02.md` records a
docs-only approval packet for the no-prompt `plan_work` current-plan miss exposed by T145. Fresh
read-only live traces show the gap is specific: explicit continuation/current-plan `plan_work`
trace `019e89ba-e8a0-7b71-bfad-23dd08bca7fd` and no-prompt `resume_session` trace
`019e89ba-e945-73e1-9115-94d0217bd0e7` return the post-T145 current plan first, while no-prompt
`plan_work` trace `019e89ba-e9e6-7ef2-9904-b4d648074d83` still returns generic guidance. Source
inspection points to the current predicate requiring query text before `plan_work` current-plan
promotion, with Brain Loop group ordering as a second local affected site. T146 does not implement
the fix; it asks for exact approval to add focused fixtures and a narrow no-prompt `plan_work`
promotion/pin that proves both `active_decisions.first()` and `brain_loop.top_items.first()`,
without changing public MCP shape, payloads, broad ranking, schema/storage/index, lifecycle,
harness files, M6, document-index behavior, runtime, or user-owned files.

T146 source/result matrix note:
`docs/BRAIN_HARNESS_T146_NO_PROMPT_PLAN_WORK_ORIENT_RESULT_2026-06-02.md` records the approved
source implementation committed as `d12b2ca` (`Fix no-prompt plan_work current-plan orient`). The
source change stays internal to `orient`: no/empty-prompt project/cwd-boundary `plan_work` promotes
the latest current-plan decision into `active_decisions` and pins it first in Brain Loop, while
specific implementation prompts and no-boundary/no-current-plan cases are guarded by focused MCP
fixtures. Validation passed for the full `memory_tests` file, the `search_tests current` subset,
the service-layer `orient_` sweep, `cargo fmt --all --check`, `cargo check -p engram-cli`, and
`git diff --check`. T146 did not install a binary or restart the daemon; live no-prompt `orient`
traces after the source commit still show stale runtime behavior.

T147 matrix note:
`docs/BRAIN_HARNESS_T147_T146_RUNTIME_REFRESH_APPROVAL_PACKET_2026-06-02.md` records the
default-deny runtime-refresh packet for the committed T146 source fix, and
`docs/BRAIN_HARNESS_T147_T146_RUNTIME_REFRESH_VALIDATION_RESULT_2026-06-03.md` records the approved
execution. Required binary-source first checks were empty from baseline
`d12b2ca17500d0979852fe9a35ff7dc6468aa091`, the installed
`/Users/yuval.meiri/.local/bin/engram` hash changed from
`3d801be9dcae4b26bd03b27cadd0d4449cc32322e7d0cb3bcff0b0ac58b6686b` to
`0cbbbc82a70f08b52f218369e4c304828037d3615c4bac71c35303957b423f22`, and the daemon restarted
from PID `10768` to PID `68053`. Direct search trace
`019e8bb8-b9e1-7ff3-921f-f3a5589b5ed2` returned active current-plan memory
`019e8a01-0720-7903-814c-db9eb4eb4d6f` first; no-prompt trace
`019e8bb8-ba85-7230-aede-84266c5721c6` and empty-prompt trace
`019e8bb8-bb3e-7af2-a765-fcbd5bbc4c50` returned the same item first in Brain Loop; explicit
implementation-prompt trace `019e8bb8-bbf7-7e21-9dac-fd1e72d91a41` did not force current-plan
promotion. T147 closes the installed-runtime gap for the T146 no-prompt `plan_work` `orient` path.
It does not authorize harness writes, lifecycle mutation, M6/migration/quarantine,
schema/storage/index changes, document-index changes, public MCP changes, payload changes,
PATH/profile/auth configuration changes, rollback, force-kill, deletion, or old-binary reinstall.

T150 matrix note:
`docs/BRAIN_HARNESS_T150_POST_T147_HARNESS_SIDE_EFFECT_AUDIT_2026-06-03.md` records the read-only
post-T147 continuation audit. T147 remains complete, and the active T146 runtime-refresh limitation
`019e89f4-7dba-7ae1-a559-85d924af31a3` is now stale because installed-runtime live validation
passed. The stale limitation is still a separate lifecycle approval gate, not an implicit archive or
supersede authorization. During read-only Claude Bridge critique, Claude Code still wrote two
session-end stub handoffs (`019e8bc0-59a2-7051-b667-e88a1a4861c0` and
`019e8bc0-59a2-7051-b667-e87463bc9b3b`), and read-only harness status/doctor checks still report
Claude Code `ready=false` with drifted installed SessionEnd hook/settings state. Therefore the next
product-moving gate is the existing exact T135 harness repair approval, and Claude Bridge should not
be used again for Engram Brain Harness consultation until T135 is approved/executed unless the user
explicitly accepts the known side-effect risk. T150 does not authorize harness writes, lifecycle
mutation, M6/migration/quarantine, schema/storage/index changes, document-index behavior changes,
ranking or `orient` changes, public MCP/payload changes, user-owned file adoption, deletion,
rollback, force-kill, or old-binary reinstall.

T152 matrix note:
`docs/BRAIN_HARNESS_T152_T135_HARNESS_REPAIR_VALIDATION_RESULT_2026-06-03.md` records the approved
T135 harness repair execution. Fresh dry-runs matched T135 before each approved one-at-a-time write:
generic policy creation, Codex skill updates, Gemini CLI command/context updates, Cursor skill
updates, and Claude Code generated SessionEnd hook plus `settings.local.json` merge. The run used
`adopt_user_owned=false`, did not edit root `AGENTS.md`, did not edit
`/Users/yuval.meiri/.claude/settings.json`, and preserved the user-owned Claude settings snippet.
Post-write live `harness(status)` and `harness(doctor)` checks report `ready=true` for generic,
Codex, Gemini CLI, Cursor, and Claude Code. Claude Code still warns about preserved user-owned
snippet state, extra legacy Engram permissions, split settings, and soft lifecycle enforcement.
`settings.local.json` parses, the installed SessionEnd hook is executable, and the hook defaults
missing `write_policy` to `nudge`. T152 closes the local generated-adapter readiness gap, but it
does not run native Claude Code behavioral validation, mutate lifecycle cleanup state, run
M6/migration/quarantine, change ranking or `orient`, change public MCP/schema/storage/index
behavior, or change document-index behavior.

T153/T154 matrix note:
`docs/BRAIN_HARNESS_T153_POST_T152_CLAUDE_STATIC_PREFLIGHT_2026-06-03.md` records static post-T152
preflight evidence only. It did not run native Claude Code, Claude Bridge, lifecycle hooks, harness
install, M6/migration/quarantine, lifecycle cleanup, ranking, `orient`, schema/storage/index,
public MCP, document-index, deletion, rollback, force-kill, or user-owned adoption actions. Static
checks found all five harnesses still `ready=true`, `settings.local.json` parses, the installed
SessionEnd hook is executable, and the hook defaults missing `write_policy` to `nudge`. Static
inspection also found existing explicit `"write_policy": "durable"` values in Claude settings for
other lifecycle hook inputs, so T153 is not proof that all effective Claude lifecycle behavior is
non-durable. `docs/BRAIN_HARNESS_T154_NATIVE_CLAUDE_VALIDATION_APPROVAL_PACKET_2026-06-03.md` is
the next default-deny approval packet for a native Claude non-session smoke limited to
`claude --version` and `claude --help`. Do not run Claude Bridge, prompt-bearing Claude commands,
interactive Claude sessions, Claude `/hooks`, or broader Engram Brain Harness validation without
separate exact approval.

T155 matrix note:
`docs/BRAIN_HARNESS_T155_COMPLETION_GATE_AUDIT_2026-06-03.md` records a read-only/docs-only
completion-gate audit after T153. The audit found the current T153 plan first in lean `orient` and
direct search, rechecked all five harnesses as `ready=true`, and used MCP lint after local CLI lint
failed on the daemon-owned database lock. Duplicate or late T135 approval does not reopen harness
writes because T135 was already executed and validated in T152. The full goal remains incomplete:
native Claude behavior, effective Claude hook configuration, lifecycle cleanup, M6/migration
completion or explicit deferral, and broader cross-harness behavioral validation are still missing
or approval-gated. Generic continuation is not T154 approval.

T156 matrix note:
`docs/BRAIN_HARNESS_T156_T154_PREFLIGHT_REFRESH_2026-06-03.md` records a read-only/static refresh
of the T154 preflight. It did not execute native Claude, Claude Bridge, Claude `/hooks`,
prompt-bearing Claude, lifecycle hooks, harness install, settings edits, M6/migration/quarantine,
lifecycle cleanup, ranking, `orient`, schema/storage/index, public MCP, document-index behavior,
deletion, rollback, force-kill, or user-owned adoption. The monitored Claude settings and
SessionEnd hook hashes still match T153, `/Users/yuval.meiri/.local/bin/claude` resolves to
`2.1.160`, Claude Code status/doctor remain `ready=true`, and the same split-settings,
legacy-permission, user-owned snippet, explicit durable-policy, and effective-hook caveats remain.
T154 still requires exact approval before any native Claude process.

T157 matrix note:
`docs/BRAIN_HARNESS_T157_STALE_CURRENT_PLAN_LIFECYCLE_APPROVAL_PACKET_2026-06-03.md` refreshes the
default-deny lifecycle request for stale current-plan MemoryItem
`019e5e0a-86b4-73e3-aa9b-ca350e83e915`. It does not archive, supersede, reject, delete, or review
any memory item and does not run `lint apply_safe`. Current evidence shows T156 current-plan memory
`019e8d05-dce0-7a82-9b23-30ce1405b5bd` is active and first for the current continuation class,
while the stale repository-scoped target still appears as active lower-rank guidance and read-only
lint reports 198 stale-current-plan feedback records plus 23 wrong-scope feedback records, both
with `safe_action=none`. T157 only asks for an exact future single-item archive after fresh matching
get/orient-or-search/lint/graph/git/obligations evidence and no intervening writes. It does not
authorize old handoff cleanup, broad lifecycle mutation, `lint apply_safe`, ranking, `orient`,
native Claude, Claude Bridge, harness writes, M6/migration/quarantine, public MCP, schema/storage/
index, document-index behavior, deletion, or user-owned-file changes.

T158 matrix note:
`docs/BRAIN_HARNESS_T158_T125_QUARANTINE_INSPECTION_APPROVAL_PACKET_2026-06-03.md`
records a docs-only/default-deny packet for the remaining T125 M6 read-only quarantine inspection
gate. It does not read quarantine files, run migration status/prioritize/apply/rerun, make candidate
decisions, mutate lifecycle state, change ranking or `orient`, change public MCP/schema/storage/
index or document-index behavior, run native Claude or Claude Bridge, or write harness files.
T123/T124 already inspected all nine review candidates from the written T68 snapshot. The next M6
inspection step now requires the exact T125 approval phrase before reading only quarantine files
0010-0011 and writing an inspection report.

T159 matrix note:
`docs/BRAIN_HARNESS_T159_STALE_T146_LIMITATION_LIFECYCLE_APPROVAL_PACKET_2026-06-03.md`
recorded a docs-only/default-deny lifecycle request for stale T146 runtime-refresh limitation
MemoryItem `019e89f4-7dba-7ae1-a559-85d924af31a3`. After exact user approval, fresh matching
read-only evidence, and no intervening writes, Codex archived exactly that item with the approved
payload. T147 validation proves the runtime-refresh gap it described is closed, read-only lint did
not flag the target, and graph depth 1 showed only evidence, project scope, and writer-session
edges, so the execution remained a human-approved manual archive, not a lint safe action. It did not
run `lint apply_safe`, archive any other item, change ranking/`orient`, run native Claude, Claude
Bridge, harness writes, M6/migration/quarantine, public MCP/schema/storage/index/document-index
behavior changes, deletion, or user-owned-file edits. Late duplicate T135 approval does not reopen
harness writes because T135 was already executed and validated in T152.

T160 matrix note:
`docs/BRAIN_HARNESS_T160_WRONG_SCOPE_CLAUDE_PROMPT_LIFECYCLE_APPROVAL_PACKET_2026-06-03.md`
recorded a docs-only/default-deny lifecycle request for wrong-scope active Claude Code prompt
capture MemoryItem `019e7f52-4fc2-7f61-93b4-9a741aba966e`. It identified a one-time telemetry
evidence-loop critique prompt captured as an active `rule`, not durable project guidance. The
packet itself did not archive the item, rerun harness writes, use Claude, run M6/migration/
quarantine, change ranking/`orient`, run `lint apply_safe`, change public MCP/schema/storage/index
or document-index behavior, delete anything, or touch user-owned files. The later exact-approved
archive execution is recorded separately in T168.

T161 matrix note:
`docs/BRAIN_HARNESS_T161_DUPLICATE_T135_COMPLETION_GATE_AUDIT_2026-06-03.md` records a
read-only/docs-only audit after a late duplicate T135 approval. T135 was already executed and
validated in T152, so the duplicate approval does not reopen harness writes. Fresh status and
doctor checks still report `ready=true` for generic, Codex, Gemini CLI, Cursor, and Claude Code,
while Claude Code keeps the known user-owned snippet, legacy-permission, split-settings, and soft
lifecycle caveats. Current T160 plan retrieval remains healthy for the duplicate-approval prompt,
but read-only lint still reports stale/wrong-scope active memory and many superseded-active items,
and telemetry feedback coverage is 42%, below the 50% confidence gate. The full goal remains
incomplete until native Claude/effective hook behavior, lifecycle cleanup, M6 migration/quarantine
completion or explicit deferral, and broader cross-harness behavior are handled under exact gates.
T161 does not run harness install, native Claude, Claude Bridge, lifecycle mutation, M6/migration/
quarantine, ranking/`orient`, `lint apply_safe`, schema/storage/index/public MCP/document-index
changes, deletion, rollback, force-kill, old-binary reinstall, or user-owned-file edits.

T162 matrix note:
`docs/BRAIN_HARNESS_T162_TELEMETRY_COVERAGE_FOLLOW_THROUGH_2026-06-03.md` records a
docs-only/telemetry-only follow-through for the T161 confidence-gate gap. The real-session eval
moved from 34% feedback coverage across two intents to 50% feedback coverage across four intents,
so the current 50-trace sample now passes the telemetry confidence gate. This is a threshold pass
in a sliding window, not durable migration readiness. Four verify-decision searches for exact
approval gates produced missing-context feedback because packet docs did not reliably surface;
`rg` against the repo packet files remains the authority for exact approval phrases until a
separately approved retrieval/document visibility slice exists. T162 does not authorize native
Claude, Claude Bridge, harness writes, lifecycle archive or `lint apply_safe`, M6/migration/
quarantine work, ranking/`orient`, public MCP/schema/storage/index/document-index behavior changes,
deletion, rollback, force-kill, old-binary reinstall, or user-owned-file edits.

T163 matrix note:
`docs/BRAIN_HARNESS_T163_RECENT_GATE_DOC_INDEX_APPROVAL_PACKET_2026-06-03.md` records a
docs-only/default-deny approval packet for exact-file indexing of seven recent Brain Harness gate
and audit docs: T154, T157, T158, T159, T160, T161, and T162. Fresh read-only document stats still
match the post-T70 state, and exact document searches for T154/T158/T160/T162 remain noisy or miss
the target docs in the top five. T163 does not run document indexing, change document-index
behavior, create packet MemoryItems, run native Claude or Claude Bridge, write harness files,
archive lifecycle memory, run `lint apply_safe`, inspect M6 quarantine files, make migration
decisions, change ranking/`orient`, change public MCP/schema/storage/index behavior, delete
anything, or touch user-owned files. Future execution requires the exact T163 approval phrase.

T164 matrix note:
`docs/BRAIN_HARNESS_T164_NO_APPROVAL_GATE_STATE_AUDIT_2026-06-03.md` records a read-only/docs-only
continuation audit after no exact approval phrase was provided. It confirms that T163 current-plan
memory still surfaces first in lean `orient` and direct current-plan search, all seven T163 target
files exist, document stats still match the post-T70/pre-T163 state, and exact document searches
for T154/T157/T158/T159/T160/T161/T162 still miss the recent target docs in the top five. Telemetry
now passes the current 50-trace confidence gate at 27/50 feedback coverage across four intents, but
the pass remains a sliding-window evidence-quality signal with missing-context records, not M6
readiness. Lint still reports stale/wrong-scope/superseded active memory with no applied safe
action. T164 does not run document indexing, lifecycle archive, `lint apply_safe`, native Claude,
Claude Bridge, harness writes, M6/migration/quarantine, ranking/`orient`, public MCP,
schema/storage/index, deletion, rollback, force-kill, old-binary reinstall, or user-owned-file
edits. The next executable product-moving gate remains exact T163 approval.

T165 matrix note:
`docs/BRAIN_HARNESS_T165_T163_DOCUMENT_INDEX_RESULT_2026-06-03.md` records the completed T163
exact-file document-visibility repair. The seven approved target files T154, T157, T158, T159,
T160, T161, and T162 were indexed individually through `docs(action="index", path=...)`, producing
7 new sources and 72 new searchable chunks with no warnings and no orphan increase. Validation
passes for T157, T158, T159, T160, T161, T162, and T154 by actual title plus exact approval phrase.
The synthetic T154 query from the T163 packet remains noisy because the document's actual title is
`T154 Native Claude Non-Session Smoke Approval Packet`. T165 does not authorize native Claude,
Claude Bridge, lifecycle archive, `lint apply_safe`, M6/migration/quarantine, ranking/`orient`,
public MCP, schema/storage/index behavior changes, document-index behavior changes, deletion, or
user-owned-file edits.

T166 matrix note:
`docs/BRAIN_HARNESS_T166_T157_LIFECYCLE_ARCHIVE_RESULT_2026-06-03.md` records execution of the
exact-approved T157 single-target lifecycle archive. After fresh matching read-only evidence and no
intervening writes, Codex archived only stale repository-scoped current-plan MemoryItem
`019e5e0a-86b4-73e3-aa9b-ca350e83e915` with the approved T157 payload. Post-archive validation
shows the target is `status=archived`, lean `orient` no longer returns it, `lint(run, write=false)`
no longer reports stale-current-plan or wrong-scope findings for that ID, and `changes_since`
showed exactly that archive state change. T166 does not run `lint apply_safe`, archive any other
memory, change handoff semantics, ranking/`orient`, public MCP/schema/storage/index/document-index
behavior, native Claude, Claude Bridge, harness files, M6/migration/quarantine, deletion, or
user-owned-file edits. Lifecycle cleanup remained partial at T166 time; T159 is now closed by T167,
T160 is now closed by T168, and broad superseded-active handoff/memory cleanup remains out of scope.

T167 matrix note:
`docs/BRAIN_HARNESS_T167_T159_LIFECYCLE_ARCHIVE_RESULT_2026-06-03.md` records execution of the
exact-approved T159 single-target lifecycle archive. After fresh matching read-only evidence and no
intervening writes, Codex archived only stale T146 runtime-refresh limitation MemoryItem
`019e89f4-7dba-7ae1-a559-85d924af31a3` with the approved T159 payload. Post-archive validation
shows the target is `status=archived`, lean `orient` continues to return the active current plan
first, targeted memory search no longer returns the archived target, `lint(run, write=false)` no
longer reports any finding for that ID, and `changes_since` showed exactly that archive state
change. T167 does not run `lint apply_safe`, archive any other memory, change handoff semantics,
ranking/`orient`, public MCP/schema/storage/index/document-index behavior, native Claude, Claude
Bridge, harness files, M6/migration/quarantine, deletion, or user-owned-file edits. Lifecycle
cleanup remained partial at T167 time; T160 is now closed by T168, while broad superseded-active
handoff/memory cleanup remains out of scope.

T168 matrix note:
`docs/BRAIN_HARNESS_T168_T160_LIFECYCLE_ARCHIVE_RESULT_2026-06-03.md` records execution of the
exact-approved T160 single-target lifecycle archive. After fresh matching read-only evidence and no
intervening writes, Codex archived only wrong-scope Claude Code prompt-capture MemoryItem
`019e7f52-4fc2-7f61-93b4-9a741aba966e` with the approved T160 payload. Post-archive validation
shows the target is `status=archived`, lean `orient` continues to return the active current plan
first, targeted memory search no longer returns the archived target, `lint(run, write=false)` no
longer reports any finding for that ID, and `changes_since` showed exactly that archive state
change. T168 does not run `lint apply_safe`, archive any other memory, change handoff semantics,
ranking/`orient`, public MCP/schema/storage/index/document-index behavior, native Claude, Claude
Bridge, harness files, M6/migration/quarantine, deletion, or user-owned-file edits. Lifecycle
cleanup remains partial: broad superseded-active handoff/memory cleanup remains out of scope.

T169 matrix note:
`docs/BRAIN_HARNESS_T169_T125_QUARANTINE_INSPECTION_REPORT_2026-06-03.md` records the exact-approved
T125 read-only inspection of quarantine candidate files 0010-0011 from the written T68 M6
review-export snapshot. T123/T124 had already inspected candidate files 0001-0009, so T169 closes
the remaining bounded inspection gap for the T68 snapshot. T169 does not decide, accept, edit,
reject, promote, archive, delete, apply, or migrate any candidate and does not run migration
status/prioritize/apply/rerun, lifecycle mutation, ranking/`orient`, public MCP/schema/storage/
index/document-index behavior changes, native Claude, Claude Bridge, harness writes, or
user-owned-file edits. The next M6 step remains separate: reviewed candidate decisions and/or a
status/dry-run/apply plan with rollback and explicit approval.

T170 matrix note:
`docs/BRAIN_HARNESS_T170_T154_NATIVE_CLAUDE_SMOKE_RESULT_2026-06-03.md` records the exact-approved
T154 native Claude non-session smoke. The only native commands executed were
`/Users/yuval.meiri/.local/bin/claude --version` and
`/Users/yuval.meiri/.local/bin/claude --help`. Both exited successfully; `--version` returned
`2.1.161 (Claude Code)`, monitored Claude settings/hook hashes remained unchanged, Memory OS
`changes_since` from the pre-command cursor returned zero items/commits after each command,
obligations stayed clean, and harness status remained ready with known warnings. T170 records a
binary-target drift from T156's observed `2.1.160` symlink target to `2.1.161`. T170 does not prove
interactive Claude Code hook behavior, prompt-bearing native Claude behavior, missing SessionEnd
`write_policy` behavior, or whole-home hidden-write absence; those remain separate approval gates.

T171 matrix note:
`docs/BRAIN_HARNESS_T171_POST_T170_COMPLETION_GATE_AUDIT_2026-06-03.md` records a docs-only/read-only
completion audit after T169/T170. Fresh evidence shows lean `orient` returns active current-plan
MemoryItem `019e8d9a-9ba3-7b31-90b5-70d296d6d63a` first, all five harnesses report `ready=true`,
Claude Code still has split-settings/user-owned-snippet/legacy-permission/effective-hook caveats,
obligations doctor is clean, and read-only lint still reports wrong-scope active-memory plus broad
superseded-active findings with no safe action applied. The current 50-trace telemetry gate fails
at 32% feedback coverage across two feedback-bearing intents. The full goal remains incomplete
until effective native Claude hook behavior, prompt-bearing native behavior, missing SessionEnd
`write_policy` behavior, M6 migration completion or explicit deferral, reviewed candidate
decisions/dry-run evidence, sliding-window telemetry confidence, and broad lifecycle cleanup or
deferral are handled under their own gates.

T172 matrix note:
`docs/BRAIN_HARNESS_T172_NATIVE_CLAUDE_EFFECTIVE_HOOK_VALIDATION_APPROVAL_PACKET_2026-06-03.md`
records a docs-only/default-deny approval packet for the next recommended Claude gate. Fresh
read-only evidence still shows Claude Code `ready=true`, the same monitored user-level Claude
settings/hook hashes as T170, binary target `2.1.161`, clean obligations, and git status clean
except the pre-existing untracked root `AGENTS.md`; project-local `.claude` files are also present
and must be monitored. The packet does not execute native Claude, Claude Bridge, `/hooks`, prompt-
bearing Claude, lifecycle writes, harness installs, settings/hook edits, M6/migration/quarantine,
ranking/`orient`, public MCP/schema/storage/index/document-index changes, deletion, rollback,
force-kill, old-binary reinstall, or user-owned-file adoption. Its proposed future approval allows
exactly one native Claude PTY session with one `/hooks` input plus EOF, pre/post state comparisons,
and observation/reporting of any side effects caused by that exact session, with no retries or
cleanup.

T173 matrix note:
`docs/BRAIN_HARNESS_T173_TELEMETRY_AND_STALE_APPROVAL_FOLLOW_THROUGH_2026-06-03.md` records a
docs-only/telemetry-only follow-through after duplicate/stale T125 and T154 approvals. Git and
docs evidence show T125 was already completed by T169 (`3dfc23d`) and T154 by T170 (`ebe835d`), so
neither was rerun. Baseline telemetry over the current 50-trace window failed at 48% feedback
coverage despite five feedback-bearing intents; after honest feedback on six assessable current
retrieval traces, the window passed at 30/50 feedback traces, 60% coverage, five intents, and zero
bad-memory use. The feedback explicitly recorded missing-context/noise where searches missed the
completed T169/T170 result reports. T173 does not authorize native Claude, Claude Bridge, `/hooks`,
prompt-bearing Claude, harness writes, lifecycle archive, `lint apply_safe`, M6/migration/
quarantine, candidate decisions, ranking/`orient`, public MCP/schema/storage/index/document-index
behavior changes, deletion, rollback, force-kill, old-binary reinstall, or user-owned-file edits.
The next product-moving gate remains exact T172 approval; M6 candidate decisions and migration
dry-run/apply planning remain separate approval gates.

T174 matrix note:
`docs/BRAIN_HARNESS_T174_M6_CANDIDATE_DECISION_DRY_RUN_SCOPING_APPROVAL_PACKET_2026-06-03.md`
records a docs-only/default-deny approval packet for the next M6 candidate-decision readiness and
dry-run scoping step after T123/T124/T169 completed inspection of the T68 review-export snapshot.
It does not execute M6 status/prioritize/apply/rerun/review-export, make candidate decisions,
mutate lifecycle state, change ranking/`orient`, change public MCP/schema/storage/index/
document-index behavior, run native Claude or Claude Bridge, edit harness files, delete, rollback,
force-kill, reinstall binaries, or touch user-owned files. The proposed future approval allows only
reading committed inspection/telemetry reports, validating and reading exact existing snapshot
files 0001-0011 plus `index.md`, at most one read-only M6 review status/readiness check if proven
safe before invocation, and writing a docs-only readiness/scoping result. T174 preserves that
candidate decisions, dry-run apply, write apply, deletion, native Claude effective-hook validation,
and broad lifecycle cleanup remain separate exact approval gates.

T175 matrix note:
`docs/BRAIN_HARNESS_T175_RECENT_GATE_DOC_INDEX_APPROVAL_PACKET_2026-06-03.md` records a
docs-only/default-deny exact-file document-index visibility packet for the recent T172, T173, and
T174 gate docs. Fresh document-only searches for the T172 title/approval phrase, T173 title, T174
title/approval phrase, and T174 filename stem returned older indexed docs such as T154, T162, T158,
T160, T159, T157, and T69 instead of the new target docs. T175 does not run document indexing or
change document-index behavior; it asks for exact future approval to index only three named files.
It does not authorize native Claude, Claude Bridge, Claude `/hooks`, prompt-bearing Claude,
harness writes, lifecycle archive, `lint apply_safe`, M6/migration/quarantine, candidate
decisions, ranking/`orient`, public MCP/schema/storage/index behavior changes, deletion, rollback,
force-kill, old-binary reinstall, or user-owned-file edits.

T176 matrix note:
`docs/BRAIN_HARNESS_T176_T175_DOCUMENT_INDEX_RESULT_2026-06-03.md` records the exact-approved T175
document-index execution. Codex indexed only the T172, T173, and T174 packet/report files named by
T175 through three file-path `docs(action="index", path=...)` calls. The run added three sources
and 37 searchable chunks with no warnings. Post-index validation returned T172 in the top five for
both approved T172 probes, T173 first for its exact title probe, and T174 in the top five for both
approved T174 probes. T176 is document visibility evidence only: it does not authorize or execute
the T172 native Claude effective-hook session, the T174 read-only M6 scoping packet, candidate
decisions, migration apply, lifecycle cleanup, harness writes, ranking/`orient`, public MCP,
schema/storage/index behavior, document-index behavior, deletion, cleanup, rollback, force-kill,
old-binary reinstall, or user-owned-file edits. The next product-moving gates remain exact T172
approval or exact T174 approval.

T177 matrix note:
`docs/BRAIN_HARNESS_T177_T176_MATRIX_DOC_INDEX_APPROVAL_PACKET_2026-06-03.md` records a
docs-only/default-deny exact-file document-index visibility packet for the T176 result report and
the updated central implementation plan. Fresh document stats still match the T175 post-index
state, and document search for the T176 title or T176 matrix-note wording does not surface the
newest T176 report/matrix content in the top five. T177 does not run document indexing or change
document-index behavior; it asks for exact future approval to index only
`docs/BRAIN_HARNESS_T176_T175_DOCUMENT_INDEX_RESULT_2026-06-03.md` and
`docs/MEMORY_OS_IMPLEMENTATION_PLAN.md`. It does not authorize native Claude, Claude Bridge,
Claude `/hooks`, prompt-bearing Claude, harness writes, lifecycle archive, `lint apply_safe`,
M6/migration/quarantine, candidate decisions, ranking/`orient`, public MCP/schema/storage/index
behavior changes, deletion, cleanup, rollback, force-kill, old-binary reinstall, or user-owned-file
edits.

T178 matrix note:
`docs/BRAIN_HARNESS_T178_T177_DOCUMENT_INDEX_RESULT_2026-06-03.md` records the exact-approved T177
document-index execution. Codex indexed only the T176 result report and
`docs/MEMORY_OS_IMPLEMENTATION_PLAN.md` through two exact file-path
`docs(action="index", path=...)` calls. The T176 report indexed as one new source with eight
chunks; the implementation plan reindexed with 336 chunks; both calls returned no warnings. Post-
index stats were `source_count=89`, `chunk_count=4346`, `searchable_chunk_count=2334`,
`orphan_chunk_count=2012`, and `embedding_dimension=384`. Validation fixed the T176 exact-title
probe: `T176 T175 Document Index Result` returned the T176 report first. The exact T176 matrix-note
probe remained noisy, but the approved `T175 document-index execution T172 T173 T174 37 searchable
chunks` probe returned the T176 report first and the newly indexed implementation-plan T175/T176
matrix chunk fifth. T178 is document visibility evidence only: it does not authorize or execute
T172 native Claude validation, T174 read-only M6 scoping, candidate decisions, migration apply,
lifecycle cleanup, harness writes, ranking/`orient`, public MCP, schema/storage/index behavior,
document-index behavior, deletion, cleanup, rollback, force-kill, old-binary reinstall, or
user-owned-file edits. The next product-moving gates remain exact T172 approval or exact T174
approval; M6 migration write-apply remains separately gated by reviewed decisions, dry-run/rollback
evidence, telemetry readiness, and explicit user approval.

T179 matrix note:
`docs/BRAIN_HARNESS_T179_T172_NATIVE_CLAUDE_EFFECTIVE_HOOK_RESULT_2026-06-03.md` records the
exact-approved T172 native Claude effective-hook validation and approved recovery attempt. Preflight
matched the packet: Claude target/version was `2.1.161`, monitored user-level hashes matched,
harness status/doctor were `ready=true`, obligations were clean, git had no tracked diff, and no
native Claude process was already running. The single native PTY emitted visible Engram
`SessionStart:startup` activation text, received only the approved `/hooks` input, but produced no
visible effective-hook configuration output. Initial EOF did not exit. The approved recovery sent
Ctrl-C, then EOF after prompt control returned; Claude still requested repeated Ctrl-D and remained
live as PID `49349`, so no further input or force-kill was run. Postflight comparisons found no
tracked git changes, no monitored Claude hash drift, no Memory OS writes, no obligation changes,
and unchanged harness readiness. T179 is a hard-stop result: it proves native startup hook output
is visible, but it does not close the effective-hook visibility gate and does not prove
prompt-bearing native Claude behavior, missing SessionEnd `write_policy` behavioral semantics,
M6 migration readiness, lifecycle cleanup, ranking/`orient`, public MCP, schema/storage/index,
document-index behavior, deletion, rollback, force-kill, or user-owned-file changes. The next step
requires explicit user approval to resolve PID `49349` or a new bounded native validation/recovery
packet; T174 remains the separate M6 read-only scoping gate.

T180 matrix note:
`docs/BRAIN_HARNESS_T180_T179_NATIVE_CLAUDE_LIVE_PROCESS_RECOVERY_APPROVAL_PACKET_2026-06-03.md`
records a docs-only/default-deny approval packet for the next native-Claude cleanup gate after
T179. Fresh read-only evidence shows PID `49349` is still live as
`/Users/yuval.meiri/.local/bin/claude`, Claude Code remains `2.1.161`, the symlink target remains
`/Users/yuval.meiri/.local/share/claude/versions/2.1.161`, monitored user-level and project-local
Claude hashes still match T179, harness status/doctor remain `ready=true` with known caveats, and
git is clean except pre-existing untracked root `AGENTS.md`. T180 does not send input, signal or
kill the process, launch native Claude, run Claude Bridge, probe `/hooks`, edit hooks/settings/
adapters, run harness install, mutate lifecycle or migration state, change ranking/`orient`, public
MCP/schema/storage/index/document-index behavior, delete, roll back, reinstall binaries, or touch
user-owned files. Its proposed future approval allows only one additional Ctrl-C to the same live
PTY if that exact PTY route is still available, followed by read-only comparisons and a docs-only
result. It explicitly forbids EOF, another Ctrl-C, process-level signals, force-kill, new native
Claude sessions, and fallback cleanup. T180 can resolve the live-process gate if executed cleanly,
but it cannot close the effective-hook visibility gate, prompt-bearing native Claude behavior,
missing SessionEnd `write_policy` behavior, M6 migration readiness, lifecycle cleanup, or broad
Brain Harness completion.

T181 matrix note:
`docs/BRAIN_HARNESS_T181_T179_T180_DOC_INDEX_APPROVAL_PACKET_2026-06-03.md` records a
docs-only/default-deny exact-file document-index visibility packet for the latest T179 result,
T180 recovery approval packet, and updated implementation plan. Fresh document-only searches for
the T180 exact title, T180 filename stem, and T179 hard-stop/PID wording returned older indexed
documents rather than the new T179/T180 files, while direct unified search still recovered current
T180 current-plan memory first. T181 does not run document indexing or change document-index
behavior; it asks for exact future approval to index only T179, T180, and
`docs/MEMORY_OS_IMPLEMENTATION_PLAN.md`. It does not authorize native Claude input, process
signals, force-kill, Claude Bridge, `/hooks`, prompt-bearing Claude, harness writes, lifecycle
archive, `lint apply_safe`, M6/migration/quarantine, candidate decisions, ranking/`orient`, public
MCP/schema/storage/index behavior changes, deletion, rollback, old-binary reinstall, or
user-owned-file edits. T180 live-process recovery and T174 M6 read-only scoping remain separate
exact approval gates.

T182 matrix note:
`docs/BRAIN_HARNESS_T182_T181_DIRECT_SEARCH_CURRENT_PLAN_APPROVAL_PACKET_2026-06-03.md`
records a docs-only/default-deny packet for a narrow direct unified `search` current-plan ranking
miss observed after T181. Fresh evidence is split: lean `orient` trace
`019e8e57-7110-7491-b72e-f0377f5a4887` and simpler direct search trace
`019e8e57-f73d-7c72-80c0-8b5973a8cd1e` both recovered active T181 current-plan memory
`019e8e54-4595-7931-8b7d-061086f9ddb4` first, but exact observed direct-search trace
`019e8e58-2498-7bc0-8520-032122b36920` for
`current next action after T181 exact approval gate non-gated work Brain Harness T180 T174 document
indexing` ranked older active rolling handoffs above the current plan. Source inspection suggests
the narrow ambiguity is `next action` plus approval-gate-context wording: current-plan guidance
recognizes `next step` but not `next action`, while `approval gate` remains contextual only when
current-plan guidance is recognized. T182 does not change ranking, `orient`, public MCP,
schema/storage/index/document-index behavior, document indexing, lifecycle state, M6/migration/
quarantine, native Claude, Claude Bridge, harness files/settings/hooks/adapters, user-owned files,
deletion, rollback, force-kill, or installed runtime configuration. It asks for exact future
approval to add a focused fixture and the smallest prompt-class source adjustment, with explicit
gate/action prompts preserved.

T183 matrix note:
`docs/BRAIN_HARNESS_T183_POST_T182_COMPLETION_GATE_AUDIT_2026-06-03.md` records a
docs-only/read-only completion and gate audit after T182 current-plan capture. Fresh lean `orient`
trace `019e8e5a-c517-71f3-ad5e-51df3ee34904` and simple direct-search trace
`019e8e5a-c6f1-7bd1-9cfa-d5fa3cbb11f0` returned active T182 current-plan memory
`019e8e59-e203-76e0-9996-6be2a3fdd8f0` first, while exact observed direct memory-search trace
`019e8e5b-4f3e-72e1-8b0c-4e53b830a233` still ranked older rolling handoffs above current-plan
guidance for `current next action after T181 exact approval gate non-gated work Brain Harness T180
T174 document indexing`. Exact T182 approval-query trace `019e8e5b-4f8b-7e11-a37f-d38c165d2b33`
returned T182 first, and explicit migration-apply trace `019e8e5b-4fb8-7951-9f90-128b1bddcd10`
kept the migration review gate first. Document search did not recover the new T182 packet because
it has not been exact-file indexed. T183 does not change source behavior, ranking, `orient`, public
MCP, schema/storage/index/document-index behavior, lifecycle state, M6/migration/quarantine,
native Claude, Claude Bridge, harness files/settings/hooks/adapters, user-owned files, deletion,
rollback, force-kill, or installed runtime configuration. The goal remains incomplete; T182, T181,
T180, T174, and high-risk migration completion remain separate exact approval gates.

T184 matrix note:
`docs/BRAIN_HARNESS_T184_T182_T183_DOC_INDEX_APPROVAL_PACKET_2026-06-03.md` records a
docs-only/default-deny exact-file document-index visibility packet for the newest T182 and T183
gate documents. Fresh document-only searches for the T182 exact title, T183 exact title, T183
commit probe `c75b509 Record T183 completion gate audit`, and T182/T183 current-plan wording
returned older indexed documents rather than the new T182/T183 files. T184 intentionally excludes
`docs/MEMORY_OS_IMPLEMENTATION_PLAN.md` because the updated implementation plan remains part of
the separate pending T181 exact-file indexing gate with T179/T180. T184 does not run document
indexing or change document-index behavior; it asks for exact future approval to index only T182
and T183. It does not authorize T182 ranking/source/test changes, T181 indexing, T180 native
Claude input or process recovery, T174 M6 scoping, lifecycle archive, `lint apply_safe`, candidate
decisions, M6/migration/quarantine work, ranking/`orient`, public MCP/schema/storage/index
behavior changes, deletion, rollback, old-binary reinstall, or user-owned-file edits.

T185 matrix note:
`docs/BRAIN_HARNESS_T185_T172_RECOVERY_OPTION_2_RERUN_RESULT_2026-06-03.md` records a bounded
rerun of the user-approved T172 recovery option 2 wording after the prior T179 hard stop. Fresh
preflight matched the T179/T180 state: PID `49349` was still live on `ttys000`, Claude Code remained
`2.1.161`, the symlink target and monitored user/project Claude hashes matched, harness
status/doctor were `ready=true` with the known warnings, obligations were clean, Memory OS had no
new item or commit changes since the pre-recovery cursor, and git had no tracked diff. Codex sent
exactly one Ctrl-C byte to `/dev/ttys000`; it did not use a process signal, send EOF, send another
Ctrl-C, run `/hooks`, send natural-language input, launch Claude, run Claude Bridge, edit
hooks/settings/adapters, run harness install, mutate lifecycle or migration state, change
ranking/`orient`, public MCP/schema/storage/index/document-index behavior, delete, roll back,
force-kill, reinstall binaries, or touch user-owned files. PID `49349` remained live after Ctrl-C.
EOF was not sent because the approval was conditional on Claude returning to a prompt, and after
context compaction Codex had no reliable transcript handle: `read_thread_terminal` had no attached
app terminal and Computer Use could not inspect the Codex app. Postflight found no tracked git
changes, no monitored hash drift, no Memory OS writes, no obligation changes, and unchanged harness
readiness. T185 does not close the live-process cleanup gate or the T172 effective-hook visibility
gate; the next cleanup step still needs explicit fresh approval that states whether EOF without
visible prompt evidence, another Ctrl-C, a process-level signal, force-kill, or user/manual
intervention is allowed.

T186 matrix note:
`docs/BRAIN_HARNESS_T186_T185_NATIVE_CLAUDE_SIGINT_CLEANUP_APPROVAL_PACKET_2026-06-03.md` records
a docs-only/default-deny approval packet for the next native-Claude cleanup gate after T185. Fresh
read-only evidence shows PID `49349` is still live as `/Users/yuval.meiri/.local/bin/claude` on
`ttys000`, Claude Code remains `2.1.161`, the symlink target remains
`/Users/yuval.meiri/.local/share/claude/versions/2.1.161`, monitored user-level and project-local
Claude hashes still match T179/T180/T185, harness status/doctor remain `ready=true` with known
caveats, obligations are clean, Memory OS has no new item or commit changes since the latest
current-plan cursor, and git is clean except pre-existing untracked root `AGENTS.md`. T186 does
not send input, signal or kill the process, launch native Claude, run Claude Bridge, probe
`/hooks`, edit hooks/settings/adapters, run harness install, mutate lifecycle or migration state,
change ranking/`orient`, public MCP/schema/storage/index/document-index behavior, delete, roll
back, reinstall binaries, or touch user-owned files. Its proposed future approval allows only one
process-level `SIGINT` using `kill -INT 49349` after fresh matching read-only preflight and no
intervening writes, followed by read-only comparisons and a docs-only result. It explicitly
forbids PTY input, EOF, Ctrl-C bytes, any other signal, a second `SIGINT`, `SIGTERM`, `SIGKILL`,
force-kill, new native Claude sessions, and fallback cleanup. T186 can resolve the live-process
gate if executed cleanly, but it cannot close the effective-hook visibility gate, prompt-bearing
native Claude behavior, missing SessionEnd `write_policy` behavior, M6 migration readiness,
lifecycle cleanup, ranking/`orient`, public MCP, schema/storage/index, document-index behavior, or
broad Brain Harness completion.

T187 matrix note:
`docs/BRAIN_HARNESS_T187_STALE_HANDOFF_LIFECYCLE_APPROVAL_PACKET_2026-06-03.md` records a
docs-only/default-deny lifecycle packet for three exact stale rolling handoff MemoryItems created
or exposed during the T186 handoff-refresh maintenance:
`019e8e6b-bb32-7832-9389-22dd04cbfcda`,
`019e8e6a-dd68-79d3-8bcb-704bc9c52fca`, and
`019e8bc0-59a2-7051-b667-e88a1a4861c0`. Fresh read-only evidence shows lean `orient` returns
active current-plan memory `019e8e6b-fac4-72b2-b702-d7df6356908c` first, `handoff(get)` returns
latest active handoff `019e8e6c-361a-73a0-933e-fcb12c599247`, direct search still surfaces stale
active handoffs as top search noise after the current plan/latest handoff, and graph/memory get
evidence shows the supersession chain `019e8e6c-361a...` -> `019e8e6b-bb32...` ->
`019e8e6a-dd68...` -> `019e8bc0-59a...`. T187 does not archive anything or run `lint apply_safe`;
it asks for exact future approval before these three archive writes. It intentionally excludes
older T140-T145 handoff noise because that requires separate target-local evidence or broader
explicit approval. T187 does not authorize handoff semantics changes, ranking, `orient`, public
MCP/schema/storage/index/document-index behavior, M6/migration/quarantine, native Claude, Claude
Bridge, process signals, harness installs/settings/hooks/adapters, lifecycle writes beyond the
three exact targets if approved, deletion, rollback, force-kill, or user-owned-file edits. T186
live-process cleanup remains the immediate product-moving gate.

T188 matrix note:
`docs/BRAIN_HARNESS_T188_T187_DOC_INDEX_APPROVAL_PACKET_2026-06-03.md` records a
docs-only/default-deny exact-file document-index approval packet for the newest T187 lifecycle
packet and this implementation plan. Fresh lean `orient` trace
`019e8e75-31b6-7042-9c1b-8730e81bee07` and direct current-plan search trace
`019e8e75-5d19-78b0-9275-bab086ea93eb` recover active current-plan memory
`019e8e70-d568-7e90-8f16-6405dd191b27` first, but exact document-layer query trace
`019e8e76-1826-7062-8227-91c58f2eb372` for the T187 title and target IDs returns older lifecycle
and document-index reports rather than T187. T188 does not run `docs(action="index")`, change
document-index behavior, archive memory, run `lint apply_safe`, signal PID `49349`, send native
Claude input, launch Claude or Claude Bridge, run harness install, mutate lifecycle or migration
state, inspect M6/quarantine candidates, change ranking or `orient`, change public
MCP/schema/storage/index behavior, delete, roll back, reinstall binaries, or touch user-owned files.
Its proposed future approval allows indexing exactly
`docs/BRAIN_HARNESS_T187_STALE_HANDOFF_LIFECYCLE_APPROVAL_PACKET_2026-06-03.md` and
`docs/MEMORY_OS_IMPLEMENTATION_PLAN.md` after fresh git/path/document-search/obligations evidence,
then read-only document-search validation and a docs-only result. T186 live-process cleanup, T187
lifecycle archive, M6/migration, and broad Brain Harness completion remain separate exact-gated
work.

T189 matrix note:
`docs/BRAIN_HARNESS_T189_TELEMETRY_COMPLETION_GATE_FOLLOW_THROUGH_2026-06-03.md` records telemetry
feedback and a docs-only completion-risk follow-through after T188. Four assessable traces from the
current turn were scored: startup orient `019e8e78-99e4-7b62-b4a3-07aad94e85cd`, current-plan
search `019e8e78-bcad-7b00-850f-b3976acc8cdb`, architecture/completion search
`019e8e78-be6f-7ae1-8ad6-aca640a426ca`, and design-philosophy search
`019e8e78-c039-7c92-9b50-b9fc590b2da1`. Feedback coverage in the current 50-trace real-session
window improved from 9 traces / 18% to 13 traces / 26%, with `bad_memory_used_count=0`, but the
confidence gate still fails because coverage is below 50% and feedback spans only one intent rather
than three. T189 does not run T186, T188, T187, document indexing, lifecycle archive, `lint
apply_safe`, M6/migration/quarantine, native Claude, Claude Bridge, process signals, harness writes,
ranking/`orient`/source/public MCP/schema/storage/index/document-index behavior changes, deletion,
rollback, old-binary reinstall, or user-owned-file edits. It records that telemetry remains a
completion blocker, not completion proof.

T190 matrix note:
`docs/BRAIN_HARNESS_T190_T172_RECOVERY_OPTION_2_REPEAT_RESULT_2026-06-03.md` records a second
bounded execution of the user-approved T172 recovery option 2 wording after T185. Fresh preflight
matched the live-process state: PID `49349` was still `/Users/yuval.meiri/.local/bin/claude` on
`ttys000`, Claude Code remained `2.1.161`, the symlink target and monitored user/project Claude
hashes matched T172/T179/T185/T186, harness status/doctor remained `ready=true` with known
warnings, obligations were clean, Memory OS had no new item or commit changes since the latest
current-plan cursor, and git had no tracked diff. Codex sent exactly one Ctrl-C byte to
`/dev/ttys000`; it did not use a process signal, send EOF, send another Ctrl-C, run `/hooks`, send
natural-language input, launch Claude, run Claude Bridge, edit hooks/settings/adapters, run harness
install, mutate lifecycle or migration state, change ranking/`orient`, public
MCP/schema/storage/index/document-index behavior, delete, roll back, force-kill, reinstall
binaries, or touch user-owned files. PID `49349` remained live after Ctrl-C. EOF was not sent
because the approval made EOF conditional on Claude returning to a prompt, and prompt-return state
could not be verified from the app terminal, open process handles, `.claude/sessions/49349.json`,
or sampled project JSONL transcript files. Postflight found no tracked git changes, no monitored
hash drift, no Memory OS writes, no obligation changes, and unchanged harness readiness. T190 does
not close the live-process cleanup gate or the T172 effective-hook visibility gate; the already
written T186 process-level SIGINT packet remains a separate exact-approval gate before any
`kill -INT 49349` signal can be sent.

T191 matrix note:
`docs/BRAIN_HARNESS_T191_POST_T190_STALE_HANDOFF_LIFECYCLE_APPROVAL_PACKET_2026-06-03.md`
records a docs-only/default-deny lifecycle packet for five active rolling handoffs now superseded
by the latest T190 handoff:
`019e8e83-4461-7500-909c-241183737348`,
`019e8e7b-5977-7f93-be7c-742da46f6831`,
`019e8e77-dfbc-7ae2-990a-df9368b75fc3`,
`019e8e71-1932-72e3-bac4-bd5abe9248f5`, and
`019e8e6c-361a-73a0-933e-fcb12c599247`. Fresh evidence shows latest
`handoff(get)` returns `019e8e84-44bf-7d31-bded-88fe36f96659`, direct current-plan search still
returns active current-plan memory `019e8e84-1927-7a11-85f0-36792b244ad1` first, focused handoff
search still surfaces the proposed targets as active handoff noise, `memory(get)` and graph
evidence show the direct supersession chain from `019e8e84-44bf...` through the five targets, and
`memory(changes_since)` from the startup cursor returned no memory item or commit changes. T191
does not archive anything, run `lint apply_safe`, change handoff semantics, ranking, `orient`,
public MCP/schema/storage/index/document-index behavior, M6/migration/quarantine, native Claude,
Claude Bridge, process signals, harness installs/settings/hooks/adapters, deletion, rollback,
force-kill, or user-owned files. It intentionally excludes already-packeted T187 targets, which
remain separately exact-gated.

T192 matrix note:
`docs/BRAIN_HARNESS_T192_T191_DOC_INDEX_APPROVAL_PACKET_2026-06-03.md` records a
docs-only/default-deny exact-file document-index approval packet for the newest T191 lifecycle
packet and this implementation plan. Fresh lean `orient` trace
`019e8e8d-5e29-7581-b681-ddde8f33879f` and direct current-plan search trace
`019e8e8d-66a5-7993-9a0c-31ddfd3991f7` recover active current-plan memory
`019e8e8a-a124-7822-87df-817e2d78be05` first, but exact document-layer probes for the T191 title,
filename stem, commit probe, and five archive target IDs return older indexed lifecycle and
document-index artifacts rather than T191. T192 does not run `docs(action="index")`, change
document-index behavior, archive memory, run `lint apply_safe`, signal PID `49349`, send native
Claude input, launch Claude or Claude Bridge, run harness install, mutate lifecycle or migration
state, inspect M6/quarantine candidates, change ranking or `orient`, change public
MCP/schema/storage/index behavior, delete, roll back, reinstall binaries, or touch user-owned
files. Its proposed future approval allows indexing exactly
`docs/BRAIN_HARNESS_T191_POST_T190_STALE_HANDOFF_LIFECYCLE_APPROVAL_PACKET_2026-06-03.md` and
`docs/MEMORY_OS_IMPLEMENTATION_PLAN.md` after fresh git/path/document-search/obligations evidence,
then read-only document-search validation and a docs-only result. T186 live-process cleanup, T191
lifecycle archive, T187 lifecycle archive, M6/migration, and broad Brain Harness completion remain
separate exact-gated work.

T193 matrix note:
`docs/BRAIN_HARNESS_T193_STALE_T106_HANDOFF_LIFECYCLE_APPROVAL_PACKET_2026-06-03.md` records a
docs-only/default-deny lifecycle packet for one active stale rolling handoff MemoryItem:
`019e839e-f061-71c2-95d3-f4c44029ac7b`. Fresh evidence shows current-plan retrieval remains healthy
with current-plan memory `019e8e8f-a725-7700-b8b6-55a13049d484` first, latest `handoff(get)`
returns `019e8e8f-fae1-77f0-8b77-68053c3173e7`, and the target is still active, project-scoped,
kind `handoff`, tagged `handoff`/`rolling`, and describes T106/T69/T70/T47-era gates. Focused
memory search still returns stale active rolling handoff noise including the target. Graph evidence
shows local supersession around the target, including `019e83a4-79fd-7f72-ab8b-4a953d3dd7b9`
superseding it, but the path from latest handoff to the target only traverses project scope and
does not prove a direct latest-to-target supersession chain. T193 therefore asks only for exact
future approval to archive this one target after fresh matching read-only preflight; it does not
archive memory, run `lint apply_safe`, change handoff semantics, ranking, `orient`, public
MCP/schema/storage/index/document-index behavior, M6/migration/quarantine, native Claude, Claude
Bridge, Claude hooks, process signals, harness installs/settings/hooks/adapters, deletion,
rollback, or user-owned files. T192 document indexing, T191/T187 lifecycle archives, T186
live-process cleanup, T172 effective-hook visibility, M6/migration, and broad Brain Harness
completion remain separate exact-gated work.

T194 matrix note:
`docs/BRAIN_HARNESS_T194_T193_DOC_INDEX_APPROVAL_PACKET_2026-06-03.md` records a
docs-only/default-deny exact-file document-index approval packet for the newest T193 lifecycle
packet and this implementation plan. Fresh lean `orient` trace
`019e8e98-107c-7830-b41e-d8e1586237ed` and direct current-plan search trace
`019e8e98-2ac4-7992-b3a2-a9b21c5e4a9d` recover active current-plan memory
`019e8e96-deb0-7bb2-918d-2f167c15430e` first, but exact document-layer probes for the T193 title,
filename stem, target ID, matrix note, and commit probe return older indexed lifecycle,
document-index, and implementation-plan artifacts rather than T193. T194 does not run
`docs(action="index")`, change document-index behavior, archive memory, run `lint apply_safe`,
signal PID `49349`, send native Claude input, launch Claude or Claude Bridge, run harness install,
mutate lifecycle or migration state, inspect M6/quarantine candidates, change ranking or `orient`,
change public MCP/schema/storage/index behavior, delete, roll back, reinstall binaries, or touch
user-owned files. Its proposed future approval allows indexing exactly
`docs/BRAIN_HARNESS_T193_STALE_T106_HANDOFF_LIFECYCLE_APPROVAL_PACKET_2026-06-03.md` and
`docs/MEMORY_OS_IMPLEMENTATION_PLAN.md` after fresh git/path/document-search/obligations evidence,
then read-only document-search validation and a docs-only result. T193/T191/T187 lifecycle
archives, T192 document indexing, T186 live-process cleanup, T172 effective-hook visibility,
M6/migration, and broad Brain Harness completion remain separate exact-gated work.

T195 matrix note:
`docs/BRAIN_HARNESS_T195_TELEMETRY_COMPLETION_GATE_AUDIT_2026-06-03.md` records telemetry feedback
and a docs-only completion-gate audit after T194. Four current traces were scored:
startup orient `019e8e9b-3074-7231-a3e3-8565b6005052`, current-plan search
`019e8e9b-539e-7f52-91b6-f981c6ec7b97`, completion-blocker search
`019e8e9b-546f-7872-9b89-0e4ed6b12552`, and design-philosophy search
`019e8e9b-553c-7871-b03b-cb473c201dff`. The current 50-trace project telemetry window now passes
the numerical confidence gate with `feedback_trace_count=39`, `feedback_coverage=78%`,
`memory_judgment_trace_coverage=82.22%`, six intents, `task_failure_count=0`, and
`bad_memory_used_count=0`; this removes the specific T189 telemetry blocker. The result is still
weak completion evidence because `external_session_trace_count=0`, `stale_memory_count=93`, direct
design-preference search still ranked stale handoffs above the reviewed preference in one trace,
and T194/T192 document indexing, T193/T191/T187 lifecycle archives, T186 native-Claude cleanup,
T172 effective-hook visibility, M6/migration, and broad Brain Harness completion remain separate
exact-gated work. T195 does not run document indexing, lifecycle archive, `lint apply_safe`, native
Claude or Claude Bridge actions, process signals, harness writes, ranking/`orient`/source/public
MCP/schema/storage/index/document-index behavior changes, deletion, rollback, old-binary reinstall,
or user-owned-file edits.

T196 matrix note:
`docs/BRAIN_HARNESS_T196_DESIGN_PREFERENCE_RETRIEVAL_RECHECK_2026-06-03.md` records a
docs-only/read-only recheck of the T195 design-preference retrieval caveat. The exact T195
`follow_user_preference` query still reproduces the issue: fresh trace
`019e8ea1-159d-7000-9671-ff10928f45fc` ranks reviewed preference
`019e6924-256b-7093-b1c5-286ec4d02461` third behind stale handoffs
`019e8475-3fa6-7080-9d80-ae81f24c9781` and
`019e838b-6b25-7011-8b4b-b4cc61dc450f`, matching the T195 trace
`019e8e9b-553c-7871-b03b-cb473c201dff`. Two close focused queries,
`019e8e9d-ea6d-74c0-bfa7-1b0c0a493247` and
`019e8ea0-533c-7882-8001-26317fef0e3f`, ranked the same reviewed preference first. T196 therefore
keeps design-preference retrieval partially validated and prompt-sensitive, and it rejects broad
ranking or `orient` churn from this evidence alone. Stale active handoff noise remains the safer
explanation and is already covered by exact-gated lifecycle packets. T196 does not run lifecycle
archive, `lint apply_safe`, document indexing, native Claude input or signals, Claude Bridge,
harness writes, M6/migration/quarantine actions, ranking/`orient`/source/public
MCP/schema/storage/index/document-index behavior changes, deletion, rollback, old-binary reinstall,
or user-owned-file edits. T194/T192 document indexing, T193/T191/T187 lifecycle archives, T186
native-Claude cleanup, T172 effective-hook visibility, M6/migration, external-session joinability,
and broad Brain Harness completion remain separate incomplete or exact-gated work.

T197 matrix note:
`docs/BRAIN_HARNESS_T197_T172_RECOVERY_PROCESS_GROUP_SIGINT_RESULT_2026-06-03.md` records the
user-approved T172 recovery-option cleanup after T179/T185/T190 left native Claude PID `49349`
live. Fresh preflight showed PID `49349` was still `/Users/yuval.meiri/.local/bin/claude` on
`ttys000`, with foreground process group `49349`, Claude Code `2.1.161`, monitored user/project
Claude hashes matching T172/T179/T185/T186/T190, harness status/doctor `ready=true` with known
warnings, obligations clean, and git clean except pre-existing untracked root `AGENTS.md`. Because
the original PTY session handle was unavailable after context compaction, Codex sent one
foreground-process-group interrupt, `kill -INT -49349`, as the Ctrl-C equivalent for that live PTY.
The process group exited within the wait window, so EOF was not sent. Postflight found no remaining
PID `49349`, no tracked git changes, no monitored Claude hash drift, no obligation changes, and
unchanged harness readiness. Memory OS `changes_since` from the pre-recovery cursor returned one
attributable SessionEnd lifecycle side effect: active handoff MemoryItem
`019e8ea5-663e-7152-b346-9c5ab7ddc93b`, written by native Claude Code and superseding
`019e8e9d-1e08-76d1-ab53-3c7f63ca0baa`. T197 resolves the live-process cleanup gate but does not
archive the new handoff, run `lint apply_safe`, close T172 effective-hook visibility, prove
prompt-bearing native Claude behavior or missing SessionEnd `write_policy` behavior, run
document indexing, lifecycle archive, M6/migration/quarantine actions, native Claude or Claude
Bridge probes, harness writes, ranking/`orient`/source/public MCP/schema/storage/index/
document-index behavior changes, deletion, rollback, old-binary reinstall, or user-owned-file
edits. T194/T192 document indexing, T193/T191/T187 lifecycle archives, T172 effective-hook
visibility, M6/migration, external-session joinability, telemetry coverage stability, and broad
Brain Harness completion remain separate incomplete or exact-gated work.

T198 matrix note:
`docs/BRAIN_HARNESS_T198_EXTERNAL_SESSION_JOINABILITY_RECHECK_2026-06-03.md` records a
docs-only/read-only external-session joinability audit after T197. Fresh runtime telemetry at
`2026-06-03T18:05:42Z` showed the current project 50-trace window has
`external_session_trace_count=0`, `distinct_external_session_count=0`,
`unspecified_external_session_trace_count=50`, `external_session_feedback_count=0`, and
`confidence_gate.passed=false` because feedback coverage dropped to 44% after new unscored traces.
Fresh `list_traces(project=engram, limit=15)` showed all 15 newest traces have
`external_session_id=null`. Source inspection found current core support intact:
`BrainHarnessTrace` and `AgentFeedback` store the optional label, feedback inherits the trace
label when omitted, reports aggregate trace/feedback external-session counts, `orient`, `search`,
`telemetry`, and `memory(changes_since)` pass caller-supplied labels through, and CLI
`orient`/`changes-since` still pass `None`. Validation passed with
`cargo test -p engram-tests --test telemetry_tests mcp_telemetry_tool_records_trace_feedback_and_stats -- --exact`
and the full `cargo test -p engram-tests --test telemetry_tests` target (`23 passed`). T198
therefore keeps external-session joinability incomplete but classifies the current zero-count gap
as caller/host adoption and host-session availability, not a core telemetry storage/report
implementation failure. It does not synthesize labels, change telemetry formulas, public MCP
parameters, `orient`, ranking, schema/storage/index/document-index behavior, lifecycle state,
harness files/settings/hooks/adapters, M6/migration/quarantine state, deletion, rollback, or
user-owned files. A real host-session contract or harness integration slice remains separately
approval-gated and should not be inferred from this audit.

T199 matrix note:
`docs/BRAIN_HARNESS_T199_EXTERNAL_SESSION_CALLER_AUDIT_2026-06-03.md` records a
docs-only/read-only caller-side audit after T198. Fresh source evidence shows the MCP tools already
expose and pass through `external_session_id` for `search`, `orient`, `telemetry`, and
`memory(changes_since)`, while the direct CLI `orient` and `memory changes-since` commands hard-code
`external_session_id: None`, and the stdio proxy forwards MCP transport session headers without
injecting Brain Harness telemetry labels. Fresh runtime telemetry at `2026-06-03T18:13:56Z` still
had `external_session_trace_count=0`, `unspecified_external_session_trace_count=50`,
`external_session_feedback_count=0`, and `confidence_gate.passed=false` because feedback coverage
fell to 40% after new unscored traces; the 20 newest project traces all had
`external_session_id=null`. AI Council and Claude Bridge agreed that read-only caller mapping is
useful, but any patch that makes live labels non-null is a host/caller or harness contract change
requiring exact approval. T199 does not change source code, add CLI flags/env defaults, mutate MCP
request/response shape, inject `mcp-session-id`, edit hooks/settings/adapters, run native Claude or
Claude Bridge actions beyond the read-only critique, run document indexing, archive memory, run
`lint apply_safe`, run M6/migration/quarantine actions, change ranking/`orient`,
schema/storage/index/document-index behavior, delete, roll back, reinstall binaries, or touch
user-owned files. External-session joinability and telemetry confidence remain incomplete; the next
implementation requires an exact-approved host/caller label contract or a separate exact-approved
M6/read-only scoping gate.

T200 matrix note:
`docs/BRAIN_HARNESS_T200_CLI_EXTERNAL_SESSION_LABEL_CONTRACT_2026-06-04.md` implements the first
narrow caller-label contract identified by T199. Direct CLI `engram orient` and
`engram memory changes-since` now accept `--external-session-id` and fall back to
`ENGRAM_EXTERNAL_SESSION_ID` when the flag is omitted. Empty or whitespace-only values normalize to
unset, and an explicit flag takes precedence over the environment. Focused CLI tests cover
flag/env precedence, whitespace normalization, and command parsing; existing telemetry tests
confirm supplied labels reach `orient` and `changes_since` traces. Validation passed with
`cargo test -p engram-cli external_session_id`,
`cargo test -p engram-tests --test telemetry_tests orient_with_intent_emits_trace_for_agent_feedback -- --exact`,
`cargo test -p engram-tests --test telemetry_tests changes_since_with_intent_emits_trace_for_agent_feedback -- --exact`,
`cargo check -p engram-cli`, and `cargo fmt --all --check`. T200 does not change MCP request or
response shape, synthesize host transcript IDs, inject MCP transport `mcp-session-id`, refresh the
installed runtime, change ranking/`orient`/schema/storage/index/document-index behavior, edit
hooks/settings/adapters/user-owned files, run lifecycle archive, M6/migration/quarantine actions,
native Claude, Claude Bridge write actions, deletion, rollback, or old-binary reinstall. Codex
Desktop live traces and ordinary MCP callers still need host/caller adoption of the existing label
field.

T201 matrix note:
`docs/BRAIN_HARNESS_T201_HANDOFF_SUPERSESSION_SEMANTICS_2026-06-04.md` implements a narrow
source-level prevention for future stale rolling handoff accumulation. `HandoffService::update`
still creates the new handoff with a `supersedes` edge to the previous matching handoff, but in
non-dry-run mode it now saves the new active handoff first and then marks that previous matching
handoff `superseded` with tool-call evidence. Dry-run remains zero-write and only returns the
planned supersedes edge. Focused validation passed with `cargo test -p engram-index handoff`,
`cargo test -p engram-tests --test harness_tests`, `cargo test -p engram-tests --test lint_tests`,
`cargo test -p engram-tests --test memory_tests test_mcp_orient_prepare_handoff_lean_surfaces_current_plan_and_gates -- --exact`,
`cargo fmt --all --check`, `cargo check -p engram-cli`, and `git diff --check`. T201 does not
archive existing stale active handoffs, run `lint apply_safe`, change handoff public request
shape, change search ranking/`orient`, schema/storage/index/document-index behavior, hooks,
settings, adapters, installed runtime, M6/migration/quarantine, native Claude, deletion, rollback,
or user-owned files. Pre-T201 stale handoffs remain visible until separately approved lifecycle
cleanup or future exact scoped work addresses them.

T202 matrix note:
`docs/BRAIN_HARNESS_T202_HANDOFF_SUPERSESSION_MCP_BOUNDARY_VALIDATION_2026-06-04.md` adds
test-only MCP boundary coverage for T201. The new integration test drives
`tools::handoff_new` with two non-dry-run `handoff(action="update")` requests in one project and
then verifies the public response plus stored previous handoff status: the second update names the
first handoff as `previous_id`, keeps the new handoff active, records the first handoff in
`supersedes`, and stores the first handoff as `superseded`. Validation passed with the focused MCP
test, full `cargo test -p engram-tests --test harness_tests`, `cargo fmt --all --check`,
`cargo check -p engram-cli`, and `git diff --check`. The initial focused test attempt failed only
because the test used `str::parse` instead of `Id::parse`; that test-code issue was corrected
before final validation. T202 does not change production behavior, installed runtime, lifecycle
state, ranking/`orient`, public MCP shape, schema/storage/index/document-index behavior, hooks,
settings, adapters, M6/migration/quarantine, native Claude, deletion, rollback, or user-owned
files.

T203 matrix note:
`docs/BRAIN_HARNESS_T203_HANDOFF_SUPERSESSION_CONVERGENCE_2026-06-04.md` tightens the T201 write
semantics so a future non-dry-run rolling handoff update converges all active same-scope handoff
predecessors, not only the newest previous item. The new handoff still saves first, preserves
`previous_id` as the newest predecessor for response compatibility, records every active matching
predecessor ID in `supersedes`, and then marks each previous matching handoff `superseded` with
tool-call evidence. Dry-run remains read-only and returns only the planned supersession links.
Validation passed with `cargo test -p engram-index handoff`,
`cargo test -p engram-tests --test harness_tests`, `cargo fmt --all --check`,
`cargo check -p engram-cli`, and `git diff --check`. T203 does not mutate existing live
MemoryItems, run `lint apply_safe`, refresh installed runtime, change search ranking/`orient`,
public MCP shape, schema/storage/index/document-index behavior, hooks, settings, adapters,
M6/migration/quarantine, native Claude, deletion, rollback, or user-owned files. Pre-T203 stale
live handoffs remain visible until a refreshed runtime performs a future handoff write or separate
lifecycle cleanup is explicitly run.

T204 matrix note:
`docs/BRAIN_HARNESS_T204_T203_RUNTIME_REFRESH_VALIDATION_2026-06-04.md` closes the immediate
runtime-drift caveat for T203. Codex installed the current `engram-cli` to
`/Users/yuval.meiri/.local/bin/engram`, producing binary hash
`39ee3b6491dca33267019376be07dd43a51b3772ffc24829cb3cf5f07385cd0c`, then cleanly stopped the old
daemon on port `8765`/PID `6516` and started the refreshed daemon on port `8765`/PID `91929`.
Before refresh, the live MCP `handoff(action="update", dry_run=true)` validation planned a
single predecessor in `item.supersedes`; after refresh, the same read-only dry-run returned the
same newest `previous_id` plus a long `supersedes` vector containing many active project-scoped
rolling handoff predecessors, and still reported `written=false`. This validates installed T203
planning behavior without mutating live handoff memory. Existing stale active handoffs remain
active until a future non-dry-run handoff update converges them or separate lifecycle cleanup is
explicitly run. T204 does not change search ranking/`orient`, public MCP shape,
schema/storage/index/document-index behavior, hooks, settings, adapters, M6/migration/quarantine,
native Claude, deletion, rollback, force-kill, or user-owned files.

T205 matrix note:
`docs/BRAIN_HARNESS_T205_HANDOFF_DOC_INDEX_RESULT_2026-06-04.md` records exact-file document
indexing for the T201, T202, T203, and T204 handoff reports. Indexing created 9, 1, 9, and 6
chunks respectively with no warnings. Targeted searches returned T203 first for its title, T204
first for its runtime-refresh query, T201 first for its title, and T202 first for the distinctive
`test_mcp_handoff_update_supersedes_previous_handoff` content query. T202 remains noisy for exact
title and filename-stem document search: `T202 Handoff Supersession MCP Boundary Validation` did
not return T202 in the top five, and the filename-stem query did not return it in the top ten.
This is an indexed-document visibility caveat, not a ranking or indexing-behavior change. T205
does not create active MemoryItems, mutate lifecycle state, run `lint apply_safe`, inspect or run
M6/migration/quarantine actions, change search ranking/`orient`, public MCP shape,
schema/storage/index/document-index behavior, hooks, settings, adapters, runtime configuration,
native Claude, deletion, rollback, force-kill, or user-owned files.

T206 matrix note:
`docs/BRAIN_HARNESS_T206_DOCUMENT_SOURCE_METADATA_SEARCH_2026-06-04.md` implements a narrow
document-search known-item repair for the T205 T202 visibility caveat. Direct document search and
unified-search document results now merge existing semantic chunk hits with metadata-only
`DocSource` title/path/basename/filename-stem matches. Exact normalized source metadata matches
score `1.0`; specific long substring matches score `0.84`; short generic terms such as
`Validation` do not trigger metadata promotion. Lexical and semantic hits for the same source are
deduplicated by promoting the existing semantic hit, and the original result limit is preserved.
Validation passed with focused source-metadata and merge tests, full document tests, full unified
search tests, `cargo fmt --all --check`, `cargo check -p engram-cli`, and `git diff --check`.
T206 does not change public MCP request/response shape, `orient`, memory ranking, memory
lifecycle, schema/storage definitions, document indexing/chunking/embedding behavior, M6/
migration/quarantine state, harness files/settings/hooks/adapters, runtime configuration,
deletion, rollback, native Claude, or user-owned files. The installed daemon still needs a
separate runtime refresh/live validation before T206 can be claimed live.

T207 matrix note:
`docs/BRAIN_HARNESS_T207_T206_RUNTIME_REFRESH_VALIDATION_2026-06-04.md` closes the immediate
installed-runtime gap for T206. Codex installed the current `engram-cli` to
`/Users/yuval.meiri/.local/bin/engram`, producing binary hash
`1475cd391ed1f2134eac59cc10226ffa6ad7c72c8049230dd19ec18a024e8058`, then cleanly stopped the old
daemon on port `8765`/PID `91929` and started the refreshed daemon on port `8765`/PID `21398`.
Before refresh, live `docs(search)` still missed T202 for exact title and filename-stem queries
while content search returned T202 first. After refresh, exact title and filename-stem document
queries both returned T202 first with score `1.0`; unified `search(layers=["document"])` for exact
title also returned T202 first with score `1.0`; generic `Validation` did not promote T202 into
the top five; and the distinctive content query still returned T202 first at semantic score
`0.6488516`. T207 does not change source code after T206, public MCP shape, `orient`, memory
ranking or lifecycle state, schema/storage definitions, document indexing/chunking/embedding
behavior, M6/migration/quarantine state, hooks, settings, adapters, deletion, rollback, native
Claude, or user-owned files.

T208 matrix note:
`docs/BRAIN_HARNESS_T208_T206_T207_DOC_INDEX_RESULT_2026-06-04.md` records exact-file document
indexing for the T206 source-change report and T207 runtime-refresh report. Indexing created 9
chunks for T206 and 1 chunk for T207 with no warnings. Document stats became `source_count=95`,
`chunk_count=4381`, `searchable_chunk_count=2369`, `orphan_chunk_count=2012`, and
`embedding_dimension=384`, so the two new sources did not increase orphan chunks. Targeted title
searches returned T206 first with score `1.0` and T207 first with score `1.0`; the raw binary-hash
query remained noisy and did not return T207 top five. This is document visibility maintenance
only. T208 does not create active MemoryItems, mutate lifecycle state, run `lint apply_safe`,
inspect or run M6/migration/quarantine actions, change ranking/`orient`, public MCP shape,
schema/storage/index/document-index behavior, hooks, settings, adapters, runtime configuration,
native Claude, deletion, rollback, force-kill, or user-owned files.

T209 matrix note:
`docs/BRAIN_HARNESS_T209_M6_READ_ONLY_SCOPING_STATUS_2026-06-04.md` records a read-only M6
scoping/status validation against the existing T68 review-export snapshot. Codex re-read the
committed T58/T68/T123/T124/T169/T173 reports, validated the snapshot index and files as the
expected generated `index.md` plus 12 regular candidate files with no symlinks, inspected source to
confirm `memory(action="migration_review_status")` delegates to dry-run apply with
`create_commit=false`, and ran exactly one status check. Status scanned 12 files, reported all 12
as `files_with_no_decision`, no skipped/conflict/not-in-index/missing files, accepted/planned/
written counts of 0, `ready_to_apply=false`, and no warnings. Candidate 0012 is accounted for as
count-drift provenance from T68, not decided or admitted to migration apply. The recommended next
gate is a docs-only candidate-disposition approval packet for 0001-0011 plus explicit separate
handling of 0012, or an exact all-snapshot 0001-0012 gate if the user intentionally expands scope.
T209 does not make candidate decisions, edit review files, run apply/prioritize/export/rerun,
mutate lifecycle state, archive memory, delete data, change ranking/`orient`, public MCP,
schema/storage/index/document-index behavior, harness files, native Claude state, runtime
configuration, or user-owned files.

T210 matrix note:
`docs/BRAIN_HARNESS_T210_M6_CANDIDATE_DISPOSITION_AUTHORIZATION_PACKET_2026-06-04.md` defines the
next M6 gate without executing it. The packet recommends a conservative future T210A gate that
records only explicit human-provided dispositions for candidates 0001-0011 and requires a separate
explicit 0012 count-drift/provenance instruction, plus an alternate T210B all-snapshot gate that
must explicitly name 0001-0012 as an intentional scope expansion beyond T58. Future execution may
edit only generated review pages with human-provided choices/notes, then run exactly one read-only
`memory(action="migration_review_status")` check and write a result report. T210 does not infer
candidate decisions, edit the review workspace, run status/apply/prioritize/export/rerun, write
active MemoryItems, mutate lifecycle state, archive memory, delete data, change ranking/`orient`,
public MCP/schema/storage/index/document-index behavior, harness files, native Claude state,
runtime configuration, or user-owned files.

T211 matrix note:
`docs/BRAIN_HARNESS_T211_T209_T210_DOC_INDEX_RESULT_2026-06-04.md` records exact-file document
indexing for the T209 M6 read-only scoping/status report and the T210 candidate-disposition
authorization packet. Pre-index document searches missed both new docs in the top five. Indexing
created 14 chunks for T209 and 13 chunks for T210 with no warnings. Post-index exact title searches
returned T209 and T210 first with score `1.0`; a content query for the T210A human-disposition gate
also returned T210 first. Document stats became `source_count=97`, `chunk_count=4408`,
`searchable_chunk_count=2396`, `orphan_chunk_count=2012`, and `embedding_dimension=384`, so orphan
count did not increase. T211 does not make candidate decisions, run M6 status/apply/prioritize/
export/rerun, mutate lifecycle state, archive memory, delete data, change ranking/`orient`, public
MCP/schema/storage/index behavior, document-index behavior, harness files, native Claude state,
runtime configuration, or user-owned files.

T212 matrix note:
`docs/BRAIN_HARNESS_T212_T211_DOC_INDEX_RESULT_2026-06-04.md` records exact-file document indexing
for the T211 T209/T210 document-index result. Pre-index exact title search missed T211 in the top
five. Indexing created 1 chunk with no warnings. Post-index exact title search returned T211 first
with score `1.0`; a stats/content query returned T211 in the top five behind older similar
index-result reports. Document stats became `source_count=98`, `chunk_count=4409`,
`searchable_chunk_count=2397`, `orphan_chunk_count=2012`, and `embedding_dimension=384`, so orphan
count did not increase. T212 does not make candidate decisions, run M6 status/apply/prioritize/
export/rerun, mutate lifecycle state, archive memory, delete data, change ranking/`orient`, public
MCP/schema/storage/index behavior, document-index behavior, harness files, native Claude state,
runtime configuration, or user-owned files.

T213 matrix note:
`docs/BRAIN_HARNESS_T213_COMPLETION_MATRIX_RECONCILIATION_2026-06-04.md` reconciles stale
completion-matrix wording after T169/T209/T210/T211/T212. The living checklist now marks T125
quarantine inspection complete from T169 evidence, and the migration matrix no longer says the two
quarantine candidates are unread. The current M6 state is: candidate inspection is complete for
0001-0011, 0012 is count-drift provenance with explicit-scope handling required, all 12 generated
snapshot files remain undecided by the read-only status check, and the next M6 progress requires
human-provided dispositions or explicit deferral. T213 does not edit the review workspace, run
M6 status/apply/prioritize/export/rerun, infer candidate decisions, mutate lifecycle state, archive
memory, delete data, change ranking/`orient`, public MCP/schema/storage/index behavior,
document-index behavior, harness files, native Claude state, runtime configuration, or user-owned
files.

T214 matrix note:
`docs/BRAIN_HARNESS_T214_HARNESS_MATRIX_RECONCILIATION_2026-06-04.md` reconciles stale
cross-harness matrix wording after T152/T170/T179/T198/T200/T204 and fresh read-only
`harness(action="doctor")` checks. The current local generated-adapter readiness state is
`ready=true` for generic, Claude Code, Codex, Gemini CLI, and Cursor. This supersedes older
completion-matrix wording that said all supported harnesses were still `ready=false` or had
required generated-adapter drift. The remaining cross-harness risks are narrower: lifecycle
compliance is still soft, Claude Code settings remain split with a user-owned snippet and extra
legacy permissions, native Claude startup guidance was observed but `/hooks` effective-hook
visibility remains unresolved, prompt-bearing native Claude behavior is not proved, external-session
joinability still depends on real caller/host labels, and stale active handoffs remain until a
future non-dry-run handoff update or explicit lifecycle cleanup. T214 does not edit hooks, settings,
adapters, runtime configuration, user-owned files, ranking/`orient`, public MCP, schema/storage/
index/document-index behavior, lifecycle state, native Claude state, M6/migration/quarantine state,
or review workspace files.
