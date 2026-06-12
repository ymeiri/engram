# Engram v0.2.0 GA Readiness Matrix

Date: 2026-06-11
Last refreshed: 2026-06-12
Status: GA preparation in progress
Validated setup-path docs checkpoint: `86dd38d0ef56bad5aa0c999578313c7f4a133e41`
Validated release-hardening checkpoint: `eb0e3a96b7a751a90d482dad95ab9ae31af76a7e`
Validated release-code baseline checkpoint: `b650a307793b576b523828a9ca2886fa41058b54`
Validated release-notes docs checkpoint: `c095770f1821c731c01b176a83fe43903618a2f8`
Pre-runbook GA release-owner review head: `1eefa11aff32e4d3802cc327ddc8d8957fd2f56f`
Documented Homebrew-gated GA release-owner review head:
`809426945cb7e0d78950552165691e29aa0191bc`

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
- Release-hardening hosted CI: main push run `27361233663` for `eb0e3a9` completed
  successfully on 2026-06-11. The `Test` job ran
  `cargo test --locked --all-targets --jobs 1` and completed from
  `16:20:13Z` to `16:47:17Z`.
- Setup-path docs hosted CI: main push run `27363378532` for `86dd38d` completed
  successfully on 2026-06-11. The `Test` job ran
  `cargo test --locked --all-targets --jobs 1` and completed from
  `16:57:15Z` to `17:25:26Z`.
- Workspace versions: every Engram workspace package now resolves to
  `0.2.0` in `cargo metadata --locked`, and `Cargo.lock` matches that version.
- Historical pre-runbook GA release-owner review CI: main push run `27379891728` for
  `1eefa11aff32e4d3802cc327ddc8d8957fd2f56f` completed successfully on
  2026-06-11 for Check, Test, Clippy, Docs, and Format.
- Historical pre-runbook full GA release gate: `scripts/release-gate-report.sh --target ga --hosted-run 27379891728
  --json` passed with `local_ci=passed`, `package_install_smoke=passed`,
  `release_scope.state=complete`, `release_gate_state=hosted_ci_passing_release_owner_review_required`,
  and `ready_for_release_owner_review=true`.
- Current Homebrew-gated GA release-owner review CI: main push run `27388790648`
  for `809426945cb7e0d78950552165691e29aa0191bc` completed successfully on
  2026-06-12 for Check, Test, Clippy, Docs, and Format. The `Test` job ran
  `cargo test --locked --all-targets --jobs 1` and completed in `28m51s`.
- Current full GA release gate: `scripts/release-gate-report.sh --target ga --hosted-run 27388790648
  --json` passed with `local_ci=passed`, `package_install_smoke=passed`,
  `homebrew_formula_render=passed`, `release_scope.state=complete`,
  `release_gate_state=hosted_ci_passing_release_owner_review_required`, and
  `ready_for_release_owner_review=true`.
