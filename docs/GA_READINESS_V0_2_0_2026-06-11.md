# Engram v0.2.0 GA Readiness Matrix

Date: 2026-06-11
Status: GA preparation in progress
Validated release-code checkpoint: `b650a307793b576b523828a9ca2886fa41058b54`
Validated release-notes docs checkpoint: `c095770f1821c731c01b176a83fe43903618a2f8`

## Summary

The expected GA target remains `v0.2.0`. Repo and release evidence show no `v0.2.0`
tag or GitHub release yet. The latest published prerelease is `v0.2.0-beta.2`.

`v0.2.0-beta.1` is verified as a signed local tag and published GitHub prerelease.
`v0.2.0-beta.2` is also verified as a signed local tag and published GitHub
prerelease with macOS Apple Silicon archive and checksum assets.

## Evidence Baseline

- Git state: `main` tracks `origin/main`, with `0 0` ahead/behind after fetch.
- Working tree: tracked tree was clean before this slice; untracked `AGENTS.md`
  remains local/user-owned instruction material and must stay unstaged unless
  explicitly requested.
- Tags: `v0.2.0-beta.1` peels to `4d6e751`; `v0.2.0-beta.2` peels to
  `ec2e263`.
- GitHub releases: `v0.2.0-beta.1` and `v0.2.0-beta.2` are published
  prereleases; `v0.1.0` remains the stable/latest release.
- Checkpoint hosted CI: main push run `27335890558` for `b650a30` completed
  successfully on 2026-06-11. The `Test` job ran
  `cargo test --locked --all-targets --jobs 1` and completed in `27m45s`.
- Release-notes hosted CI: main push run `27340971819` for `c095770` completed
  successfully on 2026-06-11. The `Test` job ran
  `cargo test --locked --all-targets --jobs 1` and completed in `28m43s`.
- Workspace versions: every Engram workspace package resolves to
  `0.2.0-beta.2` in `cargo metadata`, and `Cargo.lock` matches that version.
- Local runtime before refresh: installed `engram` and daemon still reported
  `0.2.0-beta.1`.
- Local runtime after refresh: installed binary hash
  `d1bef731d7172e3a36a716bd5a7da4a9fe8f50978123f711896374559f855b44`
  reports `engram 0.2.0-beta.2`; daemon restarted on port `8765`, PID `9401`,
  with spawn version `0.2.0-beta.2`; `/health` returns
  `{"status":"ok","service":"engram","version":"0.2.0-beta.2"}`.

## Matrix

