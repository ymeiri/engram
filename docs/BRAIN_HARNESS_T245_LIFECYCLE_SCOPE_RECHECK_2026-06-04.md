# Brain Harness T245 Lifecycle Scope Recheck

Date: 2026-06-04
Status: completed docs-only read-only lifecycle scoping update. No lifecycle archive,
`lint apply_safe`, migration/quarantine action, harness write, native Claude action, ranking,
`orient`, public MCP, schema/storage/index, document-index behavior, deletion, rollback,
force-kill, legacy simplification, or user-owned-file change was executed.

## Scope

T245 records a narrow lifecycle completion-matrix correction after T166, T167, and T168.
Those result reports show that the exact T157, T159, and T160 Engram lifecycle archive targets
were executed and validated. Fresh lint still reports lifecycle pressure, but the leading sampled
findings are mixed-scope/global, not those exact Engram targets.

This slice updates only:

- `docs/BRAIN_HARNESS_ARCHITECTURE.md`
- `docs/MEMORY_OS_IMPLEMENTATION_PLAN.md`
- this report

## Research Question

How should the completion matrix describe lifecycle cleanup now that T157/T159/T160 exact targets
are archived, while fresh lint still reports active wrong-scope and superseded-active pressure?

## Hypotheses

| Type | Hypothesis | Result |
| --- | --- | --- |
| Preferred | Mark T157/T159/T160 exact targets closed, but keep lifecycle cleanup incomplete and exact-target-review-gated. | Supported. |
| Null | Keep lifecycle wording unchanged because lint still reports pressure. | Partially rejected: broad lifecycle pressure remains, but the known exact targets are no longer open. |
| Simpler alternative | Rely on T166/T167/T168 reports only and avoid another matrix update. | Rejected because T243/T244 current-plan wording still risks making future agents re-open closed exact targets. |
| Failure | Treat a sampled lint output as proof that no Engram-scoped lifecycle debt remains, or use T245 as approval for archive/apply actions. | Avoided by bounding the claim to exact targets and preserving the broader gate. |

## Evidence

Fresh startup orientation trace `019e925b-fcd0-7c30-8753-662b392bfdd5` returned
current-plan MemoryItem `019e9259-6147-7431-80d6-158ee4e5ff28` first and kept obligations clean.

Fresh read-only `lint(action="run", write=false, limit=20)` at
`2026-06-04T11:19:29Z` returned:

- `feedback_wrong_scope_active_memory` for `019e8d2e-df0e-77b1-8ab8-f425346224d4`
  (`Claude Code tool failure: Read`), with `safe_action=none`.
- `feedback_wrong_scope_active_memory` for `019e8d39-50ca-7961-a424-ea5a638beb3d`
  (`Claude Code tool failure: mcp__playwright__browser_wait_for`), with `safe_action=none`.
- `feedback_wrong_scope_active_memory` for `019e8d3a-d918-7843-a288-4da7db1c1176`
  (`Claude Code tool failure: mcp__playwright__browser_wait_for`), with `safe_action=none`.
- `superseded_item_still_active` for `019dd5cd-a403-7b53-9010-47bd94bba51a`,
  with `safe_action=archive_memory_item`.

Representative read-only `memory(get)` checks showed:

- `019e8d2e-df0e-77b1-8ab8-f425346224d4` is an active `dd-source`
  `session_insight`, not Engram-scoped.
- `019e8d39-50ca-7961-a424-ea5a638beb3d` is an active `dd-source`
  `session_insight`, not Engram-scoped.
- `019dd5cd-a403-7b53-9010-47bd94bba51a` is an active
  `ide-mcp-eval-replay-stringification-verification` handoff.
- `graph(action="around", node="019dd5cd-a403-7b53-9010-47bd94bba51a", depth=1)`
  showed it is superseded by `019dd7ff-0041-7e33-b825-cb65d299bfa9` in the same
  non-Engram project scope.

Exact target checks showed:

- T157 target `019e5e0a-86b4-73e3-aa9b-ca350e83e915` is `status=archived`.
- T159 target `019e89f4-7dba-7ae1-a559-85d924af31a3` is `status=archived`.
- T160 target `019e7f52-4fc2-7f61-93b4-9a741aba966e` is `status=archived`.

This evidence is not an exhaustive lifecycle inventory. The lint run was capped at 20 findings,
and `memory(action="list", project_name="engram", scope_type="project", status_filter="active",
limit=30)` still showed active Engram items that may need later exact review, including older
telemetry guidance such as `019e8291-40aa-71a0-b16b-9ba7b6446cc6`. T245 therefore does not claim
that all Engram-scoped lifecycle debt is gone.

## AI Consultation

AI Council recall found prior lifecycle guidance for T48/T108/T139: lifecycle packets should be
default-deny, exact-target only, and must not bundle broader cleanup, `lint apply_safe`, ranking,
`orient`, M6, harness, schema/storage/index, or deletion work.

A bounded AI Council broadcast and isolated read-only Claude Bridge critique agreed on the main
blind spots:

- `limit=20` lint output is a sample, not a full scope inventory.
- `safe_action=none` means no automated safe action, not non-actionability.
- Leading non-Engram lint findings do not prove no Engram-scoped items exist deeper in the queue.
- T166/T167/T168 close their exact targets, not lifecycle cleanup as a whole.

This consultation is blind-spot evidence only. The decision is based on the repo reports and fresh
read-only Engram checks.

## Completion-Matrix Effect

| Area | Current State | Evidence | Remaining Gate |
| --- | --- | --- | --- |
| T157/T159/T160 exact lifecycle targets | Closed | `memory(get)` shows all three target IDs archived; result reports T166/T167/T168 recorded validation | None for those exact IDs |
| Broad lifecycle cleanup | Incomplete | Fresh sampled lint still reports wrong-scope and superseded-active pressure; no safe actions applied | Broader exact-target review before any archive or `lint apply_safe` |
| Engram-scoped lifecycle debt | Not exhaustively inventoried by T245 | `limit=20` lint sample plus active Engram memory list are insufficient for a global clean claim | Future scoped inventory or exact target packet |
| Hot path / runtime / harness / M6 | Unchanged | No source, runtime, harness, ranking, `orient`, migration, or schema/index action ran | Existing gates remain |

## Decision

Update the architecture and implementation-plan matrix to prevent future agents from re-opening
the already archived T157/T159/T160 targets. Keep lifecycle cleanup listed as incomplete because
fresh lint still reports pressure and T245 is not a full inventory.

The next safe lifecycle work is not `lint apply_safe`. It is a scoped, exact-target review that
names the candidate IDs, confirms their current state with fresh read-only evidence, and either
prepares an approval packet or explicitly defers them.

## Validation

Validation for this docs-only slice:

- `git status --short`
- `git log --oneline -8`
- `lint(action="run", write=false, limit=20)`
- `memory(action="get")` for T157/T159/T160 and representative lint items
- `graph(action="around", node="019dd5cd-a403-7b53-9010-47bd94bba51a", depth=1)`
- AI Council recall and bounded broadcast
- isolated read-only Claude Bridge critique
- `git diff --check`
- exact document indexing for this report, `docs/BRAIN_HARNESS_ARCHITECTURE.md`, and
  `docs/MEMORY_OS_IMPLEMENTATION_PLAN.md`
- document-search visibility for T245
- `obligations(action="doctor", project="engram")`
- focused commit with only intended repo docs