- Current local asset-dir release verifier:
  `scripts/verify-published-release-install.sh --tag v0.2.0 --asset-dir dist --json`
  passed with `assets.source=asset_dir`,
  `expected_git_head=809426945cb7e0d78950552165691e29aa0191bc`, `install_smoke=passed`,
  and `release_actions_performed=false`. Newer verifier output distinguishes this local
  rehearsal from published-release proof with `asset_install_verified=true` and
  `published_install_verified=false` when `assets.downloaded=false`.
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
| Versioning | Validated on Homebrew-gated owner-review head / gate command authoritative | Workspace metadata and lockfile are consistent at the intended `0.2.0` GA version on head `8094269`; full GA gate confirmed `workspace_version_matches_release=true`. | Rerun exact-head CI and the full GA gate after this readiness refresh or any other release-facing change. |
| Hosted CI | Validated on Homebrew-gated owner-review head / gate command authoritative | Main push CI run `27388790648` passed for head `8094269`; earlier runs `27379891728`, `27372009309`, `27363378532`, `27361233663`, `27335890558`, and `27340971819` passed for prior release-owner, release-gate, setup-path docs, release-hardening, release-code, and release-notes checkpoints. | Re-run exact-head hosted CI after any GA version/docs/package changes. |
| Local runtime | Validated for GA package smoke / installed global still beta.2 | Full GA gate package/install smoke passed for `engram 0.2.0` on `8094269`; local asset-dir verification also passed without downloads or publishing. Local `--asset-dir` verifier output now reports `asset_install_verified=true` while keeping `published_install_verified=false`. The previously installed global binary and daemon remain beta.2 evidence until an explicit post-release refresh. | After release publication, verify the published install path and then refresh local runtime evidence if desired. |
| `orient` hot path | Validated / preserve | Lean `orient` returned compact scope, cursor, Brain Loop guidance, candidate IDs, and no open obligations. | Do not expand `orient`; only add focused regressions if GA changes touch ranking or lifecycle. |
| Memory obligations | Validated | `engram obligations doctor --scope-project engram --cwd ...` returned `open=[]`, `warnings=[]`. | Re-run after every meaningful GA commit. |
| Generated vault | Validated | Canonical vault status after memory updates reports `generated_file_count=2902`, `expected_generated_file_count=2902`, `user_file_count=0`. | Re-run before final GA release if memory writes occur. |
| Native Claude production gate | Blocked | Fresh `scripts/native-claude-gate-preflight.sh --json` on `a082a63` reports `gate_state=blocked`: branch synced, tracked tree clean, obligations clean, vault aligned at `2888/2888`, Claude Code `2.1.173` hash matches, daemon reports `0.2.0-beta.2`, and the blocker is an already-running native Claude CLI process on `ttys001`. | Do not claim native Claude prompt-bearing, `/hooks`, or live host-label proof until a clean process window allows the fail-closed preflight and proof run. |
| Claude static harness readiness | Partially validated | `engram harness doctor --harness claude-code --json` reports `ready=true` with warnings about user-owned snippet, extra permissions, split settings, and unproved live hook visibility. | Resolve or explicitly scope warnings before GA claims depend on live Claude hook behavior. |
| Codex setup/runtime path | Validated for generated adapter install and current MCP use | `engram setup --agent codex --root <temp> --write --yes` wrote the two required Codex skills plus `AGENTS.engram.md`; `engram harness status/doctor --harness codex --root <temp> --json` reported required adapters installed and `ready=true`. Current Codex session also used MCP `orient` successfully. | Repeat on the final GA versioned head; live lifecycle compliance remains advisory and host-driven. |
| Cursor setup/runtime path | Validated for generated adapter install | `engram setup --agent cursor --root <temp> --write --yes` wrote the three required Cursor skills; `engram harness status/doctor --harness cursor --root <temp> --json` reported required adapters installed and `ready=true`. | Repeat on the final GA versioned head; no live Cursor host session has been claimed. |
| Release notes and changelog | Drafted / needs final validation | `docs/RELEASE_NOTES_V0_2_0.md` now exists with install, upgrade, first-run, and known-limitation text; changelog still has only Unreleased entries. | Review and finalize the notes on the versioned GA head, then promote changelog entries only after GA scope is fixed. |
| Package artifacts | Validated on Homebrew-gated owner-review head / unpublished | Beta.2 GitHub release has archive and checksum assets; full GA package-install smoke passed for `engram-0.2.0-aarch64-apple-darwin.tar.gz` plus checksum on clean head `8094269`. Local release packaging still fails closed on tracked changes by default, and the install smoke rejects checksum files that do not name exactly the expected archive. | Publish and verify assets only after owner approval and a fresh full gate on the release head. |
| Homebrew | Validated by full GA gate / unpublished | The full GA gate for `8094269` rendered `dist/homebrew/Formula/engram.rb`, `ruby -c` reported `Syntax OK`, and the gate rejected beta-specific Homebrew wording. The renderer also requires the adjacent `.sha256` asset to name and hash the same archive before writing formula text. The remote tap `ymeiri/homebrew-engram` still points at beta.2 until explicitly updated. | Update the tap only after release approval, fresh package evidence, and published asset verification. |
| Docs consistency | Partially hardened | README, MCP setup, and security policy now use a `0.2.x` support-scope framing for supported setup paths while preserving the current fact that `v0.2.0-beta.2` is the latest published artifact. Historical docs still contain beta-specific caveats by design. | Re-check release-facing docs after the final `0.2.0` version bump and artifact publication; do not rewrite historical T-doc evidence. |
| Memory lifecycle / M6 | Scoped for GA / final validation required | Legacy layers remain supported substrate; broad lifecycle cleanup and unrestricted automated lifecycle mutation are not proven GA-complete and are explicitly outside the current `v0.2.0` release claims. `scripts/release-gate-report.sh --target ga` now checks that the GA release notes retain those scope acknowledgements. | Keep the release-notes scope acknowledgements through the final version bump and full GA gate; do not broaden lifecycle/M6 claims without fresh implementation and validation evidence. |
| Git release mechanics | Runbook prepared / not published | No `v0.2.0` tag, release, or package publication exists. `scripts/release-gate-report.sh --target ga --hosted-run 27388790648 --json` passed on the Homebrew-gated owner-review head and reported `ready_for_release_owner_review=true`. `docs/GA_RELEASE_OWNER_APPROVAL_V0_2_0_2026-06-12.md` names the fail-closed post-approval command sequence and requires a fresh full gate on the release head. The GA gate now defaults `expected_branch=main` so owner-review evidence cannot be collected from a synced non-main branch unless explicitly overridden. | Tag, publish, update Homebrew, and verify only after explicit release-owner approval and fresh exact-head gate evidence. |

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