| Area | Status | Evidence | GA Gap / Next Action |
| --- | --- | --- | --- |
| GA target | Validated | Current prerelease line is `0.2.0-beta.2`; no `v0.2.0` tag/release exists. | Keep GA target as `v0.2.0` unless a later release decision changes it. |
| Beta baseline | Validated | Local tags and GitHub prereleases exist for beta.1 and beta.2 with release assets. | Use beta.2 plus current `main` as the GA baseline. |
| Versioning | Partially validated | Workspace metadata and lockfile are consistent at `0.2.0-beta.2`. | Bump workspace version and lockfile to `0.2.0` only after GA blockers are closed. |
| Hosted CI | Validated for listed checkpoints | Main push CI run `27335890558` passed for release-code checkpoint `b650a30`; run `27340971819` passed for release-notes docs checkpoint `c095770`. | Re-run exact-head hosted CI after any GA version/docs/package changes. |
| Local runtime | Validated for beta.2 source | Release build passed; installed binary and daemon now match `0.2.0-beta.2`. | Repeat install/daemon smoke on the final GA versioned head. |
| `orient` hot path | Validated / preserve | Lean `orient` returned compact scope, cursor, Brain Loop guidance, candidate IDs, and no open obligations. | Do not expand `orient`; only add focused regressions if GA changes touch ranking or lifecycle. |
| Memory obligations | Validated | `engram obligations doctor --scope-project engram --cwd ...` returned `open=[]`, `warnings=[]`. | Re-run after every meaningful GA commit. |
| Generated vault | Validated | Canonical vault compiled after memory updates to `generated_file_count=2860`, `expected_generated_file_count=2860`, `user_file_count=0`. | Re-run before final GA release if memory writes occur. |
| Native Claude production gate | Blocked | Preflight baseline now matches Claude Code `2.1.173` and current Engram daemon/vault; the only remaining blocker is an already-running native Claude CLI process, most recently PID `61303` on `ttys001`. | Do not claim native Claude prompt-bearing, `/hooks`, or live host-label proof until a clean process window allows the fail-closed preflight and proof run. |
| Claude static harness readiness | Partially validated | `engram harness doctor --harness claude-code --json` reports `ready=true` with warnings about user-owned snippet, extra permissions, split settings, and unproved live hook visibility. | Resolve or explicitly scope warnings before GA claims depend on live Claude hook behavior. |
| Codex setup/runtime path | Partially validated | Beta.2 docs and setup support Codex; current task used native Codex MCP `orient` successfully. | Run a final supported-path setup/orient smoke on the GA head. |
| Cursor setup/runtime path | Implemented / needs validation | Beta.2 adds guided setup for Cursor. | Run or explicitly defer Cursor smoke before GA support claims. |
| Release notes and changelog | Drafted / needs final validation | `docs/RELEASE_NOTES_V0_2_0.md` now exists with install, upgrade, first-run, and known-limitation text; changelog still has only Unreleased entries. | Review and finalize the notes on the versioned GA head, then promote changelog entries only after GA scope is fixed. |
| Package artifacts | Partially validated | Beta.2 GitHub release has archive and checksum assets; local package-install smoke passed in an isolated temp `DIST_DIR` for the pre-GA `0.2.0-beta.2` workspace. | Run `scripts/package-release.sh`, `scripts/package-install-smoke.sh`, and publish/verify assets for final `v0.2.0`. |
| Homebrew | Implemented / needs GA validation | Homebrew rendering exists; formula errors and release-facing packaging docs now use GA-neutral wording while preserving the macOS Apple Silicon scope. A local beta.2 formula render produced Ruby-valid formula text with no remaining beta-specific Homebrew wording. | Render/audit the formula against the final GA archive and update the tap if publishing allows it. |
| Docs consistency | Risky | README and MCP setup are beta.2-oriented; historical docs contain many beta-specific caveats by design. | Update only release-facing docs for GA; do not rewrite historical T-doc evidence. |
| Memory lifecycle / M6 | Risky | Legacy layers remain substrate; broad lifecycle cleanup and M6 write-apply expansion are not proven GA-complete. | Either close specific lifecycle/M6 gates with evidence or explicitly scope them out of `v0.2.0` GA claims. |
| Git release mechanics | Missing | No `v0.2.0` tag, release, or package publication exists. | Tag, publish, and verify only after final validation passes. |

## First GA Slice Completed

This slice refreshed the native-Claude production preflight baseline from stale Claude Code
`2.1.169` metadata to the currently observed Claude Code `2.1.173` target and SHA-256, then
recompiled the canonical generated vault and refreshed the installed Engram runtime from current
source. The stale Claude-version, vault-count, and installed-runtime blockers are now removed from
the current preflight evidence.

Remaining high-risk GA question: whether `v0.2.0` requires live native-Claude prompt-bearing,
effective-hook, and host-label proof. If yes, the next blocking condition is external process
state: a pre-existing native Claude CLI process is running, and the fail-closed preflight correctly
refuses to proceed.

## Exact-Head CI Refresh

After this first GA slice was committed as `b650a30`, GitHub Actions main CI run
`27335890558` completed successfully for that exact commit. Format, Check, Docs, Clippy, and Test
all passed; the serialized Test job completed in `27m45s`.

This proves the `b650a30` readiness checkpoint. Any later docs, version, package, or release commit
still needs its own exact-head CI evidence before it can be used as a final GA release head.
This document intentionally treats its own later maintenance commits as evidence updates, not as
the release-code checkpoint.

