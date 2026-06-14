# Engram v0.2.0 GA Readiness Matrix

Date: 2026-06-11
Last refreshed: 2026-06-14
Status: GA preparation in progress
Validated setup-path docs checkpoint: `86dd38d0ef56bad5aa0c999578313c7f4a133e41`
Validated release-hardening checkpoint: `eb0e3a96b7a751a90d482dad95ab9ae31af76a7e`
Validated release-code baseline checkpoint: `b650a307793b576b523828a9ca2886fa41058b54`
Validated release-notes docs checkpoint: `c095770f1821c731c01b176a83fe43903618a2f8`
Pre-runbook GA release-owner review head: `1eefa11aff32e4d3802cc327ddc8d8957fd2f56f`
Documented Homebrew-gated GA release-owner review head:
`809426945cb7e0d78950552165691e29aa0191bc`
GA hardening evidence baseline for this refresh:
`4978711b5bc27d350f0d57983698758d331a3f16`
Release-packaging payload-hash guard checkpoint:
`35a5b9ca6ebb790ed8987ac6425d29e3e2e6e402`
Release-gate disk threshold override guard checkpoint:
`6ba403efce7dc0893a1fd40aa6fddc39fbaa6fe5`
Release package output overwrite guard checkpoint:
`2b34e50b556f1e595368775ca160e6d77dd3ddd3`
Generated-artifact proof-state checkpoint:
`d0454b9c1ea0aeaa6ed0294d81017ea926bc92cb`
Latest recorded release-gate behavior evidence checkpoint:
`3a95713895e0ee90675d61a03659061cc3a244cc`

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
- Historical Homebrew-gated GA release-owner review CI: main push run `27388790648`
  for `809426945cb7e0d78950552165691e29aa0191bc` completed successfully on
  2026-06-12 for Check, Test, Clippy, Docs, and Format. The `Test` job ran
  `cargo test --locked --all-targets --jobs 1` and completed in `28m51s`.
- Current release-gate behavior CI: main push run `27507336718` for
  `fd4c61ce689dd46a2f6172b7fcba153e471a684e` completed successfully on
  2026-06-14 for Check, Test, Clippy, Docs, and Format. The post-CI GA gate
  assertions passed for the quick `evidence_incomplete` state, the default
  `generated_outputs_cleanup_required` full-gate stop, the read-only
  `generated_output_cleanup_fingerprints_verified` cleanup verifier state, and the exact
  three-path cleanup allowlist for the archive, checksum, and Homebrew formula.
- Historical full GA release gate: `scripts/release-gate-report.sh --target ga --hosted-run 27388790648
  --json` passed with `local_ci=passed`, `package_install_smoke=passed`,
  `homebrew_formula_render=passed`, `release_scope.state=complete`,
  `release_gate_state=hosted_ci_passing_release_owner_review_required`, and
  `ready_for_release_owner_review=true`.
- Historical local asset-dir release verifier:
  `scripts/verify-published-release-install.sh --tag v0.2.0 --asset-dir dist --json`
  passed with `assets.source=asset_dir`,
  `expected_git_head=809426945cb7e0d78950552165691e29aa0191bc`, `install_smoke=passed`,
  and `release_actions_performed=false`. Newer verifier output distinguishes this local
  rehearsal from published-release proof with `asset_install_verified=true` and
  `published_install_verified=false` when `assets.downloaded=false`.
- GA hardening baseline CI: main push run `27401954970` for
  `4978711b5bc27d350f0d57983698758d331a3f16` completed successfully on
  2026-06-12 for Check, Test, Clippy, Docs, and Format. The `Test` job restored
  and warmed `engram-tests/.fastembed_cache` before running
  `cargo test --locked --all-targets --jobs 1`.
- GA hardening baseline quick gate: `ALLOW_RELEASE_GATE_MIN_FREE_OVERRIDE=1
  RELEASE_GATE_MIN_FREE_KIB=1 scripts/release-gate-report.sh --target ga --hosted-run
  27401954970 --quick --json`
  passed for exact head `4978711`, with hosted CI passing, local CI/package smoke,
  Homebrew formula render, and disk checks intentionally skipped, and
  `release_actions_performed=false`.
- Release-packaging payload-hash guard CI: main push run `27417397670` for
  `35a5b9ca6ebb790ed8987ac6425d29e3e2e6e402` completed successfully on
  2026-06-12 for Check, Test, Clippy, Docs, and Format. The `Test` job ran for
  `28m7s`.
- Release-packaging payload-hash quick gate:
  `scripts/release-gate-report.sh --target ga --hosted-run 27417397670 --quick --json`
  passed for exact head `35a5b9c` with hosted CI passing, release target
  `v0.2.0` still available, no local/remote tag, no GitHub release, no owner-review
  readiness, and no release actions.
- Release-gate disk threshold override guard CI: main push run `27425319893` for
  `6ba403efce7dc0893a1fd40aa6fddc39fbaa6fe5` completed successfully on
  2026-06-12 for Check, Test, Clippy, Docs, and Format.
- Release-gate disk threshold override quick gate:
  `scripts/release-gate-report.sh --target ga --hosted-run 27425319893 --quick --json`
  passed for exact head `6ba403e` with hosted CI passing, release target
  `v0.2.0` still available, no local/remote tag, no GitHub release,
  `disk_space.min_required_kib=10485760`, no owner-review readiness, and no release actions.
- Generated-artifact proof-state CI: main push run `27475243716` for
  `d0454b9c1ea0aeaa6ed0294d81017ea926bc92cb` completed successfully on
  2026-06-13 for Check, Test, Clippy, Docs, and Format. The `Test` job restored
  and warmed `engram-tests/.fastembed_cache` before running the serialized
  `cargo test --locked --all-targets --jobs 1` step.
- Generated-artifact readiness CI: main push run `27478418090` for
  `b970c07db3847571665eba878a90cd8f63c61a3a` completed successfully on
  2026-06-13 for Check, Test, Clippy, Docs, and Format. The `Test` job restored
  and warmed `engram-tests/.fastembed_cache` before running
  `cargo test --locked --all-targets --jobs 1`.
- Release-gate configuration-preflight evidence CI: main push run
  `27485035688` for `a2e18d99d7fb20ac9af55fc7bbea71fe5c1327e6` completed successfully
  on 2026-06-14 for Check, Test, Clippy, Docs, and Format. The `Test` job restored and
  warmed `engram-tests/.fastembed_cache` before running
  `cargo test --locked --all-targets --jobs 1`.
- Prior release-gate behavior evidence CI: main push run
  `27491186766` for `1ddc317df74bfe9a67640107b0dca0708aaeb9f1` completed successfully
  on 2026-06-14 for Check, Test, Clippy, Docs, and Format. The `Test` job restored and
  warmed `engram-tests/.fastembed_cache` before running
  `cargo test --locked --all-targets --jobs 1`.
- Scoped observation-search fix CI: main push run `27499686731` for
  `d4c0cd9454c304471ee127ae230c112bb4fdf89f` completed successfully on
  2026-06-14 for Check, Test, Clippy, Docs, and Format. The local focused and full
  entity-test validation included `cargo test -p engram-tests --test entity_tests`,
  and the live MCP `entity_observe(action="search", entity="live-debugger-mcp", ...)`
  regression query returned `count=5` after the installed daemon refresh. This fixes
  the prior false negative for long natural-language scoped observation searches; it
  does not change the release-owner cleanup gate.
- Cleanup path allowlist hardening CI: main push run `27501370037` for
  `3a95713895e0ee90675d61a03659061cc3a244cc` completed successfully on
  2026-06-14 for Check, Test, Clippy, Docs, and Format. The runbook now requires the
  verifier's `will_write=true` paths to diff exactly against
  `dist/engram-0.2.0-aarch64-apple-darwin.tar.gz`,
  `dist/engram-0.2.0-aarch64-apple-darwin.tar.gz.sha256`, and
  `dist/homebrew/Formula/engram.rb`, and the deletion loop uses the same case-sensitive
  allowlist plus `test -f` before removing anything.
- Live-debugger scoped-search regression CI: main push run `27507336718` for
  `fd4c61ce689dd46a2f6172b7fcba153e471a684e` completed successfully on
  2026-06-14 for Check, Test, Clippy, Docs, and Format. Commit `fd4c61c` added exact
  MCP regression coverage for the user-reported Claude Code zero-result query against
  `entity_observe(action="search", entity="live-debugger-mcp", ...)`. A fresh live MCP
  query for `debug with ai how skill is loaded code-gen-backend global_skills vs MCP
  load_datadog_skill catalogue` returned results on the current daemon. A post-fetch
  branch check for this checkpoint had `HEAD...origin/main` at `0 0`, with `HEAD`,
  `origin/main`, and remote `refs/heads/main` all at `fd4c61c`.
- Prior quick GA gate:
  `scripts/release-gate-report.sh --target ga --hosted-run 27475243716 --quick --json`
  passed for exact head `d0454b9`, with hosted CI passing, release target `v0.2.0`
  still available, `generated_outputs.state=cleanup_required` as read-only partial
  evidence, all generated outputs reporting `will_write=false`, and
  `release_actions_performed=false`.
- Generated-artifact readiness quick GA gate:
  `scripts/release-gate-report.sh --target ga --hosted-run 27478418090 --quick
  --allow-tracked-changes --json` passed for exact head `b970c07`, with hosted CI
  passing, release target `v0.2.0` still available, generated outputs reported as
  read-only cleanup evidence with regular-file size and SHA-256 fingerprints, and
  `release_actions_performed=false`.
- Current cleanup-approval quick GA gate:
  `scripts/release-gate-report.sh --target ga --hosted-run 27507336718 --quick --json`
  passed for exact head `fd4c61c`, with hosted CI
  passing, release target `v0.2.0` still available, generated outputs reported as
  read-only cleanup evidence with regular-file size and SHA-256 fingerprints, and
  `release_actions_performed=false`.
- Prior generated-output, forced-disk, and release-target failure rehearsals against
  run `27475243716` all keep `release_actions_performed=false` and report
  `generated_artifacts.state=not_checked`, `generated_artifacts.host_triple=null`, and
  an empty post-run artifact list. This proves that preflight failures do not claim
  archive, checksum, or Homebrew formula publication evidence.
- Current post-CI full-gate rehearsal against run `27507336718` keeps
  `release_actions_performed=false` and reports `generated_artifacts.state=not_checked`
  on the generated-output preflight failure.
- The latest recorded `v0.2.0-beta.2` release-target conflict rehearsal against run
  `27480555463` keeps `release_actions_performed=false` and reports
  `ready_for_release_owner_review=false`.
- The latest exact-head default GA gate on this host passes disk preflight with
  `disk_space.state=passed` and `min_required_kib=10485760`, then fails before local
  CI/package smoke on head `fd4c61c` with
  `release_gate_state=generated_outputs_cleanup_required`,
  `failure.kind=generated_outputs_preflight`, and `release_target.state=available`.
  Exact `free_space_kib` values are host-local and should be read from the final gate JSON.
- Current generated release evidence in `dist/` includes stale `v0.2.0` archive/checksum assets
  and a rendered Homebrew formula. Local package and Homebrew render scripts now refuse to
  overwrite those outputs unless the corresponding local-rehearsal overwrite flag is explicit.
  The GA release gate now reports those paths under `generated_outputs` so final owner-review
  evidence can require `generated_outputs.state=clear` before claiming local package/Homebrew
  proof. Preflight failure JSON also reports `generated_artifacts.state=not_checked` so consumers
  do not mistake a stopped full gate for post-run artifact publication evidence.
- When disk headroom is forced through for a non-final local rehearsal, the GA release gate now
  fails closed at `release_gate_state=generated_outputs_cleanup_required` if any generated output
  that the full gate would write already exists. This keeps stale `dist/` evidence from reaching
  local CI, package smoke, or Homebrew formula validation.
- Local runtime before the latest refresh: installed `engram` and daemon reported
  `0.2.0-beta.2`.
- Local runtime after the latest refresh: installed binary hash
  `df9b8a72f9a7c5775bab0429dbc27270c13c230bdfb95529d11bb35c9868e17c`
  reports `engram 0.2.0`; daemon restarted on port `8765` with spawn version
  `0.2.0`; `/health` returns `{"status":"ok","service":"engram","version":"0.2.0"}`.

## Matrix

| Area | Status | Evidence | GA Gap / Next Action |
| --- | --- | --- | --- |
| GA target | Validated | Current prerelease line is `0.2.0-beta.2`; no `v0.2.0` tag/release exists. | Keep GA target as `v0.2.0` unless a later release decision changes it. |
| Beta baseline | Validated | Local tags and GitHub prereleases exist for beta.1 and beta.2 with release assets. | Use beta.2 plus current `main` as the GA baseline. |
| Versioning | Validated historically / fresh full local gate pending | Workspace metadata and lockfile are consistent at the intended `0.2.0` GA version on head `8094269`; full GA gate confirmed `workspace_version_matches_release=true`. The latest recorded `fd4c61c` release-gate behavior evidence checkpoint has exact-head hosted CI and quick-gate evidence, but not a fresh full local gate because generated-output cleanup is still approval-gated before local CI/package smoke. | Obtain release-owner approval for generated-output cleanup, preserve disk cleanup approval as conditional if the preflight regresses, then rerun exact-head CI if needed and the full GA gate on the final release head. |
| Hosted CI | Validated on latest recorded release-gate evidence checkpoint | Main push CI run `27507336718` passed for head `fd4c61c`. Earlier runs `27506588055`, `27501370037`, `27499686731`, `27497957513`, `27497239164`, `27491186766`, `27489912065`, `27485035688`, `27484385982`, `27483700305`, `27483057249`, `27482421136`, `27481797605`, `27480555463`, `27479860353`, `27479117658`, `27478418090`, `27475243716`, `27474449994`, `27473620301`, `27472826808`, `27472166352`, `27425319893`, `27417397670`, `27401954970`, `27399073606`, `27388790648`, `27379891728`, `27372009309`, `27363378532`, `27335890558`, and `27340971819` passed for live-debugger scoped-search regression, remote-head freshness guard, cleanup path allowlist hardening, scoped observation-search fix, cleanup manifest fingerprinting, cleanup manifest evidence, hosted-verifier config JSON, published-verifier config JSON, configuration-failure JSON, hosted-CI failure JSON, branch-sync, cleanup-runbook, artifact-proof, runbook, output-guard, disk, package, owner-review, setup-path, release-code, and release-notes checkpoints. | Re-run exact-head hosted CI after any GA version/docs/package changes. |
| Local runtime | Source-installed 0.2.0 validated / published install pending | Full GA gate package/install smoke passed for `engram 0.2.0` on `8094269`; local asset-dir verification also passed without downloads or publishing. After the scoped-search fix, `/Users/yuval.meiri/.local/bin/engram` was source-installed from current main, reports `engram 0.2.0`, and the restarted daemon reports spawn version `0.2.0`; `/health` returns `{"status":"ok","service":"engram","version":"0.2.0"}`. This is local source-install evidence, not published-release evidence. | After release publication, verify the published install path and then refresh local runtime evidence from the published asset. |
| Scoped observation search | Validated | Commit `d4c0cd9` fixed `entity_observe(action="search")` so long natural-language queries rank relevant scoped observations by key/content term overlap instead of requiring a full contiguous substring. Commit `fd4c61c` added exact MCP regression coverage for the literal `live-debugger-mcp` Claude Code query involving `load_datadog_skill` and `global_skills`. Local `entity_tests`, exact-head CI runs `27499686731` and `27507336718`, and fresh live MCP queries returning results cover the prior false negative. | No remaining GA blocker from this bug; keep broad unified search and scoped observation search behavior distinct in future regressions. |
| `orient` hot path | Validated / preserve | Lean `orient` returned compact scope, cursor, Brain Loop guidance, candidate IDs, and no open obligations. | Do not expand `orient`; only add focused regressions if GA changes touch ranking or lifecycle. |
| Memory obligations | Validated | `engram obligations doctor --scope-project engram --cwd ...` returned `open=[]`, `warnings=[]`. | Re-run after every meaningful GA commit. |
| Generated vault | Validated | The generated vault status for this evidence checkpoint reports `generated_file_count=3144`, `expected_generated_file_count=3144`, `user_file_count=0`. | Re-run before final GA release if memory writes occur. |
| Native Claude production gate | Blocked | The native preflight baseline now matches the installed Claude Code `2.1.174` path/hash, default-denies non-canonical branch/binary/vault overrides, and the generated vault was regenerated to `3134/3134`. On exact head `cada64a`, the strict preflight reports no tracked changes but blocks on the local untracked `.engram-vault/` copy plus an already-running native Claude CLI process. The scoped preflight with worktree changes explicitly allowed isolates the production gate and reports the only remaining native-Claude blocker as an already-running native Claude CLI process. The preflight now emits `configuration_preflight_failed` JSON for early `--json` operator/configuration failures without launching Claude, running `/hooks`, signaling processes, or taking release actions. | Do not claim native Claude prompt-bearing, `/hooks`, or live host-label proof until a clean process window and clean release-evidence worktree allow the proof run. |
| Claude static harness readiness | Partially validated | `engram harness doctor --harness claude-code --json` reports `ready=true` with warnings about user-owned snippet, extra permissions, split settings, and unproved live hook visibility. | Resolve or explicitly scope warnings before GA claims depend on live Claude hook behavior. |
| Codex setup/runtime path | Validated for generated adapter install and current MCP use | `engram setup --agent codex --root <temp> --write --yes` wrote the two required Codex skills plus `AGENTS.engram.md`; `engram harness status/doctor --harness codex --root <temp> --json` reported required adapters installed and `ready=true`. Current Codex session also used MCP `orient` successfully. | Repeat on the final GA versioned head; live lifecycle compliance remains advisory and host-driven. |
| Cursor setup/runtime path | Validated for generated adapter install | `engram setup --agent cursor --root <temp> --write --yes` wrote the three required Cursor skills; `engram harness status/doctor --harness cursor --root <temp> --json` reported required adapters installed and `ready=true`. | Repeat on the final GA versioned head; no live Cursor host session has been claimed. |
| Release notes and changelog | Drafted / needs final validation | `docs/RELEASE_NOTES_V0_2_0.md` now exists with install, upgrade, first-run, and known-limitation text; changelog still has only Unreleased entries. | Review and finalize the notes on the versioned GA head, then promote changelog entries only after GA scope is fixed. |
| Package artifacts | Validated historically / unpublished | Beta.2 GitHub release has archive and checksum assets; full GA package-install smoke passed for `engram-0.2.0-aarch64-apple-darwin.tar.gz` plus checksum on clean head `8094269`. Local release packaging still fails closed on tracked changes by default, refuses stale archive/checksum overwrites, stages new archive/checksum outputs as temporary files, and checksum-verifies them before moving them into final `dist/` paths. The install smoke rejects checksum files that do not name exactly the expected archive. Later hardening heads have not refreshed `dist/` because generated-output cleanup is still approval-gated; with disk sufficient, the gate now fails before local package proof if stale generated outputs would be written. The GA release gate separates preflight `generated_outputs` cleanup evidence from post-run `generated_artifacts` publication evidence, failure JSON reports artifact proof as `not_checked` until local proof runs, successful post-run artifact proof fingerprints existing regular artifacts with `file_type`, `size_bytes`, and `sha256`, and owner-review readiness is blocked if required post-run artifacts are missing, non-regular, empty, or unfingerprinted. | Publish and verify assets only after owner approval, generated-output cleanup, disk cleanup if required, and a fresh full gate on the release head; require `generated_artifacts.state=present` with regular-file fingerprints and reuse the archive/checksum produced by that successful full gate instead of regenerating into guarded output paths. |
| Homebrew | Validated historically / unpublished | The full GA gate for `8094269` rendered `dist/homebrew/Formula/engram.rb`, `ruby -c` reported `Syntax OK`, and the gate rejected beta-specific Homebrew wording. The renderer also requires the adjacent `.sha256` asset to name and hash the same archive, verifies packaged `MANIFEST.json` release identity, rejects archive members outside the expected root, checks packaged payload hashes before writing formula text, refuses to overwrite existing formula output unless explicitly allowed for local rehearsals, and now stages formula text in a temporary file with Ruby syntax validation before moving it into the final output path. The GA release-owner runbook audits the rendered formula before committing the tap update, installs or upgrades the fully-qualified `ymeiri/engram/engram` formula, runs `brew test ymeiri/engram/engram`, and checks the formula prefix binary version directly instead of relying on `PATH`. The remote tap `ymeiri/homebrew-engram` still points at beta.2 until explicitly updated. | Update the tap only after release approval, fresh package evidence, generated-output cleanup, published asset verification, formula audit, qualified tap install/test proof, and direct formula-binary version proof. |
| Docs consistency | Partially hardened | README, MCP setup, and security policy now use a `0.2.x` support-scope framing for supported setup paths while preserving the current fact that `v0.2.0-beta.2` is the latest published artifact. Historical docs still contain beta-specific caveats by design. | Re-check release-facing docs after the final `0.2.0` version bump and artifact publication; do not rewrite historical T-doc evidence. |
| Memory lifecycle / M6 | Scoped for GA / final validation required | Legacy layers remain supported substrate; broad lifecycle cleanup and unrestricted automated lifecycle mutation are not proven GA-complete and are explicitly outside the current `v0.2.0` release claims. `scripts/release-gate-report.sh --target ga` now checks that the GA release notes retain those scope acknowledgements. | Keep the release-notes scope acknowledgements through the final version bump and full GA gate; do not broaden lifecycle/M6 claims without fresh implementation and validation evidence. |
| Git release mechanics | Runbook prepared / not published | No `v0.2.0` tag, release, or package publication exists. `scripts/release-gate-report.sh --target ga --hosted-run 27388790648 --json` passed on the historical Homebrew-gated owner-review head and reported `ready_for_release_owner_review=true`. The exact-head post-CI full gate for `fd4c61c` reports `release_target.state=available`, `disk_space.state=passed`, `release_gate_state=generated_outputs_cleanup_required`, and `generated_artifacts.state=not_checked` before local CI/package/Homebrew proof because stale generated outputs still exist; cleanup manifest verification reported `generated_output_cleanup_fingerprints_verified` with manifest SHA-256 `1081aabd3f5f08366445ee3e74f88ee5814a258de0792c9019187c64b93ee98d`. `docs/GA_RELEASE_OWNER_APPROVAL_V0_2_0_2026-06-12.md` names the fail-closed post-approval command sequence, verifies the exact stale-output fingerprints before cleanup, warns against using `git pull` to satisfy branch-sync checks, requires the cleanup paths to match the exact expected archive/checksum/formula allowlist before deletion, requires a fresh full gate on the release head, reuses the full-gate archive/checksum/formula artifacts for publication instead of rerunning guarded generators, and passes `--verify-tag` to `gh release create` so release publication aborts if the pushed signed tag is missing instead of auto-creating a tag from the default branch. The GA gate now defaults `expected_branch=main` so owner-review evidence cannot be collected from a synced non-main branch unless explicitly overridden, reports `release_target.state=available` only when the intended local tag, remote Git tag, and GitHub release are all absent, fails with `generated_outputs_cleanup_required` if stale generated archive/checksum/formula outputs would be written by final proof, supports read-only `--verify-generated-output-cleanup <gate-json>` fingerprint verification before any owner-approved deletion with structured mismatch JSON and full-gate manifest validation including exact-head hosted-CI proof, owner-decision fields, expected remaining actions, and required all-false release-action keys, emits the validated manifest's hosted-CI/release-target/disk/no-action summary as `generated_output_cleanup_verification.manifest_evidence`, fingerprints that source manifest with `manifest_size_bytes` and `manifest_sha256`, emits structured `configuration_preflight_failed` JSON for early operator/configuration failures, and reports post-run `generated_artifacts` so owner-review JSON proves the artifacts to publish exist after the full gate with regular-file size and SHA-256 fingerprints. `engram-tests/tests/release_gate_script_tests.rs` now covers ahead-of-upstream and stale-remote-head GA branch-sync failures in temporary git repositories and asserts `branch_sync_required`, no release actions, authoritative remote-head mismatch evidence, and the stop-and-inspect `git pull` warning. | Tag, publish, update Homebrew, and verify only after explicit release-owner approval, generated-output cleanup, disk cleanup if required, fresh exact-head gate evidence, generated artifact fingerprint evidence, and release-target availability evidence. |

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