## Release Packaging Dirty-State Guard

`scripts/package-release.sh` now refuses to build an archive when tracked working-tree or index
changes are present unless `ALLOW_TRACKED_CHANGES=1` is set. This keeps final release artifacts
fail-closed by default while preserving explicit local rehearsal support.

The guard was checked against the development diff for this slice: default `scripts/package-release.sh`
exited with the tracked-change error before building, while
`ALLOW_TRACKED_CHANGES=1 DIST_DIR=<temp> scripts/package-install-smoke.sh` still built and validated
a dirty rehearsal archive, including manifest verification and packaged HTTP `/health`.

## Published Release Verification Guard

`scripts/verify-published-release-install.sh` now verifies release identity before downloading
assets. The requested tag must match the workspace package version, the local tag must peel to the
expected packaged Git commit, draft releases fail closed, and prerelease state must match
`--expected-prerelease` or the default tag-based inference. Tags with a suffix such as
`v0.2.0-beta.2` infer prerelease `true`; stable tags such as `v0.2.0` infer prerelease `false`.

This keeps the final `v0.2.0` post-publish verifier from accepting a draft or prerelease GitHub
release even if the archive assets install successfully. It also prevents `v0.2.0` evidence from
being collected on an unbumped `0.2.0-beta.2` checkout, or from a published archive whose manifest
commit does not match the local release tag.

The verifier also distinguishes local pre-publish rehearsals from actual published-install proof:
`--asset-dir` mode can prove `asset_install_verified=true`, but it now keeps
`published_install_verified=false` because no GitHub release assets were downloaded.

## Package Manifest Verification Guard

`scripts/package-install-smoke.sh` now parses packaged `MANIFEST.json` with `jq` instead of
line-oriented text matching. The install smoke checks release metadata, boolean dirty-state
provenance, Cargo.lock hash provenance, and each packaged file hash through structured JSON queries.

This keeps local package rehearsals and published-release install verification from accepting
malformed or misleading manifest JSON during the final `v0.2.0` artifact proof.

## Release Manifest Build Guard

`scripts/package-release.sh` now validates the generated `MANIFEST.json` with `jq` before the
release archive is created. The release builder checks package identity, version, host triple,
archive name, Git commit, dirty-state provenance, Cargo.lock hash, required package files, and
SHA-256 shape before any tarball or checksum is written.

