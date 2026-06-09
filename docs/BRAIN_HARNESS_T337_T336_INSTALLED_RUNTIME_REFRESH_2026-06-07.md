# Brain Harness T337 T336 Installed Runtime Refresh

Date: 2026-06-07
Status: executed and locally validated

## Scope

T337 refreshes the installed local Engram runtime so the T336 project-scoped lint API is available
through the installed CLI, the restarted daemon, and a fresh MCP `tools/list` / `tools/call` path.

This is a runtime-adoption slice for the already-committed T336 code. It does not change source
behavior, add new lint rules, run `lint apply_safe`, archive memory, mutate obligations, run native
Claude, change harness adapters, or touch user-owned files.

## Research Question

Does the installed local/Codex runtime now expose and execute the T336 `lint(project=...)` surface?

## Hypotheses

| Type | Hypothesis | Evidence |
| --- | --- | --- |
| Preferred | Installing the current `engram-cli` and restarting the daemon makes `lint.project` live for installed MCP clients. | Fresh installed `tools/list` reports `project` in the `lint` input schema, and installed `tools/call` with `project=engram` returns JSON. |
| Null | The source code is current, but the installed runtime remains stale. | Rejected after `/Users/yuval.meiri/.local/bin/engram lint run --help` exposed `--scope-project` and daemon status showed the refreshed installed path. |
| Failure | Restarting the daemon leaves path drift or a dead daemon. | Avoided by checking `daemon status` after restart: PID `57356`, spawned by `/Users/yuval.meiri/.local/bin/engram`, current CLI `/Users/yuval.meiri/.local/bin/engram`. |

## Execution

Pre-state:

- Branch was synced with upstream: `git rev-list --left-right --count HEAD...@{u}` returned `0 0`.
- Installed binary hash was
  `01b171ec654da95ea5b1f8363bc109e3069c0ff78bdb38581a202e472f9fd09b`.
- The installed CLI did not expose `--scope-project` for `lint run`, `lint list`, or
  `lint apply-safe`.
- The daemon was running as PID `75180` from `/Users/yuval.meiri/.local/bin/engram serve --http
  --port 8765`.

Action:

```bash
cargo install --path engram-cli --force --root /Users/yuval.meiri/.local
/Users/yuval.meiri/.local/bin/engram daemon stop
/Users/yuval.meiri/.local/bin/engram daemon start
```

Post-state:

- Installed binary hash is
  `b775efa0946862eba8d4d8993bb946f0926372d8a3fe9bbfea98ea38e786e7c2`.
- `/Users/yuval.meiri/.local/bin/engram --version` reports `engram 0.2.0-beta.1`.
- `/Users/yuval.meiri/.local/bin/engram lint run --help` includes
  `--scope-project <SCOPE_PROJECT>`.
- `daemon status` reports PID `57356`, spawned by `/Users/yuval.meiri/.local/bin/engram`, spawn
  version `0.2.0-beta.1`, and current CLI `/Users/yuval.meiri/.local/bin/engram`.

## Validation

Installed CLI isolated smoke passed:

```bash
/Users/yuval.meiri/.local/bin/engram lint --data-dir "$(mktemp -d)" run --scope-project engram --limit 5 --json
```

It returned:

```json
{
  "findings": [],
  "applied_safe_actions": 0
}
```

Fresh installed MCP proxy smoke passed through the restarted daemon:

- `tools/list` returned `lint_has_project_property=True`.
- `tools/list` returned `lint_description_mentions_project=True`.
- `lint.project` schema was
  `{"description":"Optional project scope to lint.","nullable":true,"type":"string"}`.
- `tools/call` with `{"action":"run","project":"engram","limit":1}` returned JSON with
  `applied_safe_actions=0`.

## Non-Claims

T337 proves installed-runtime adoption for T336 only. It does not prove broad lifecycle cleanup,
production parity, hosted CI, native Claude prompt-bearing behavior, effective-hook visibility, live
host labels, or safe broad `lint apply_safe` execution.