## GA Release Notes Draft

`docs/RELEASE_NOTES_V0_2_0.md` has been added as the release-notes artifact that
`scripts/package-release.sh` will require once the workspace version is bumped to `0.2.0`.
The notes include install, upgrade, first-run, and known-limitation sections, and they explicitly
avoid claiming native Claude prompt-bearing, live `/hooks`, or live host-label proof unless the
final release gate records that evidence.

The notes are not by themselves a release decision. They still need final review on the versioned
GA head before tag, package, Homebrew, or GitHub release publication.

The release-notes draft commit `c095770` has exact-head GitHub Actions evidence: run
`27340971819` completed successfully for Format, Check, Docs, Clippy, and Test. This validates the
release-notes docs checkpoint, not a final GA versioned release head.

## Package and Homebrew Rehearsal

A local pre-GA package rehearsal ran in an isolated temp `DIST_DIR` on the `0.2.0-beta.2`
workspace version. `scripts/package-install-smoke.sh` built the release binary, created and checked
the `engram-0.2.0-beta.2-aarch64-apple-darwin.tar.gz` archive and checksum, verified safe archive
paths and manifest hashes, installed the packaged binary into a temp prefix, and started the
packaged HTTP server long enough to verify:

```json
{"status":"ok","service":"engram","version":"0.2.0-beta.2"}
```

The same temp archive was then used to render a Homebrew formula with
`scripts/render-homebrew-formula.sh`. `ruby -c` reported `Syntax OK`, and a targeted search found no
remaining `Homebrew beta`, `beta Homebrew`, or `Homebrew beta currently` wording in the rendered
formula. This is rehearsal evidence only; the final `v0.2.0` archive still needs the same package
and Homebrew checks after the workspace version bump.

## Validation Run

- `git fetch --tags --prune origin`
- `git rev-list --left-right --count main...origin/main`
- `gh release list --repo ymeiri/engram --limit 20`
- `gh release view v0.2.0-beta.1 --repo ymeiri/engram ...`
- `gh release view v0.2.0-beta.2 --repo ymeiri/engram ...`
- `cargo metadata --no-deps --format-version 1`
- `cargo fmt --all --check`
- `git diff --check`
- `bash -n scripts/native-claude-gate-preflight.sh`
- `cargo build --locked --release -p engram-cli`
- `engram vault compile /Users/yuval.meiri/.engram/vault`
- `engram daemon stop`
- `engram daemon start`
- `curl -fsS http://127.0.0.1:8765/health`
- `engram vault status /Users/yuval.meiri/.engram/vault --json`
- `engram harness doctor --harness claude-code --json`
- `engram obligations doctor --scope-project engram --cwd /Users/yuval.meiri/projects/engram --limit 20 --json`
- `scripts/native-claude-gate-preflight.sh --allow-worktree-changes --json`
- `gh run watch 27335890558 --repo ymeiri/engram --exit-status --interval 30`
- `scripts/native-claude-gate-preflight.sh --json`
- `gh run watch 27340971819 --repo ymeiri/engram --exit-status --interval 30`
- `git diff --check`
- `bash -n scripts/render-homebrew-formula.sh`
- `bash -n scripts/package-release.sh`
- `bash -n scripts/package-install-smoke.sh`
- `cargo fmt --all --check`
- `DIST_DIR=<temp> scripts/package-install-smoke.sh`
- `DIST_DIR=<temp> FORMULA_OUTPUT=<temp>/homebrew/Formula/engram.rb HOMEBREW_HOST_TRIPLE=aarch64-apple-darwin scripts/render-homebrew-formula.sh`
- `ruby -c <temp>/homebrew/Formula/engram.rb`
- `if rg -n "Homebrew beta|beta Homebrew|Homebrew beta currently" <temp>/homebrew/Formula/engram.rb; then exit 1; fi`