This keeps final `v0.2.0` packaging from publishing an archive whose manifest is malformed or
structurally inconsistent before the downstream install smoke ever runs.

## Package Checksum Filename Guard

`scripts/package-install-smoke.sh` now checks the copied `.sha256` file before running
`shasum -a 256 -c`. The checksum file must contain exactly one line, the digest must be a
64-character SHA-256 hex value, and the filename must exactly match the expected release archive
basename.

This keeps local package rehearsals and published-release install verification from accepting a
checksum asset whose digest is valid but whose filename points at a different path or archive name.

## Homebrew Checksum Asset Guard

`scripts/render-homebrew-formula.sh` now requires the release checksum file next to the archive
before it writes `Formula/engram.rb`. The checksum file must contain exactly one line, name the
expected archive basename, contain a 64-character SHA-256 digest, and match the digest computed
from the archive used for the formula.

This keeps the final tap update from being rendered from an archive/checksum pair that would upload
inconsistent GitHub release assets.

## Hosted CI Multi-Session Test Stabilization

GitHub Actions run `27358951202` for commit `900101f` failed in the `Test` job while Format,
Check, Docs, and Clippy passed. The failure was isolated to
`engram-tests/tests/multi_session_tests.rs`: all 17 tests failed to start a daemon because the Rust
test harness ran daemon-spawning tests concurrently and each timed out waiting for `/health`.

The test harness now serializes each `TestDaemon` lifetime with a process-local async lock. This
keeps multi-session coverage intact while preventing hosted CI from starting one daemon per test at
the same time.

The follow-up commit `eb0e3a9` has exact-head hosted CI evidence: run `27361233663` completed
successfully for Format, Docs, Check, Clippy, and Test. This makes `eb0e3a9` the validated
release-hardening checkpoint before any future GA version, package, or release publication changes.

## Supported Setup Path Rehearsal

Codex and Cursor setup paths were rehearsed in isolated temp roots so the validation did not modify
the user's real harness configuration.

Codex:

- `target/debug/engram setup --agent codex --root /tmp/engram-codex-setup.BWMgtQ --write --yes`
  wrote `.codex/skills/engram-memory-session/SKILL.md`,
  `.codex/skills/engram-resume-session/SKILL.md`, and `AGENTS.engram.md`.
- `target/debug/engram harness status --harness codex --root /tmp/engram-codex-setup.BWMgtQ
  --json` and `target/debug/engram harness doctor --harness codex --root
  /tmp/engram-codex-setup.BWMgtQ --json` reported required adapters installed and `ready=true`.

Cursor:

- `target/debug/engram setup --agent cursor --root /tmp/engram-cursor-setup.ShHnjs --write --yes`
  wrote `.cursor/skills/engram-memory-session/SKILL.md`,
  `.cursor/skills/engram-resume-session/SKILL.md`, and
  `.cursor/skills/engram-end-session/SKILL.md`.
- `target/debug/engram harness status --harness cursor --root /tmp/engram-cursor-setup.ShHnjs
  --json` and `target/debug/engram harness doctor --harness cursor --root
  /tmp/engram-cursor-setup.ShHnjs --json` reported required adapters installed and `ready=true`.

This proves generated adapter install/status behavior for clean roots. It does not claim a live
Cursor host session, and it does not upgrade advisory lifecycle compliance into a hard runtime
guarantee.

## Native Claude Gate Refresh

A fresh read-only native Claude production-gate preflight ran again on release-gate checkpoint
`a082a63`. The script reported:

- `gate_state=blocked`
- branch `main`, upstream `origin/main`, `ahead=0`, `behind=0`
- `tracked_changes_present=false`
- Claude Code target `/Users/yuval.meiri/.local/share/claude/versions/2.1.173`,
  version `2.1.173 (Claude Code)`, SHA-256
  `235c1bacdcc7f9d8d92368c95a0c66c26fcac98f878f21b10c73af340bc331ab`
- Engram daemon running from `/Users/yuval.meiri/.local/bin/engram`, spawn version
  `0.2.0-beta.2`