`scripts/package-release.sh` now also validates `ALLOW_TRACKED_CHANGES` itself as exactly `0` or
`1` before release binary builds or artifact writes. This keeps the dirty-state escape hatch
consistent with the release-gate and Homebrew override guards: typos fail closed instead of being
silently treated as some third mode.

`scripts/package-release.sh` now also fails closed if `DIST_DIR` points anywhere other than the
repository `dist` directory unless `ALLOW_PACKAGE_DIST_DIR_OVERRIDE=1` is set for an explicit local
rehearsal. The dist path must be non-empty. This keeps final package/checksum evidence from
silently landing in an ambient temp directory or old release-asset checkout. Package install smoke
still supports temp package rehearsals by passing the producer approval flag only when it is
building into a non-default `DIST_DIR`.

Targeted validation for this override guard on a development diff:

- `bash -n scripts/package-release.sh scripts/package-install-smoke.sh
  scripts/release-gate-report.sh scripts/beta-release-gate-report.sh`
- `DIST_DIR=/tmp/engram-package-dist-test scripts/package-release.sh` failed before release binary
  builds or artifact writes with `DIST_DIR override requires explicit package approval`.
- `ALLOW_PACKAGE_DIST_DIR_OVERRIDE=yes DIST_DIR=/tmp/engram-package-dist-test
  scripts/package-release.sh` failed with
  `ALLOW_PACKAGE_DIST_DIR_OVERRIDE must be 0 or 1, got yes`.
- `DIST_DIR= scripts/package-release.sh` failed with `DIST_DIR must not be empty`.
- `ALLOW_TRACKED_CHANGES=yes scripts/package-release.sh` failed with
  `ALLOW_TRACKED_CHANGES must be 0 or 1, got yes`.
- `ALLOW_PACKAGE_DIST_DIR_OVERRIDE=1 DIST_DIR=/tmp/engram-default-dirty-test
  scripts/package-release.sh` still failed with the
  tracked-change guard and no release artifact write.
- `ALLOW_TRACKED_CHANGES=1 ALLOW_PACKAGE_DIST_DIR_OVERRIDE=1
  DIST_DIR=/tmp/engram-allow-tracked-package-test
  scripts/package-release.sh` built a local rehearsal archive.
- `ALLOW_PACKAGE_BUILD_SKIP=1 SKIP_PACKAGE_BUILD=1
  DIST_DIR=/tmp/engram-allow-tracked-package-test
  EXPECTED_TRACKED_CHANGES_PRESENT=true scripts/package-install-smoke.sh` verified the archive,
  installed `engram 0.2.0`, and checked packaged HTTP `/health`.
- `ALLOW_TRACKED_CHANGES=1 DIST_DIR=<temp> scripts/package-install-smoke.sh` also built and
  verified a temp package, proving the smoke harness preserves intentional temp-package rehearsals
  without requiring direct producer runs to accept ambient `DIST_DIR` overrides.

## Package Install Smoke DIST_DIR Guard

`scripts/package-install-smoke.sh` now also guards the consumer side of package asset selection:
if `DIST_DIR` points outside the repository `dist` directory, the smoke requires
`ALLOW_PACKAGE_DIST_DIR_OVERRIDE=1`, and an explicit empty `DIST_DIR` fails instead of silently
falling back to the default. This keeps final package/install evidence from consuming assets out
of an ambient temp directory or stale release checkout unless the run is explicitly marked as a
local rehearsal.

`scripts/verify-published-release-install.sh` passes `ALLOW_PACKAGE_DIST_DIR_OVERRIDE=1` and
`ALLOW_PACKAGE_BUILD_SKIP=1` only for its internally managed asset directory, after it has either
downloaded the exact GitHub release assets or accepted an explicit `--asset-dir` rehearsal. The
normal GA release gate still runs package install smoke against the default repository `dist`
directory.

Targeted validation for this smoke `DIST_DIR` guard on a development diff:

- `bash -n scripts/package-install-smoke.sh scripts/verify-published-release-install.sh`
- `DIST_DIR=/tmp/engram-smoke-dist-test SKIP_PACKAGE_BUILD=1
  scripts/package-install-smoke.sh` failed before release asset reads with
  `DIST_DIR override requires explicit package approval`.
- `DIST_DIR= SKIP_PACKAGE_BUILD=1 scripts/package-install-smoke.sh` failed before release asset
  reads with `DIST_DIR must not be empty`.
- `ALLOW_PACKAGE_DIST_DIR_OVERRIDE=yes DIST_DIR=/tmp/engram-smoke-dist-test
  SKIP_PACKAGE_BUILD=1 scripts/package-install-smoke.sh` failed with
  `ALLOW_PACKAGE_DIST_DIR_OVERRIDE must be 0 or 1, got yes`.
- `ALLOW_PACKAGE_DIST_DIR_OVERRIDE=1 ALLOW_PACKAGE_BUILD_SKIP=1
  DIST_DIR=/tmp/engram-smoke-dist-test
  SKIP_PACKAGE_BUILD=1 scripts/package-install-smoke.sh` passed the approval guard and then
  failed at the expected missing-tarball check.

## Package Install Smoke Build-Skip Guard

`scripts/package-install-smoke.sh` now requires `ALLOW_PACKAGE_BUILD_SKIP=1` whenever
`SKIP_PACKAGE_BUILD=1` is used. This keeps final package/install evidence from accidentally
validating stale assets already present in `dist/` instead of rebuilding the release archive for
the current head. The published-release verifier still sets the approval because validating
downloaded GitHub assets or an explicit local `--asset-dir` is its intended path.

Targeted validation for this build-skip guard on a development diff:

- `bash -n scripts/package-install-smoke.sh scripts/verify-published-release-install.sh`
- `SKIP_PACKAGE_BUILD=1 scripts/package-install-smoke.sh` failed before release asset reads with
  `SKIP_PACKAGE_BUILD=1 requires explicit package build-skip approval`.
- `ALLOW_PACKAGE_BUILD_SKIP=yes SKIP_PACKAGE_BUILD=1 scripts/package-install-smoke.sh` failed with
  `ALLOW_PACKAGE_BUILD_SKIP must be 0 or 1, got yes`.
- `ALLOW_PACKAGE_BUILD_SKIP=1 SKIP_PACKAGE_BUILD=1 scripts/package-install-smoke.sh` passed the
  build-skip approval guard and then proceeded to existing release-asset validation.

`scripts/package-install-smoke.sh` now also validates its own release-rehearsal overrides before
package extraction or packaged server startup. `SKIP_PACKAGE_BUILD` must be exactly `0` or `1`,
and an explicit `EXPECTED_TRACKED_CHANGES_PRESENT` value must be non-empty and exactly `true` or
`false`.
This keeps post-publish verifier and local `--asset-dir` rehearsals from silently accepting typoed
override intent.

Targeted validation for this smoke override guard on a development diff:

- `bash -n scripts/package-install-smoke.sh scripts/package-release.sh
  scripts/release-gate-report.sh scripts/beta-release-gate-report.sh`
- `SKIP_PACKAGE_BUILD=yes DIST_DIR=/tmp/engram-smoke-override-test
  scripts/package-install-smoke.sh` failed with
  `SKIP_PACKAGE_BUILD must be 0 or 1, got yes`.
- `ALLOW_PACKAGE_BUILD_SKIP=1 SKIP_PACKAGE_BUILD=1
  DIST_DIR=/tmp/engram-smoke-override-test
  EXPECTED_TRACKED_CHANGES_PRESENT=maybe scripts/package-install-smoke.sh` failed with
  `EXPECTED_TRACKED_CHANGES_PRESENT must be true or false, got maybe`.
- `ALLOW_PACKAGE_BUILD_SKIP=1 SKIP_PACKAGE_BUILD=1
  DIST_DIR=/tmp/engram-smoke-override-test
  EXPECTED_TRACKED_CHANGES_PRESENT= scripts/package-install-smoke.sh` failed before package
  extraction with `EXPECTED_TRACKED_CHANGES_PRESENT must not be empty`.
- `ALLOW_PACKAGE_BUILD_SKIP=1 SKIP_PACKAGE_BUILD=1
  DIST_DIR=/tmp/engram-smoke-override-test
  EXPECTED_TRACKED_CHANGES_PRESENT=true scripts/package-install-smoke.sh` still verified the
  local rehearsal archive, installed `engram 0.2.0`, and checked packaged HTTP `/health`.

`scripts/package-install-smoke.sh` now also validates explicit `SMOKE_PORT` overrides before
package extraction or packaged server startup. The value must be a numeric TCP port in
`1..65535`; otherwise the smoke fails closed before it can produce ambiguous server or health-check
evidence. If the requested port is already in use on `127.0.0.1`, the smoke also fails closed
before release asset reads, avoiding ambiguous `/health` evidence from a pre-existing local
service. Automatic port selection remains the default when `SMOKE_PORT` is unset.
An explicitly empty `SMOKE_PORT=` now fails before package extraction instead of being treated as
unset.

Targeted validation for this smoke port override guard on a development diff:

- `bash -n scripts/package-install-smoke.sh scripts/package-release.sh
  scripts/release-gate-report.sh scripts/beta-release-gate-report.sh`
- `ALLOW_PACKAGE_BUILD_SKIP=1 SMOKE_PORT=abc SKIP_PACKAGE_BUILD=1
  DIST_DIR=/tmp/engram-smoke-port-test
  scripts/package-install-smoke.sh` failed with
  `SMOKE_PORT must be a numeric TCP port, got abc`.
- `ALLOW_PACKAGE_BUILD_SKIP=1 SMOKE_PORT= SKIP_PACKAGE_BUILD=1
  scripts/package-install-smoke.sh` failed with
  `SMOKE_PORT must not be empty`.
- `ALLOW_PACKAGE_BUILD_SKIP=1 SMOKE_PORT=70000 SKIP_PACKAGE_BUILD=1
  DIST_DIR=/tmp/engram-smoke-port-test
  scripts/package-install-smoke.sh` failed with
  `SMOKE_PORT must be between 1 and 65535, got 70000`.
- `ALLOW_PACKAGE_BUILD_SKIP=1 SMOKE_PORT=8766 SKIP_PACKAGE_BUILD=1
  DIST_DIR=/tmp/engram-smoke-port-test
  scripts/package-install-smoke.sh` while a temporary local server occupied the port failed before
  release asset reads with `SMOKE_PORT is already in use on 127.0.0.1: 8766`.
- `ALLOW_PACKAGE_BUILD_SKIP=1 SMOKE_PORT=8766 SKIP_PACKAGE_BUILD=1
  DIST_DIR=/tmp/engram-smoke-port-test
  EXPECTED_TRACKED_CHANGES_PRESENT=true scripts/package-install-smoke.sh` still verified the
  local rehearsal archive, installed `engram 0.2.0`, and checked packaged HTTP `/health`.

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

The published-release path also validates GitHub release asset metadata before downloading. The
release must expose exactly the expected archive and checksum asset names, both assets must be in
the uploaded state with nonzero size and `sha256:` digests, and the verifier checks the downloaded
bytes against those GitHub asset digests before running package/install smoke. JSON evidence reports
`assets.release_asset_list_verified` and `assets.release_asset_digests_verified`.

The published-release path also verifies the release tag itself before asset download. The local
tag must pass `git tag -v`, the remote Git tag object in `ymeiri/engram` must match the local
signed tag object, and the remote peeled tag commit must match the expected release head. JSON
evidence reports `tag_object`, `local_tag_signature_verified`, and
`remote_tag.{object,commit,verified}`.

The published-release verifier now also validates the expected release head before release metadata
or asset checks. `--expected-git-head` must be a 40-character Git SHA, which keeps post-publish and
local asset-verification rehearsals from treating malformed operator-supplied release-head
expectations as meaningful evidence.

## Package Manifest Verification Guard

`scripts/package-install-smoke.sh` now parses packaged `MANIFEST.json` with `jq` instead of
line-oriented text matching. The install smoke checks release metadata, boolean dirty-state
provenance, Cargo.lock hash provenance, and each packaged file hash through structured JSON queries.

This keeps local package rehearsals and published-release install verification from accepting
malformed or misleading manifest JSON during the final `v0.2.0` artifact proof.

`scripts/package-install-smoke.sh` and `scripts/render-homebrew-formula.sh` now also validate
explicit package identity expectation overrides before package identity evidence is collected.
`EXPECTED_PACKAGE_GIT_HEAD` must be a 40-character Git SHA, and
`EXPECTED_CARGO_LOCK_SHA256` must be a 64-character SHA-256 hex value. This keeps local package
rehearsals, published-release install verification, and Homebrew formula rendering from treating
malformed operator-supplied expectations as meaningful release evidence.

Targeted validation for this package identity expectation guard on a development diff:

- `bash -n scripts/package-install-smoke.sh scripts/package-release.sh
  scripts/render-homebrew-formula.sh scripts/release-gate-report.sh
  scripts/beta-release-gate-report.sh`
- `ALLOW_PACKAGE_BUILD_SKIP=1 EXPECTED_PACKAGE_GIT_HEAD=abc SKIP_PACKAGE_BUILD=1
  DIST_DIR=/tmp/engram-identity-override-test scripts/package-install-smoke.sh` failed before
  release asset reads with `EXPECTED_PACKAGE_GIT_HEAD must be a 40-character Git SHA, got abc`.
- `ALLOW_PACKAGE_BUILD_SKIP=1 EXPECTED_CARGO_LOCK_SHA256=abc SKIP_PACKAGE_BUILD=1
  DIST_DIR=/tmp/engram-identity-override-test scripts/package-install-smoke.sh` failed before
  release asset reads with `EXPECTED_CARGO_LOCK_SHA256 must be a SHA-256 hex value, got abc`.
- `ALLOW_HOMEBREW_DIST_DIR_OVERRIDE=1 EXPECTED_PACKAGE_GIT_HEAD=abc
  DIST_DIR=/tmp/engram-no-assets-test
  scripts/render-homebrew-formula.sh` failed before archive reads with
  `EXPECTED_PACKAGE_GIT_HEAD must be a 40-character Git SHA, got abc`.
- `ALLOW_HOMEBREW_DIST_DIR_OVERRIDE=1 EXPECTED_CARGO_LOCK_SHA256=abc
  DIST_DIR=/tmp/engram-no-assets-test
  scripts/render-homebrew-formula.sh` failed before archive reads with
  `EXPECTED_CARGO_LOCK_SHA256 must be a SHA-256 hex value, got abc`.
