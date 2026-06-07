# Brain Harness T112 Recommendation Surface Audit

Status: Completed docs-only recommendation-surface audit; no telemetry behavior change
Date: 2026-06-01
Scope: Audit whether the existing `RealSessionEvalReport.recommendations` surface is advisory-only
inside this repository before choosing any T111 behavior change.

This slice did not add a recommendation string, change `DEFAULT_REAL_SESSION_EVAL_LIMIT`, change
confidence formulas, change public MCP request parameters or response fields, generate calibration
traces, run M6 inspection/export/apply/deletion, mutate lifecycle state, run
`lint(action="apply_safe")`, index documents, change ranking, expand `orient`, change
schema/storage/index behavior, change document-index behavior, or write harness adapters/hooks.

## Research Question

Does the existing repo treat `RealSessionEvalReport.recommendations` as operator-facing advisory
text, or are there automated consumers that make adding a T111 default-window advisory riskier than
the prior source read suggested?

## Hypotheses

- Preferred: the field is serialized as advisory report text, with tests checking only targeted
  substrings and no repo-local control-flow consumer. This lowers, but does not eliminate, the risk
  of a future narrow advisory string.
- Null: the prior T111 handoff has enough evidence; no surface audit is needed before asking the
  user to choose a direction.
- Simpler alternative: pause again and ask for Option A or Option B without adding any repo
  artifact.
- Failure: a repo-local caller parses, counts, orders, or treats recommendations as machine-stable
  semantics, so changing the list is a stronger public behavior change than expected.

## Measurement

Source and repo search before any edit:

- `rg -n "recommendations"` across the repo.
- A targeted `rg` search for `RealSessionEvalReport`, `real_session_eval_report`,
  `DEFAULT_REAL_SESSION_EVAL_LIMIT`, `fn recommendations`, and `recommendations(` across
  `engram-core`, `engram-index`, `engram-mcp`, and `engram-tests`.
- Source reads in `engram-core/src/telemetry.rs`, `engram-index/src/telemetry.rs`,
  `engram-mcp/src/tools.rs`, and `engram-tests/tests/telemetry_tests.rs`.
- Prior T111 state from Engram current-plan memory `019e8475-b81f-71f3-82bb-ef842b5e49e0` and
  handoff `019e8475-3fa6-7080-9d80-ae81f24c9781`.

## Findings

The field is an existing public report field:

- `engram-core/src/telemetry.rs` documents `RealSessionEvalReport.recommendations` as
  "Operator-facing follow-up recommendations."
- `engram-index/src/telemetry.rs` builds the report, computes the confidence gate, then assigns
  `report.recommendations = recommendations(&report)`.
- The existing `recommendations` function emits advisory strings for confidence-gate failure,
  empty trace samples, low feedback coverage, multiple feedback records per trace, unspecified
  intent, missing external session labels, missing scenario/arm labels, missing scores, missing
  outcome feedback, missing memory attribution, trace warnings, or a default "use this evidence for
  ranking calibration" message when no other recommendation applies.
- `engram-mcp/src/tools.rs` serializes the whole report as JSON for
  `telemetry(action="real_session_eval")`.

No repo-local automated control-flow consumer was found:

- The only `RealSessionEvalReport.recommendations` usages found in telemetry tests assert that
  some recommendation contains specific substrings: `Keep M6 write-apply blocked`,
  `feedback_records_per_trace`, and `memory attribution fields`.
- No source path found by the search parses the recommendation list to decide migration, lifecycle,
  ranking, `orient`, indexing, harness, or schema behavior.
- Other `recommendations` hits belong to the separate Tool Intelligence layer or documentation.

That does not make a T111 behavior change free:

- The field is serialized through MCP, so adding a string still changes observable report content.
- Agents or external callers outside this repository may read recommendations as actionable
  guidance.
- The prior AI Council disagreement remains material: Claude Sonnet favored docs-only, while
  GPT-5.4 and Gemini favored a contextual advisory if tested.

## Interpretation

T112 narrows the public-behavior risk from "unknown repo consumer" to "observable advisory content
change." It does not resolve the T111 design choice. The safest current state remains no telemetry
behavior change until the user explicitly chooses between:

- implementing a contextual default-window advisory in the existing recommendation channel with
  focused tests and docs; or
- keeping the T111 result docs-only and relying on T110's executable regression plus explicit
  recent-window calls for confidence checks.

## Completion Matrix Delta

- Evidence and feedback loop: slightly clearer, still partial. The recommendation surface is now
  audited as advisory in repo-local code.
- Telemetry behavior: unchanged. No Rust code or report-generation behavior changed.
- Public MCP behavior: unchanged. No fields, parameters, defaults, or response contents changed.
- T111 eval design: still unresolved. Prior model disagreement remains; this audit only reduces one
  uncertainty.
- M6 migration: still gated. T69 exact inspection approval remains required.
- Lifecycle: still gated. `safe_action=none` and exact-ID lifecycle gates remain unchanged.
- Harness readiness: still gated/risky. T47 exact harness-write gate remains pending.

## Next Gate

If the project owner wants the T111 behavior change, require an exact approval such as:

`Approve T111 Option A: add the contextual default-window recommendation string.`

If the project owner wants to keep the report unchanged, use:

`Approve T111 Option B: keep T111 docs-only and do not change real_session_eval recommendations.`

Neither phrase authorizes migration apply/deletion/simplification, lifecycle writes, document
indexing, harness writes, broad ranking changes, `orient` expansion, public MCP request-parameter
changes, schema/storage/index behavior changes, or document-index behavior changes.