- `harness_status.ready=true` and `harness_doctor.ready=true`, with static warnings
  that live effective-hook visibility still requires Claude Code `/hooks` verification
- snippet-only harness install dry-run planned no generated-file changes
- obligations doctor returned no open items or warnings
- canonical vault status was aligned at `generated_file_count=2888`,
  `expected_generated_file_count=2888`, `user_file_count=0`
- the blocker was an already-running native Claude CLI process on `ttys001`
- no native Claude launch, `/hooks` command, process signal, or release action was performed

Strict mode also failed closed as intended: `scripts/native-claude-gate-preflight.sh --json
--require-ready` returned exit code `2` while reporting the same blocked gate state.

This refresh keeps the production gate open. It narrows the current native-Claude blocker to
process-state availability for a future proof run; it does not prove prompt-bearing behavior,
effective-hook visibility, or live host-label attribution.

This refresh also confirms the divergent-branch warning seen during a prior pull
attempt is not the live repo state: after `git fetch --tags --prune origin`, `main` and
`origin/main` were still aligned at `a082a63` with `ahead=0`, `behind=0`.

## Release-Facing Docs Scope Cleanup

README, MCP setup, and security policy wording were updated so the supported setup path is framed
as a `0.2.x` support scope rather than a beta-only support matrix. The docs still preserve the
current release fact that `v0.2.0-beta.2` is the latest published artifact and that final
`v0.2.0` artifacts require exact-head release validation before publication.

This narrows the release-facing documentation gap without changing historical T-doc evidence,
version metadata, tags, packages, Homebrew output, or GitHub releases.

## GA Release Gate Report

The previous release-owner evidence collector was beta-specific and defaulted to PR #3. That PR is
merged and no longer points at current `main`, so it cannot validate the current GA baseline.

`scripts/release-gate-report.sh` now supports an explicit `--target ga` mode that verifies the
current branch, upstream sync, tracked-change state, and a hosted push CI run for the exact current
head without requiring a PR. GA mode defaults the expected branch to `main` and fails closed on a
branch mismatch; `--expected-branch` or `EXPECTED_BRANCH` must be explicit if a future release uses
a non-main release branch. `scripts/beta-release-gate-report.sh` remains as a compatibility wrapper
for beta PR gates. The script remains evidence-only: it does not accept fallbacks, mark a PR ready,
merge, tag, publish, mutate harness state, or change release scope.

GA mode also separates the current workspace package version from the intended release version.
While the workspace still reports `0.2.0-beta.2`, the report defaults the intended GA release
version to `0.2.0`, emits `workspace_version_matches_release=false`, and keeps
`release_gate_state=version_bump_required`. This prevents pre-GA evidence from suggesting a
`tag_v0.2.0-beta.2` action for the GA path.

The GA report also verifies release-note scope acknowledgements for high-risk items that remain
outside the current `v0.2.0` claim: native Claude prompt-bearing/live `/hooks` proof and broad
legacy lifecycle/M6 mutation. If those acknowledgements are removed before the final release gate,
the report emits `release_scope.state=incomplete` and blocks with
`release_gate_state=release_scope_acknowledgement_required` instead of marking the release ready
for owner review.

## GA Version Bump Checkpoint

This release slice moves workspace package metadata from the validated
`0.2.0-beta.2` prerelease baseline to the intended `0.2.0` GA version. This is a validation-head
change only: it does not create a tag, upload final release assets, update Homebrew, or publish a
GitHub release.

Local validation on the version-bump worktree confirmed:

- `cargo metadata --locked --no-deps --format-version 1` reports all seven workspace packages at
  `0.2.0`.
- `cargo pkgid --locked -p engram-cli` resolves `engram-cli#0.2.0`.
- `scripts/local-ci.sh` passed, including format, check, clippy, tests, and docs.
- `ALLOW_TRACKED_CHANGES=1 DIST_DIR=<temp> scripts/package-install-smoke.sh` passed for
  `engram-0.2.0-aarch64-apple-darwin.tar.gz`, including checksum, manifest, temp install, packaged
  `engram 0.2.0`, and packaged HTTP `/health` returning
  `{"status":"ok","service":"engram","version":"0.2.0"}`.
