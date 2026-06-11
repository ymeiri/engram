# Engram v0.2.0 GA Readiness Matrix

Date: 2026-06-11
Status: GA preparation in progress
Base inspected: `main` at `40b8f0fff5b3de886d14bc8c3ac673bf303853bc`

## Summary

The expected GA target remains `v0.2.0`. Repo and release evidence show no `v0.2.0`
tag or GitHub release yet. The latest published prerelease is `v0.2.0-beta.2`, and
the current `main` branch is four commits past that tag.

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
- Current hosted CI: main push run `27276034000` for `40b8f0f` completed
  successfully on 2026-06-10.
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
| Hosted CI | Validated for current main | Main push CI run `27276034000` passed on current head. | Re-run exact-head hosted CI after any GA version/docs/package changes. |
| Local runtime | Validated for beta.2 source | Release build passed; installed binary and daemon now match `0.2.0-beta.2`. | Repeat install/daemon smoke on the final GA versioned head. |
| `orient` hot path | Validated / preserve | Lean `orient` returned compact scope, cursor, Brain Loop guidance, candidate IDs, and no open obligations. | Do not expand `orient`; only add focused regressions if GA changes touch ranking or lifecycle. |
| Memory obligations | Validated | `engram obligations doctor --scope-project engram --cwd ...` returned `open=[]`, `warnings=[]`. | Re-run after every meaningful GA commit. |
| Generated vault | Validated | Canonical vault compiled to `generated_file_count=2851`, `expected_generated_file_count=2851`, `user_file_count=0`. | Re-run before final GA release if memory writes occur. |
| Native Claude production gate | Blocked | Preflight baseline now matches Claude Code `2.1.173` and current Engram daemon/vault; the only remaining blocker is an already-running native Claude CLI process. | Do not claim native Claude prompt-bearing, `/hooks`, or live host-label proof until a clean process window allows the fail-closed preflight and proof run. |
| Claude static harness readiness | Partially validated | `engram harness doctor --harness claude-code --json` reports `ready=true` with warnings about user-owned snippet, extra permissions, split settings, and unproved live hook visibility. | Resolve or explicitly scope warnings before GA claims depend on live Claude hook behavior. |
| Codex setup/runtime path | Partially validated | Beta.2 docs and setup support Codex; current task used native Codex MCP `orient` successfully. | Run a final supported-path setup/orient smoke on the GA head. |
| Cursor setup/runtime path | Implemented / needs validation | Beta.2 adds guided setup for Cursor. | Run or explicitly defer Cursor smoke before GA support claims. |
| Release notes and changelog | Missing for GA | Beta.1/beta.2 notes and changelog exist; no GA release notes exist yet. | Add `v0.2.0` release notes and promote changelog entries only after the GA scope is fixed. |
| Package artifacts | Partially validated | Beta.2 GitHub release has archive and checksum assets; package scripts exist. | Run `scripts/package-release.sh`, `scripts/package-install-smoke.sh`, and publish/verify assets for final `v0.2.0`. |
| Homebrew | Implemented / needs GA validation | Beta Homebrew rendering exists and post-beta style fixes landed. | Render/audit the formula against the final GA archive and update the tap if publishing allows it. |
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
