# Brain Harness T339 Canonical Vault Resync

Date: 2026-06-07
Status: completed and validated

## Scope

T339 refreshes the durable generated Memory OS vault after the latest T338/T339 memory writes. A
fresh preflight `vault(action=status, vault_path="/Users/yuval.meiri/.engram/vault")` showed the
vault was initialized and generated-only, but stale by two generated files:

```text
total_file_count=2566
generated_file_count=2566
user_file_count=0
expected_generated_file_count=2568
memory_item_count=1790
knowledge_commit_count=654
```

This slice runs only the generated-vault compile path and validates the postflight count alignment.
It does not mutate repository source, run `lint apply_safe`, archive memory, run native Claude,
change harness adapters, close hosted CI, or mark the beta release ready.

## Research Question

Can the canonical generated vault be safely resynchronized after the latest Memory OS writes without
touching user-owned files or changing product behavior?

## Hypotheses

| Type | Hypothesis | Evidence |
| --- | --- | --- |
| Preferred | `vault(action=compile)` will write only Engram-generated vault files and restore count alignment. | The generated marker policy and prior T277/T278 vault execution evidence cover this path. |
| Null | The count drift is harmless and should be left until release tagging. | Rejected because the goal definition treats canonical vault validation as a completion gate. |
| Broader alternative | Add a new release-readiness command that checks every beta gate. | Deferred because this slice only needs to refresh an already-supported vault surface. |
| Failure | Compile skips files, touches user files, or leaves counts mismatched. | Guarded by postflight status and zero skipped files in the compile result. |

## Validation

`vault(action=compile, vault_path="/Users/yuval.meiri/.engram/vault")` completed with:

- `files_skipped=[]`;
- `memory_item_count=1790`;
- `knowledge_commit_count=654`;
- `repository_count=9`;
- `entity_count=32`;
- `project_count=79`.

Postflight `vault(action=status, vault_path="/Users/yuval.meiri/.engram/vault")` returned:

```text
exists=true
initialized=true
missing_directories=[]
total_file_count=2568
generated_file_count=2568
user_file_count=0
memory_item_count=1790
knowledge_commit_count=654
repository_count=9
entity_count=32
project_count=79
expected_generated_file_count=2568
```

`obligations(action=doctor, project=engram)` returned no open obligations and no warnings.

`lint(action=run, vault_path="/Users/yuval.meiri/.engram/vault", limit=30)` did not report a vault
marker/frontmatter/count finding in the bounded result; the returned findings were historical
stale-feedback and unrelated global open-obligation signals with `safe_action=none`.

## Non-Claims

T339 proves the durable generated vault is count-aligned after the latest Memory OS writes. It does
not prove hosted CI, native Claude prompt-bearing behavior, effective-hook visibility, live host
labels, production parity, direct legacy deprecation, or broad lifecycle cleanup.