- `scripts/render-homebrew-formula.sh` rendered a `v0.2.0` macOS Apple Silicon formula from that
  archive; `ruby -c` reported `Syntax OK`, and targeted search found no beta-specific Homebrew
  wording.
- Hosted CI run `27378308443` for the initial version-bump head failed in `work_tests` because
  CRUD/MCP work-management fixtures used `WorkService::with_defaults`, which tried to retrieve
  `model.onnx` on an empty hosted embedding cache. The follow-up test hardening changes those
  fixtures to `WorkService::new` so non-embedding work tests stay offline-deterministic; explicit
  model-dependent semantic tests remain opt-in/ignored.

The follow-up hosted-CI hardening commit `1eefa11` fixed incidental model-cache dependence in
work-management CRUD/MCP tests. Exact-head hosted CI run `27379891728` passed for that head, and
the full GA release gate passed with local CI, package/install smoke, and release-scope
acknowledgements complete. This makes `1eefa11` ready for release-owner review, not published.

## Pre-Runbook GA Release-Owner Review Checkpoint

Head `1eefa11aff32e4d3802cc327ddc8d8957fd2f56f` is the pre-runbook owner-review candidate.

Evidence collected on this head:

- `gh run view 27379891728 --repo ymeiri/engram --json headSha,status,conclusion,jobs,url`
  confirmed exact-head hosted CI success for Check, Test, Clippy, Docs, and Format.
- `scripts/release-gate-report.sh --target ga --hosted-run 27379891728 --json` reran local CI,
  generated docs, ran package/install smoke, and emitted
  `release_gate_state=hosted_ci_passing_release_owner_review_required`.
- Package smoke produced `dist/engram-0.2.0-aarch64-apple-darwin.tar.gz` with checksum
  `57e404714d3ebb2df3dd4622075748742526c9ead63725e4844cd775ce4b9642`.
- `scripts/render-homebrew-formula.sh` and `ruby -c dist/homebrew/Formula/engram.rb` validated
  a tap-ready formula from that archive.
- `gh release view v0.2.0 --repo ymeiri/engram` still reports `release not found`, and
  `git tag --list 'v*' --sort=version:refname` still stops at `v0.2.0-beta.2`.

`docs/GA_RELEASE_OWNER_APPROVAL_V0_2_0_2026-06-12.md` is the default-deny release-owner approval
runbook for the remaining manual release actions. Because adding that runbook changes `main`, the
release head must still get fresh exact-head hosted CI and a full GA release gate before tag,
publish, or Homebrew tap update.

## Homebrew-Gated GA Release-Owner Review Checkpoint

Head `809426945cb7e0d78950552165691e29aa0191bc` is the documented owner-review
checkpoint with full GA gate evidence including Homebrew formula render validation.

Evidence collected on this head:

- `gh run watch 27388790648 --repo ymeiri/engram --exit-status --interval 30`
  confirmed exact-head hosted CI success for Check, Test, Clippy, Docs, and Format.
  The `Test` job completed in `28m51s`.
- `scripts/release-gate-report.sh --target ga --hosted-run 27388790648 --json`
  reran local CI, generated docs, ran package/install smoke, rendered the Homebrew
  formula, checked it with Ruby, and emitted
  `release_gate_state=hosted_ci_passing_release_owner_review_required`.
- The saved gate JSON assertion required `head=809426945cb7e0d78950552165691e29aa0191bc`,
  `hosted_ci.run.headSha=809426945cb7e0d78950552165691e29aa0191bc`,
  `hosted_ci.state=passing`, `local_ci=passed`, `package_install_smoke=passed`,
  `homebrew_formula_render=passed`, `release_scope.state=complete`, and
  `ready_for_release_owner_review=true`.