- `ALLOW_TRACKED_CHANGES=1 ALLOW_PACKAGE_DIST_DIR_OVERRIDE=1
  DIST_DIR=/tmp/engram-identity-override-test
  scripts/package-release.sh` built a local rehearsal archive.
- `ALLOW_PACKAGE_BUILD_SKIP=1 SKIP_PACKAGE_BUILD=1
  DIST_DIR=/tmp/engram-identity-override-test
  EXPECTED_TRACKED_CHANGES_PRESENT=true scripts/package-install-smoke.sh` still verified the
  local rehearsal archive, installed `engram 0.2.0`, and checked packaged HTTP `/health`.
- `ALLOW_HOMEBREW_DIST_DIR_OVERRIDE=1 ALLOW_HOMEBREW_FORMULA_OUTPUT_OVERRIDE=1
  DIST_DIR=/tmp/engram-identity-override-test EXPECTED_TRACKED_CHANGES_PRESENT=true
  FORMULA_OUTPUT=/tmp/engram-identity-override-test/homebrew/Formula/engram.rb
  scripts/render-homebrew-formula.sh` rendered a formula from the same archive, and `ruby -c`
  accepted the result.

## Package Identity Override Guard

`scripts/package-install-smoke.sh` and `scripts/render-homebrew-formula.sh` now default package
identity evidence to the current Git head and current `Cargo.lock` SHA-256. Valid non-default
`EXPECTED_PACKAGE_GIT_HEAD` and `EXPECTED_CARGO_LOCK_SHA256` values require
`ALLOW_PACKAGE_IDENTITY_OVERRIDE=1` before package identity evidence is collected, and explicit
empty values fail instead of silently falling back to the default.

This closes the gap left by the earlier malformed-input guard: final package and Homebrew evidence
can no longer be redirected to a different, syntactically valid package identity by ambient
environment values. Local asset rehearsals against a known older head remain possible, but they
must be explicitly marked as rehearsals. The published-release verifier passes the approval through
to the install smoke only when its own expected-head path differs from current `HEAD`; published
verification still performs local and remote tag parity checks before package smoke.

Targeted validation for this guard:

- `bash -n scripts/package-install-smoke.sh scripts/render-homebrew-formula.sh
  scripts/verify-published-release-install.sh`
- `ALLOW_PACKAGE_BUILD_SKIP=1
  EXPECTED_PACKAGE_GIT_HEAD=0000000000000000000000000000000000000000 SKIP_PACKAGE_BUILD=1
  DIST_DIR=/tmp/engram-no-assets scripts/package-install-smoke.sh` failed before release asset
  reads with `EXPECTED_PACKAGE_GIT_HEAD override requires explicit package identity approval`.
- `ALLOW_PACKAGE_BUILD_SKIP=1 EXPECTED_PACKAGE_GIT_HEAD= SKIP_PACKAGE_BUILD=1
  DIST_DIR=/tmp/engram-no-assets scripts/package-install-smoke.sh` failed before release asset
  reads with `EXPECTED_PACKAGE_GIT_HEAD must not be empty`.
- `ALLOW_PACKAGE_IDENTITY_OVERRIDE=yes EXPECTED_PACKAGE_GIT_HEAD=0000000000000000000000000000000000000000
  ALLOW_PACKAGE_BUILD_SKIP=1 SKIP_PACKAGE_BUILD=1 DIST_DIR=/tmp/engram-no-assets
  scripts/package-install-smoke.sh` failed with
  `ALLOW_PACKAGE_IDENTITY_OVERRIDE must be 0 or 1, got yes`.
- `EXPECTED_CARGO_LOCK_SHA256=0000000000000000000000000000000000000000000000000000000000000000
  ALLOW_PACKAGE_BUILD_SKIP=1 SKIP_PACKAGE_BUILD=1 DIST_DIR=/tmp/engram-no-assets
  scripts/package-install-smoke.sh` failed before release asset reads with
  `EXPECTED_CARGO_LOCK_SHA256 override requires explicit package identity approval`.
- `EXPECTED_PACKAGE_GIT_HEAD=0000000000000000000000000000000000000000
  scripts/render-homebrew-formula.sh` failed before release asset reads or formula writes with
  `EXPECTED_PACKAGE_GIT_HEAD override requires explicit package identity approval`.
- `EXPECTED_CARGO_LOCK_SHA256=0000000000000000000000000000000000000000000000000000000000000000
  scripts/render-homebrew-formula.sh` failed before release asset reads or formula writes with
  `EXPECTED_CARGO_LOCK_SHA256 override requires explicit package identity approval`.
- `ALLOW_PACKAGE_IDENTITY_OVERRIDE=1 EXPECTED_PACKAGE_GIT_HEAD=0000000000000000000000000000000000000000
  ALLOW_PACKAGE_BUILD_SKIP=1 SKIP_PACKAGE_BUILD=1 DIST_DIR=/tmp/engram-no-assets
  scripts/package-install-smoke.sh` passed the approval guard and then failed at the expected
  missing-tarball check.
- `scripts/release-gate-report.sh --target ga --hosted-run 27451944394 --quick
  --allow-tracked-changes --json` still accepted the default package identity path, reported
  `release_target.state=available`, `release_gate_state=evidence_incomplete`, and no release
  actions.

This is development-diff validation on top of head `30c29f5`; after this guard is committed, rerun
exact-head hosted CI and the GA gate before tag, publish, or Homebrew tap update.

## Hosted CI Run ID Guard

`scripts/release-gate-report.sh` and `scripts/verify-hosted-ci-prestep-blocker.sh` now validate
explicit hosted CI run IDs before querying GitHub run metadata. `HOSTED_RUN_ID`, `--hosted-run`,
`GITHUB_RUN_ID`, and the pre-step verifier positional run ID must be numeric GitHub Actions run
IDs. `scripts/release-gate-report.sh` also requires `PR_NUMBER` and `--pr` to be numeric before
beta pull-request metadata is queried. This keeps release-owner evidence collection and hosted-CI
fallback evidence from turning operator typos into ambiguous `gh` failures or malformed JSON
conversion later in the gate.

The pre-step verifier now distinguishes an unset `GITHUB_RUN_ID` from an explicitly empty
`GITHUB_RUN_ID=`. Unset keeps the existing latest-run discovery behavior, while an explicitly empty
value fails before discovery so CI or release automation cannot accidentally attach fallback
evidence to an ambient latest run.

The empty `GITHUB_RUN_ID` validation was checked on a development diff on top of head `111ad1f`.

The pre-step verifier also validates `EXPECTED_HEAD_SHA` before GitHub run discovery or inspection.
It must be a 40-character Git SHA, so standalone fallback evidence cannot be gathered against a
malformed expected head. `EXPECTED_HEAD_SHA` now defaults to current `HEAD` only when the variable
is unset; an explicitly empty value fails closed instead of silently falling back to current `HEAD`.

Targeted validation for this hosted CI run ID guard on a development diff:

- `bash -n scripts/package-install-smoke.sh scripts/package-release.sh
  scripts/render-homebrew-formula.sh scripts/release-gate-report.sh
  scripts/beta-release-gate-report.sh scripts/verify-hosted-ci-prestep-blocker.sh`
- `HOSTED_RUN_ID=abc scripts/release-gate-report.sh --target ga --quick
  --allow-tracked-changes --json` failed before GitHub run queries with
  `HOSTED_RUN_ID/--hosted-run must be a numeric GitHub Actions run id, got abc`.
- `scripts/release-gate-report.sh --target ga --hosted-run abc --quick
  --allow-tracked-changes --json` failed with the same run-id validation error.
- `GITHUB_RUN_ID=abc scripts/verify-hosted-ci-prestep-blocker.sh --json` failed before GitHub run
  queries with
  `GITHUB_RUN_ID/positional run id must be a numeric GitHub Actions run id, got abc`.
- `GITHUB_RUN_ID= scripts/verify-hosted-ci-prestep-blocker.sh --json` failed before GitHub run
  discovery with `GITHUB_RUN_ID/positional run id must not be empty`.
- `scripts/verify-hosted-ci-prestep-blocker.sh abc --json` failed with the same verifier run-id
  validation error.
- `scripts/release-gate-report.sh --target beta --pr abc --quick --allow-tracked-changes --json`
  failed before GitHub PR queries with
  `PR_NUMBER/--pr must be a numeric GitHub pull request number, got abc`.
- `EXPECTED_HEAD_SHA=abc scripts/verify-hosted-ci-prestep-blocker.sh --json` failed before GitHub
  run discovery with `EXPECTED_HEAD_SHA must be a 40-character Git SHA, got abc`.
- `EXPECTED_HEAD_SHA= scripts/verify-hosted-ci-prestep-blocker.sh --json` failed before GitHub
  run discovery with `EXPECTED_HEAD_SHA must be a 40-character Git SHA, got `.
- `scripts/verify-published-release-install.sh --tag v0.2.0 --expected-git-head abc --json`
  failed before release metadata or asset checks with
  `--expected-git-head must be a 40-character Git SHA, got abc`.
- `scripts/release-gate-report.sh --target ga --hosted-run 27435286033 --quick
  --allow-tracked-changes --json` still accepted a numeric run ID and produced quick GA evidence
  for head `89e784a8a566b1e1e16a4d60ad51ba072bd061ad` with hosted CI passing, release target
  available, and no release actions.

## Release Gate Explicit-Empty Input Guard

`scripts/release-gate-report.sh` now distinguishes unset release selectors from explicitly empty
release selectors. Unset values keep their existing defaults, but explicit empty values fail before
release-target lookup, hosted CI discovery, beta PR fallback, disk preflight, local CI, package
smoke, Homebrew rendering, tag publication, or asset publication. This covers `RELEASE_TARGET`,
`HOSTED_RUN_ID`, `--hosted-run`, `PR_NUMBER`, `--pr`, `RELEASE_VERSION`,
`--release-version`, `RELEASE_REPOSITORY`, and `RELEASE_GATE_MIN_FREE_KIB`.

Targeted validation for this guard on a development diff:

- `bash -n scripts/release-gate-report.sh`
- `RELEASE_TARGET= scripts/release-gate-report.sh --quick --json` failed with
  `--target must be ga or beta`.
- `HOSTED_RUN_ID= scripts/release-gate-report.sh --target ga --quick --json` failed with
  `HOSTED_RUN_ID/--hosted-run must not be empty`.
- `scripts/release-gate-report.sh --target ga --hosted-run '' --quick --json` failed with
  `HOSTED_RUN_ID/--hosted-run must not be empty`.
- `PR_NUMBER= scripts/release-gate-report.sh --target beta --quick --json` failed with
  `PR_NUMBER/--pr must not be empty`.
- `scripts/release-gate-report.sh --target beta --pr '' --quick --json` failed with
  `PR_NUMBER/--pr must not be empty`.
- `RELEASE_VERSION= scripts/release-gate-report.sh --target ga --quick --json` failed with
  `RELEASE_VERSION/--release-version must not be empty`.
- `scripts/release-gate-report.sh --target ga --release-version '' --quick --json` failed with
  `RELEASE_VERSION/--release-version must not be empty`.
- `RELEASE_REPOSITORY= scripts/release-gate-report.sh --target ga --hosted-run 27454978118
  --quick --json` failed with `RELEASE_REPOSITORY must not be empty`.
- `RELEASE_GATE_MIN_FREE_KIB= scripts/release-gate-report.sh --target ga --hosted-run
  27454978118 --quick --json` failed with `RELEASE_GATE_MIN_FREE_KIB must not be empty`.
- `scripts/release-gate-report.sh --target ga --hosted-run 27454978118 --quick
  --allow-tracked-changes --json` still accepted the exact-head hosted run and reported
  `hosted_ci.state=passing`, `release_target.state=available`, `disk_space.state=skipped`,
  `release_gate_state=evidence_incomplete`, and no release actions.

## Release Manifest Build Guard

`scripts/package-release.sh` now validates the generated `MANIFEST.json` with `jq` before the
release archive is created. The release builder checks package identity, version, host triple,
archive name, Git commit, dirty-state provenance, Cargo.lock hash, required package files, and
SHA-256 shape before any tarball or checksum is written. It also recomputes the staged payload
hashes for `engram`, `README.md`, `LICENSE`, `CHANGELOG.md`, and `RELEASE_NOTES.md` and fails if
any manifest entry does not match the staged file.

This keeps final `v0.2.0` packaging from publishing an archive whose manifest is malformed or
structurally inconsistent with the actual staged payload before the downstream install smoke ever
runs.

Exact-head validation for the payload-hash guard:

- GitHub Actions main push run `27417397670` passed for
  `35a5b9ca6ebb790ed8987ac6425d29e3e2e6e402`.
- `scripts/release-gate-report.sh --target ga --hosted-run 27417397670 --quick --json`
  passed with `release_target.state=available`, no `v0.2.0` local tag, no remote Git tag, no
  GitHub release, no owner-review readiness, and no release actions.
- `scripts/release-gate-report.sh --target ga --hosted-run 27417397670 --json`
  still failed closed at disk preflight before local CI, package/install smoke, or Homebrew render.
  The fresh run reported `free_kib=5907104`, `min_required_kib=10485760`,
  `shortfall_kib=4578656`, and cleanup candidates `target=103776236 KiB` and
  `dist=74608 KiB`. These values are host-local point-in-time evidence, not approval to delete
  either path.

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

## Homebrew Manifest Identity Guard

`scripts/render-homebrew-formula.sh` now extracts packaged `MANIFEST.json` from the release archive
before it writes `Formula/engram.rb`. The renderer checks package identity, workspace version, host
triple, archive name, expected Git commit, tracked-change provenance, and `Cargo.lock` hash
provenance against the current checkout unless explicit expected values are supplied.

This keeps the final tap update from being rendered from a stale or wrong release archive even when
the archive/checksum pair is internally consistent.

## Homebrew Archive Payload Guard

`scripts/render-homebrew-formula.sh` now inspects release archive paths before it writes
`Formula/engram.rb`. The renderer rejects empty archives, unsafe member names, members outside the
expected archive root, and archives missing the package files that the formula installs. It also
checks each packaged payload hash against the corresponding `MANIFEST.json` entry.

This keeps the final tap update from being rendered from a tampered or structurally wrong archive
even when the archive checksum and release-identity manifest fields are otherwise consistent.

## Homebrew Release URL Guard

`scripts/render-homebrew-formula.sh` now fails closed if `HOMEBREW_RELEASE_BASE_URL` differs from
the default `https://github.com/ymeiri/engram/releases/download/v<package_version>` URL base unless
`ALLOW_HOMEBREW_RELEASE_BASE_URL_OVERRIDE=1` is set. Any rendered URL base must also use `https://`
and must not end with a slash. An explicitly empty `HOMEBREW_RELEASE_BASE_URL` is also rejected
before release asset validation or formula rendering; leaving it unset still uses the canonical
GitHub release URL base.

This keeps the final tap formula from silently pointing at a wrong repository, tag, or release path
because of an ambient environment override. The explicit override remains available for local
rehearsals, not final release-owner evidence.

## Tracked-Changes Expectation Empty Guard

`scripts/package-install-smoke.sh` and `scripts/render-homebrew-formula.sh` now fail closed when
`EXPECTED_TRACKED_CHANGES_PRESENT` is explicitly set to an empty value. Leaving it unset still
derives the expected manifest flag from the current tracked working-tree state, while any explicit
value must be either `true` or `false`.

This keeps package smoke, published-release local asset rehearsals, and Homebrew formula rendering
from silently converting an empty expectation into local default evidence.

## Homebrew Host Triple Guard

`scripts/render-homebrew-formula.sh` now fails closed if `HOMEBREW_HOST_TRIPLE` selects a target
triple other than the local `rustc -vV` host unless `ALLOW_HOMEBREW_HOST_TRIPLE_OVERRIDE=1` is set
for an explicit local rehearsal. The host triple must be non-empty, must look like a Rust target
triple token, and the renderer still only supports `aarch64-apple-darwin`.

This keeps final formula evidence from silently reading a stale or wrong-platform archive because
of an ambient target override. Intentional cross-host archive rehearsals remain possible, but they
must opt in explicitly and still target the only supported Homebrew package triple.

Targeted validation for this guard:

- `HOMEBREW_HOST_TRIPLE=x86_64-apple-darwin scripts/render-homebrew-formula.sh` failed before
  release asset reads or formula writes with `HOMEBREW_HOST_TRIPLE override requires explicit
  approval`.
- `ALLOW_HOMEBREW_HOST_TRIPLE_OVERRIDE=yes HOMEBREW_HOST_TRIPLE=aarch64-apple-darwin
  scripts/render-homebrew-formula.sh` failed with
  `ALLOW_HOMEBREW_HOST_TRIPLE_OVERRIDE must be 0 or 1, got yes`.
- `HOMEBREW_HOST_TRIPLE= scripts/render-homebrew-formula.sh` failed with
  `HOMEBREW_HOST_TRIPLE must not be empty`.
- `ALLOW_HOMEBREW_HOST_TRIPLE_OVERRIDE=1 HOMEBREW_HOST_TRIPLE=bad/triple
  scripts/render-homebrew-formula.sh` failed with
  `HOMEBREW_HOST_TRIPLE must be a Rust target triple`.
- `ALLOW_HOMEBREW_HOST_TRIPLE_OVERRIDE=1 HOMEBREW_HOST_TRIPLE=x86_64-apple-darwin
  scripts/render-homebrew-formula.sh` failed with
  `Homebrew formula currently supports aarch64-apple-darwin only`.
- `ALLOW_HOMEBREW_DIST_DIR_OVERRIDE=1
  DIST_DIR=<temp> EXPECTED_PACKAGE_GIT_HEAD=6a0d5c32b0ae3ad40835116ece1386c0428d3222
  EXPECTED_TRACKED_CHANGES_PRESENT=false
  EXPECTED_CARGO_LOCK_SHA256=990db0cb3620338b48531fce661a6685f0765817700fc986210fbfe8c4c799b8
  scripts/render-homebrew-formula.sh` rendered the default temp formula path and `ruby -c` reported
  `Syntax OK`.
- `ALLOW_HOMEBREW_DIST_DIR_OVERRIDE=1 ALLOW_HOMEBREW_HOST_TRIPLE_OVERRIDE=1
  HOMEBREW_HOST_TRIPLE=aarch64-apple-darwin
  DIST_DIR=<temp> EXPECTED_PACKAGE_GIT_HEAD=6a0d5c32b0ae3ad40835116ece1386c0428d3222
  EXPECTED_TRACKED_CHANGES_PRESENT=false
  EXPECTED_CARGO_LOCK_SHA256=990db0cb3620338b48531fce661a6685f0765817700fc986210fbfe8c4c799b8
  scripts/render-homebrew-formula.sh` rendered the supported host-triple rehearsal formula and
  `ruby -c` reported `Syntax OK`.

## Homebrew Dist Directory Guard

`scripts/render-homebrew-formula.sh` now fails closed if `DIST_DIR` points anywhere other than the
repository `dist` directory unless `ALLOW_HOMEBREW_DIST_DIR_OVERRIDE=1` is set for an explicit
local rehearsal. The dist path must be non-empty.

