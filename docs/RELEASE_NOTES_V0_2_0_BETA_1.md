# Engram v0.2.0-beta.1 Release Notes

Release date: 2026-06-07
Status: Pre-release candidate

## Supported Beta Path

This beta is scoped to the local/Codex Brain Loop path:

- local `engram serve` MCP operation,
- lean `orient` with current-plan retrieval, trace/cursor fields, used-memory candidate IDs, and
  obligation summary,
- generated Memory OS vault inspection,
- review-gated M6 inventory/export/status paths,
- scoped obligations doctor and advisory harness lifecycle guidance,
- preserved approval boundaries for destructive or broad writes.

## Deferred From This Beta

The following remain production-hardening or host-parity gates, not blockers for this beta:

- native Claude prompt-bearing proof,
- effective-hook visibility proof,
- live Claude host-label proof,
- full multi-host parity,
- direct legacy deprecation/deletion,
- broad lifecycle cleanup or broad `lint apply_safe`,
- exhaustive telemetry completeness,
- OIDC/Vault/native-Claude auth/debugging edge cases,
- new feature work.

T306 resolved the current Rustdoc warning set for this candidate. Future Rustdoc polish remains a
production-hardening activity, not an initial-beta blocker.

## Release Gate

Before tagging this beta, the candidate commit must have normal exact-head hosted CI proof, or an
explicit release-owner decision accepting local validation as a fallback while hosted Actions is
externally account-blocked. The expected gate remains:

- exact-head CI green for Format, Docs, Check, Clippy, and Test,
- `cargo fmt --all --check` passing locally,
- a focused local/Codex smoke confirming current source-rendered harness guidance,
- canonical generated vault status count-aligned,
- `obligations(action=doctor, project=engram, cwd=/Users/yuval.meiri/projects/engram)` clean,
- a refreshed installed runtime/adapters check.

The local fallback command is `./scripts/local-ci.sh`. It mirrors the exact-head local validation
sequence used for this candidate: whitespace diff check, rustfmt, check, clippy, tests with CI-like
incremental/debug settings, and docs.

Recent phase-1 local evidence is strong: T317 validated PR #3 head
`78f14d0bebd980070a4fcb8d1f259be47517c704` with `cargo fmt --all --check`,
`git diff --check`, `cargo check --all-targets`,
`cargo clippy --all-targets -- -D warnings`, `cargo test --all-targets --jobs 1`, and
`cargo doc --no-deps`. T318 reran hosted GitHub Actions run `27091138284`, creating attempt 2 on
that same head, but all five jobs failed before runner assignment with zero steps, `runner_id=0`,
and billing/spending-limit annotations. That external account gate does not contradict the local
validation, but the normal exact-head hosted-CI release proof is still missing until Actions can run
on the head intended for release or the release owner explicitly accepts the local fallback.

T329 advanced the draft PR #3 head to `fe46d0a73d39e3309b149703dda4c108da91fc02` through
docs-only release evidence plus exact lifecycle archive records. Local validation for that head
passed `git diff --check`, `cargo fmt --all --check`, `cargo check --all-targets`, canonical vault
compile with zero skipped files, and cached diff checks. Hosted GitHub Actions run `27096981016`
on the same head again failed before workflow steps ran with the same billing/spending-limit
annotations. Treat this as the current hosted-CI blocker.

Fresh AI Council review after T329 places the initial local/Codex beta at about `88-92%` ready while
hosted CI is externally blocked and local fallback evidence is accepted, or about `95%` ready once
GitHub Actions billing is fixed and exact-head checks pass or the release owner explicitly accepts
local validation as the beta fallback. T330 also records a current-head local/Codex smoke with lean
`orient`, obligations doctor, vault status/compile, lint-sample evidence, and bounded M6
inventory/temp-export/status/dry-run-apply evidence. This is not a production/GA readiness claim;
production readiness remains materially lower because native Claude proof, effective hooks,
host-label proof, host parity, telemetry completeness, and operational hardening remain open.

T331 closes the next exact superseded rolling-handoff lifecycle batch:
`019e7cf7-560c-70e2-bbeb-3448f4637055`,
`019e7d27-32d6-7200-944c-ef5945436f8c`,
`019e7d28-add4-70e3-a55c-453f8fe8695d`,
`019e7d29-0f3c-7961-9588-c1adbe4628af`, and
`019e7da0-d384-7b12-b43a-d7188b1a8c38`. Post-archive lint advances to
`019e7db8-de1e-7251-87ba-fea21bed17f7`, so broad lifecycle cleanup remains deferred and
exact-target-gated rather than part of the beta release gate.