- Package smoke produced `dist/engram-0.2.0-aarch64-apple-darwin.tar.gz` with checksum
  `6878ceae0622f98f41e9eb93e3c172e2715861825b242457e7385e60620f0420`.
- `scripts/verify-published-release-install.sh --tag v0.2.0 --asset-dir dist --json`
  passed as a non-publishing local asset-dir verifier with `assets.source=asset_dir`,
  `expected_git_head=809426945cb7e0d78950552165691e29aa0191bc`,
  `install_smoke=passed`, and `release_actions_performed=false`. On the hardened verifier
  contract, local `--asset-dir` evidence reports `asset_install_verified=true` and
  `published_install_verified=false`; only downloaded GitHub release assets can set
  `published_install_verified=true`.
- `git tag --list 'v0.2.0'` returned no tag, and
  `gh release view v0.2.0 --repo ymeiri/engram` still reported `release not found`.

This document refresh changes `main` after that checkpoint. If this refresh commit, or any later
release-facing commit, is included in the actual release head, rerun exact-head hosted CI and the
full GA release gate before tag, publish, or Homebrew tap update.

## GA Branch Guard Hardening

`scripts/release-gate-report.sh --target ga` now defaults the expected branch to `main`, emits
`expected_branch` in text and JSON evidence, and fails before accepting evidence if the current
branch does not match. This closes a release-management gap where a synced non-main branch with an
exact-head hosted run could otherwise look like GA owner-review evidence.

Targeted validation for this guard:

- `bash -n scripts/release-gate-report.sh scripts/beta-release-gate-report.sh`
- `git diff --check`
- `scripts/release-gate-report.sh --target ga --hosted-run 27390228134 --quick
  --allow-tracked-changes --json`, with a JSON assertion requiring `branch=main`,
  `expected_branch=main`, `upstream.name=origin/main`, `upstream.ahead=0`,
  `upstream.behind=0`, exact head `4ad8fdabe9442754ebeb0cef9453e41f3c84d3b4`,
  hosted CI state `passing`, push event, run ID `27390228134`, and no release actions.
- `EXPECTED_BRANCH=release/0.2 scripts/release-gate-report.sh --target ga --hosted-run
  27390228134 --quick --allow-tracked-changes --json`, expected failure with
  `branch mismatch: expected release/0.2, got main`.

This branch-guard slice changes release-facing code and docs. If it becomes the final release head,
rerun exact-head hosted CI and the full GA release gate before tag, publish, or Homebrew tap update.

## Local Disk-Space Release Gate Preflight

`scripts/release-gate-report.sh --target ga` now checks local free space before running local CI or
package/install smoke. Full local owner-review evidence uses the default
`RELEASE_GATE_MIN_FREE_KIB=10485760` threshold (10 GiB) and reports `disk_space.state`,
`disk_space.free_kib`, and `disk_space.min_required_kib` in JSON once the preflight runs. The
override is available for controlled rehearsals, but lowering it weakens local release evidence.

This closes the current local validation failure mode where Cargo can spend substantial time in
`cargo check`, clippy, or tests before surfacing `No space left on device`. On current head
`13c08294a838e19e7497e893bea2bcb301449bdf`, hosted CI run `27393027654` is green, but the full
local GA gate remains pending because the filesystem has less than 1 GiB free while `target/` is
about 99 GiB. Cleanup of generated build artifacts still requires explicit approval before rerunning
the full gate and refreshing GA `dist/` assets.

## Validation Run

- `git fetch --tags --prune origin`
- `git rev-list --left-right --count main...origin/main`
- `gh run view 27363378532 --json status,conclusion,headSha,url,jobs`
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
- `scripts/native-claude-gate-preflight.sh --json --require-ready` (expected exit code `2`)
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
- `scripts/package-release.sh` with tracked development changes present (expected failure)
- `ALLOW_TRACKED_CHANGES=1 DIST_DIR=<temp> scripts/package-install-smoke.sh`
- `scripts/verify-published-release-install.sh --tag v0.2.0 --asset-dir <temp> --json` (expected failure)
- `scripts/verify-published-release-install.sh --tag v0.2.0-beta.2 --expected-prerelease false --json` (expected failure)
- `scripts/verify-published-release-install.sh --tag v0.2.0-beta.2 --expected-git-head
  ec2e263541f149fbbbbe8408d3546f4b183e0d02 --json`