This keeps final formula evidence from silently reading release assets from, or writing the default
formula under, an ambient temp directory or tap checkout. Intentional temp-archive rehearsals remain
available, but they must name the approval flag; independent `FORMULA_OUTPUT` overrides still need
their own approval flag.

Targeted validation for this guard:

- `DIST_DIR=/tmp/engram-homebrew-dist-test scripts/render-homebrew-formula.sh` failed before
  release asset reads or formula writes with `DIST_DIR override requires explicit Homebrew
  approval`.
- `ALLOW_HOMEBREW_DIST_DIR_OVERRIDE=yes DIST_DIR=/tmp/engram-homebrew-dist-test
  scripts/render-homebrew-formula.sh` failed with
  `ALLOW_HOMEBREW_DIST_DIR_OVERRIDE must be 0 or 1, got yes`.
- `DIST_DIR= scripts/render-homebrew-formula.sh` failed with `DIST_DIR must not be empty`.
- `ALLOW_HOMEBREW_DIST_DIR_OVERRIDE=1 DIST_DIR=<temp>
  EXPECTED_PACKAGE_GIT_HEAD=6a0d5c32b0ae3ad40835116ece1386c0428d3222
  EXPECTED_TRACKED_CHANGES_PRESENT=false
  EXPECTED_CARGO_LOCK_SHA256=990db0cb3620338b48531fce661a6685f0765817700fc986210fbfe8c4c799b8
  scripts/render-homebrew-formula.sh` rendered the explicitly approved temp dist formula and
  `ruby -c` reported `Syntax OK`.
- `ALLOW_HOMEBREW_DIST_DIR_OVERRIDE=1 ALLOW_HOMEBREW_FORMULA_OUTPUT_OVERRIDE=1
  FORMULA_OUTPUT=<temp>/Formula/engram.rb DIST_DIR=<temp>
  EXPECTED_PACKAGE_GIT_HEAD=6a0d5c32b0ae3ad40835116ece1386c0428d3222
  EXPECTED_TRACKED_CHANGES_PRESENT=false
  EXPECTED_CARGO_LOCK_SHA256=990db0cb3620338b48531fce661a6685f0765817700fc986210fbfe8c4c799b8
  scripts/render-homebrew-formula.sh` rendered the explicitly approved temp formula output and
  `ruby -c` reported `Syntax OK`.

## Homebrew Formula Output Guard

`scripts/render-homebrew-formula.sh` now fails closed if `FORMULA_OUTPUT` points anywhere other
than the default `<dist>/homebrew/Formula/engram.rb` path unless
`ALLOW_HOMEBREW_FORMULA_OUTPUT_OVERRIDE=1` is set for an explicit local rehearsal. The output path
must be non-empty and must end with `engram.rb`.

This keeps final tap evidence from silently writing the generated formula to the wrong file or an
ambient Homebrew tap checkout. Intentional temp-output rehearsals remain available, but they must
name the approval flag and still write a file named `engram.rb`.

Targeted validation for this guard:

- `FORMULA_OUTPUT=/tmp/engram.rb scripts/render-homebrew-formula.sh` failed before release asset
  reads or formula writes with `FORMULA_OUTPUT override requires explicit approval`.
- `ALLOW_HOMEBREW_FORMULA_OUTPUT_OVERRIDE=yes FORMULA_OUTPUT=/tmp/engram.rb
  scripts/render-homebrew-formula.sh` failed with
  `ALLOW_HOMEBREW_FORMULA_OUTPUT_OVERRIDE must be 0 or 1, got yes`.
- `FORMULA_OUTPUT= scripts/render-homebrew-formula.sh` failed with
  `FORMULA_OUTPUT must not be empty`.
- `ALLOW_HOMEBREW_FORMULA_OUTPUT_OVERRIDE=1 FORMULA_OUTPUT=/tmp/not-engram.txt
  scripts/render-homebrew-formula.sh` failed with
  `FORMULA_OUTPUT must end with engram.rb`.
- `ALLOW_HOMEBREW_DIST_DIR_OVERRIDE=1
  DIST_DIR=<temp> EXPECTED_PACKAGE_GIT_HEAD=6a0d5c32b0ae3ad40835116ece1386c0428d3222
  EXPECTED_TRACKED_CHANGES_PRESENT=false
  EXPECTED_CARGO_LOCK_SHA256=990db0cb3620338b48531fce661a6685f0765817700fc986210fbfe8c4c799b8
  scripts/render-homebrew-formula.sh` rendered the default temp formula path and `ruby -c` reported
  `Syntax OK`.
- `ALLOW_HOMEBREW_DIST_DIR_OVERRIDE=1 ALLOW_HOMEBREW_FORMULA_OUTPUT_OVERRIDE=1
  FORMULA_OUTPUT=<temp>/Formula/engram.rb
  DIST_DIR=<temp>
  EXPECTED_PACKAGE_GIT_HEAD=6a0d5c32b0ae3ad40835116ece1386c0428d3222
  EXPECTED_TRACKED_CHANGES_PRESENT=false
  EXPECTED_CARGO_LOCK_SHA256=990db0cb3620338b48531fce661a6685f0765817700fc986210fbfe8c4c799b8
  scripts/render-homebrew-formula.sh` rendered an explicitly approved temp formula path and
  `ruby -c` reported `Syntax OK`.

## Homebrew Formula Final Output Staging

`scripts/render-homebrew-formula.sh` now writes formula text to a hidden temporary file beside the
final `engram.rb`, runs `ruby -c` on that temporary file, then moves it into the final output path
only after a final overwrite/race check. The renderer also rejects a directory named `engram.rb` as
an output path.

This keeps final tap evidence from leaving a partial formula at `dist/homebrew/Formula/engram.rb`
after a failed or interrupted formula render.

Development-diff validation for this guard:

- `bash -n scripts/render-homebrew-formula.sh`
- With a temporary failing `ruby` wrapper and `DIST_DIR=<temp>`,
  `scripts/render-homebrew-formula.sh` failed during temporary formula syntax validation and left
  no final formula or hidden temporary formula output.
- With `DIST_DIR=<temp>`,
  `scripts/render-homebrew-formula.sh` rendered an explicitly approved temp formula path and the
  internal Ruby syntax check passed.

This is development-diff validation on top of head `afb5865`; after this guard is committed, rerun
exact-head hosted CI and the GA gate before tag, publish, or Homebrew tap update.

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

## Hosted CI Embedding Cache Warmup

GitHub Actions run `27396872082` for commit `b192c60` failed in the `Test` job while Format,
Check, Docs, and Clippy passed. The failure was isolated to daemon startup in
`engram-tests/tests/multi_session_tests.rs`: the hosted runner used an empty fastembed cache at
`engram-tests/.fastembed_cache`, then daemon startup failed with `Failed to retrieve model.onnx`.

The test job now sets `ENGRAM_EMBED_CACHE_DIR` to that deterministic cache path, restores it with
`actions/cache`, and runs `engram warmup embeddings` with retries before
`cargo test --locked --all-targets --jobs 1`. This keeps hosted multi-session tests from depending
on every daemon test process being able to download the ONNX model during startup. The cache warmup
is CI validation hardening only; it does not change release artifacts or publish anything.

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

The native Claude production-gate preflight was refreshed again after the installed Claude Code
runtime moved from `2.1.173` to `2.1.174`. `scripts/native-claude-gate-preflight.sh` now defaults
to:

- Claude binary `/Users/yuval.meiri/.local/bin/claude`
- Claude target `/Users/yuval.meiri/.local/share/claude/versions/2.1.174`
- Claude version `2.1.174 (Claude Code)`
- Claude target SHA-256
  `20c5380b4423be9963c510f5464cc1f443235a9b4423179f9c01f28021b81bad`
- Engram binary `/Users/yuval.meiri/.local/bin/engram`
- canonical vault path `/Users/yuval.meiri/.engram/vault`
- expected branch `main`

The same preflight now also fails closed before evidence collection if a caller changes any of
those evidence targets without an explicit local-rehearsal approval flag:
`ALLOW_NATIVE_CLAUDE_BRANCH_OVERRIDE`, `ALLOW_NATIVE_CLAUDE_BIN_OVERRIDE`,
`ALLOW_NATIVE_CLAUDE_IDENTITY_OVERRIDE`, `ALLOW_NATIVE_CLAUDE_ENGRAM_BIN_OVERRIDE`, or
`ALLOW_NATIVE_CLAUDE_VAULT_PATH_OVERRIDE`.

A development-diff preflight with worktree changes explicitly allowed first showed two live
blockers after the `2.1.174` baseline refresh: canonical vault generated-count drift and an
already-running native Claude CLI process. Regenerating the canonical generated vault removed the
vault blocker. The refreshed preflight then reported:

- `gate_state=blocked`
- branch `main`, upstream `origin/main`, `ahead=0`, `behind=0`
- `tracked_changes_present=true` because this code/docs slice was still uncommitted
- no extra untracked files beyond the local `AGENTS.md` allowance
- Claude Code target `/Users/yuval.meiri/.local/share/claude/versions/2.1.174`,
  version `2.1.174 (Claude Code)`, SHA-256
  `20c5380b4423be9963c510f5464cc1f443235a9b4423179f9c01f28021b81bad`
- canonical vault status aligned at `generated_file_count=2977`,
  `expected_generated_file_count=2977`, `user_file_count=0`
- the only remaining blocker was `native Claude CLI processes are already running`
- no native Claude launch, `/hooks` command, process signal, or release action was performed

This refresh keeps the production gate open. It narrows the current native-Claude blocker to
process-state availability for a future proof run; it does not prove prompt-bearing behavior,
effective-hook visibility, or live host-label attribution. Rerun the same preflight on the clean
committed exact head before using it as owner-review evidence.

This refresh also confirms the divergent-branch warning seen during a prior pull
attempt is not the live repo state: after `git fetch --tags --prune origin`, `main` and
`origin/main` were still aligned at `a082a63` with `ahead=0`, `behind=0`.

## Native Claude Preflight Checkpoint at cada64a

After the GA evidence docs refresh was committed as `cada64a`, the canonical generated vault had
drifted behind memory at `generated_file_count=3028`,
`expected_generated_file_count=3051`, `user_file_count=0`. Recompiling
`/Users/yuval.meiri/.engram/vault` wrote only generated vault files and restored the canonical
vault to `generated_file_count=3051`, `expected_generated_file_count=3051`,
`user_file_count=0`.

The current strict native-Claude preflight on exact head `cada64a` then reported:

- `gate_state=blocked`
- branch `main`, upstream `origin/main`, `ahead=0`, `behind=0`
- `tracked_changes_present=false`
- `extra_untracked_count=3036`, from the local untracked `.engram-vault/` copy
- canonical vault status aligned at `generated_file_count=3051`,
  `expected_generated_file_count=3051`, `user_file_count=0`
- blockers `unexpected untracked files are present` and
  `native Claude CLI processes are already running`
- no native Claude launch, `/hooks` command, process signal, or release action was performed

A scoped rerun with worktree changes explicitly allowed was used only to isolate the production
native-Claude gate from the local untracked vault copy. It reported the same exact head `cada64a`,
no tracked changes, canonical vault alignment at `3051/3051`, Claude Code target
`/Users/yuval.meiri/.local/share/claude/versions/2.1.174`, version `2.1.174 (Claude Code)`,
SHA-256 `20c5380b4423be9963c510f5464cc1f443235a9b4423179f9c01f28021b81bad`, and the only
remaining blocker `native Claude CLI processes are already running`. It also confirmed
`actions_performed.native_claude_launch=false`, `actions_performed.hooks_command=false`,
`actions_performed.process_signals=false`, and `actions_performed.release_actions=false`.

This keeps native Claude prompt-bearing, live `/hooks`, and live host-label proof outside the
current GA release claim. Final owner-review evidence should either run in a clean worktree without
the local `.engram-vault/` copy or receive explicit local handling for that generated vault copy,
and it still needs a clean native-Claude process window.

## Native Claude Configuration Failure JSON

`scripts/native-claude-gate-preflight.sh --json` now emits structured failure evidence for early
native-Claude operator and approval-input preflights before exiting nonzero. Unknown flags,
non-default evidence target overrides without the matching approval flag, malformed approval flag
values, and malformed Claude identity inputs report
`gate_state=configuration_preflight_failed` with `failure.kind=configuration_preflight`.

Those failures also keep `actions_performed.native_claude_launch=false`,
`actions_performed.hooks_command=false`, `actions_performed.process_signals=false`, and
`actions_performed.release_actions=false`, so automation can distinguish config rejection from a
preflight collection crash or a live native-Claude proof attempt.
The native-Claude JSON paths also retain the legacy `release_actions_performed=false`
compatibility field, matching the release gate and post-publish verifier no-action contracts.

Validation:

- `bash -n scripts/native-claude-gate-preflight.sh`
- `scripts/native-claude-gate-preflight.sh --bogus --json` failed with
  `gate_state=configuration_preflight_failed`,
  `failure.message="unknown option: --bogus"`, `release_actions_performed=false`, and all action
  booleans false.
- `scripts/native-claude-gate-preflight.sh --expected-branch dev --json` failed with
  `gate_state=configuration_preflight_failed`,
  `failure.message="EXPECTED_BRANCH override requires explicit native Claude approval"`, and
  all action booleans false.
- `ALLOW_NATIVE_CLAUDE_BRANCH_OVERRIDE=yes scripts/native-claude-gate-preflight.sh --json`
  failed with `gate_state=configuration_preflight_failed`,
  `failure.message="ALLOW_NATIVE_CLAUDE_BRANCH_OVERRIDE must be 0 or 1, got yes"`, and all
  action booleans false.
- `CLAUDE_BIN=/tmp/claude scripts/native-claude-gate-preflight.sh --json` failed with
  `gate_state=configuration_preflight_failed`,
  `failure.message="CLAUDE_BIN override requires explicit native Claude approval"`, and all
  action booleans false.
- `EXPECTED_CLAUDE_SHA256=abc scripts/native-claude-gate-preflight.sh --json` failed with
  `gate_state=configuration_preflight_failed`,
  `failure.message` beginning `EXPECTED_CLAUDE_SHA256 must be a SHA-256 hex value`, and all
  action booleans false.
- `scripts/native-claude-gate-preflight.sh --allow-worktree-changes --json` still emitted the
  normal blocked preflight evidence shape; with this development diff present, it reported
  `tracked_changes_present=true`, canonical vault alignment, `release_actions_performed=false`,
  and the same live native-Claude process blocker.

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
branch mismatch; `--expected-branch` or `EXPECTED_BRANCH` values that target a non-`main` branch
now require `ALLOW_EXPECTED_BRANCH_OVERRIDE=1` for an explicit local rehearsal. Branch names are
validated with Git before any release evidence is collected. `scripts/beta-release-gate-report.sh`
remains as a compatibility wrapper for beta PR gates. The script remains evidence-only: it does not
accept fallbacks, mark a PR ready, merge, tag, publish, mutate harness state, or change release
scope.

GA mode also separates the current workspace package version from the intended release version.
While the workspace still reports `0.2.0-beta.2`, the report defaults the intended GA release
version to `0.2.0`, emits `workspace_version_matches_release=false`, and keeps
`release_gate_state=version_bump_required`. This prevents pre-GA evidence from suggesting a
`tag_v0.2.0-beta.2` action for the GA path.

The intended release version is validated before the report constructs the release tag or checks
local/remote release-target availability. `RELEASE_VERSION` and `--release-version` must be
`x.y.z` with an optional prerelease suffix such as `-beta.2`; malformed values fail closed instead
of producing ambiguous release-owner evidence for malformed tags.

The GA report also verifies release-note scope acknowledgements for high-risk items that remain
outside the current `v0.2.0` claim: native Claude prompt-bearing/live `/hooks` proof and broad
legacy lifecycle/M6 mutation. If those acknowledgements are removed before the final release gate,
the report emits `release_scope.state=incomplete` and blocks with
`release_gate_state=release_scope_acknowledgement_required` instead of marking the release ready
for owner review.

The release notes source is now default-deny as release-owner evidence. `RELEASE_NOTES_PATH`
overrides fail before release-scope checks unless `ALLOW_RELEASE_NOTES_PATH_OVERRIDE=1` is set for
an explicit local rehearsal, so accidental environment drift cannot point the GA gate at the wrong
scope document.

Hosted CI expected-event inputs are now validated before GitHub run discovery or inspection.
`EXPECTED_EVENT`, GA gate `--expected-event`, and hosted CI pre-step verifier `--event` values must
be GitHub event-name tokens such as `push` or `pull_request`; malformed values fail closed instead
of querying for ambiguous run evidence.

Hosted CI workflow-name selection is also default-deny for release evidence. Both the GA release
gate and the hosted CI pre-step verifier default to the repo's `CI` workflow and reject
`EXPECTED_WORKFLOW_NAME` drift before GitHub run discovery or inspection unless
`ALLOW_EXPECTED_WORKFLOW_NAME_OVERRIDE=1` is set for an explicit local rehearsal. Workflow names
must be non-empty and limited to letters, numbers, spaces, dot, underscore, and hyphen.

Validation on the workflow-name guard:

- `EXPECTED_WORKFLOW_NAME=CI-copy scripts/release-gate-report.sh --target ga --hosted-run
  27442002997 --quick --json` failed before hosted run inspection with
  `EXPECTED_WORKFLOW_NAME override requires explicit approval`.
- `ALLOW_EXPECTED_WORKFLOW_NAME_OVERRIDE=yes EXPECTED_WORKFLOW_NAME=CI-copy
  scripts/release-gate-report.sh --target ga --hosted-run 27442002997 --quick --json` failed with
  `ALLOW_EXPECTED_WORKFLOW_NAME_OVERRIDE must be 0 or 1, got yes`.
- `ALLOW_EXPECTED_WORKFLOW_NAME_OVERRIDE=1 EXPECTED_WORKFLOW_NAME=CI/extra
  scripts/release-gate-report.sh --target ga --hosted-run 27442002997 --quick --json` failed with
  `EXPECTED_WORKFLOW_NAME must contain only letters, numbers, spaces, dot, underscore, and hyphen`.
- `EXPECTED_WORKFLOW_NAME= scripts/verify-hosted-ci-prestep-blocker.sh --json` failed before
  GitHub run discovery with `EXPECTED_WORKFLOW_NAME must not be empty`.
- `EXPECTED_WORKFLOW_NAME=CI-copy scripts/verify-hosted-ci-prestep-blocker.sh --json` failed before
  GitHub run discovery with `EXPECTED_WORKFLOW_NAME override requires explicit approval`.
- `scripts/release-gate-report.sh --target ga --hosted-run 27442002997 --quick
  --allow-tracked-changes --json` still accepted the default `CI` workflow path and reported
  `hosted_ci.state=passing`, `hosted_ci.run.workflowName=CI`, and
  `release_gate_state=evidence_incomplete` without release actions.

## Release Gate Workspace Package Version Guard

`scripts/release-gate-report.sh` now parses command-line options before requiring Cargo, so
`--help` remains available without a Rust toolchain. For real evidence collection, the gate now
requires `cargo` explicitly, derives the workspace package version with `cargo pkgid --locked -p
engram-cli`, and validates that version as SemVer before deriving `RELEASE_VERSION` defaults or
constructing a release tag.

This closes a release-evidence gap where a missing Cargo binary or malformed package-version output
could fail with an indirect shell error, or could be misreported as a release-version problem after
release evidence had already started to initialize.

