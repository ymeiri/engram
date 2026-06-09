# Brain Harness T359 Obligations CLI Scope

Date: 2026-06-08
Status: implemented, source-validated, installed, and installed-path smoked

## Scope

T359 hardens the local/Codex beta operator path for agent-native obligations. MCP already supported
`project` and `cwd` filters for `obligations(action="list")` and
`obligations(action="doctor")`, but the CLI exposed no equivalent flags for `engram obligations
list` or `engram obligations doctor`. During normal daemon-backed operation this made the health
path noisier than the MCP path and forced operators toward unscoped obligation output.

This slice changes only CLI plumbing and tests:

- adds `--scope-project` and `--cwd` to `engram obligations list`;
- adds `--scope-project` and `--cwd` to `engram obligations doctor`;
- passes those filters through daemon-backed CLI calls;
- passes those filters through direct-store fallback calls when `--data-dir` is supplied;
- keeps unscoped list/doctor behavior unchanged when no project or cwd scope is requested.

## Research Question

Can Engram expose the same obligation list/doctor scoping through the CLI that MCP already supports
without changing obligation storage, daemon semantics, or the beta release scope?

## Hypotheses

| Type | Hypothesis | Result |
| --- | --- | --- |
| Preferred | CLI flag plumbing can make obligation health checks project/cwd-scoped while reusing the existing service behavior. | Supported. |
| Null | MCP scoping is sufficient; CLI operators do not need matching scope controls. | Rejected because T357 proved CLI admin commands are part of the supported daemon-backed beta path. |
| Simpler alternative | Add only `--scope-project` to `doctor`. | Rejected because MCP supports both project and cwd filters for list and doctor, and direct/daemon parity is the lower-risk contract. |
| Failure | The CLI path diverges from MCP or changes unscoped behavior. | Avoided; scoped tests and smokes pass, and unscoped calls still omit `project`/`cwd`. |

## Implementation

- Extended `ObligationCommands::List` with `scope_project` and `cwd`.
- Extended `ObligationCommands::Doctor` with `scope_project` and `cwd`.
- Updated `obligation_daemon_arguments` to serialize `project` and `cwd` only when requested,
  while preserving top-level `engram obligations --project <name>` as the default scope.
- Updated direct-store execution to call `ObligationService::list(status, project, cwd, limit)`
  and `ObligationService::doctor(project, cwd, limit)`.
- Added focused daemon-argument regressions for scoped `list` and `doctor`.

## Validation

Focused source validation passed:

```text
cargo test -p engram-cli obligation_daemon_arguments
cargo check -p engram-cli
cargo fmt --all --check
git diff --check
cargo test -p engram-tests --test obligation_tests
```

Source CLI help exposes the new flags:

```text
cargo run -q -p engram-cli -- obligations doctor --help
cargo run -q -p engram-cli -- obligations list --help
```

Source daemon-backed CLI smokes passed:

```text
cargo run -q -p engram-cli -- obligations doctor --scope-project engram --cwd /Users/yuval.meiri/projects/engram --limit 3 --json
cargo run -q -p engram-cli -- obligations list --scope-project engram --cwd /Users/yuval.meiri/projects/engram --limit 3 --json
```

Source direct-store fallback smokes passed with isolated temp data directories:

```text
engram obligations --data-dir <tempdir> doctor --scope-project engram --cwd /Users/yuval.meiri/projects/engram --limit 3 --json
engram obligations --data-dir <tempdir> list --scope-project engram --cwd /Users/yuval.meiri/projects/engram --limit 3 --json
```

Installed runtime refresh:

```text
cargo install --path engram-cli --force --root /Users/yuval.meiri/.local
```

This replaced installed hash:

```text
62c9955925f74fba706ad466416033cc0bdbc211cf0443a373d4e5925760589a
```

with:

```text
ae45c01ab2a4c5046508e916a7c381655a71611f223fd8fc7989392cd3879f79
```

The installed binary reports:

```text
engram 0.2.0-beta.1
```

Installed daemon-backed CLI smokes passed without restarting the daemon because this slice changes
only CLI request construction:

```text
/Users/yuval.meiri/.local/bin/engram obligations doctor --scope-project engram --cwd /Users/yuval.meiri/projects/engram --limit 3 --json
/Users/yuval.meiri/.local/bin/engram obligations list --scope-project engram --cwd /Users/yuval.meiri/projects/engram --status open --limit 3 --json
```

The scoped doctor returned:

```json
{"open":[],"warnings":[]}
```

and the scoped open list returned:

```json
[]
```

Final exact-head validation passed:

```text
git diff --check
cargo fmt --all --check
cargo check --all-targets
cargo clippy --all-targets -- -D warnings
env CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 cargo test --all-targets --jobs 1
cargo doc --no-deps
./scripts/package-install-smoke.sh
```

The first `./scripts/local-ci.sh` run completed through check and clippy, then its test step
stalled in `rustc` with no CPU activity. That hung validation process was terminated, the exact
same serialized test step was rerun directly and passed, and `cargo doc --no-deps` passed. The
package install smoke created and verified:

```text
dist/engram-0.2.0-beta.1-aarch64-apple-darwin.tar.gz
dist/engram-0.2.0-beta.1-aarch64-apple-darwin.tar.gz.sha256
```

and confirmed packaged HTTP `/health` returned:

```json
{"status":"ok","service":"engram","version":"0.2.0-beta.1"}
```

## Gate Impact

T359 improves the supported local/Codex beta operator path by making obligation health checks
scoped and daemon-compatible through the CLI. It does not mark PR #3 ready, merge, tag, publish,
close hosted CI, run native Claude, prove effective-hook visibility, prove live host labels, mutate
lifecycle state, run broad `lint apply_safe`, or change the supported beta scope.