- `ALLOW_TRACKED_CHANGES=1 DIST_DIR=<temp> scripts/package-install-smoke.sh` after the `jq`
  manifest parser change
- `scripts/verify-published-release-install.sh --tag v0.2.0-beta.2 --expected-git-head
  9204cdea38361acb6647ffb4c7b2399590c349f2 --json` (expected tag-commit failure)
- `ALLOW_TRACKED_CHANGES=1 DIST_DIR=<temp> scripts/package-install-smoke.sh` after the
  package-release manifest build guard
- `cargo test -p engram-tests --test multi_session_tests`
- `target/debug/engram setup --agent codex --root /tmp/engram-codex-setup.BWMgtQ --write --yes`
- `target/debug/engram harness status --harness codex --root /tmp/engram-codex-setup.BWMgtQ
  --json`
- `target/debug/engram harness doctor --harness codex --root /tmp/engram-codex-setup.BWMgtQ
  --json`
- `target/debug/engram setup --agent cursor --root /tmp/engram-cursor-setup.ShHnjs --write --yes`
- `target/debug/engram harness status --harness cursor --root /tmp/engram-cursor-setup.ShHnjs
  --json`
- `target/debug/engram harness doctor --harness cursor --root /tmp/engram-cursor-setup.ShHnjs
  --json`
- `rg -n "Beta scope|this beta|beta support|published beta|supported beta|beta setup|Guided beta" README.md docs/MCP_SETUP.md SECURITY.md CONTRIBUTING.md docs/RELEASE_NOTES_V0_2_0.md -S` (expected no output)
- `rg -n "v0\\.2\\.0-beta|0\\.2\\.0-beta" README.md docs/MCP_SETUP.md SECURITY.md CONTRIBUTING.md docs/RELEASE_NOTES_V0_2_0.md -S`
- `bash -n scripts/release-gate-report.sh scripts/beta-release-gate-report.sh`
- `scripts/release-gate-report.sh --target ga --hosted-run 27367795100 --quick
  --allow-tracked-changes --json`
- `scripts/release-gate-report.sh --target ga --quick --allow-tracked-changes --json`
- `scripts/release-gate-report.sh --target ga --hosted-run 27367795100 --quick`
  with tracked development changes present (expected failure)
- `scripts/release-gate-report.sh --target ga --hosted-run 27370049604 --quick
  --allow-tracked-changes --json`
  expected `release_version=0.2.0`, `workspace_version_matches_release=false`,
  `release_gate_state=version_bump_required`, and no `tag_v0.2.0-beta.2`
  remaining action
- `scripts/release-gate-report.sh --target ga --hosted-run 27373857951 --quick
  --allow-tracked-changes --json`
  expected `release_scope.state=complete`, native Claude proof limits acknowledged,
  lifecycle/M6 limits acknowledged, `release_gate_state=version_bump_required`, and no
  `tag_v0.2.0-beta.2` remaining action
- `RELEASE_NOTES_PATH=<temp-empty-file> scripts/release-gate-report.sh --target ga
  --release-version 0.2.0-beta.2 --hosted-run 27373857951 --quick --allow-tracked-changes --json`
  expected `release_scope.state=incomplete`,
  `release_gate_state=release_scope_acknowledgement_required`, and remaining action
  `restore_release_notes_ga_scope_acknowledgements`
- `scripts/native-claude-gate-preflight.sh --json | jq .`
  expected `head=a082a63969df1be1179f38a75a02ee23ff815166`,
  `gate_state=blocked`, vault `2888/2888`, no tracked changes, no extra untracked files, and
  blocker `native Claude CLI processes are already running`
- `scripts/native-claude-gate-preflight.sh --json --require-ready` (expected exit code `2`)