Targeted validation for this guard on a development diff:

- `env PATH=/usr/bin:/bin scripts/release-gate-report.sh --help` printed usage and exited `0`
  without requiring Cargo.
- `env PATH=/usr/bin:/bin scripts/release-gate-report.sh --target ga --quick` failed before
  release evidence collection with `required tool is missing: cargo`.
- With a fake `cargo pkgid` output of `file:///tmp/engram#not-a-version`,
  `scripts/release-gate-report.sh --target ga --hosted-run 27461887713 --quick
  --allow-tracked-changes --json` failed before release-target or hosted-run checks with
  `workspace package version must be x.y.z with an optional prerelease suffix, got not-a-version`.
- `scripts/release-gate-report.sh --target ga --hosted-run 27461887713 --quick
  --allow-tracked-changes --json` still accepted the current workspace package version
  `0.2.0`, reported `release_target.state=available`, and kept
  `release_gate_state=evidence_incomplete` without release actions.

This is development-diff validation on top of head `89ec467`; after this guard is committed, rerun
exact-head hosted CI and the GA gate before tag, publish, or Homebrew tap update.

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
branch does not match. Non-`main` expected-branch overrides now also require
`ALLOW_EXPECTED_BRANCH_OVERRIDE=1` and the supplied branch name must pass `git check-ref-format`.
Explicitly empty `EXPECTED_BRANCH` and `--expected-branch` selectors now fail closed before the
default `main` branch is applied. This closes a release-management gap where an accidental
`EXPECTED_BRANCH`/`--expected-branch` override could make a synced non-main branch, or a miswired
empty branch selector, look like GA owner-review evidence.

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
- `EXPECTED_BRANCH=release/0.2 scripts/release-gate-report.sh --target ga --hosted-run
  27443650460 --quick --json` failed before release-target lookup or hosted run inspection with
  `EXPECTED_BRANCH override requires explicit approval`.
- `ALLOW_EXPECTED_BRANCH_OVERRIDE=yes EXPECTED_BRANCH=release/0.2
  scripts/release-gate-report.sh --target ga --hosted-run 27443650460 --quick --json` failed with
  `ALLOW_EXPECTED_BRANCH_OVERRIDE must be 0 or 1, got yes`.
- `ALLOW_EXPECTED_BRANCH_OVERRIDE=1 EXPECTED_BRANCH=bad..branch
  scripts/release-gate-report.sh --target ga --hosted-run 27443650460 --quick --json` failed with
  `EXPECTED_BRANCH/--expected-branch must be a valid Git branch name, got bad..branch`.
- `EXPECTED_BRANCH= scripts/release-gate-report.sh --target ga --hosted-run 27459091318 --quick
  --json` failed before release-target lookup with
  `EXPECTED_BRANCH/--expected-branch must not be empty`.
- `scripts/release-gate-report.sh --target ga --expected-branch '' --hosted-run 27459091318
  --quick --json` failed before release-target lookup with
  `EXPECTED_BRANCH/--expected-branch must not be empty`.
- `scripts/release-gate-report.sh --target ga --hosted-run 27443650460 --quick
  --allow-tracked-changes --json` still accepted the default `main` branch path and reported
  `branch=main`, `expected_branch=main`, hosted CI passing, `release_target.state=available`,
  `release_gate_state=evidence_incomplete`, and no release actions.

This branch-guard slice changes release-facing code and docs. If it becomes the final release head,
rerun exact-head hosted CI and the full GA release gate before tag, publish, or Homebrew tap update.

## Local Disk-Space Release Gate Preflight

`scripts/release-gate-report.sh --target ga` now checks local free space before running local CI or
package/install smoke. Full local owner-review evidence uses the default
`RELEASE_GATE_MIN_FREE_KIB=10485760` threshold (10 GiB) and reports `disk_space.state`,
`disk_space.free_kib`, `disk_space.min_required_kib`, `disk_space.shortfall_kib`, and
`disk_space.cleanup_candidates` in JSON once the preflight runs. The
override is available only for controlled rehearsals with
`ALLOW_RELEASE_GATE_MIN_FREE_OVERRIDE=1`; lowering it weakens local release evidence and is not
final owner-review proof.
If the preflight fails in `--json` mode, the command still exits nonzero but now writes structured
failure evidence with `release_gate_state=disk_space_cleanup_required`, `local_ci=not_run`,
`package_install_smoke=not_run`, and `failure.kind=disk_space_preflight`.

This closes the current local validation failure mode where Cargo can spend substantial time in
`cargo check`, clippy, or tests before surfacing `No space left on device`. On this host,
exact-head hosted CI for the `4978711` hardening baseline is green, but the full local GA gate
remains pending because this filesystem still has less than the default 10 GiB release-gate
threshold while `target/` is about 99 GiB. Cleanup of generated build artifacts still requires
explicit approval before rerunning the full gate and refreshing GA `dist/` assets. The cleanup
candidate list is intentionally
non-destructive evidence for that approval or triage step; it is not authorization for the release
gate to delete `target/`, `dist/`, or other local artifacts.

## Disk Threshold Override Guard

`scripts/release-gate-report.sh` now fails closed if `RELEASE_GATE_MIN_FREE_KIB` differs from the
default `10485760` threshold unless `ALLOW_RELEASE_GATE_MIN_FREE_OVERRIDE=1` is set for an explicit
local rehearsal. The allow flag must be exactly `0` or `1`, and the threshold must remain a
non-negative integer. Final owner-review evidence must use the default threshold; the release-owner
runbook now asserts `disk_space.min_required_kib == 10485760`.

Targeted validation for this guard:

- `bash -n scripts/release-gate-report.sh scripts/beta-release-gate-report.sh`
- `RELEASE_GATE_MIN_FREE_KIB=1 scripts/release-gate-report.sh --target ga --hosted-run
  27423199238 --quick --json` failed before release-target lookup with
  `RELEASE_GATE_MIN_FREE_KIB override requires explicit approval`.
- `ALLOW_RELEASE_GATE_MIN_FREE_OVERRIDE=yes RELEASE_GATE_MIN_FREE_KIB=1
  scripts/release-gate-report.sh --target ga --hosted-run 27423199238 --quick --json` failed with
  `ALLOW_RELEASE_GATE_MIN_FREE_OVERRIDE must be 0 or 1, got yes`.
- `ALLOW_RELEASE_GATE_MIN_FREE_OVERRIDE=1 RELEASE_GATE_MIN_FREE_KIB=abc
  scripts/release-gate-report.sh --target ga --hosted-run 27423199238 --quick --json` failed with
  `RELEASE_GATE_MIN_FREE_KIB must be a non-negative integer`.
- `ALLOW_RELEASE_GATE_MIN_FREE_OVERRIDE=1 RELEASE_GATE_MIN_FREE_KIB=1
  scripts/release-gate-report.sh --target ga --hosted-run 27423199238 --quick
  --allow-tracked-changes --json` passed as rehearsal evidence with
  `disk_space.state=skipped`, `disk_space.min_required_kib=1`,
  `release_gate_state=evidence_incomplete`, and no release actions.
- `scripts/release-gate-report.sh --target ga --hosted-run 27423199238 --quick
  --allow-tracked-changes --json` passed with the default
  `disk_space.min_required_kib=10485760`, `release_target.repository=ymeiri/engram`,
  `release_target.state=available`, and no release actions.

This is development-diff validation on top of head `760fb29`; after this guard is committed, rerun
exact-head hosted CI and the GA gate before tag, publish, or Homebrew tap update.

After this guard was committed as `6ba403efce7dc0893a1fd40aa6fddc39fbaa6fe5`, GitHub Actions main
push run `27425319893` passed for Check, Test, Clippy, Docs, and Format. The exact-head quick GA
gate with hosted run `27425319893` passed with `release_target.state=available`, no local or
remote `v0.2.0` tag, no GitHub release, default `disk_space.min_required_kib=10485760`, no
owner-review readiness, and no release actions. The exact-head full GA gate still failed closed at
disk preflight with default threshold evidence before local CI, package/install smoke, Homebrew
render, or any release action.

## GA Hardening Evidence Baseline

The release-facing hardening baseline used by this matrix refresh is
`4978711b5bc27d350f0d57983698758d331a3f16`. It includes the release-owner
runbook disk-preflight approval gate on top of disk cleanup-candidate reporting,
deterministic hosted fastembed cache warmup, and the `actions/cache@v5` CI update.

Validation on this checkpoint:

- GitHub Actions main push run `27401954970` passed for Check, Test, Clippy, Docs, and Format.
- The Test job restored and warmed `engram-tests/.fastembed_cache`, then ran
  `cargo test --locked --all-targets --jobs 1`.
- `ALLOW_RELEASE_GATE_MIN_FREE_OVERRIDE=1 RELEASE_GATE_MIN_FREE_KIB=1
  scripts/release-gate-report.sh --target ga --hosted-run 27401954970 --quick --json` passed as
  partial exact-head rehearsal evidence with no release actions.
- The default full gate still fails closed at disk preflight on this host with
  `release_gate_state=disk_space_cleanup_required`, `release_target.state=available`,
  `target=103776236 KiB`, and `dist=74608 KiB`. Exact `free_space_kib` and `shortfall_kib`
  values are host-local and should be read from the gate JSON emitted for the final release head.
- A fresh `git fetch --tags --prune origin` followed by
  `git rev-list --left-right --count main...origin/main` returned `0 0`, so the
  recurring divergent-branch pull hint is not current evidence to run `git pull`,
  merge, rebase, or set pull-policy configuration.
- `git tag --list 'v0.2.0*'` still lists only `v0.2.0-beta.1` and `v0.2.0-beta.2`; `gh release
  view v0.2.0 --repo ymeiri/engram` still reports `release not found`.

This checkpoint is not final owner-review evidence because the full local GA gate and refreshed
GA package/Homebrew assets are still blocked by local disk cleanup approval.

## Release Packaging Payload-Hash Checkpoint

The latest release-packaging hardening checkpoint before this matrix maintenance update is
`35a5b9ca6ebb790ed8987ac6425d29e3e2e6e402`. It adds producer-side staged payload hash checks to
`scripts/package-release.sh`, so `MANIFEST.json` cannot claim a different digest than the file
that is about to be archived.

Validation on this checkpoint:

- GitHub Actions main push run `27417397670` passed for Check, Test, Clippy, Docs, and Format.
- Positive package evidence used `ALLOW_TRACKED_CHANGES=1 ALLOW_PACKAGE_DIST_DIR_OVERRIDE=1
  DIST_DIR=<temp> scripts/package-release.sh`, then `ALLOW_PACKAGE_BUILD_SKIP=1 DIST_DIR=<temp>
  SKIP_PACKAGE_BUILD=1 EXPECTED_TRACKED_CHANGES_PRESENT=true
  scripts/package-install-smoke.sh`; the packaged binary reported `engram 0.2.0` and packaged
  HTTP `/health` returned
  `{"status":"ok","service":"engram","version":"0.2.0"}`.
- Negative package evidence used a temporary `shasum` wrapper that returned a second mismatched
  `README.md` digest; `scripts/package-release.sh` failed with
  `manifest hash mismatch for README.md` and wrote no tarball.
- The exact-head quick GA gate with hosted run `27417397670` passed as partial evidence with
  `release_target.state=available` and no release actions.
- The exact-head full GA gate with hosted run `27417397670` still failed closed at
  `disk_space_preflight` before local CI/package/Homebrew steps. The run reported
  `free_kib=5907104`, `min_required_kib=10485760`, `shortfall_kib=4578656`,
  `target=103776236 KiB`, and `dist=74608 KiB`.

This checkpoint is not final owner-review evidence because the full local GA gate remains blocked
by disk cleanup approval. It is evidence that the current package-manifest producer guard is
validated on an exact hosted-CI head.

## GA Release Target Availability Guard

`scripts/release-gate-report.sh --target ga` now checks the intended release target before local
CI, package/install smoke, or Homebrew formula validation can contribute owner-review evidence. For
the GA path, JSON output reports `release_target.tag`, `release_target.repository`,
`release_target.state`, `release_target.local_tag_exists`,
`release_target.remote_git_tag_exists`, and `release_target.github_release_exists`, while text
output reports the corresponding `release_target_*` fields. The gate fails closed with
`release_gate_state=release_target_unavailable` if the local tag, remote Git tag, or GitHub release
already exists, and with `release_gate_state=release_target_check_failed` if GitHub release lookup
or remote Git tag lookup cannot distinguish absence from an access or transport error.

Targeted validation for this guard:

- `bash -n scripts/release-gate-report.sh scripts/beta-release-gate-report.sh`
- `scripts/release-gate-report.sh --target ga --hosted-run 27414400008 --quick
  --allow-tracked-changes --json`, with a JSON assertion requiring exact head
  `f1c6c6287d32c424d08460f11c63f5a1202fc2ac`, hosted CI state `passing`,
  `release_target.tag=v0.2.0`, `release_target.repository=ymeiri/engram`,
  `release_target.state=available`, no local tag, no remote Git tag, no GitHub release, no
  owner-review readiness, and no release actions.
- `scripts/release-gate-report.sh --target ga --release-version 0.2.0-beta.2 --hosted-run
  27414400008 --quick --allow-tracked-changes --json`, expected failure with
  `release_gate_state=release_target_unavailable`, `failure.kind=release_target_preflight`,
  `release_target.local_tag_exists=true`, `release_target.remote_git_tag_exists=true`,
  `release_target.github_release_exists=true`, `local_ci=not_run`, `package_install_smoke=not_run`,
  `disk_space.state=not_checked`, no owner-review readiness, and no release actions.
- `scripts/release-gate-report.sh --target ga --release-version 0.2.0-remote-only --hosted-run
  27414400008 --quick --allow-tracked-changes --json` with a mocked `git ls-remote`, expected
  failure with `release_gate_state=release_target_unavailable`,
  `release_target.local_tag_exists=false`, `release_target.remote_git_tag_exists=true`,
  `release_target.github_release_exists=false`, `disk_space.state=not_checked`, and no release
  actions.
- `ALLOW_RELEASE_REPOSITORY_OVERRIDE=1 RELEASE_REPOSITORY=ymeiri/engram-does-not-exist
  scripts/release-gate-report.sh --target ga --hosted-run 27414400008 --quick
  --allow-tracked-changes --json`, expected failure with
  `release_gate_state=release_target_check_failed`, `release_target.state=unknown`,
  `disk_space.state=not_checked`, no owner-review readiness, and no release actions.

The remote Git tag extension is development-diff validation on top of the `f1c6c62` head. If this
guard becomes part of the release head, rerun exact-head hosted CI and the full GA release gate
before tag, publish, or Homebrew tap update.

## Release Repository Override Guard

`scripts/release-gate-report.sh` and `scripts/verify-published-release-install.sh` now default the
release repository to `ymeiri/engram` and fail closed when an ambient repository override points
elsewhere. `RELEASE_REPOSITORY` in the GA gate and `GITHUB_REPOSITORY` or `--repo` in the
published-release verifier require `ALLOW_RELEASE_REPOSITORY_OVERRIDE=1` before they can target a
different repository, and the effective repository must still be an `owner/name` value.

This prevents final `v0.2.0` gate, tag, or published-asset evidence from being accidentally
collected against a fork, scratch repository, or malformed repository string. Overrides remain
available for explicit local rehearsals, not final release-owner evidence.

Targeted validation for this guard:

- `bash -n scripts/release-gate-report.sh scripts/verify-published-release-install.sh
  scripts/package-release.sh scripts/package-install-smoke.sh scripts/render-homebrew-formula.sh`
- `RELEASE_REPOSITORY=example/engram scripts/release-gate-report.sh --target ga --hosted-run
  27421217595 --quick --json` failed before release-target lookup with
  `RELEASE_REPOSITORY override requires explicit approval`.
- `ALLOW_RELEASE_REPOSITORY_OVERRIDE=yes RELEASE_REPOSITORY=example/engram
  scripts/release-gate-report.sh --target ga --hosted-run 27421217595 --quick --json` failed with
  `ALLOW_RELEASE_REPOSITORY_OVERRIDE must be 0 or 1, got yes`.
- `ALLOW_RELEASE_REPOSITORY_OVERRIDE=1 RELEASE_REPOSITORY=bad/repo/extra
  scripts/release-gate-report.sh --target ga --hosted-run 27421217595 --quick --json` failed with
  `release repository must be owner/name`.
- `GITHUB_REPOSITORY=example/engram scripts/verify-published-release-install.sh --tag v0.2.0
  --asset-dir /tmp/engram-no-such-assets` failed with
  `release repository override requires explicit approval`.
- `ALLOW_RELEASE_REPOSITORY_OVERRIDE=1 GITHUB_REPOSITORY=bad/repo/extra
  scripts/verify-published-release-install.sh --tag v0.2.0
  --asset-dir /tmp/engram-no-such-assets` failed with
  `release repository must be owner/name`.
- The default-repository quick GA gate with hosted run `27421217595` passed for head `a1a7da5`
  while reporting `release_target.repository=ymeiri/engram`,
  `release_target.state=available`, no `v0.2.0` local tag, no remote Git tag, no GitHub release,
  and no release actions.
- A local asset-dir verifier rehearsal from `/tmp/engram-repo-guard-assets.X4Vj1a` passed with
  `repo=ymeiri/engram`, `assets.source=asset_dir`, `asset_install_verified=true`,
  `published_install_verified=false`, and no release actions.

This is development-diff validation on top of head `a1a7da5`; after this guard is committed, rerun
exact-head hosted CI and the GA gate before tag, publish, or Homebrew tap update.

## Published Release Verifier Explicit-Empty Input Guard

`scripts/verify-published-release-install.sh` now distinguishes unset post-publish verification
selectors from explicitly empty selectors. Unset values keep their existing defaults, but explicit
empty values fail before GitHub release metadata inspection, asset download, or local asset
validation. This covers `GITHUB_REPOSITORY`, `--repo`, `--tag`, `--host-triple`,
`--expected-git-head`, and `--asset-dir`.

This prevents final published-install proof from accidentally verifying the default repository,
default tag, current host triple, current `HEAD`, or downloaded GitHub release assets when an
operator or automation supplied an empty value.

Targeted validation for this guard on a development diff:

- `bash -n scripts/verify-published-release-install.sh`
- `GITHUB_REPOSITORY= scripts/verify-published-release-install.sh --tag v0.2.0
  --asset-dir /tmp/engram-no-such-assets --json` failed with
  `GITHUB_REPOSITORY/--repo must not be empty`.
- `scripts/verify-published-release-install.sh --repo '' --tag v0.2.0
  --asset-dir /tmp/engram-no-such-assets --json` failed with
  `GITHUB_REPOSITORY/--repo must not be empty`.
- `scripts/verify-published-release-install.sh --tag ''
  --asset-dir /tmp/engram-no-such-assets --json` failed with `--tag must not be empty`.
- `scripts/verify-published-release-install.sh --tag v0.2.0 --host-triple ''
  --asset-dir /tmp/engram-no-such-assets --json` failed with
  `--host-triple must not be empty`.
- `scripts/verify-published-release-install.sh --tag v0.2.0 --expected-git-head ''
  --asset-dir /tmp/engram-no-such-assets --json` failed with
  `--expected-git-head must not be empty`.
- `scripts/verify-published-release-install.sh --tag v0.2.0 --asset-dir '' --json` failed with
  `--asset-dir must not be empty`.
