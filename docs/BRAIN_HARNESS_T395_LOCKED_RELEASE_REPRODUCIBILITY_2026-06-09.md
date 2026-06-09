# Brain Harness T395 Locked Release Reproducibility

Status: completed release reproducibility hardening
Date: 2026-06-09

## Research Question

Can the beta release path use one committed Cargo dependency graph across local CI, hosted CI, and
package builds?

## Hypotheses

| Hypothesis | Expected result |
| --- | --- |
| Preferred | Commit `Cargo.lock` and run CI/package cargo commands with `--locked`, so dependency drift fails early instead of silently changing release inputs. |
| Null | Leaving `Cargo.lock` ignored is sufficient because Cargo resolves compatible dependencies from manifests on each build. |
| Simpler alternative | Track `Cargo.lock` but leave scripts and hosted CI unlocked. |
| Failure | The lockfile is stale or `--locked` breaks local CI/package validation. |

## Measurement

- `cargo metadata --locked --no-deps --format-version 1`
- `cargo pkgid --locked -p engram-cli`
- `bash -n scripts/local-ci.sh scripts/package-release.sh scripts/package-install-smoke.sh`
- `cargo check --locked --all-targets`
- `cargo clippy --locked --all-targets -- -D warnings`
- `./scripts/package-install-smoke.sh`
- `./scripts/local-ci.sh`

## Evidence

`Cargo.lock` already existed locally and `cargo metadata --locked --no-deps --format-version 1`
succeeded before the policy change. The release gap was that `.gitignore` excluded the lockfile and
the CI/package scripts did not require locked dependency resolution.

T395 changes that boundary:

- `.gitignore` no longer ignores `Cargo.lock`;
- `Cargo.lock` is tracked for the workspace;
- `.github/workflows/ci.yml` uses `--locked` for check, test, clippy, and docs jobs;
- `./scripts/local-ci.sh` mirrors those locked CI commands;
- `./scripts/package-release.sh` uses `cargo pkgid --locked` and
  `cargo build --locked --release -p engram-cli`;
- `./scripts/package-install-smoke.sh` resolves the expected package version with
  `cargo pkgid --locked`;
- README development commands now show the locked build/test/clippy path.

## Validation

The locked dependency graph is current and the updated command surfaces passed:

- `cargo metadata --locked --no-deps --format-version 1`
- `cargo pkgid --locked -p engram-cli`
- `bash -n scripts/local-ci.sh scripts/package-release.sh scripts/package-install-smoke.sh`
- `git diff --check`
- `cargo fmt --all --check`
- `cargo check --locked --all-targets`
- `cargo clippy --locked --all-targets -- -D warnings`
- `./scripts/package-install-smoke.sh`
- `./scripts/local-ci.sh`

The package/install smoke rebuilt the locked release package, verified the checksum, inspected the
archive paths, installed the binary into a temporary prefix, confirmed `engram 0.2.0-beta.1`, and
verified packaged HTTP `/health` returned:

```json
{"status":"ok","service":"engram","version":"0.2.0-beta.1"}
```

## Decision

Accept the preferred hypothesis. For a binary-producing beta candidate, a committed lockfile plus
locked CI/package commands is the smallest release-reproducibility step that prevents unreviewed
dependency graph changes between local validation, hosted CI, and release archive creation.

## Boundary

T395 does not change Engram runtime behavior, accept the hosted-CI fallback, mark PR #3 ready,
merge, tag, publish, launch native Claude, prove hooks or host labels, mutate M6, run lifecycle
cleanup, or make Engram production/GA ready.