T332 closes the next single exact superseded rolling-handoff target,
`019e7db8-de1e-7251-87ba-fea21bed17f7`, after successor review showed it is directly superseded
by active handoff `019e844c-6a05-7a10-858b-5212d117a4bb`. Post-archive lint no longer returns the
T332 target in the first ten sampled findings; the bounded sample now reports stale-feedback review
signals rather than a superseded-active warning.

## Current Installation Status

T305 refreshed the installed local binary, generated Codex adapter, and global daemon from the
`0.2.0-beta.1` candidate. The installed binary now reports:

```text
engram 0.2.0-beta.1
```

The installed Codex harness is `Ready: true`, and both source-rendered and installed Codex harness
guidance include scoped final-response obligation checks:

```text
obligations(action=doctor, project=..., cwd=...)
```

Already-open agent UI sessions may still need a fresh session or tool reload before they ingest the
updated skill text. This does not change the beta deferrals for native Claude, effective hooks,
host labels, or full multi-host parity.

## Claude Code Adapter Safety Follow-Up

T315 adds source-level coverage for the T314 repair path later executed by T333. The new harness
test proves that `HarnessSettingsTarget::SnippetOnly` can repair generated Claude Code adapters
without rewriting an existing `settings.json`, `settings.local.json`, or
`engram-settings-snippet.json`.

T333 executes the prepared T314 command with the installed CLI:

```text
/Users/yuval.meiri/.local/bin/engram harness install --harness claude-code --settings-target snippet-only --write --json
```

The write updated exactly the three generated Claude Code adapters, left `settings.json`,
`settings.local.json`, and `engram-settings-snippet.json` unchanged, and made Claude Code
`harness status` and `harness doctor` report `ready=true`.

This closes generated-adapter drift only. It still does not prove native Claude prompt-bearing
execution, effective-hook visibility, live host labels, or full production parity.

## Native Claude Post-Repair Preflight

T334 reruns the native-Claude/effective-hook/host-label preflight after the T333 adapter repair.
The Claude `2.1.168` path, target, SHA-256, installed daemon, obligations doctor, canonical vault
status, generated adapter hashes, snippet-only dry-run, and Claude Code harness readiness now
match the expected state. The preflight still hard-stops before launching native Claude because
existing native Claude CLI sessions are live on `ttys001` and `ttys005`, making attribution
ambiguous for a new prompt-bearing run.

This narrows the remaining production-hardening gap: adapter drift is closed, but native Claude
prompt-bearing proof, effective-hook visibility, and live host labels remain deferred from the
initial beta.

## Effective-Hook Successor Packet

T335 records a docs-only successor for the T269 effective-hook visibility packet under the observed
Claude Code `2.1.168` runtime. It updates the future `/hooks` preflight baseline from the stale
`2.1.161` target to the T334-observed path, version, and SHA-256, while preserving the strict
one-`/hooks` transcript observation contract, bounded cleanup, T312/T270 separation, and T334
attribution hard-stop.

T335 does not launch native Claude, run `/hooks`, mutate settings or adapters, prove
effective-hook visibility, or change the initial beta scope.

## Project-Scoped Lint

T336 adds optional project filtering to Memory OS lint. MCP callers can pass `project`, and the CLI
can run `engram lint run --scope-project <name>` to focus memory, stale-session, and open-obligation
findings on the current project while preserving global/user memory checks and unchanged unscoped
lint behavior.

This improves project health visibility for beta closeout. It does not run `lint apply_safe`,
archive memory, or close lifecycle cleanup.

## Project-Scoped Lint Installed Runtime Refresh

T337 installs the current `engram-cli` into `/Users/yuval.meiri/.local/bin/engram` and restarts the
daemon so the T336 project-scoped lint surface is live in the installed local/Codex path. The
installed binary hash changed from
`01b171ec654da95ea5b1f8363bc109e3069c0ff78bdb38581a202e472f9fd09b` to
`b775efa0946862eba8d4d8993bb946f0926372d8a3fe9bbfea98ea38e786e7c2`, `daemon status` now reports
PID `57356` spawned by `/Users/yuval.meiri/.local/bin/engram`, and fresh installed MCP smoke
confirmed `lint.project` appears in `tools/list` and `tools/call` accepts `project=engram`.

T337 is installed-runtime adoption for T336 only. It does not run `lint apply_safe`, mutate memory,
or change the beta deferrals.