- `scripts/verify-published-release-install.sh --tag v0.2.0
  --asset-dir /tmp/engram-no-such-assets --json` still accepted the non-empty local asset
  directory selector and reached the intended local validation error:
  `asset directory does not exist: /tmp/engram-no-such-assets`.

## Published Release Verifier Workspace Defaults Guard

`scripts/verify-published-release-install.sh` now parses help and explicit selectors before it
discovers workspace package metadata, the current Rust host triple, or the current Git head. After
parsing, the verifier explicitly requires `cargo`, `rustc`, `git`, and `jq`; then it validates that
`cargo pkgid --locked -p engram-cli` returned an `x.y.z` package version with an optional
prerelease suffix before defaulting `--tag`, deriving archive/checksum asset names, or inspecting
release metadata.

This keeps post-publish verification fail-closed when local tool discovery is unavailable or
malformed, while still allowing operators to read `--help` without a complete Rust/Git toolchain in
`PATH`.

Targeted validation for this guard on a development diff:

- `bash -n scripts/verify-published-release-install.sh`
- `env PATH=/usr/bin:/bin scripts/verify-published-release-install.sh --help` printed usage and
  exited `0` without requiring Cargo.
- `env PATH=/usr/bin:/bin scripts/verify-published-release-install.sh
  --asset-dir /tmp/engram-no-such-assets --json` failed before package metadata discovery with
  `required tool is missing: cargo`.
- With a temporary `cargo` wrapper returning `file:///tmp/engram#not-a-version`,
  `scripts/verify-published-release-install.sh --asset-dir /tmp/engram-no-such-assets --json`
  failed before host-triple discovery, asset-dir validation, or release checks with
  `workspace package version must be x.y.z with an optional prerelease suffix, got not-a-version`.
- With a temporary `cargo` wrapper exiting nonzero, the same verifier command failed with
  `could not determine workspace package version for engram-cli`.
- With the current workspace package metadata,
  `scripts/verify-published-release-install.sh --asset-dir /tmp/engram-no-such-assets --json`
  still reached the intended local validation error:
  `asset directory does not exist: /tmp/engram-no-such-assets`.
- With a temporary `rustc -vV` wrapper omitting `host:`, the verifier still failed before archive
  naming with `host triple could not be determined from rustc -vV; pass --host-triple explicitly`.

This is development-diff validation on top of head `f9c998e`; after this guard is committed, rerun
exact-head hosted CI and the GA gate before tag, publish, or Homebrew tap update.

## Published Release Verifier Host Triple Discovery Guard

`scripts/verify-published-release-install.sh` now validates the final host triple before deriving
release archive and checksum names. A missing host line from `rustc -vV` fails with an explicit
message that asks the operator to pass `--host-triple`; malformed auto-detected or overridden host
triples fail before GitHub release metadata inspection, asset download, or local asset validation.

This prevents final published-install proof from looking for ambiguous asset names such as
`engram-<version>-.tar.gz` or syntactically invalid host archives when local Rust toolchain output is
unexpected.

Targeted validation for this guard on a development diff:

- `bash -n scripts/verify-published-release-install.sh`
- With a fake `rustc -vV` output that omitted `host:`, `scripts/verify-published-release-install.sh
  --tag v0.2.0 --asset-dir /tmp/engram-no-such-assets --json` failed with
  `host triple could not be determined from rustc -vV; pass --host-triple explicitly`.
- With a fake `rustc -vV` output containing `host: bad`, the same verifier command failed with
  `--host-triple must be a Rust target triple, got bad`.
- `scripts/verify-published-release-install.sh --tag v0.2.0 --host-triple bad
  --asset-dir /tmp/engram-no-such-assets --json` failed with
  `--host-triple must be a Rust target triple, got bad`.
- `scripts/verify-published-release-install.sh --tag v0.2.0
  --asset-dir /tmp/engram-no-such-assets --json` still accepted the current Rust host triple and
  reached the intended local validation error:
  `asset directory does not exist: /tmp/engram-no-such-assets`.

This is development-diff validation on top of head `eae6924`; after this guard is committed, rerun
exact-head hosted CI and the GA gate before tag, publish, or Homebrew tap update.

## Local Package Host Triple Discovery Guard

`scripts/package-release.sh` and `scripts/package-install-smoke.sh` now validate the auto-detected
Rust host triple before deriving local release archive/checksum paths. A missing host line from
`rustc -vV` fails before release binary builds, package extraction, or artifact writes; malformed
host triples fail before any archive name such as `engram-<version>-bad.tar.gz` can become release
evidence.

Targeted validation for this guard on a development diff:

- `bash -n scripts/package-release.sh scripts/package-install-smoke.sh
  scripts/verify-published-release-install.sh scripts/render-homebrew-formula.sh
  scripts/release-gate-report.sh`
- With a fake `rustc -vV` output that omitted `host:`, `scripts/package-release.sh` failed with
  `host triple could not be determined from rustc -vV`.
- With a fake `rustc -vV` output containing `host: bad`, `scripts/package-release.sh` failed with
  `host triple must be a Rust target triple, got bad`.
- With a fake `rustc -vV` output that omitted `host:`,
  `SKIP_PACKAGE_BUILD=1 ALLOW_PACKAGE_BUILD_SKIP=1
  DIST_DIR=/tmp/engram-no-such-assets ALLOW_PACKAGE_DIST_DIR_OVERRIDE=1
  scripts/package-install-smoke.sh` failed with
  `host triple could not be determined from rustc -vV`.
- With a fake `rustc -vV` output containing `host: bad`, the same install-smoke command failed
  with `host triple must be a Rust target triple, got bad`.
- With the current Rust host, `scripts/package-release.sh` still reached the intended dirty-tree
  guard, and `SKIP_PACKAGE_BUILD=1 ALLOW_PACKAGE_BUILD_SKIP=1
  DIST_DIR=/tmp/engram-no-such-assets ALLOW_PACKAGE_DIST_DIR_OVERRIDE=1
  scripts/package-install-smoke.sh` still reached the intended missing-tarball validation error.

This is development-diff validation on top of head `23f3e87`; after this guard is committed, rerun
exact-head hosted CI and the GA gate before tag, publish, or Homebrew tap update.

## Package Producer Workspace Package Version Guard

`scripts/package-release.sh` now validates the workspace package version returned by
`cargo pkgid --locked -p engram-cli` before deriving the release notes path, archive name, staging
directory, manifest version, tarball path, or checksum path. The package producer now fails closed
if Cargo cannot resolve `engram-cli`, or if the discovered package version is not `x.y.z` with an
optional prerelease suffix.

This closes a local release-packaging gap where malformed package metadata could have been copied
into artifact names such as `engram-not-a-version-<host>.tar.gz` before the release gate had a
chance to reject it.

Targeted validation for this guard on a development diff:

- `bash -n scripts/package-release.sh scripts/package-install-smoke.sh
  scripts/render-homebrew-formula.sh scripts/release-gate-report.sh
  scripts/verify-published-release-install.sh scripts/verify-hosted-ci-prestep-blocker.sh`
- With a temporary `cargo` wrapper returning `file:///tmp/engram#not-a-version`,
  `scripts/package-release.sh` failed before host-triple discovery, release binary builds, or
  artifact writes with
  `workspace package version must be x.y.z with an optional prerelease suffix, got not-a-version`.
- With a temporary `cargo` wrapper exiting nonzero, `scripts/package-release.sh` failed with
  `could not determine workspace package version for engram-cli`.
- With the current workspace package metadata, `scripts/package-release.sh` still reached the
  intended tracked-worktree guard before building or writing release assets:
  `tracked working-tree or index changes are present; commit changes first`.

This is development-diff validation on top of head `93cd113`; after this guard is committed, rerun
exact-head hosted CI and the GA gate before tag, publish, or Homebrew tap update.

## Package Install Smoke Workspace Package Version Guard

`scripts/package-install-smoke.sh` now captures `cargo pkgid --locked -p engram-cli` explicitly and
validates the discovered workspace package version before deriving the local release archive name,
tarball path, or checksum path. This closes the package-consumer side of the workspace metadata
gap so malformed package versions cannot produce smoke-test paths such as
`engram-not-a-version-<host>.tar.gz`.

Targeted validation for this guard on a development diff:

- `bash -n scripts/package-install-smoke.sh scripts/package-release.sh
  scripts/render-homebrew-formula.sh scripts/release-gate-report.sh
  scripts/verify-published-release-install.sh`
- With a temporary `cargo` wrapper returning `file:///tmp/engram#not-a-version`,
  `SKIP_PACKAGE_BUILD=1 ALLOW_PACKAGE_BUILD_SKIP=1
  DIST_DIR=/tmp/engram-no-such-assets ALLOW_PACKAGE_DIST_DIR_OVERRIDE=1
  scripts/package-install-smoke.sh` failed before host-triple discovery or asset lookup with
  `workspace package version must be x.y.z with an optional prerelease suffix, got not-a-version`.
- With a temporary `cargo` wrapper exiting nonzero, the same install-smoke command failed with
  `could not determine workspace package version for engram-cli`.
- With the current workspace package metadata, the same install-smoke command still reached the
  intended missing-tarball validation error:
  `release tarball not found at /tmp/engram-no-such-assets/engram-0.2.0-aarch64-apple-darwin.tar.gz`.

This is development-diff validation on top of head `44bd867`; after this guard is committed, rerun
exact-head hosted CI and the GA gate before tag, publish, or Homebrew tap update.

## Homebrew Workspace Package Version Guard

`scripts/render-homebrew-formula.sh` now captures `cargo pkgid --locked -p engram-cli` explicitly
and validates the discovered workspace package version before deriving the default GitHub release
URL base, release archive name, tarball path, checksum path, or formula URL. This closes the
Homebrew consumer side of the workspace metadata gap so malformed package versions cannot produce
formula evidence for paths such as `engram-not-a-version-<host>.tar.gz` or release URLs such as
`/download/vnot-a-version`.

Targeted validation for this guard on a development diff:

- `bash -n scripts/render-homebrew-formula.sh scripts/package-release.sh
  scripts/package-install-smoke.sh scripts/release-gate-report.sh
  scripts/verify-published-release-install.sh`
- With a temporary `cargo` wrapper returning `file:///tmp/engram#not-a-version`,
  `ALLOW_HOMEBREW_DIST_DIR_OVERRIDE=1 DIST_DIR=/tmp/engram-no-such-assets
  scripts/render-homebrew-formula.sh` failed before host-triple discovery, release URL validation,
  or asset lookup with
  `workspace package version must be x.y.z with an optional prerelease suffix, got not-a-version`.
- With a temporary `cargo` wrapper exiting nonzero, the same renderer command failed with
  `could not determine workspace package version for engram-cli`.
- With the current workspace package metadata, the same renderer command still reached the intended
  missing-tarball validation error:
  `release tarball not found at /tmp/engram-no-such-assets/engram-0.2.0-aarch64-apple-darwin.tar.gz`.

This is development-diff validation on top of head `eba0d06`; after this guard is committed, rerun
exact-head hosted CI and the GA gate before tag, publish, or Homebrew tap update.

## Homebrew Host Triple Discovery Guard

`scripts/render-homebrew-formula.sh` now validates the auto-detected Rust host triple before
deriving Homebrew release archive/checksum paths. The renderer already default-denies
non-default `HOMEBREW_HOST_TRIPLE` archive selection; this additional guard closes the default
toolchain-discovery path so malformed `rustc -vV` output cannot construct names such as
`engram-<version>-.tar.gz` before failing.

Targeted validation for this guard on a development diff:

- `bash -n scripts/package-release.sh scripts/package-install-smoke.sh
  scripts/verify-published-release-install.sh scripts/render-homebrew-formula.sh
  scripts/release-gate-report.sh`
- With a fake `rustc -vV` output that omitted `host:`,
  `ALLOW_HOMEBREW_DIST_DIR_OVERRIDE=1 DIST_DIR=/tmp/engram-no-such-assets
  scripts/render-homebrew-formula.sh` failed with
  `host triple could not be determined from rustc -vV`.
- With a fake `rustc -vV` output containing `host: bad`, the same renderer command failed with
  `host triple must be a Rust target triple, got bad`.
- With the current Rust host,
  `ALLOW_HOMEBREW_DIST_DIR_OVERRIDE=1 DIST_DIR=/tmp/engram-no-such-assets
  scripts/render-homebrew-formula.sh` still reached the intended missing-tarball validation error:
  `release tarball not found at /tmp/engram-no-such-assets/engram-0.2.0-aarch64-apple-darwin.tar.gz`.

This is development-diff validation on top of head `4916508`; after this guard is committed, rerun
exact-head hosted CI and the GA gate before tag, publish, or Homebrew tap update.

## Hosted CI Event Override Guard

`scripts/release-gate-report.sh` and `scripts/verify-hosted-ci-prestep-blocker.sh` now fail closed
when release evidence tries to select a non-default GitHub Actions event without explicit
approval. The GA gate defaults to `push`, beta/fallback pre-step evidence defaults to
`pull_request`, and `EXPECTED_EVENT`, `--expected-event`, or `--event` values that differ from
those defaults require `ALLOW_EXPECTED_EVENT_OVERRIDE=1` before GitHub run discovery or
inspection. Explicit empty event values fail instead of silently falling back to the default.

This closes a release-evidence gap left by the earlier event-token validation: syntactically valid
but wrong event names can no longer make final GA evidence search the wrong class of hosted runs
unless the run is clearly marked as an explicit local rehearsal. The expected workflow and branch
guards still apply separately.

Targeted validation for this guard:

- `bash -n scripts/release-gate-report.sh scripts/verify-hosted-ci-prestep-blocker.sh`
- `EXPECTED_EVENT=pull_request scripts/release-gate-report.sh --target ga --hosted-run
  27451012359 --quick --allow-tracked-changes --json` failed before hosted run inspection with
  `EXPECTED_EVENT override requires explicit approval`.
- `ALLOW_EXPECTED_EVENT_OVERRIDE=yes EXPECTED_EVENT=pull_request
  scripts/release-gate-report.sh --target ga --hosted-run 27451012359 --quick
  --allow-tracked-changes --json` failed with
  `ALLOW_EXPECTED_EVENT_OVERRIDE must be 0 or 1, got yes`.
- `EXPECTED_EVENT= scripts/release-gate-report.sh --target ga --hosted-run 27451012359 --quick
  --allow-tracked-changes --json` failed before hosted run inspection with
  `EXPECTED_EVENT/--expected-event must not be empty`.
- `scripts/verify-hosted-ci-prestep-blocker.sh --event push --json` failed before GitHub run
  discovery with `EXPECTED_EVENT override requires explicit approval`.
- `ALLOW_EXPECTED_EVENT_OVERRIDE=yes scripts/verify-hosted-ci-prestep-blocker.sh --event push
  --json` failed with `ALLOW_EXPECTED_EVENT_OVERRIDE must be 0 or 1, got yes`.
- `EXPECTED_EVENT= scripts/verify-hosted-ci-prestep-blocker.sh --json` failed before GitHub run
  discovery with `EXPECTED_EVENT/--event must not be empty`.
- `scripts/release-gate-report.sh --target ga --hosted-run 27451012359 --quick
  --allow-tracked-changes --json` still accepted the default GA `push` event and reported
  `hosted_ci.state=passing`, `release_target.state=available`,
  `release_gate_state=evidence_incomplete`, and no release actions.

This is development-diff validation on top of head `b9617dd`; after this guard is committed, rerun
exact-head hosted CI and the GA gate before tag, publish, or Homebrew tap update.

## Release Gate Remote Branch Head Guard

`scripts/release-gate-report.sh` now checks the upstream remote branch head directly before release
evidence can proceed. The gate still requires `HEAD` to be `0 0` ahead/behind relative to its local
tracking ref, but it also resolves the branch's upstream remote/ref and compares `git ls-remote`
output with the local `HEAD`.

This keeps GA owner-review evidence from relying on a stale local `origin/main` ref. A checkout can
no longer claim synced-branch release evidence if the authoritative remote branch has advanced since
the last fetch. JSON evidence now reports `upstream.remote`, `upstream.remote_ref`,
`upstream.remote_head`, and `upstream.matches_remote_head`.

Branch-sync failure output now also carries the release-owner runbook guidance inline: fetch and
inspect local/remote commits, do not treat `git pull` as release approval, and rerun exact-head
hosted CI plus the gate after any reconciliation changes the release head.

Targeted validation for this guard:

- `bash -n scripts/release-gate-report.sh`
- In a temporary local clone with `main` one commit ahead of `origin/main`,
  `scripts/release-gate-report.sh --target ga --quick --json` failed before hosted CI, disk,
  package, or Homebrew evidence with `branch is not synced with origin/main: ahead=1 behind=0` and
  the stop-and-inspect hint.
- With a temporary `git` wrapper returning a mismatched SHA for the upstream branch,
  `scripts/release-gate-report.sh --target ga --hosted-run 27465173036 --quick
  --allow-tracked-changes --json` failed before hosted CI, disk, package, or Homebrew evidence with
  `branch is not synced with remote origin/main`.
- `scripts/release-gate-report.sh --target ga --hosted-run 27465173036 --quick
  --allow-tracked-changes --json` still accepted the current remote branch head and reported
  `upstream.matches_remote_head=true`, `hosted_ci.state=passing`, `release_target.state=available`,
  `release_gate_state=evidence_incomplete`, and no release actions.

This is development-diff validation on top of head `582d40d`; after this guard is committed, rerun
exact-head hosted CI and the GA gate before tag, publish, or Homebrew tap update.

## Release Gate Repository-State Failure JSON

`scripts/release-gate-report.sh --json` now emits structured failure evidence for early
repository-state preflights before exiting nonzero. Branch mismatch, branch sync, and tracked
working-tree/index failures still stop before release-target lookup, hosted CI verification, disk
preflight, generated-output inventory, package smoke, or Homebrew render, but automation can now
distinguish those release-head failures from a script crash.

The failure JSON uses `release_target.state=not_checked`, `hosted_ci.state=not_checked`,
`disk_space.state=not_checked`, `generated_outputs.state=not_checked`,
`generated_artifacts.state=not_checked`, and `release_actions_performed=false` because no release
evidence or release action has run yet.

Targeted validation for this guard:

- `bash -n scripts/release-gate-report.sh scripts/beta-release-gate-report.sh`
- On a dirty development tree, `scripts/release-gate-report.sh --target ga --hosted-run
  27483057249 --quick --json` failed with `release_gate_state=tracked_changes_present`,
  `failure.kind=tracked_changes_preflight`, all downstream release evidence states
  `not_checked`, and `release_actions_performed=false`.
- With `ALLOW_EXPECTED_BRANCH_OVERRIDE=1 EXPECTED_BRANCH=release/0.2`,
  `scripts/release-gate-report.sh --target ga --hosted-run 27483057249 --quick --json` failed
  with `release_gate_state=branch_mismatch`, `failure.kind=branch_preflight`, and no downstream
  release evidence.
- In a temporary local clone with `main` one commit ahead of `origin/main`,
  `scripts/release-gate-report.sh --target ga --quick --json` failed with
  `release_gate_state=branch_sync_required`, `failure.kind=branch_sync_preflight`,
  `upstream.ahead=1`, `upstream.behind=0`, the stop-and-inspect `git pull` warning, and
  `release_actions_performed=false`.

