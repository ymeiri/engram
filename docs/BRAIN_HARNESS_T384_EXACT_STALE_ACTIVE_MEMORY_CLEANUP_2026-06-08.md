# Brain Harness T384 Exact Stale Active Memory Cleanup

Date: 2026-06-08
Status: Completed
Scope: Exact Memory OS lifecycle hygiene only

## Question

Can the current project-scoped stale-active-memory lint sample be reduced without broad cleanup,
source changes, release-state changes, or native-Claude execution?

## Boundary

T384 archived exactly these active MemoryItems:

- `019dd080-612a-7540-a028-42991c20ef1b`
  (`Installed Engram MCP binary refreshed after quarantine prioritization change`)
- `019dd083-e014-74f1-95e5-b1eef478e894`
  (`Memory OS implementation plan is not fully complete`)
- `019dd3a8-138d-7453-9991-d724f96a128f`
  (`Memory OS is ready for daily Codex use with Claude hooks caveat`)
- `019dd3e4-9143-7721-9bff-b3fb505c8859`
  (`Memory OS full-plan remaining gaps on 2026-04-28`)
- `019e68a7-3375-7943-8ef0-dc0dde64c8bd`
  (`Direct search current-plan live smoke passed after install`)

The records are preserved as archived history. T384 did not delete data, run broad
`lint apply_safe`, end sessions, mutate M6, change ranking or `orient`, accept hosted-CI fallback,
mark PR #3 ready, merge, tag, publish, launch native Claude, prove hooks, or prove host labels.

## Evidence

Read-only preflight:

- `git fetch origin --prune` left branch `yuval.meiri/memory-os-phase1` synchronized with origin.
- `gh pr view 3` reported PR #3 head
  `8368b689d1f741d7ad18d918bfd29193d0b4f8e2`, draft, with hosted CI still failing before
  workflow steps.
- `./target/debug/engram lint run --scope-project engram --limit 30 --json` reported the five
  exact `feedback_stale_active_memory` findings above, each with `safe_action="none"`.
- Direct CLI `memory get` hit the expected RocksDB `LOCK` while the daemon owned the store, so the
  exact item reads and archives were run through the daemon HTTP MCP path.
- `memory(action="get")` showed the five targets were active project-scoped records with dated
  runtime, implementation-plan, readiness, gap, or direct-search live-smoke content from
  2026-04-27 through 2026-05-27.
- `graph(action="around", depth=1)` showed each target linked only to its historical evidence,
  project scope, and original commit/session context; no active successor edge made a blind
  supersession claim.
- Current search and file evidence surfaced newer active replacements, including T337/T355/T378
  runtime/operator evidence and the T383 current matrix and handoff.
- `./target/debug/engram daemon status` reported daemon PID `39185`, spawned by
  `/Users/yuval.meiri/.local/bin/engram`, spawn version `0.2.0-beta.1`.
- `./target/debug/engram vault status /Users/yuval.meiri/.engram/vault --json` reported aligned
  generated vault counts: `generated_file_count=2749`,
  `expected_generated_file_count=2749`, `user_file_count=0`.
- `./target/debug/engram obligations doctor --scope-project engram --cwd
  /Users/yuval.meiri/projects/engram --limit 20 --json` returned no open obligations or warnings.

Archive execution:

- All five `memory(action="archive")` calls returned `status="archived"` with T384 archive reasons.

Postflight:

- `./target/debug/engram lint run --scope-project engram --limit 30 --json` no longer reports any
  of the five archived `feedback_stale_active_memory` findings.
- The remaining sampled findings are missing-evidence and stale-session debt with
  `safe_action="none"`.
- Vault status remained aligned at `2749/2749`.
- Scoped obligations doctor remained clean.

## Result

T384 reduces stale active-memory pressure in current project-scoped retrieval by removing five
dated current-state records from active guidance. The remaining production/GA gates are unchanged:
hosted-CI or release-owner fallback acceptance, native Claude prompt-bearing proof, effective-hook
proof, live host-label proof, direct legacy cleanup, and further exact lifecycle hygiene.