## Rolling Handoff Evidence

T338 makes rolling handoff updates evidence-backed. After T337, project-scoped lint showed the
current rolling handoff was active but missing evidence. `HandoffService::update` now adds
tool-call evidence for global, project, and session handoff writes while preserving existing
session-event evidence for session-scoped handoffs.

The T338 runtime was installed to `/Users/yuval.meiri/.local/bin/engram` with hash
`e53765568a2232c55c2d17a8a48480e745b2c2fda044a8d087681c20534e3dc5`, daemon PID `92750`, and the
new active handoff `019ea34a-c3ac-74d0-ae42-52cd6adcb610` carries
`handoff(action=update,project=engram)` evidence. Installed project-scoped lint no longer flags
that active handoff as missing evidence.

This improves Memory OS hygiene for future handoffs. It does not run `lint apply_safe`, archive
historical handoffs, or change the beta deferrals.

## Canonical Vault Resync

T339 refreshes the durable generated Memory OS vault after the latest T338/T339 memory writes. The
preflight status showed the vault was initialized and generated-only, but count-stale:
`generated_file_count=2566` while `expected_generated_file_count=2568`.

`vault(action=compile, vault_path="/Users/yuval.meiri/.engram/vault")` completed with
`files_skipped=[]`, and postflight status returned `generated_file_count=2568`,
`user_file_count=0`, and `expected_generated_file_count=2568`.

This restores the canonical vault validation gate for the current Memory OS state. It does not
change source behavior, run `lint apply_safe`, close hosted CI, or change the beta deferrals.

## Native Claude Preflight Refresh

T340 reruns the read-only native-Claude/effective-hook/host-label preflight at PR #3 head
`6d0467e933a880f9039fd943b34848c2ca93f069`. The Claude `2.1.168` path, target, SHA-256, Engram
daemon, obligations doctor, canonical vault status, Claude Code harness readiness, and snippet-only
install dry-run all match the expected state. The canonical vault is currently count-aligned at
`2573` generated files, zero user files, and `2573` expected generated files.

The preflight still hard-stops before launching native Claude because native Claude CLI sessions
remain live on `ttys001` and `ttys005`, making new-session attribution ambiguous. T340 does not
launch native Claude, run `/hooks`, signal processes, mutate settings or adapters, prove
prompt-bearing behavior, prove effective-hook visibility, prove live host labels, close hosted CI,
or change the initial beta scope.

## Exact-Head Local CI Fallback

T341 validates PR #3 head `2fa5b577bda8ab6141e0f7272736044d441a7e88` with the full
CI-equivalent local workflow after hosted run `27101388242` again failed all five jobs before
executing any workflow steps. `gh run view` reports `steps: []` for Check, Test, Format, Clippy,
and Docs, matching the prior external account/billing/spending-limit pattern.

That T341 head passed:

- `git diff --check`
- `cargo check --all-targets`
- `CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 cargo test --all-targets --jobs 1`
- `cargo fmt --all --check`
- `cargo clippy --all-targets -- -D warnings`
- `cargo doc --no-deps`

Fresh AI Council consensus after T341 puts the scoped local/Codex MVP beta at about `95%`
complete and shippable if the release owner accepts local validation as the fallback. Hosted CI
passing is still not achieved and remains either an ops/GA hygiene item or a beta gate only if the
release owner requires it. T341 does not mark PR #3 ready, merge, tag, publish, close hosted CI,
or change the production/GA deferrals.

## Beta Scope Consensus Refresh

T343 refreshes the release-scope decision on the current PR #3 head
`966dc00d5248ac342b156974b5392700706f3139`. The PR body records that this exact head passed the
full local CI-equivalent workflow before push, while hosted run `27101972733` failed Check, Test,
Format, Clippy, and Docs before workflow-step execution with `steps: []`.

Fresh AI Council consensus keeps the scoped local/Codex MVP beta at about `95-98%` ready before
release mechanics if the release owner accepts local validation as fallback. The remaining beta gate
is either explicit release-owner fallback acceptance or restored exact-head hosted CI. Production/GA
readiness remains separate and materially lower because native Claude prompt-bearing proof,
effective-hook visibility, live host labels, full multi-host parity, broad lifecycle cleanup,
direct legacy deprecation/deletion, exhaustive telemetry, auth edge hardening, packaging,
performance, and cross-platform polish remain open.

T343 does not mark PR #3 ready, merge, tag, publish, close hosted CI, run native Claude, prove
hooks or host labels, or change the supported beta scope.