This is development-diff validation on top of head `2d46c40`; after this guard is committed, rerun
exact-head hosted CI and the GA gate before tag, publish, or Homebrew tap update.

## Release Gate Hosted CI Failure JSON

`scripts/release-gate-report.sh --json` now emits structured failure evidence for GA hosted CI
preflight failures before exiting nonzero. Hosted run discovery, run inspection, head/status/
conclusion/workflow/event checks, and expected job validation still stop before release-target
lookup, disk preflight, generated-output inventory, local CI, package smoke, or Homebrew render,
but automation can now distinguish those hosted-CI evidence failures from a script crash.

The failure JSON reports states such as `hosted_ci_inspection_failed`,
`hosted_ci_head_mismatch`, and `hosted_ci_workflow_mismatch`, includes the effective
`hosted_ci.repository`, expected workflow/event/jobs, the inspected run object when available,
and any parsed check rows available before failure. It keeps `release_target.state=not_checked`,
`disk_space.state=not_checked`, `generated_outputs.state=not_checked`,
`generated_artifacts.state=not_checked`, and `release_actions_performed=false` because no
downstream release evidence or release action has run yet.

Targeted validation for this guard:

- `bash -n scripts/release-gate-report.sh scripts/beta-release-gate-report.sh`
- `scripts/release-gate-report.sh --target ga --hosted-run 27483057249 --quick
  --allow-tracked-changes --json` failed with
  `release_gate_state=hosted_ci_head_mismatch`, `failure.kind=hosted_ci_preflight`, inspected run
  `27483057249`, downstream release evidence states `not_checked`, and
  `release_actions_performed=false`.
- `ALLOW_EXPECTED_WORKFLOW_NAME_OVERRIDE=1 EXPECTED_WORKFLOW_NAME=Release
  scripts/release-gate-report.sh --target ga --hosted-run 27483700305 --quick
  --allow-tracked-changes --json` failed with
  `release_gate_state=hosted_ci_workflow_mismatch`, expected workflow `Release`, inspected
  workflow `CI`, downstream release evidence states `not_checked`, and no release actions.
- `scripts/release-gate-report.sh --target ga --hosted-run 0 --quick
  --allow-tracked-changes --json` failed with
  `release_gate_state=hosted_ci_inspection_failed`, `hosted_ci.run=null`,
  `hosted_ci.checks=[]`, downstream release evidence states `not_checked`, and no release
  actions.
- `scripts/release-gate-report.sh --target ga --hosted-run 27483700305 --quick
  --allow-tracked-changes --json` still passed for exact head `0f650e8`, with hosted CI passing,
  release target `v0.2.0` available, generated outputs reported as read-only cleanup evidence,
  `release_gate_state=evidence_incomplete`, and no release actions.

This is development-diff validation on top of head `0f650e8`; after this guard is committed, rerun
exact-head hosted CI and the GA gate before tag, publish, or Homebrew tap update.

## Release Gate Configuration Failure JSON

`scripts/release-gate-report.sh --json` now emits structured failure evidence for early
configuration and approval-input preflights before exiting nonzero. Malformed CLI flags,
non-default release-gate override attempts without approval, malformed release selectors,
repository overrides, and disk-threshold overrides still stop before branch/repository-state
inspection, release-target lookup, hosted CI verification, disk preflight, generated-output
inventory, local CI, package smoke, or Homebrew render, but automation can now distinguish those
configuration failures from a script crash.

The failure JSON uses `release_gate_state=configuration_preflight_failed` and
`failure.kind=configuration_preflight`. It keeps `release_target.state=not_checked`,
`hosted_ci.state=not_checked`, `disk_space.state=not_checked`,
`generated_outputs.state=not_checked`, `generated_artifacts.state=not_checked`,
`local_ci=not_run`, `package_install_smoke=not_run`, and `release_actions_performed=false`
because no release evidence or release action has run yet.

Targeted validation for this guard:

- `bash -n scripts/release-gate-report.sh scripts/beta-release-gate-report.sh`
- `git diff --check`
- `scripts/release-gate-report.sh --bogus --json` failed with
  `release_gate_state=configuration_preflight_failed`,
  `failure.kind=configuration_preflight`, all downstream release evidence states
  `not_checked`, and no release actions.
- `EXPECTED_EVENT=workflow_dispatch scripts/release-gate-report.sh --target ga --quick --json`
  failed with `release_gate_state=configuration_preflight_failed`,
  `failure.kind=configuration_preflight`, downstream release evidence states `not_checked`, and
  no release actions.
- `RELEASE_GATE_MIN_FREE_KIB=abc scripts/release-gate-report.sh --target ga --quick --json`
  failed with `release_gate_state=configuration_preflight_failed`,
  `failure.kind=configuration_preflight`, downstream release evidence states `not_checked`, and
  no release actions.
- `RELEASE_REPOSITORY=example/engram scripts/release-gate-report.sh --target ga --quick --json`
  failed with `release_gate_state=configuration_preflight_failed`,
  `failure.kind=configuration_preflight`, downstream release evidence states `not_checked`, and
  no release actions.
- `scripts/release-gate-report.sh --target ga --hosted-run 27484385982 --quick
  --allow-tracked-changes --json` still passed for exact head `3bb9810`, with hosted CI passing,
  release target `v0.2.0` available, generated outputs reported as read-only cleanup evidence,
  `release_gate_state=evidence_incomplete`, and no release actions.

After this guard was committed as `a2e18d9`, exact-head hosted CI run `27485035688` passed for
Check, Format, Docs, Clippy, and Test. Post-CI exact-head GA gate assertions against that run also
passed for the read-only quick gate and the default full generated-output cleanup preflight. The
full gate remains intentionally blocked by the same three stale generated outputs until cleanup is
explicitly approved and executed with the release-owner runbook checks.

## Release Gate Structured Action Evidence

`scripts/release-gate-report.sh --json` now keeps the existing
`release_actions_performed=false` compatibility field and also emits an `actions_performed` object
on every release-gate JSON path. The object currently reports
`release_actions=false`, `git_tag=false`, `github_release=false`,
`package_asset_upload=false`, `homebrew_tap_update=false`, and
`generated_output_cleanup=false`.

This makes release-gate no-side-effect evidence machine-checkable across early configuration
failures, repository-state failures, quick evidence reports, hosted-CI failures, disk/generated
output preflights, and final owner-review evidence reports. It does not mark validation-only work
such as local CI or package smoke as release publication; those remain represented by their
existing state fields.

Development-diff validation for this structured action evidence:

- `bash -n scripts/release-gate-report.sh scripts/beta-release-gate-report.sh`
- `git diff --check`
- `scripts/release-gate-report.sh --bogus --json` failed with
  `release_gate_state=configuration_preflight_failed`, `failure.kind=configuration_preflight`,
  `release_actions_performed=false`, and every `actions_performed` value false.
- `scripts/release-gate-report.sh --target ga --hosted-run 27486939343 --quick --json` failed
  on the intentional development diff with `release_gate_state=tracked_changes_present`,
  `failure.kind=tracked_changes_preflight`, `tracked_changes_present=true`,
  `release_actions_performed=false`, and every `actions_performed` value false.
- `scripts/release-gate-report.sh --target ga --hosted-run 27486939343 --quick
  --allow-tracked-changes --json` still emitted partial exact-head evidence for `fa7fbf1`, with
  hosted CI passing, release target available, generated outputs requiring cleanup,
  `release_gate_state=evidence_incomplete`, `release_actions_performed=false`, and every
  `actions_performed` value false.
- `scripts/release-gate-report.sh --target ga --hosted-run 27486939343
  --allow-tracked-changes --json` still failed before local proof on the stale generated outputs
  with `release_gate_state=generated_outputs_cleanup_required`,
  `failure.kind=generated_outputs_preflight`, `disk_space.state=passed`,
  `generated_artifacts.state=not_checked`, `release_actions_performed=false`, and every
  `actions_performed` value false.

## Hosted CI Pre-Step Structured Action Evidence

`scripts/verify-hosted-ci-prestep-blocker.sh --json` now keeps
`hosted_ci_fallback_accepted=false` and `release_actions_performed=false`, and also emits an
`actions_performed` object on successful pre-step-blocker verification. The object reports
`hosted_ci_fallback_acceptance=false`, `release_actions=false`, `git_tag=false`,
`github_release=false`, `package_asset_upload=false`, `homebrew_tap_update=false`, and
`generated_output_cleanup=false`.

The verifier also emits structured JSON for configuration preflight failures in `--json` mode.
Those failures report `verification_state=configuration_preflight_failed`,
`condition_verified=false`, `failure.kind=configuration_preflight`,
`hosted_ci_fallback_accepted=false`, `release_actions_performed=false`, and the same all-false
`actions_performed` object. Later hosted-run evidence mismatches are still outside this
configuration-preflight class and continue to emit no success JSON.

This keeps the hosted-CI fallback verifier aligned with the GA release-gate JSON contract: the
verifier can prove that a hosted run failed before workflow steps ran, but it still does not accept
the fallback, mark a PR ready, merge, tag, publish, upload assets, update Homebrew, or clean release
outputs.

Development-diff validation for this verifier evidence:

- `bash -n scripts/verify-hosted-ci-prestep-blocker.sh`
- `git diff --check`
- `EXPECTED_HEAD_SHA=0de4f2745ba627266200b8f6e03d1b06edb2dc82
  scripts/verify-hosted-ci-prestep-blocker.sh --json 27190538964` passed against the historical
  pre-step-blocker run with `condition_verified=true`, `hosted_ci_fallback_accepted=false`,
  `release_actions_performed=false`, and every `actions_performed` value false.
- `EXPECTED_HEAD_SHA=0000000000000000000000000000000000000000
  scripts/verify-hosted-ci-prestep-blocker.sh --json 27190538964` failed closed with a run-head
  mismatch and emitted no success JSON.
- `scripts/verify-hosted-ci-prestep-blocker.sh --bogus --json` failed closed with
  `verification_state=configuration_preflight_failed`, `condition_verified=false`,
  `failure.kind=configuration_preflight`, `hosted_ci_fallback_accepted=false`,
  `release_actions_performed=false`, and every `actions_performed` value false.
- `GITHUB_REPOSITORY=example/engram scripts/verify-hosted-ci-prestep-blocker.sh --json`
  failed closed with the same structured configuration-preflight JSON and no release actions.
- `EXPECTED_WORKFLOW_NAME=Release scripts/verify-hosted-ci-prestep-blocker.sh --json` and
  `EXPECTED_EVENT=schedule scripts/verify-hosted-ci-prestep-blocker.sh --json` failed closed with
  structured configuration-preflight JSON and no release actions.
- `EXPECTED_HEAD_SHA=bad scripts/verify-hosted-ci-prestep-blocker.sh --json` failed closed with
  structured configuration-preflight JSON and no release actions.
- `EXPECTED_HEAD_SHA=0000000000000000000000000000000000000000
  scripts/verify-hosted-ci-prestep-blocker.sh --json 27490518203` failed after hosted-run
  inspection with a run-head mismatch, stdout empty, and no configuration-preflight JSON.

## Published Verifier Structured Action Evidence

`scripts/verify-published-release-install.sh --json` now keeps the existing
`release_actions_performed=false` compatibility field and also emits an `actions_performed`
object on successful asset-install verification. The object reports `release_actions=false`,
`git_tag=false`, `github_release=false`, `package_asset_upload=false`,
`homebrew_tap_update=false`, and `generated_output_cleanup=false`.

The verifier also emits structured JSON for configuration preflight failures in `--json` mode.
Those failures report `verification_state=configuration_preflight_failed`,
`failure.kind=configuration_preflight`, `asset_install_verified=false`,
`published_install_verified=false`, `release_actions_performed=false`, and the same all-false
`actions_performed` object.

This keeps local asset-dir rehearsal evidence and future post-publish verifier evidence aligned
with the release-gate no-side-effect JSON contract. Local `--asset-dir` evidence can prove
`asset_install_verified=true`, but it still reports `published_install_verified=false` and no
release actions because no GitHub release assets were downloaded or mutated.

Development-diff validation for this verifier evidence:

- `bash -n scripts/verify-published-release-install.sh`
- `scripts/verify-published-release-install.sh --tag v0.2.0 --asset-dir dist
  --expected-git-head 6a0d5c32b0ae3ad40835116ece1386c0428d3222 --json` passed with
  `assets.source=asset_dir`, `assets.downloaded=false`, `install_smoke=passed`,
  `asset_install_verified=true`, `published_install_verified=false`,
  `release_actions_performed=false`, and every `actions_performed` value false.
- `scripts/verify-published-release-install.sh --tag v0.2.0 --asset-dir dist
  --expected-git-head 0000000000000000000000000000000000000000 --json` failed closed with a
  manifest Git-head mismatch and emitted no success JSON.
- `scripts/verify-published-release-install.sh --bogus --json` failed closed with
  `verification_state=configuration_preflight_failed`, `failure.kind=configuration_preflight`,
  `asset_install_verified=false`, `published_install_verified=false`,
  `release_actions_performed=false`, and every `actions_performed` value false.
- `GITHUB_REPOSITORY=example/engram scripts/verify-published-release-install.sh --tag v0.2.0
  --json` failed closed with the same structured configuration-preflight JSON and no release
  actions.

## Hosted CI Repository Anchor Guard

`scripts/release-gate-report.sh` now passes the effective release repository into hosted CI run
discovery, hosted CI run inspection, beta PR evidence, and hosted CI pre-step fallback verification.
`scripts/verify-hosted-ci-prestep-blocker.sh` now accepts `--repo <owner/name>` or
`GITHUB_REPOSITORY`, defaults to `ymeiri/engram`, and rejects non-default or malformed repository
targets unless `ALLOW_RELEASE_REPOSITORY_OVERRIDE=1` is set for an explicit local rehearsal.

This keeps hosted CI, beta PR, and pre-step fallback evidence from depending on the ambient
repository inferred by `gh`. Release-gate JSON now reports `hosted_ci.repository`; pre-step verifier
JSON reports both top-level `repo` and `run.repo`. No release action is performed by either script.

Targeted validation for this guard:

- `bash -n scripts/release-gate-report.sh scripts/verify-hosted-ci-prestep-blocker.sh`
- `GITHUB_REPOSITORY=example/engram scripts/verify-hosted-ci-prestep-blocker.sh --json` failed
  before GitHub run discovery with `release repository override requires explicit approval`.
- `ALLOW_RELEASE_REPOSITORY_OVERRIDE=yes GITHUB_REPOSITORY=ymeiri/engram
  scripts/verify-hosted-ci-prestep-blocker.sh --json` failed with
  `ALLOW_RELEASE_REPOSITORY_OVERRIDE must be 0 or 1, got yes`.
- `ALLOW_RELEASE_REPOSITORY_OVERRIDE=1 GITHUB_REPOSITORY=bad/repo/extra
  scripts/verify-hosted-ci-prestep-blocker.sh --json` failed with
  `release repository must be owner/name, got bad/repo/extra`.
- `GITHUB_REPOSITORY=ymeiri/engram EXPECTED_HEAD_SHA=69aebc5b4a947b6f4e20526038a6d3b45d66b754
  EXPECTED_EVENT=push ALLOW_EXPECTED_EVENT_OVERRIDE=1
  scripts/verify-hosted-ci-prestep-blocker.sh 27465917208 --json` reached the explicit
  `ymeiri/engram` run and failed with `run conclusion is not failure: success`, as expected for a
  passing run.
- `scripts/release-gate-report.sh --target ga --hosted-run 27465917208 --quick
  --allow-tracked-changes --json` passed on head `69aebc5`, reported
  `hosted_ci.repository=ymeiri/engram`, `hosted_ci.state=passing`,
  `release_target.state=available`, `release_gate_state=evidence_incomplete`, and no release
  actions.

This is development-diff validation on top of head `69aebc5`; after this guard is committed, rerun
exact-head hosted CI and the GA gate before tag, publish, or Homebrew tap update.

## Release Package Output Overwrite Guard

`scripts/package-release.sh` now refuses to overwrite an existing release archive or adjacent
checksum at the expected output path. The guard runs before the release binary build, so stale
`dist/engram-<version>-<host>.tar.gz` or `.sha256` files stop packaging before new local evidence
can replace old artifact evidence. Overwrite remains available only for explicit local rehearsals
with `ALLOW_PACKAGE_ASSET_OVERWRITE=1`.

This protects final owner-review packaging from accidentally replacing a previous archive/checksum
while retaining the existing default `dist` path and `ALLOW_PACKAGE_DIST_DIR_OVERRIDE=1` rehearsal
escape hatch.

Targeted validation for this guard:

- `bash -n scripts/package-release.sh`
- `ALLOW_PACKAGE_ASSET_OVERWRITE=yes scripts/package-release.sh` failed before release binary
  build with `ALLOW_PACKAGE_ASSET_OVERWRITE must be 0 or 1, got yes`.
- With `ALLOW_PACKAGE_DIST_DIR_OVERRIDE=1 DIST_DIR=<temp>` and a placeholder expected archive
  already present, `scripts/package-release.sh` failed before release binary build with
  `release package output already exists; refusing to overwrite`.
- `scripts/package-release.sh` also failed before release binary build on the current checkout
  because the default `dist/` directory still contains stale
  `engram-0.2.0-aarch64-apple-darwin.tar.gz` and `.sha256` outputs from prior rehearsals.

This is development-diff validation on top of head `fbab2b3`; after this guard is committed, rerun
exact-head hosted CI and the GA gate before tag, publish, or Homebrew tap update.

## Release Package Final Output Staging

`scripts/package-release.sh` now writes the release archive and checksum as hidden temporary files
under the selected `dist` directory, computes the checksum from the staged archive, and verifies the
final checksum after moving both files into place. If archive creation, checksum creation, or final
checksum verification fails, the cleanup trap removes temporary files and any final outputs created
by that failed run.

This keeps final owner-review packaging from leaving a partial archive or checksum at the canonical
`dist/engram-<version>-<host>.tar.gz` paths after an interrupted or failed artifact write.

Development-diff validation for this guard:

- `bash -n scripts/package-release.sh`
- With a temporary failing `tar` wrapper and `DIST_DIR=<temp>`, `scripts/package-release.sh`
  failed during archive creation and left no final archive/checksum outputs in the temp `dist`
  directory.
- With `ALLOW_TRACKED_CHANGES=1 ALLOW_PACKAGE_DIST_DIR_OVERRIDE=1 DIST_DIR=<temp>`,
  `scripts/package-release.sh` created a local rehearsal archive/checksum and the final checksum
  verification passed.

This is development-diff validation on top of head `f0397a0`; after this guard is committed, rerun
exact-head hosted CI and the GA gate before tag, publish, or Homebrew tap update.

## Release Gate Generated Output Preflight

`scripts/release-gate-report.sh` now fails a full local-proof preflight when a generated output
that the gate would write already exists. The preflight runs after disk space passes and before
local CI, package/install smoke, or Homebrew formula validation, so stale
`dist/engram-<version>-<host>.tar.gz`, `.sha256`, or `dist/homebrew/Formula/engram.rb` files
cannot be silently reused or overwritten by final owner-review evidence.

The structured failure state is `generated_outputs_cleanup_required`, with
`failure.kind=generated_outputs_preflight`, `generated_artifacts.state=not_checked`,
`ready_for_release_owner_review=false`, and remaining actions limited to cleanup approval plus
rerunning the full gate.

