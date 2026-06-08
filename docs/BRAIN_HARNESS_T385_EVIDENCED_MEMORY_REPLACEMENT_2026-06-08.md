# Brain Harness T385 Evidenced Memory Replacement

Date: 2026-06-08
Branch: `yuval.meiri/memory-os-phase1`
Head before slice: `fe559f7b15e3ddc78165c32eb8ee528965bfe68b`

## Question

Can the next project-scoped Memory OS lint findings be resolved without broad cleanup or
destructive session mutation, while preserving useful implementation facts as evidenced active
memory?

## Scope

This slice addresses exactly the first five project-scoped lint findings from:

```bash
./target/debug/engram lint run --scope-project engram --limit 20 --json
```

The findings were:

- `missing-evidence:019dcaa6-0223-73a2-9fe4-76b61ff14faa`
- `missing-evidence:019dddbe-d369-7523-ac91-9bfeb016463b`
- `missing-evidence:019dfecb-16ca-71c2-b391-e3c216601590`
- `missing-evidence:019dfee0-90f8-7a61-a548-c2bddefcf897`
- `missing-evidence:019dfee6-c83a-7c02-a3ed-78fc9e80329b`

No broad `lint apply_safe`, session ending, direct legacy deletion, M6 mutation, ranking change,
native-Claude launch, hook execution, PR ready/merge/tag/publish action, or hosted-CI fallback
acceptance was performed.

## Evidence Review

The useful facts had durable source evidence:

- `Digest extraction apply implemented` is backed by commit `f200c82`, `engram-mcp/src/tools.rs`,
  `engram-index/src/digest.rs`, and
  `engram-tests/tests/memory_tests.rs:test_mcp_memory_digest_extraction_apply_empty_batch_dry_run`.
- `Orient contract checkpoint committed` is backed by commit `c535ed5`,
  `docs/ORIENT_CONTRACT.md`, `engram-tests/tests/memory_tests.rs`, and
  `engram-tests/tests/obligation_tests.rs`.
- `M3 real-session telemetry eval report added` is backed by commit `5d6c4c4`,
  `engram-core/src/telemetry.rs`, `engram-index/src/telemetry.rs`,
  `engram-mcp/src/tools.rs`, and `engram-tests/tests/telemetry_tests.rs`.

The two stale operational records had no useful active guidance:

- `019dddbe-d369-7523-ac91-9bfeb016463b` was a global 2026-04-30 Claude Code handoff with
  unknown CWD and no evidence.
- `019dfee6-c83a-7c02-a3ed-78fc9e80329b` was a May 6 installed-runtime snapshot with old
  daemon PID `38875` and obsolete 16 percent telemetry coverage.

`graph(action="around", depth=1)` for all five targets showed only scope edges, so there were no
dependent memory graph edges to preserve beyond scope.

## Changes

Created evidenced active replacements:

- `019ea8e5-d8ba-7623-abb0-b4151504ad14`:
  `Digest extraction apply support is implemented with evidence`
- `019ea8e6-19b0-7353-bc93-7124bfea5b61`:
  `Orient contract checkpoint is implemented with evidence`
- `019ea8e6-59e6-7980-9a97-e08ab77073be`:
  `Real-session telemetry eval report is implemented with evidence`

Superseded evidence-less active facts:

- `019dcaa6-0223-73a2-9fe4-76b61ff14faa`
- `019dfecb-16ca-71c2-b391-e3c216601590`
- `019dfee0-90f8-7a61-a548-c2bddefcf897`

Archived stale operational records:

- `019dddbe-d369-7523-ac91-9bfeb016463b`
- `019dfee6-c83a-7c02-a3ed-78fc9e80329b`

## Validation

Post-change project-scoped lint no longer reports the five missing-evidence targets. The first
returned findings are now stale active-session warnings with `safe_action="none"`.

```bash
./target/debug/engram lint run --scope-project engram --limit 10 --json
```

Canonical vault alignment was restored after compiling the generated vault:

```bash
./target/debug/engram vault compile /Users/yuval.meiri/.engram/vault --json
./target/debug/engram vault status /Users/yuval.meiri/.engram/vault --json
```

Final status:

```json
{
  "total_file_count": 2755,
  "generated_file_count": 2755,
  "user_file_count": 0,
  "memory_item_count": 1916,
  "knowledge_commit_count": 714,
  "expected_generated_file_count": 2755
}
```

After recording KnowledgeCommit `019ea8e8-f0ee-7f91-b78b-5b96060b9f0a` for the T385 memory
changes and recompiling again, final vault status is:

```json
{
  "total_file_count": 2756,
  "generated_file_count": 2756,
  "user_file_count": 0,
  "memory_item_count": 1916,
  "knowledge_commit_count": 715,
  "expected_generated_file_count": 2756
}
```

Scoped obligations remain clean:

```bash
./target/debug/engram obligations doctor --scope-project engram --cwd /Users/yuval.meiri/projects/engram --json
```

Output:

```json
{
  "open": [],
  "warnings": []
}
```

## Remaining Gates

This slice improves evidence quality and active-memory trustworthiness. It does not change the
remaining beta or production gates:

- PR #3 still needs release-owner local-validation fallback acceptance or restored hosted CI, then
  ready/merge/tag/publish mechanics.
- Prompt-bearing native Claude, effective-hook visibility, and live host-label proof remain
  separate production gates.
- Stale active-session lint findings remain visible and intentionally expose no automatic safe
  action.
- Direct legacy deprecation/deletion and broad lifecycle cleanup remain separate exact-scope work.
