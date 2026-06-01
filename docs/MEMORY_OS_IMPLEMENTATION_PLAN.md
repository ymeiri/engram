# Engram Memory OS: Design and Implementation Plan

Status: Implementation in progress
Date: 2026-04-26
Last Updated: 2026-06-01
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
- [ ] Migration completion run: explicit read-only inventory scope approval, inventory,
      review export, prioritize/dedupe, human review, dry-run apply, explicit write-apply approval,
      knowledge commit, vault compile, lint run.

## Current Completion Matrix

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
`candidates/0012-skip-plan.md`. Until that exact approval arrives, M6 remains paused.
T70 records a follow-up document visibility gap: T68 and T69 are not yet visible through top
document-search results, and T59 still has stale pre-export chunks indexed. Source inspection shows
exact-file `docs(action="index")` reuses the existing source identity and replaces chunks for that
source. T70 asks for explicit approval to index exactly T59, T68, and T69; it does not grant T69
inspection approval or any M6 migration action.
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
| Migration from legacy layers | Review export produced a stopped review workspace; apply still gated | Review-gated migration/digest flows exist; one accidental broad read-only inventory returned `3934` sources and `3641` candidates with no writes; the live feedback batch found no current M6 authorization and rejected older migration/export approvals as stale for this gate; T14 explicit migration-apply prompts now retrieve the paused migration review gate first; T58 completed an explicitly approved inventory-only scope with 115 sources scanned, 11 candidates returned, no truncation, and no writes; T59 prepared the exact review-export approval packet later executed as T68; T64 explicit review-export probes preserved default-deny gate context but did not surface the T59 packet itself in top memory results; T65 prepared a pending bounded document-index visibility approval packet for T58/T59/T64 docs; T66 source-only preflight confirmed exact-file MCP indexing is executable if approved; T67 executed the approved exact-file indexing and improved T59 document-search visibility for title, filename-stem, and explicit review-export probes; T68 executed the approved exact review-export call but stopped because the export returned 12 candidates, including one `skip`, instead of the expected 11; T69 prepares the next exact read-only inspection approval packet; T70 prepares a separate exact-file indexing approval packet because T68/T69 are absent from top document results and stale T59 chunks remain visible | Do not run review apply, prioritize, candidate decisions, deletion, lifecycle mutation, schema/storage/index behavior change, ranking, public MCP changes, `orient` expansion, or harness writes without separate explicit approval. T68's count drift requires a user decision before further M6 progress; T69 asks for explicit approval to inspect only `index.md` and `candidates/0012-skip-plan.md` from the written export snapshot. T70 asks for exact-file indexing approval only and does not replace the T69 inspection gate. Absolute-path semantic document search remains weak, and T59 was edited after T67 indexing, so read repo docs before M6 decisions. Do not write-apply/delete/simplify without reviewed candidates, dry-run report, rollback plan, and explicit approval. |

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