The generated-output inventory now also fingerprints existing regular files with `file_type`,
`size_bytes`, and `sha256` in both JSON and text output. This keeps cleanup approval evidence
non-destructive while still identifying exactly which stale archive, checksum, or formula files
blocked final local proof. Missing outputs and non-regular paths keep null fingerprints rather than
being treated as publishable artifact evidence.

The post-run generated-artifact proof uses the same fingerprint fields for successful full-gate
artifacts. Final owner-review evidence should require every required archive, checksum, and formula
artifact to be a regular file with nonzero `size_bytes` and a 64-character SHA-256 `sha256`, so
publication reuses an auditable artifact set instead of accepting path existence alone. The gate now
keeps owner-review readiness blocked with `generated_artifacts_missing` if a required post-run
artifact is missing, non-regular, empty, or unfingerprinted.

Development-diff validation for this guard:

- `bash -n scripts/release-gate-report.sh scripts/beta-release-gate-report.sh`
- Extracted and executed the real `append_generated_artifact` function in a temporary harness:
  required regular nonempty files remain `present` with a 64-character SHA-256 fingerprint, while
  missing paths, symlinks, directories, and zero-byte files set `generated_artifacts_state=missing`
  with a path-specific reason.
- `scripts/release-gate-report.sh --target ga --hosted-run 27477567656 --quick
  --allow-tracked-changes --json` still passed as partial evidence with
  `release_gate_state=evidence_incomplete`, `generated_outputs.state=cleanup_required`, and all
  generated outputs reporting `will_write=false`; existing generated-output files report
  `file_type=file`, nonzero `size_bytes`, and 64-character SHA-256 fingerprints.
- `scripts/release-gate-report.sh --target ga --hosted-run 27477567656
  --allow-tracked-changes --json` failed before local CI, package, or Homebrew validation with
  `release_gate_state=generated_outputs_cleanup_required`, `failure.kind=generated_outputs_preflight`,
  `disk_space.state=passed`, `generated_artifacts.state=not_checked`,
  `ready_for_release_owner_review=false`, and all three stale generated outputs reporting
  `exists=true`, `will_write=true`, `file_type=file`, nonzero `size_bytes`, and SHA-256
  fingerprints.
- `scripts/release-gate-report.sh --target ga --release-version 0.2.0-beta.2 --hosted-run
  27477567656 --quick --allow-tracked-changes --json` failed with
  `release_gate_state=release_target_unavailable`, `failure.kind=release_target_preflight`, and
  `ready_for_release_owner_review=false`.

After this guard was committed as `b970c07`, exact-head hosted CI run `27478418090` passed for
Check, Format, Docs, Clippy, and Test. Post-CI exact-head GA gate assertions against that run also
passed for the read-only quick gate, the default full generated-output cleanup preflight, and the
`v0.2.0-beta.2` release-target conflict rehearsal. The full gate remains intentionally blocked by
stale generated outputs until cleanup is explicitly approved.

After the cleanup runbook was tightened as `8396e92`, exact-head hosted CI run `27479860353`
passed for Check, Format, Docs, Clippy, and Test. Post-CI exact-head GA gate assertions against
that run also passed for the read-only quick gate, the default full generated-output cleanup
preflight, and the `v0.2.0-beta.2` release-target conflict rehearsal. The full gate remains
intentionally blocked by stale generated outputs until cleanup is explicitly approved and executed
with the fingerprint checks in the release-owner runbook.

After cleanup checkpoint evidence was refreshed as `e86d795`, exact-head hosted CI run
`27480555463` passed for Check, Format, Docs, Clippy, and Test. Post-CI exact-head GA gate
assertions against that run also passed for the read-only quick gate, the default full
generated-output cleanup preflight, and the `v0.2.0-beta.2` release-target conflict rehearsal. The
full gate remains intentionally blocked by the same three fingerprinted stale generated outputs
until cleanup is explicitly approved and executed with the release-owner runbook checks.

After branch-sync handling was clarified as `0aa009a`, exact-head hosted CI run `27481797605`
passed for Check, Format, Docs, Clippy, and Test. Post-CI exact-head GA gate assertions against
that run also passed for the read-only quick gate and the default full generated-output cleanup
preflight. The full gate remains intentionally blocked by the same three fingerprinted stale
generated outputs until cleanup is explicitly approved and executed with the release-owner
runbook checks.

After configuration-preflight JSON was added as `a2e18d9`, exact-head hosted CI run `27485035688`
passed for Check, Format, Docs, Clippy, and Test. Post-CI exact-head GA gate assertions against
that run also passed for the read-only quick gate and the default full generated-output cleanup
preflight. The full gate remains intentionally blocked by the same three fingerprinted stale
generated outputs until cleanup is explicitly approved and executed with the release-owner
runbook checks.

After published-verifier configuration JSON was added as `fa90465`, exact-head hosted CI run
`27489912065` passed for Check, Format, Docs, Clippy, and Test. Post-CI exact-head GA gate
assertions against that run also passed for the read-only quick gate and the default full
generated-output cleanup preflight. The full gate remains intentionally blocked by the same three
fingerprinted stale generated outputs until cleanup is explicitly approved and executed with the
release-owner runbook checks.

After hosted-CI verifier configuration JSON was added as `1ddc317`, exact-head hosted CI run
`27491186766` passed for Check, Format, Docs, Clippy, and Test. Post-CI exact-head GA gate
assertions against that run also passed for the read-only quick gate and the default full
generated-output cleanup preflight. The full gate still reports `release_target.state=available`,
`disk_space.state=passed`, `release_gate_state=generated_outputs_cleanup_required`,
`failure.kind=generated_outputs_preflight`, `generated_artifacts.state=not_checked`, and all
release action flags false. The blocking generated-output fingerprints are unchanged from the
release-owner runbook, so cleanup remains explicit owner-gated work before final local proof.

## Validation Run

- `git fetch --tags --prune origin`
- `git rev-list --left-right --count main...origin/main`
- `gh run view 27491186766 --repo ymeiri/engram --json
  databaseId,headSha,status,conclusion,workflowName,event,jobs,url`
- `scripts/release-gate-report.sh --target ga --hosted-run 27491186766 --quick --json`
- `scripts/release-gate-report.sh --target ga --hosted-run 27491186766 --json` (expected
  generated-output cleanup preflight failure with no release actions)
- `gh run view 27363378532 --json status,conclusion,headSha,url,jobs`
- `gh release list --repo ymeiri/engram --limit 20`
- `gh release view v0.2.0-beta.1 --repo ymeiri/engram ...`
- `gh release view v0.2.0-beta.2 --repo ymeiri/engram ...`
- `cargo metadata --no-deps --format-version 1`
- `cargo fmt --all --check`
- `git diff --check`
- `bash -n scripts/native-claude-gate-preflight.sh`
- `scripts/native-claude-gate-preflight.sh --expected-branch dev --json`
  failed before binary execution with `EXPECTED_BRANCH override requires explicit native Claude
  approval`
- `ALLOW_NATIVE_CLAUDE_BRANCH_OVERRIDE=yes scripts/native-claude-gate-preflight.sh --json`
  failed with `ALLOW_NATIVE_CLAUDE_BRANCH_OVERRIDE must be 0 or 1`
- `CLAUDE_BIN=/tmp/claude scripts/native-claude-gate-preflight.sh --json` failed with
  `CLAUDE_BIN override requires explicit native Claude approval`
- `CLAUDE_BIN= scripts/native-claude-gate-preflight.sh --json` failed with
  `CLAUDE_BIN must not be empty`
- `EXPECTED_CLAUDE_SHA256=abc scripts/native-claude-gate-preflight.sh --json` failed with
  `EXPECTED_CLAUDE_SHA256 must be a SHA-256 hex value`
- `EXPECTED_CLAUDE_VERSION=bad scripts/native-claude-gate-preflight.sh --json` failed with
  `Claude identity override requires explicit native Claude approval`
- `ENGRAM_BIN=/tmp/engram scripts/native-claude-gate-preflight.sh --json` failed with
  `ENGRAM_BIN override requires explicit native Claude approval`
- `ENGRAM_VAULT_PATH=/tmp/engram-vault scripts/native-claude-gate-preflight.sh --json`
  failed with `ENGRAM_VAULT_PATH override requires explicit native Claude approval`
- `ENGRAM_BIN= scripts/native-claude-gate-preflight.sh --json` failed with
  `ENGRAM_BIN must not be empty`
- `ENGRAM_VAULT_PATH= scripts/native-claude-gate-preflight.sh --json` failed with
  `ENGRAM_VAULT_PATH must not be empty`
- `ALLOW_NATIVE_CLAUDE_BRANCH_OVERRIDE=1 scripts/native-claude-gate-preflight.sh
  --expected-branch dev --allow-worktree-changes --json` reported the expected branch-mismatch
  blocker instead of failing on the approval guard
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
- `ALLOW_HOMEBREW_DIST_DIR_OVERRIDE=1 ALLOW_HOMEBREW_FORMULA_OUTPUT_OVERRIDE=1
  DIST_DIR=<temp> FORMULA_OUTPUT=<temp>/homebrew/Formula/engram.rb
  HOMEBREW_HOST_TRIPLE=aarch64-apple-darwin scripts/render-homebrew-formula.sh`
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
- `scripts/verify-published-release-install.sh` after the GitHub asset metadata guard with a mocked
  matching GitHub release and local `0.2.0` assets, expected
  `published_install_verified=true`, `assets.release_asset_list_verified=true`, and
  `assets.release_asset_digests_verified=true`
- `scripts/verify-published-release-install.sh` after the GitHub asset metadata guard with a mocked
  extra release asset, expected failure before download with the exact-asset-list error
- `scripts/verify-published-release-install.sh` after the GitHub asset metadata guard with a mocked
  bad GitHub asset digest, expected `GitHub asset digest mismatch`
- `scripts/verify-published-release-install.sh` after the published tag parity guard with a mocked
  matching GitHub release and local `0.2.0` assets, expected
  `local_tag_signature_verified=true`, `remote_tag.verified=true`, and
  `remote_tag.commit` matching the expected release head
- `scripts/verify-published-release-install.sh` after the published tag parity guard with a mocked
  local tag signature failure, expected failure before asset download during
  `verify local release tag signature`
- `scripts/verify-published-release-install.sh` after the published tag parity guard with a mocked
  remote tag object mismatch, expected failure before asset download with
  `remote tag object mismatch`
- `ALLOW_TRACKED_CHANGES=1 DIST_DIR=<temp> scripts/package-install-smoke.sh` after the
  package-release manifest build guard
- `ALLOW_TRACKED_CHANGES=1 ALLOW_PACKAGE_DIST_DIR_OVERRIDE=1 DIST_DIR=<temp>
  scripts/package-release.sh` after the package-release producer payload-hash guard, followed by
  `ALLOW_PACKAGE_BUILD_SKIP=1 DIST_DIR=<temp> SKIP_PACKAGE_BUILD=1
  EXPECTED_TRACKED_CHANGES_PRESENT=true scripts/package-install-smoke.sh`
- `SKIP_PACKAGE_BUILD=yes DIST_DIR=<temp> scripts/package-install-smoke.sh`, expected failure
  before package extraction with `SKIP_PACKAGE_BUILD must be 0 or 1`
- `ALLOW_PACKAGE_BUILD_SKIP=1 SKIP_PACKAGE_BUILD=1 DIST_DIR=<temp>
  EXPECTED_TRACKED_CHANGES_PRESENT=maybe scripts/package-install-smoke.sh`, expected failure
  before package extraction with `EXPECTED_TRACKED_CHANGES_PRESENT must be true or false`
- `scripts/package-release.sh` with a temporary `shasum` wrapper returning a second, mismatched
  `README.md` digest, expected failure with `manifest hash mismatch for README.md` and no tarball
  written
- `gh run watch 27417397670 --repo ymeiri/engram --exit-status --interval 30`
- `scripts/release-gate-report.sh --target ga --hosted-run 27417397670 --quick --json`
- `scripts/release-gate-report.sh --target ga --hosted-run 27417397670 --json` (expected
  disk-preflight failure with `release_gate_state=disk_space_cleanup_required`, no local
  CI/package/Homebrew steps, and no release actions)
- `ALLOW_TRACKED_CHANGES=1 ALLOW_PACKAGE_DIST_DIR_OVERRIDE=1 DIST_DIR=<temp>
  scripts/package-release.sh` after the Homebrew manifest identity guard
- `ALLOW_HOMEBREW_DIST_DIR_OVERRIDE=1 ALLOW_HOMEBREW_FORMULA_OUTPUT_OVERRIDE=1
  DIST_DIR=<temp> EXPECTED_TRACKED_CHANGES_PRESENT=true FORMULA_OUTPUT=<temp>/homebrew/Formula/engram.rb
  HOMEBREW_HOST_TRIPLE=aarch64-apple-darwin scripts/render-homebrew-formula.sh`
- `ruby -c <temp>/homebrew/Formula/engram.rb`
- `EXPECTED_TRACKED_CHANGES_PRESENT= scripts/render-homebrew-formula.sh`, expected failure before
  release asset validation with `EXPECTED_TRACKED_CHANGES_PRESENT must not be empty`
- `ALLOW_HOMEBREW_DIST_DIR_OVERRIDE=1 ALLOW_HOMEBREW_FORMULA_OUTPUT_OVERRIDE=1
  EXPECTED_PACKAGE_GIT_HEAD=0000000000000000000000000000000000000000 DIST_DIR=<temp>
  EXPECTED_TRACKED_CHANGES_PRESENT=true FORMULA_OUTPUT=<temp>/homebrew/Formula/engram.rb
  HOMEBREW_HOST_TRIPLE=aarch64-apple-darwin scripts/render-homebrew-formula.sh` (expected failure:
  manifest git head mismatch)
- `ALLOW_TRACKED_CHANGES=1 ALLOW_PACKAGE_DIST_DIR_OVERRIDE=1 DIST_DIR=<temp>
  scripts/package-release.sh` after the Homebrew archive payload guard
- `ALLOW_HOMEBREW_DIST_DIR_OVERRIDE=1 ALLOW_HOMEBREW_FORMULA_OUTPUT_OVERRIDE=1
  DIST_DIR=<temp> EXPECTED_TRACKED_CHANGES_PRESENT=true FORMULA_OUTPUT=<temp>/homebrew/Formula/engram.rb
  HOMEBREW_HOST_TRIPLE=aarch64-apple-darwin scripts/render-homebrew-formula.sh`
- `ruby -c <temp>/homebrew/Formula/engram.rb`
- Repacked the temp archive with a corrupted `README.md` manifest hash and recomputed its
  `.sha256`; `scripts/render-homebrew-formula.sh` failed with `manifest hash mismatch for
  README.md`
- Repacked the temp archive with an extra `other/` root and recomputed its `.sha256`;
  `scripts/render-homebrew-formula.sh` failed with `release archive member is outside expected
  root`
- `HOMEBREW_RELEASE_BASE_URL=https://example.com/engram scripts/render-homebrew-formula.sh`
  failed before reading release assets with `HOMEBREW_RELEASE_BASE_URL override requires explicit
  approval`
- `HOMEBREW_RELEASE_BASE_URL= scripts/render-homebrew-formula.sh` failed before reading release
  assets with `HOMEBREW_RELEASE_BASE_URL must not be empty`
- `ALLOW_HOMEBREW_RELEASE_BASE_URL_OVERRIDE=1 HOMEBREW_RELEASE_BASE_URL=https://example.com/engram/
  scripts/render-homebrew-formula.sh` failed before reading release assets with `Homebrew release
  URL base must not end with a slash`
- `scripts/release-gate-report.sh --target ga --hosted-run 27408174338 --quick
  --allow-tracked-changes --json` after the release-target availability guard, expected
  `release_target.state=available` for `v0.2.0`
- `scripts/release-gate-report.sh --target ga --release-version 0.2.0-beta.2 --hosted-run
  27408174338 --quick --allow-tracked-changes --json` after the release-target availability guard,
  expected `release_gate_state=release_target_unavailable` before disk or local validation
- `scripts/release-gate-report.sh --target ga --release-version nope --hosted-run 27437010801
  --quick --allow-tracked-changes --json` failed before release-target checks with
  `RELEASE_VERSION/--release-version must be x.y.z with an optional prerelease suffix, got nope`
- `scripts/release-gate-report.sh --target ga --release-version v0.2.0 --hosted-run 27437010801
  --quick --allow-tracked-changes --json` failed before release-target checks with
  `RELEASE_VERSION/--release-version must be x.y.z with an optional prerelease suffix, got
  v0.2.0`
- `RELEASE_VERSION=0.2 scripts/release-gate-report.sh --target ga --hosted-run 27437010801
  --quick --allow-tracked-changes --json` failed before release-target checks with
  `RELEASE_VERSION/--release-version must be x.y.z with an optional prerelease suffix, got 0.2`
- `RELEASE_NOTES_PATH=/tmp/engram-release-notes.md scripts/release-gate-report.sh --target ga
  --hosted-run 27438605838 --quick --json` failed before release-scope checks with
  `RELEASE_NOTES_PATH override requires explicit approval`
- `ALLOW_RELEASE_NOTES_PATH_OVERRIDE=maybe RELEASE_NOTES_PATH=/tmp/engram-release-notes.md
  scripts/release-gate-report.sh --target ga --hosted-run 27438605838 --quick --json` failed
  before release-scope checks with `ALLOW_RELEASE_NOTES_PATH_OVERRIDE must be 0 or 1`
- `RELEASE_NOTES_PATH= scripts/release-gate-report.sh --target ga --hosted-run 27438605838
  --quick --json` failed before release-scope checks with `RELEASE_NOTES_PATH must not be empty`
- `EXPECTED_EVENT=pull-request scripts/release-gate-report.sh --target ga --hosted-run
  27440399236 --quick --json` failed before GitHub run inspection with
  `EXPECTED_EVENT/--expected-event must be a GitHub event name token, got pull-request`
- `scripts/verify-hosted-ci-prestep-blocker.sh --event pull-request --json` failed before GitHub
  run discovery with `EXPECTED_EVENT/--event must be a GitHub event name token, got pull-request`
- `ALLOW_RELEASE_REPOSITORY_OVERRIDE=1 RELEASE_REPOSITORY=ymeiri/engram-does-not-exist
  scripts/release-gate-report.sh --target ga --hosted-run 27408174338 --quick
  --allow-tracked-changes --json` after the release-target availability guard, expected
  `release_gate_state=release_target_check_failed`
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
- `ALLOW_RELEASE_NOTES_PATH_OVERRIDE=1 RELEASE_NOTES_PATH=<temp-empty-file>
  scripts/release-gate-report.sh --target ga --release-version 0.2.0-beta.2 --hosted-run
  27373857951 --quick --allow-tracked-changes --json`
  expected `release_scope.state=incomplete`,
  `release_gate_state=release_scope_acknowledgement_required`, and remaining action
  `restore_release_notes_ga_scope_acknowledgements`
- `scripts/native-claude-gate-preflight.sh --json | jq .`
  expected `head=a082a63969df1be1179f38a75a02ee23ff815166`,
  `gate_state=blocked`, vault `2888/2888`, no tracked changes, no extra untracked files, and
  blocker `native Claude CLI processes are already running`
- `scripts/native-claude-gate-preflight.sh --json --require-ready` (expected exit code `2`)
