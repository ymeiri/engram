# Engram v0.2.0-beta.1 Release Notes

Release date: TBD
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

The local pre-publish packaging command is `./scripts/package-release.sh`. It builds the release
binary, verifies that `engram --version` matches the workspace package version, and writes a tarball
plus SHA-256 checksum under ignored `dist/`.

The local install-smoke command is `./scripts/package-install-smoke.sh`. It builds the package,
verifies the checksum, extracts the archive, installs the packaged binary into a temporary prefix,
confirms `PATH` resolution and `engram --version`, starts the packaged binary with
`engram serve --http --memory`, and verifies `/health`. The smoke starts the server from the
temporary install workspace and sets `ENGRAM_EMBED_CACHE_DIR` explicitly, so package validation no
longer relies on the repository root as the process cwd for embedding model cache discovery.

### Release-Owner Signoff Checklist

The remaining beta decision is explicit release-owner signoff, not additional product scope. Before
tagging `v0.2.0-beta.1`, the release owner should confirm:

1. `./scripts/local-ci.sh` on the signed-off head is accepted as the local CI-equivalent beta gate
   while hosted GitHub Actions is externally blocked before workflow-step execution.
2. `./scripts/package-install-smoke.sh` on the same head is accepted as package/checksum/install
   and packaged HTTP `/health` proof.
3. The current hosted run can be waived for this beta only if it failed before workflow-step
   execution with `steps=[]`, matching the external billing/spending-limit blocker rather than a
   source or packaging failure.
4. The known beta limitations listed in this file are accepted: native Claude prompt-bearing
   behavior, effective-hook visibility, live Claude host labels, full multi-host parity, direct
   legacy deprecation/deletion, broad lifecycle cleanup or broad `lint apply_safe`, M6 write-apply
   expansion, and exhaustive telemetry/auth/ops/performance/cross-platform hardening remain
   deferred.
5. After signoff, PR #3 may be marked ready, merged, tagged as `v0.2.0-beta.1`, published with the
   release archive and checksum, and verified with the published install commands.

## Beta Install Quickstart

For source installs, build and place the binary on `PATH` before running `engram init` or
`engram serve`:

```bash
git clone https://github.com/ymeiri/engram.git
cd engram
cargo build --release

mkdir -p "$HOME/.local/bin"
install -m 755 ./target/release/engram "$HOME/.local/bin/engram"
export PATH="$HOME/.local/bin:$PATH"
engram --version
```

After `v0.2.0-beta.1` is published, install the macOS arm64 release artifact with checksum
verification:

```bash
version=0.2.0-beta.1
archive="engram-${version}-aarch64-apple-darwin.tar.gz"

curl -LO "https://github.com/ymeiri/engram/releases/download/v${version}/${archive}"
curl -LO "https://github.com/ymeiri/engram/releases/download/v${version}/${archive}.sha256"
shasum -a 256 -c "${archive}.sha256"
tar -xzf "${archive}"

mkdir -p "$HOME/.local/bin"
install -m 755 "engram-${version}-aarch64-apple-darwin/engram" "$HOME/.local/bin/engram"
export PATH="$HOME/.local/bin:$PATH"
engram --version
```

The expected version output for this beta is:

```text
engram 0.2.0-beta.1
```

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
direct legacy deprecation/deletion, exhaustive telemetry, auth edge hardening, performance, and
cross-platform polish remain open.

T343 does not mark PR #3 ready, merge, tag, publish, close hosted CI, run native Claude, prove
hooks or host labels, or change the supported beta scope.

## One-Command Local CI and Package Proof

T345 adds `./scripts/local-ci.sh` as the single exact-head local CI-equivalent fallback command for
release validation. It runs whitespace diff checks, rustfmt, cargo check, clippy with warnings as
errors, the full test suite with CI-like incremental/debug settings, and rustdoc generation.

T346 adds `./scripts/package-release.sh` as the local pre-publish packaging command. It builds the
release binary, checks that `engram --version` matches the workspace package version, packages the
binary with README, LICENSE, changelog, and these release notes, and writes a SHA-256 checksum under
ignored `dist/`.

T347 validates PR #3 head `b0d8e075a04b5e35f4ed7c4d60654231ed5c1324` with both commands. The
exact head passed `./scripts/local-ci.sh`, and `./scripts/package-release.sh` produced
`dist/engram-0.2.0-beta.1-aarch64-apple-darwin.tar.gz` plus its `.sha256` file. The archive
contents were inspected, the checksum verified, and the extracted packaged binary reported
`engram 0.2.0-beta.1`.

Hosted GitHub Actions run `27102953577` on that same head still failed before workflow-step
execution with `steps: []` and account billing/spending-limit annotations. The remaining beta gate
is therefore explicit release-owner fallback acceptance or restored exact-head hosted CI. T347 does
not mark PR #3 ready, merge, tag, publish, close hosted CI, run native Claude, prove hooks or host
labels, or change the supported beta scope.

T350 closes the first-user onboarding gap on PR #3 head
`54d01eb71e2020960fa62c0d6b72a05b5c00aee4`. README and these release notes now show source
install steps that put `engram` on `PATH` before `engram init` or `engram serve`, plus published
tarball download, checksum, install, and expected-version commands. That head passed
`./scripts/local-ci.sh`, `./scripts/package-release.sh`, checksum verification, packaged release
note inspection, and a manual isolated package install/health smoke.

T351 adds `./scripts/package-install-smoke.sh` so the install proof is repeatable instead of
manual. T351 package-smoke candidate head `4d05f6fc2f4fb4c6309c431c083ea55540c32380` passed
`./scripts/local-ci.sh` and `./scripts/package-install-smoke.sh`. The smoke builds the package,
verifies the checksum, extracts the archive, checks required packaged files, installs the packaged
binary into a temporary prefix, confirms `PATH` resolution and `engram 0.2.0-beta.1`, starts the
packaged binary with `engram serve --http --memory`, and verifies `/health` returns
`{"status":"ok","service":"engram","version":"0.2.0-beta.1"}`.

Hosted GitHub Actions run `27114219090` on `4d05f6f` still failed before workflow-step execution:
Check, Test, Format, Clippy, and Docs all report `steps: []`; the sampled Check annotation says
recent account payments failed or the spending limit must be increased. Fresh AI Council consensus
on 2026-06-08 gives a conservative release-management estimate of about `92%` complete until
release-owner approval, ready/merge/tag/publish, and a practical local/Codex beta readiness of
`98-99%` if the release owner accepts exact-head local CI plus package/install validation as the
hosted-CI fallback.

T351 does not mark PR #3 ready, merge, tag, publish, close hosted CI, run native Claude, prove
hooks or host labels, or change the supported beta scope.

T353 makes embedding model cache discovery deterministic for beta installs. `EmbedConfig::default`
now uses `~/.engram/cache/fastembed` by default, accepts `ENGRAM_EMBED_CACHE_DIR` as the
Engram-specific override, and preserves upstream `FASTEMBED_CACHE_DIR` compatibility. `HF_HOME`
still remains an upstream Hugging Face override when set. The package install smoke now starts the
packaged binary from the temporary install workspace with `ENGRAM_EMBED_CACHE_DIR` set explicitly,
so `/health` validation no longer depends on repository-root cwd.

T353 validation passed `cargo test -p engram-embed config::tests`,
`cargo test -p engram-tests --test multi_session_tests`, `./scripts/package-install-smoke.sh`, and
`./scripts/local-ci.sh`. The first full local CI attempt exposed that isolated multi-session test
daemons also needed an explicit stable cache path; the test daemon helper now sets
`ENGRAM_EMBED_CACHE_DIR` to the configured cache override or package-local `.fastembed_cache`.

T353 does not mark PR #3 ready, merge, tag, publish, close hosted CI, run native Claude, prove
hooks or host labels, or change the supported beta scope.

T354 refreshes the installed local binary after T353. `cargo install --path engram-cli --force
--root /Users/yuval.meiri/.local` replaced the previous installed hash
`e53765568a2232c55c2d17a8a48480e745b2c2fda044a8d087681c20534e3dc5` with
`a47edffa8c8ed955a311adac85033ce8a28235c37007b5149ea81a8ffeb456de`, while
`/Users/yuval.meiri/.local/bin/engram --version` still reports `engram 0.2.0-beta.1`.

An isolated installed-runtime smoke started `/Users/yuval.meiri/.local/bin/engram serve --http
--memory` from a temporary workspace with temp `HOME`, temp `ENGRAM_DATA_DIR`, and explicit
`ENGRAM_EMBED_CACHE_DIR` pointing at the warmed repo cache. `GET /health` returned
`{"status":"ok","service":"engram","version":"0.2.0-beta.1"}`. The existing global daemon was not
restarted during this validation.

T354 does not mark PR #3 ready, merge, tag, publish, close hosted CI, run native Claude, prove
hooks or host labels, or change the supported beta scope.

T355 restarts the live global daemon onto the refreshed T354 installed binary. Before restart,
`engram daemon status` reported PID `92750`, spawned by `/Users/yuval.meiri/.local/bin/engram`,
with spawn version `0.2.0-beta.1`. After `engram daemon stop`, the first `engram daemon start`
attempt exited before readiness because the daemon log reported a transient RocksDB
`~/.engram/data/LOCK` conflict. A follow-up status check reported no running daemon, and the
retry succeeded.

The refreshed live daemon now reports PID `64693` on port `8765`, spawned by
`/Users/yuval.meiri/.local/bin/engram`, with spawn version `0.2.0-beta.1` and installed hash
`a47edffa8c8ed955a311adac85033ce8a28235c37007b5149ea81a8ffeb456de`. `GET /health`
returned `{"status":"ok","service":"engram","version":"0.2.0-beta.1"}`, and a live MCP
`orient(response_shape="lean")` smoke for the current MVP-status prompt returned trace
`019ea620-b1be-7dc1-a022-15f49badddf6`.

T355 does not mark PR #3 ready, merge, tag, publish, close hosted CI, run native Claude, prove
hooks or host labels, or change the supported beta scope.

T356 reruns the read-only native-Claude/effective-hook/host-label preflight after the T355 live
daemon refresh. The branch is synchronized with its upstream, and `origin/main...HEAD` is `0 48`,
so the earlier divergent-branches pull hint is not current for `yuval.meiri/memory-os-phase1`.
PR #3 remains draft at `6a561863154d7b3fabc0e176d3105ed1f0dedd7a`; hosted run
`27122932006` still fails all five jobs before workflow-step execution with `steps: []`.

The Claude binary preflight still matches the T312/T335/T340 baseline:
`/Users/yuval.meiri/.local/bin/claude`, version `2.1.168 (Claude Code)`, SHA-256
`377f0ecedba8246bdabdf312ce8b7cc8ae1160997b26f5edca352a4a8d61dc78`. Claude Code
`harness status` and `harness doctor` report `ready=true`; snippet-only install dry-run reports
`planned=[]`; the global daemon remains PID `64693` from `/Users/yuval.meiri/.local/bin/engram`
with spawn version `0.2.0-beta.1`; `/health` returns
`{"status":"ok","service":"engram","version":"0.2.0-beta.1"}`; canonical vault status is
count-aligned at `2628` generated files; and obligations doctor is clean.

The preflight still hard-stops before launching native Claude because native Claude CLI sessions
remain live as PID `60453` on `ttys001` and PID `311` on `ttys005`, making new-session attribution
ambiguous. T356 does not launch native Claude, run `/hooks`, signal processes, mutate settings or
adapters, prove prompt-bearing behavior, prove effective-hook visibility, prove live host labels,
close hosted CI, or change the supported beta scope.

## CLI Daemon Admin Routing

T357 hardens local beta administration while the daemon owns the store. `engram lint`,
`engram obligations`, and `engram vault` now prefer the healthy global or project daemon when no
explicit `--data-dir` is supplied, using the existing one-shot MCP proxy path. Explicit data-dir
commands still use direct RocksDB access, and the direct path remains the fallback when no matching
daemon is healthy.

This fixes the practical lock failure observed when running installed CLI health checks while the
global daemon held `~/.engram/data/LOCK`. Focused validation passed `cargo fmt --all --check`,
`git diff --check`, `cargo check -p engram-cli`, and `cargo test -p engram-cli`. Live source-binary
and installed-path smokes proved `lint run --scope-project engram`, `vault status`, and
`obligations doctor --limit 3` return daemon-backed output instead of the lock error while daemon
PID `64693` remains running.

The same T357 head also passed `./scripts/local-ci.sh` and
`./scripts/package-install-smoke.sh`. The package smoke rebuilt
`engram-0.2.0-beta.1-aarch64-apple-darwin.tar.gz`, verified its checksum, installed the packaged
binary into a temporary prefix, confirmed `engram 0.2.0-beta.1`, started packaged
`engram serve --http --memory`, and verified `/health` returned
`{"status":"ok","service":"engram","version":"0.2.0-beta.1"}`.

`cargo install --path engram-cli --force --root /Users/yuval.meiri/.local` refreshed the installed
CLI hash from `a47edffa8c8ed955a311adac85033ce8a28235c37007b5149ea81a8ffeb456de` to
`fa91efbd228683dae608881f5828bdc1ffe55b67376e414653f8ac8eb92ba8c9`; `engram --version`
continues to report `engram 0.2.0-beta.1`.

T357 does not mark PR #3 ready, merge, tag, publish, close hosted CI, run native Claude, prove
hooks or host labels, mutate lifecycle state, run broad `lint apply_safe`, or change the supported
beta scope.

## Lint Project Schema Exposure

T358 hardens the public MCP metadata for project-scoped lint. `LintRequest.project` now has an
explicit schema description, and the test suite guards both the generated request schema and the
HTTP daemon `tools/list` response so `lint.inputSchema.properties.project` remains visible to agent
callers.

Focused validation passed `cargo fmt --all --check`, `git diff --check`,
`cargo test -p engram-mcp lint_request_schema_exposes_project_filter`, `cargo test -p engram-mcp`,
`cargo test -p engram-tests --test lint_tests test_mcp_lint_project_filter_excludes_unrelated_project_memory`,
and
`cargo test -p engram-tests --test multi_session_tests test_mcp_tools_list_lint_schema_exposes_project_filter`.

`cargo install --path engram-cli --force --root /Users/yuval.meiri/.local` refreshed the installed
CLI hash from `fa91efbd228683dae608881f5828bdc1ffe55b67376e414653f8ac8eb92ba8c9` to
`62c9955925f74fba706ad466416033cc0bdbc211cf0443a373d4e5925760589a`; `engram --version`
continues to report `engram 0.2.0-beta.1`. The live global daemon was restarted onto that binary
as PID `36562` on port `8765`, and an installed-daemon `tools/list` smoke returned `true` for
`lint.inputSchema.properties.project.description == "Optional project scope to lint."`.

T358 does not mark PR #3 ready, merge, tag, publish, close hosted CI, run native Claude, prove
hooks or host labels, mutate lifecycle state, run broad `lint apply_safe`, or change the supported
beta scope.

T359 hardens scoped obligation health checks in the CLI. `engram obligations list` and
`engram obligations doctor` now accept `--scope-project` and `--cwd`, matching the existing MCP
`project`/`cwd` filters for obligation list and doctor actions. The daemon-backed CLI path serializes
those filters into `tools/call`; the direct-store fallback path passes them to
`ObligationService::list` and `ObligationService::doctor`. Unscoped list/doctor behavior remains
unchanged, and top-level `engram obligations --project <name>` continues to act as the default
project scope.

Focused validation passed `cargo test -p engram-cli obligation_daemon_arguments`,
`cargo check -p engram-cli`, `cargo fmt --all --check`, `git diff --check`, and
`cargo test -p engram-tests --test obligation_tests`. Source CLI help exposes the new flags for
both commands. Source daemon-backed smokes for scoped `doctor` and `list` passed, and isolated
`--data-dir` smokes covered the direct-store fallback path.

`cargo install --path engram-cli --force --root /Users/yuval.meiri/.local` refreshed the installed
CLI hash from `62c9955925f74fba706ad466416033cc0bdbc211cf0443a373d4e5925760589a` to
`ae45c01ab2a4c5046508e916a7c381655a71611f223fd8fc7989392cd3879f79`; `engram --version`
continues to report `engram 0.2.0-beta.1`. Installed daemon-backed smokes passed:
`engram obligations doctor --scope-project engram --cwd /Users/yuval.meiri/projects/engram --limit 3 --json`
returned `{"open":[],"warnings":[]}`, and
`engram obligations list --scope-project engram --cwd /Users/yuval.meiri/projects/engram --status open --limit 3 --json`
returned `[]`.

Final exact-head validation passed the local CI-equivalent steps (`git diff --check`,
`cargo fmt --all --check`, `cargo check --all-targets`,
`cargo clippy --all-targets -- -D warnings`, serialized `cargo test --all-targets --jobs 1`, and
`cargo doc --no-deps`) plus `./scripts/package-install-smoke.sh`. The first `./scripts/local-ci.sh`
run completed through check and clippy before its test step stalled in `rustc` with no CPU activity;
the stalled process was terminated, and the same serialized test step plus docs step were rerun
directly and passed. The package smoke verified the release tarball checksum, installed the package
into a temporary prefix, confirmed `engram 0.2.0-beta.1`, and verified packaged HTTP `/health`
returned `{"status":"ok","service":"engram","version":"0.2.0-beta.1"}`.

T359 does not mark PR #3 ready, merge, tag, publish, close hosted CI, run native Claude, prove
hooks or host labels, mutate lifecycle state, run broad `lint apply_safe`, or change the supported
beta scope.

T360 hardens public MCP metadata for scoped obligation health checks. `ObligationRequest.project`
now explicitly describes its supported detect/add/list/open/doctor scope, and
`ObligationRequest.cwd` now explicitly describes detect/list/open/doctor scoping. This matches the
runtime MCP behavior and the T359 CLI flags, so agents reading `tools/list` can discover the scoped
obligation list/doctor path without guessing.

Focused validation passed `cargo test -p engram-mcp obligation_request_schema_exposes_scope_filters`,
`cargo test -p engram-tests --test multi_session_tests test_mcp_tools_list_obligations_schema_exposes_scope_filters`,
`cargo fmt --all --check`, `git diff --check`, `cargo test -p engram-mcp`, and
`cargo test -p engram-tests --test multi_session_tests test_mcp_tools_list`.

`cargo install --path engram-cli --force --root /Users/yuval.meiri/.local` refreshed the installed
CLI hash from `ae45c01ab2a4c5046508e916a7c381655a71611f223fd8fc7989392cd3879f79` to
`ff16b90be46e54d089ce66e5b360630449bffc9f874da031beb10884f994756b`; `engram --version`
continues to report `engram 0.2.0-beta.1`. The live global daemon was restarted onto that binary
as PID `48118` on port `8765`, `/health` returned
`{"status":"ok","service":"engram","version":"0.2.0-beta.1"}`, and a live `tools/list` smoke
confirmed the obligations schema exposes:
`project = "Optional project scope for detect, add, list, open, and doctor."` and
`cwd = "Current working directory for detect/list/open/doctor scoping."`.

Final exact-head validation passed `./scripts/local-ci.sh` and `./scripts/package-install-smoke.sh`.
The package smoke verified the release tarball checksum, installed the package into a temporary
prefix, confirmed `engram 0.2.0-beta.1`, and verified packaged HTTP `/health` returned
`{"status":"ok","service":"engram","version":"0.2.0-beta.1"}`.

T360 does not mark PR #3 ready, merge, tag, publish, close hosted CI, run native Claude, prove
hooks or host labels, mutate lifecycle state, run broad `lint apply_safe`, or change the supported
beta scope.

T361 hardens public MCP metadata for project-scoped telemetry reports. `TelemetryRequest.project`
now explicitly describes its support for `record_trace`, `list_traces`, `list_feedback`,
`stats_by_intent`, and `real_session_eval`. This matches the runtime telemetry behavior, so agents
reading `tools/list` can discover the project-filtered telemetry/eval path without guessing.

Focused validation passed
`cargo test -p engram-mcp telemetry_request_schema_exposes_project_filter` and
`cargo test -p engram-tests --test multi_session_tests test_mcp_tools_list_telemetry_schema_exposes_project_filter`.
Broader validation passed `cargo fmt --all --check`, `git diff --check`, `cargo test -p
engram-mcp`, `cargo test -p engram-tests --test multi_session_tests test_mcp_tools_list`,
`./scripts/local-ci.sh`, and `./scripts/package-install-smoke.sh`.

`cargo install --path engram-cli --force --root /Users/yuval.meiri/.local` refreshed the installed
CLI hash from `ff16b90be46e54d089ce66e5b360630449bffc9f874da031beb10884f994756b` to
`6c278872d2f71a5ce96fba3e1777b3cc2f4690e6d6c9caf74df093fb4fd7e49a`; `engram --version`
continues to report `engram 0.2.0-beta.1`. The live global daemon was restarted onto that binary
as PID `2865` on port `8765`, `/health` returned
`{"status":"ok","service":"engram","version":"0.2.0-beta.1"}`, and a live `tools/list` smoke
confirmed the telemetry schema exposes
`project = "Optional project scope for record_trace, list_traces, list_feedback, stats_by_intent, and real_session_eval."`.

T361 does not mark PR #3 ready, merge, tag, publish, close hosted CI, run native Claude, prove
hooks or host labels, mutate lifecycle state, change telemetry filtering semantics, or change the
supported beta scope.

T362 hardens public MCP metadata for the core `orient` Brain Loop entrypoint. `OrientRequest.cwd`
now explicitly describes repository/project resolution and scoped memory selection,
`OrientRequest.project` describes project resolution and project-scoped memory selection, and
`OrientRequest.response_shape` describes the full/lean response shape contract for compact
trace/cursor/Brain Loop guidance.

Focused validation passed
`cargo test -p engram-mcp orient_request_schema_exposes_context_contract` and
`cargo test -p engram-tests --test multi_session_tests test_mcp_tools_list_orient_schema_exposes_context_contract`.
Broader validation passed `cargo fmt --all --check`, `git diff --check`, `cargo test -p engram-mcp`,
`cargo test -p engram-tests --test multi_session_tests test_mcp_tools_list`, and
`./scripts/local-ci.sh`. `./scripts/package-install-smoke.sh` rebuilt the release package, verified
the checksum, installed the packaged binary in a temporary prefix, reported `engram 0.2.0-beta.1`,
and verified packaged HTTP `/health`. The installed runtime hash is
`77a08e895614bea3b02816e67bafd64087ea0634f4b0ca58b8199a9ef7855633`; the live daemon was
restarted as PID `47577`; an installed-daemon `tools/list` smoke confirmed the three `orient`
schema descriptions.

T362 does not mark PR #3 ready, merge, tag, publish, close hosted CI, run native Claude, prove
hooks or host labels, mutate lifecycle state, change orientation ranking, or change the supported
beta scope.

T363 refreshes the native Claude prompt-bearing, effective-hook, and live host-label preflight after
T362. The read-only assertions still match: Claude Code resolves to
`/Users/yuval.meiri/.local/share/claude/versions/2.1.168`, reports `2.1.168 (Claude Code)`, and
has SHA-256 `377f0ecedba8246bdabdf312ce8b7cc8ae1160997b26f5edca352a4a8d61dc78`; Claude Code
harness status/doctor report `ready=true`; snippet-only install dry-run plans no writes; daemon PID
`47577` is healthy; canonical vault status is count-aligned at `2651` generated files with zero
user files; scoped obligations doctor is clean.

T363 hard-stops before native execution because process inventory shows an active native `claude`
process on `ttys004` (`PID 34797`). That process makes a new single-session native transcript
ambiguous for T312 prompt-bearing proof, T335 `/hooks` effective-hook proof, and T270 live
host-label proof.

T363 does not launch native Claude, send prompts, run `/hooks`, signal processes, mutate settings
or adapters, mark PR #3 ready, merge, tag, publish, close hosted CI, or change the supported beta
scope.

T364 performs exact lifecycle cleanup for two stale, evidence-less Claude-harness MemoryItems:
`019dd4e3-bcec-7c02-9174-ba0ac0380d45` (`Claude Code native hook harness implemented`) and
`019dd509-46f2-71c0-aff7-ebe777810825` (`Claude Code Engram-native harness activated`). Fresh
`memory(get)` checks showed both were active with empty evidence; `graph(around, depth=1)` showed
only project scope; and project-scoped lint flagged them as stale-feedback/missing-evidence active
memory. Both were archived with explicit reasons because newer evidenced T333/T340/T363 records now
carry the current Claude-harness state: generated adapter drift is closed, while native Claude
prompt-bearing behavior, effective-hook visibility, and live host labels remain unproved.

Post-archive validation confirmed both IDs are `status=archived`, live daemon project-scoped
`lint(action=run, project=engram, limit=30)` returned `archived_targets_present=[]`, and direct
search for the old activation titles now surfaces active evidenced successor/limitation records
first.

T364 does not change source code, launch native Claude, run `/hooks`, signal processes, mutate
settings or adapters, run broad `lint apply_safe`, delete data, mark PR #3 ready, merge, tag,
publish, close hosted CI, or change the supported beta scope.

T365 performs exact lifecycle cleanup for two stale, evidence-less May 6 orient-contract and
architecture checkpoint MemoryItems: `019dfed3-519d-7f01-8c46-c9245ba0045b`
(`AI Council and Claude next-step synthesis after orient contract`) and
`019dfed5-1875-7110-b355-8d1060e6d04a`
(`Brain Harness Architecture synced after orient contract checkpoint`). Fresh `memory(get)` checks
showed both were active with empty evidence; `graph(around, depth=1)` showed only project scope; and
project-scoped lint flagged both as stale-feedback/missing-evidence active memory. Both records were
archived with explicit reasons because their May 6 next-step/checkpoint guidance is historical, not
current.

Post-archive validation confirmed both IDs are `status=archived`, live daemon project-scoped
`lint(action=run, project=engram, limit=40)` returned `archived_targets_present=[]` and reduced the
returned finding count from `20` to `16`, and direct search for the old titles still returns active
evidenced current-plan/beta-scope records first.

T365 does not change source code, run broad `lint apply_safe`, delete data, mark PR #3 ready,
merge, tag, publish, close hosted CI, run native Claude, prove hooks or host labels, or change the
supported beta scope.

T366 refreshes local beta acceptance evidence for PR #3 while hosted GitHub Actions remains
externally blocked. The local CI-equivalent command set passed: `git diff --check`,
`cargo fmt --all --check`, `cargo check --all-targets`,
`CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 cargo test --all-targets --jobs 1`,
`cargo clippy --all-targets -- -D warnings`, and `cargo doc --no-deps`.
`./scripts/package-install-smoke.sh` also passed, rebuilding the release package, verifying the
tarball checksum, installing the packaged binary into a temporary prefix, confirming
`engram 0.2.0-beta.1`, and verifying packaged HTTP `/health` returned
`{"status":"ok","service":"engram","version":"0.2.0-beta.1"}`.

T366 narrows the initial beta gate to either restored exact-head hosted CI or explicit
release-owner acceptance of this local CI plus package/install fallback, followed by marking PR #3
ready, merging, tagging, and publishing. It does not mark PR #3 ready, merge, tag, publish, close
hosted CI, run native Claude, prove hooks or host labels, or change production/GA readiness.

T367 refreshes the native Claude prompt-bearing, effective-hook, and live host-label preflight
after T366. Fresh read-only process inventory no longer shows an active native `claude` CLI
process, so the T363 attribution hard stop is cleared for the native CLI process class. Claude
Code `2.1.168`, the resolved SHA-256, installed Engram `0.2.0-beta.1`, daemon health, harness
status/doctor `ready=true`, snippet-only no-write dry-run, canonical vault status, and scoped
obligations doctor all match the preflight contract. Ambient Claude-family helper processes still
exist and must be listed in any future execution report. T367 does not launch native Claude or run
`/hooks`; T312, T335, and T270 remain default-deny until exact approval names the intended packet.

T368 reruns the same native-Claude/effective-hook/live-host-label preflight and finds that the
T367 attribution-clear state is no longer current. A live native `claude` CLI process is present
again on `ttys004` as PID `34797`, started on 2026-06-08 at 11:49:15 local time. That process
hard-stops T312 prompt-bearing proof, T335 `/hooks` effective-hook proof, and T270 live
host-label proof until it exits naturally or the user gives exact approval for a process action
and a fresh preflight passes. T368 is read-only and does not launch native Claude, run `/hooks`,
signal the process, mark PR #3 ready, merge, tag, publish, or change the beta scope.

T369 refreshes exact-head local beta validation after T368. `./scripts/local-ci.sh` passed on the
current PR #3 candidate, covering `git diff --check`, `cargo fmt --all --check`,
`cargo check --all-targets`, `cargo clippy --all-targets -- -D warnings`,
CI-like `cargo test --all-targets --jobs 1`, and `cargo doc --no-deps`.
`./scripts/package-install-smoke.sh` also passed, rebuilding the release package, verifying the
tarball checksum, installing the packaged binary into a temporary prefix, confirming
`engram 0.2.0-beta.1`, starting packaged `engram serve --http --memory`, and verifying packaged
HTTP `/health` returned `{"status":"ok","service":"engram","version":"0.2.0-beta.1"}`.
Hosted run `27138579667` on T368 head `8dad84e37b6bc033904319c675683345c7b60972` still fails all
jobs before workflow-step execution with `steps=[]`, so the remaining beta gate is explicit
release-owner acceptance of the local fallback or restored exact-head hosted CI green, followed by
ready/merge/tag/publish mechanics.

T370 refreshes real-session telemetry confidence for the Brain Harness evidence trail. The
50-trace project-scoped report first failed at `13/50` feedback records (`26%` coverage), then
improved to `19/50` (`38%`) after the initial feedback catch-up, and finally passed at `26/50`
(`52%`) after feedback was added only for judgeable recent traces. The 20-trace window now passes
at `18/20` (`90%`). This strengthens sampled retrieval-feedback evidence, but it is not beta
release approval and does not authorize M6 write-apply, lifecycle cleanup, native Claude,
effective hooks, live host labels, or production/GA readiness.

T371 refreshes local validation after T370 moved the PR head. `./scripts/local-ci.sh` passed on the
current candidate tree, covering whitespace diff check, rustfmt, check, clippy, serialized tests,
and docs. `./scripts/package-install-smoke.sh` also passed, rebuilding the release package,
verifying the tarball checksum, installing the packaged binary into a temporary prefix, confirming
`engram 0.2.0-beta.1`, starting packaged `engram serve --http --memory`, and verifying packaged
HTTP `/health` returned `{"status":"ok","service":"engram","version":"0.2.0-beta.1"}`.
Hosted run `27141590404` on T370 head `4249855bee0fe4b33a9bd343d7750ce7a8da368f` still fails all
jobs before workflow-step execution with `steps=[]`, so the remaining beta gate stays explicit
release-owner local-fallback acceptance or restored exact-head hosted CI green.

T372 refreshes current PR #3 and native-Claude production-gate evidence after T371. Current hosted
run `27142919365` targets head `3cf0e3d453fe4f02a0e1019bcf79fe8779e72cde` and still fails Docs,
Clippy, Test, Format, and Check before workflow-step execution with `steps=[]`. Claude Code
`2.1.168`, SHA-256 `377f0ecedba8246bdabdf312ce8b7cc8ae1160997b26f5edca352a4a8d61dc78`,
installed Engram daemon `0.2.0-beta.1`, harness status/doctor `ready=true`, clean obligations, and
canonical vault counts match the expected preflight state. A live native `claude` PID `34797` on
`ttys004` still blocks unambiguous T312 prompt-bearing proof, T335 `/hooks` effective-hook proof,
and T270 live host-label proof. T372 is read-only for runtime state and does not launch native
Claude, run `/hooks`, signal processes, mutate settings or adapters, mark PR #3 ready, merge, tag,
publish, or change the beta scope.

T373 makes stale active-session lint findings more actionable for production lifecycle review.
`engram-index/src/lint.rs` now includes project, agent, RFC3339 `started_at`, and `age_hours` in
each `stale_active_session` finding while preserving `safe_action=none`. This improves cleanup
review safety without ending sessions, archiving memory, running `lint apply_safe`, mutating M6,
changing ranking/`orient`, launching native Claude, or changing release scope. Focused lint tests,
public lint integration tests, fmt, check, clippy, local CI, and package/install smoke pass for the
T373 candidate.

T374 installs the T373 source behavior into the live local/Codex runtime. Before install,
`/Users/yuval.meiri/.local/bin/engram` and `./target/release/engram` both reported
`0.2.0-beta.1` but had different SHA-256 hashes. After `cargo build --release` and installing the
fresh binary, both paths hash to
`2446fe249b0b24745f47fafd356eec62fde2ca585b16c4f865f56d5e7c4c6a6c`; the daemon was restarted on
PID `30394`, `/health` returned `{"status":"ok","service":"engram","version":"0.2.0-beta.1"}`, and
installed `engram lint run --scope-project engram --limit 20 --json` showed stale active-session
messages with project, agent, `started_at`, and `age_hours`. T374 does not run `lint apply_safe`,
archive sessions or memory, mutate M6, change `orient`, launch native Claude, mark PR #3 ready,
merge, tag, publish, or change the beta scope.

T375 makes ready harness-doctor lifecycle evidence more actionable. When
`engram harness doctor` reports a ready harness, the soft lifecycle warning now names the exact
advisory trigger set from the policy: `task_start_orient`,
`before_major_decision_changes_since`, `after_discovery_record`,
`before_final_changes_since`, `before_final_obligations`,
`before_context_compaction_save`, `session_end_handoff`, and
`commit_workflow_consult_memory`. Focused harness unit tests, MCP harness integration tests, fmt,
check, clippy, release build, live installed runtime validation, daemon restart, `/health`, and the
existing dist checksum pass. T375 improves production-gate evidence quality only; it does not
install or mutate hooks/adapters, enforce lifecycle behavior, run lifecycle cleanup or M6, launch
native Claude, mark PR #3 ready, merge, tag, publish, or change the beta scope.

T376 makes lifecycle state machine-readable in harness reports. `HarnessStatusReport` now includes
a `lifecycle` object with `soft_contract=true`, `enforced=false`, the advisory trigger list, and a
summary message; the CLI text output also prints the lifecycle contract before adapter checks. This
lets agents and release checks inspect adapter readiness separately from advisory lifecycle
compliance without parsing warning text. Focused service and MCP harness tests, fmt, check, clippy,
release build, live installed runtime validation, daemon restart, `/health`, live JSON/text
`harness doctor`, live JSON `harness status`, and the existing dist checksum pass. T376 improves
structured evidence only; it does not install or mutate hooks/adapters, enforce lifecycle behavior,
run lifecycle cleanup or M6, launch native Claude, mark PR #3 ready, merge, tag, publish, or change
the beta scope.

T377 makes MCP tool availability state machine-readable in harness reports. `HarnessStatusReport`
now includes an `mcp_tools` object with `checked`, `required_tools`, `observed_tools`,
`missing_tools`, and `message`, while preserving the compatibility `missing_mcp_tools` field. This
removes the ambiguity where `missing_mcp_tools=[]` could mean either "all required tools were
observed" or "no observed tool list was supplied." The CLI text output now states whether MCP tools
were checked, checked complete, or checked with missing tool names. Focused service and MCP harness
tests, fmt, check, clippy, release build, daemon restart, `/health`, live installed JSON/text
`harness doctor`, live MCP checked missing-tool status, and dist checksum validation passed. The
installed CLI does not expose an observed-tool flag, so the checked path was not validated through
CLI text flags. T377 improves structured evidence only; it does not install or mutate
hooks/adapters, enforce lifecycle behavior, run lifecycle cleanup or M6, launch native Claude, mark
PR #3 ready, merge, tag, publish, or change the beta scope.

T378 exposes the T377 checked MCP tool path through the CLI. `engram harness status` and
`engram harness doctor` now accept repeatable `--observed-mcp-tool <TOOL>` flags and pass those
names into the same status/doctor service used by MCP. Omitting the flag preserves the unchecked
behavior. Focused CLI parse/check/clippy validation passed; the rebuilt installed binary hash is
`d7e17ae33bdfd48c84fd24070b1d10b17a284c0e31993e7a9b190c7450180b34`, the daemon restarted on PID
`39185`, `/health` returned ok, and installed CLI checked-complete plus checked-missing smokes
passed. T378 improves release/operator evidence only; it does not install or mutate hooks/adapters,
enforce lifecycle behavior, run lifecycle cleanup or M6, launch native Claude, mark PR #3 ready,
merge, tag, publish, or change the beta scope.

T379 refreshes the implementation-plan completion matrix after T378 closeout. The current matrix
now names PR #3 head `c876374db987252f4ad7ed88885ab55f30860b8a`, exact-head local CI plus
package-install smoke evidence, hosted run `27153053722` with `steps=0` on every job, and the
current native-Claude attribution blocker. T379 is a docs-only freshness correction; it does not
mark PR #3 ready, merge, tag, publish, accept hosted-CI fallback, close hosted CI, run native
Claude, prove effective hooks or live host labels, run lifecycle cleanup, mutate M6, or change the
beta scope.

T380 archives exactly the stale active M6 checkpoint MemoryItem
`019dd35d-1a48-7103-b0e2-390225f8b418` after T278 made its active
`Memory OS completion is paused at migration review gate` guidance stale for the current review
batch. Post-archive `memory(get)` reports `status=archived`, current T278 search no longer returns
that checkpoint in active memory results, project-scoped lint no longer reports its stale-feedback
finding, and obligations doctor is clean. T380 is exact lifecycle hygiene only; it does not delete
data, run broad `lint apply_safe`, run M6 write-apply, change source behavior, change ranking or
`orient`, accept hosted-CI fallback, mark PR #3 ready, merge, tag, publish, launch native Claude,
prove effective hooks, prove live host labels, or change the beta scope.

T383 restores the current sampled telemetry confidence signal after the 50-trace project window
regressed to `21/50` feedback records (`42%`). Four judgeable recent traces were scored:
current continuation orient `019ea8a5-f60b-76c2-9e53-016980300077`, T382 orient
`019ea884-8d3b-7f92-9433-2c3b35159208`, T382 `changes_since`
`019ea886-d4a6-7890-ac40-b1f3f52baf2d`, and T380 post-archive search
`019ea83c-150b-7011-998f-54f61ba618d4`. Fresh
`telemetry(action="real_session_eval", project="engram", limit=50)` now passes at `25/50`
feedback records (`50%`) with zero task failures, zero bad-memory-used, zero stale-memory, and
zero wrong-scope-memory outcomes. The 20-trace window also passes at `10/20` feedback records
(`50%`) with the same clean outcome counters. T383 changes no source behavior, telemetry formula,
or release scope; it does not accept hosted-CI fallback, mark PR #3 ready, merge, tag, publish,
run M6, run lifecycle cleanup, launch native Claude, prove hooks or host labels, or make the system
production/GA ready.

T384 archives exactly five stale active MemoryItems that project-scoped lint reported as
`feedback_stale_active_memory`: `019dd080-612a-7540-a028-42991c20ef1b`,
`019dd083-e014-74f1-95e5-b1eef478e894`, `019dd3a8-138d-7453-9991-d724f96a128f`,
`019dd3e4-9143-7721-9bff-b3fb505c8859`, and
`019e68a7-3375-7943-8ef0-dc0dde64c8bd`. The targets were April/May runtime,
implementation-plan, readiness, gap, and direct-search live-smoke snapshots now superseded by
current T337-T383 evidence. Post-archive project-scoped lint no longer reports those five stale
active-memory findings; the remaining sample is missing-evidence and stale-session debt with
`safe_action="none"`. Vault status remains aligned at `2749/2749`, and scoped obligations doctor
is clean. T384 preserves history and changes lifecycle state only; it does not delete data, run
broad `lint apply_safe`, end sessions, mutate M6, change ranking or `orient`, accept hosted-CI
fallback, mark PR #3 ready, merge, tag, publish, launch native Claude, prove hooks or host labels,
or make the system production/GA ready.

T385 replaces or archives the next five project-scoped missing-evidence MemoryItems. Three useful
implementation facts now have evidenced active successors: digest extraction apply
`019ea8e5-d8ba-7623-abb0-b4151504ad14`, orient contract checkpoint
`019ea8e6-19b0-7353-bc93-7124bfea5b61`, and real-session telemetry eval
`019ea8e6-59e6-7980-9a97-e08ab77073be`. Their old evidence-less records
`019dcaa6-0223-73a2-9fe4-76b61ff14faa`,
`019dfecb-16ca-71c2-b391-e3c216601590`, and
`019dfee0-90f8-7a61-a548-c2bddefcf897` are superseded. The stale global rolling handoff
`019dddbe-d369-7523-ac91-9bfeb016463b` and stale May 6 installed-runtime snapshot
`019dfee6-c83a-7c02-a3ed-78fc9e80329b` are archived. Post-change project-scoped lint starts at
stale active-session warnings with `safe_action="none"`, canonical vault status is aligned at
`2756/2756` after KnowledgeCommit `019ea8e8-f0ee-7f91-b78b-5b96060b9f0a`, and scoped obligations
doctor is clean. T385 changes Memory OS lifecycle/evidence state only; it does not run broad
cleanup, end sessions, mutate M6, change ranking or `orient`, accept hosted-CI fallback, mark PR #3
ready, merge, tag, publish, launch native Claude, prove hooks or host labels, or make the system
production/GA ready.

T386 closes exactly five project-scoped stale active-session lint findings while preserving durable
session history. The completed session records are
`019dd063-ee4f-7943-8409-0450bac3a724`,
`019e7d38-68d7-7652-b62f-3e8e635253ae`,
`019e7e6e-73e2-7e72-9351-da62e69686af`,
`019e8470-5d37-7db0-ac21-1725184849e7`, and
`019e990f-e4fb-7a02-a840-77a38dceab3e`. Matching stale coordination rows with empty
components/current_file were unregistered for all but `019e7d38`, which had no coordination row.
Post-cleanup project-scoped lint returns no findings, and the targeted coordination rows no
longer appear in `coord(list)`. T386 changes session lifecycle/coordination state only; it does
not delete session events, run broad `lint apply_safe`, mutate M6, change source behavior, accept
hosted-CI fallback, mark PR #3 ready, merge, tag, publish, launch native Claude, prove hooks or
host labels, clean unrelated coordination-only rows, or make the system production/GA ready.

T387 removes the remaining Engram-project coordination-only orphan rows after T386. The exact
rows were `019e683b-1560-7361-b535-53b012e04aa5`,
`d238eb81-870e-4da7-8e5d-381014f151b0`, and
`d96b3edb-f8ac-4d45-b644-92be32d0eae4`: all had empty components, null current_file, stale
June 1 heartbeats, and no durable session record from `session(get)`. Source inspection confirmed
`coord.unregister` deletes only the ephemeral `active_session` row, and the row contents are
preserved in `docs/BRAIN_HARNESS_T387_EXACT_COORD_ORPHAN_CLEANUP_2026-06-08.md`. Post-cleanup
`coord(list, project="engram")` returns `count=0`, project-scoped lint remains empty, and scoped
obligations doctor remains clean. T387 does not delete session events, run broad `lint apply_safe`,
mutate M6, change source behavior, accept hosted-CI fallback, mark PR #3 ready, merge, tag,
publish, launch native Claude, prove hooks or host labels, or make the system production/GA ready.

T388 refreshes exact-head release validation after T384-T387 lifecycle/evidence/session/coordination
cleanup. `./scripts/local-ci.sh` passed on the T388 head, covering `git diff --check`,
`cargo fmt --all --check`, `cargo check --all-targets`,
`cargo clippy --all-targets -- -D warnings`, CI-like `cargo test --all-targets --jobs 1`, and
`cargo doc --no-deps`. `./scripts/package-install-smoke.sh` also passed, rebuilding the release
package, verifying the checksum, installing the packaged binary into a temporary prefix,
confirming `engram 0.2.0-beta.1`, and verifying packaged HTTP `/health` returned
`{"status":"ok","service":"engram","version":"0.2.0-beta.1"}`. T388 refreshes local release
evidence only; it does not accept hosted-CI fallback, mark PR #3 ready, merge, tag, publish,
launch native Claude, prove hooks or host labels, mutate M6, or make the system production/GA
ready.

T390 hardens deterministic memory ranking for `ungated`, `un-gated`, and `not gated`
continuation wording. These variants now share the same continuation gate-negation normalization as
`non-gated` / `non gated`, so an M6 continuation prompt does not accidentally match the `gated`
substring and promote migration-gate context. Explicit modal gate prompts such as "should we
proceed with migration apply" still trigger gate guidance. Focused ranker and search regression
tests, formatter check, `cargo check -p engram-index`, focused clippy, and `git diff --check`
passed. Exact-worktree `./scripts/local-ci.sh` and `./scripts/package-install-smoke.sh` also
passed; the package smoke rebuilt the release archive, verified the checksum, installed the
packaged binary into a temporary prefix, confirmed `engram 0.2.0-beta.1`, and verified packaged
HTTP `/health` returned `{"status":"ok","service":"engram","version":"0.2.0-beta.1"}`. T390
changes only deterministic ranking query classification; it does not accept the hosted-CI fallback,
mark PR #3 ready, merge, tag, publish, launch native Claude, prove hooks or host labels, mutate M6,
or make the system production/GA ready.

T391 tightens deterministic gate-language matching after T390. Bare gate terms such as `gate`,
`gated`, `must`, `blocked`, `cannot`, and `never` now require ASCII word boundaries, while
multi-word or hyphenated phrases such as `approval gate`, `review-gated`, `do not`, `should not`,
and `requires approval` remain substring checks. This prevents unrelated words such as `gateway`,
`gatekeeper`, and `gatedness` from creating contextual M6 gate promotion. Focused ranker tests,
existing M6/current-plan search regressions, formatter check, `cargo check -p engram-index`,
focused clippy, and `git diff --check` passed. Exact-worktree `./scripts/local-ci.sh` and
`./scripts/package-install-smoke.sh` also passed; the package smoke rebuilt the release archive,
verified the checksum, installed the packaged binary into a temporary prefix, confirmed
`engram 0.2.0-beta.1`, and verified packaged HTTP `/health` returned
`{"status":"ok","service":"engram","version":"0.2.0-beta.1"}`. T391 changes only deterministic
ranking query classification; it does not accept the hosted-CI fallback, mark PR #3 ready, merge,
tag, publish, launch native Claude, prove hooks or host labels, mutate M6, or make the system
production/GA ready.

T392 tightens deterministic decision-gate action matching. Fallback action terms in
`asks_for_decision_gate` and migration-apply action terms in
`asks_for_explicit_migration_apply_gate` now require ASCII word boundaries, so incidental substrings
such as `mustache`, `blockchain`, `unblocked`, `allowance`, and `safetybelt` do not force
decision-gate mode. Explicit modal prompts and real boundary-delimited gate terms such as `must`,
`blocked`, `allowed`, `safety`, and `write-apply` still trigger gate classification. Focused ranker
tests, existing M6/current-plan search regressions, formatter check, `cargo check -p engram-index`,
focused clippy, and `git diff --check` passed. Exact-worktree `./scripts/local-ci.sh` and
`./scripts/package-install-smoke.sh` also passed; the package smoke rebuilt the release archive,
verified the checksum, installed the packaged binary into a temporary prefix, confirmed
`engram 0.2.0-beta.1`, and verified packaged HTTP `/health` returned
`{"status":"ok","service":"engram","version":"0.2.0-beta.1"}`. T392 changes only deterministic
ranking query classification; it does not accept the hosted-CI fallback, mark PR #3 ready, merge,
tag, publish, launch native Claude, prove hooks or host labels, mutate M6, or make the system
production/GA ready.

T393 adds an explicit beta release-owner signoff checklist in
`docs/BRAIN_HARNESS_T393_BETA_RELEASE_SIGNOFF_CHECKLIST_2026-06-09.md` and this release note. The
checklist names the exact local evidence, hosted-CI waiver condition, known beta limitations, and
manual release actions required after signoff. It does not accept the hosted-CI fallback, mark PR #3
ready, merge, tag, publish, launch native Claude, prove hooks or host labels, mutate M6, or make the
system production/GA ready.

T394 hardens `./scripts/package-install-smoke.sh` for release-artifact validation. Before extracting
the tarball, the smoke now rejects empty archive listings, absolute or parent-directory paths,
members outside the expected `engram-<version>-<host-triple>/` root, and archives missing required
package members. The `/health` checks now use fixed-string matching. The valid package smoke passed,
including checksum verification, archive-path inspection, temp-prefix install,
`engram 0.2.0-beta.1`, and packaged HTTP `/health`; a synthetic wrong-root archive failed closed
before extraction. T394 does not accept the hosted-CI fallback, mark PR #3 ready, merge, tag,
publish, launch native Claude, prove hooks or host labels, mutate M6, or make the system
production/GA ready.

T395 commits `Cargo.lock` and makes the local/hosted CI plus release-package paths use the locked
dependency graph. `.github/workflows/ci.yml`, `./scripts/local-ci.sh`, `./scripts/package-release.sh`,
and `./scripts/package-install-smoke.sh` now use `--locked` for dependency-resolving Cargo commands,
and README development commands reflect the locked workflow. This makes dependency drift fail
explicitly instead of silently changing release inputs between local validation, hosted CI, and
archive creation. Validation passed locked metadata/pkgid checks, script syntax checks, locked
check/clippy, package/install smoke, and the full locked `./scripts/local-ci.sh`. T395 does not
accept the hosted-CI fallback, mark PR #3 ready, merge, tag, publish, launch native Claude, prove
hooks or host labels, mutate M6, or make the system production/GA ready.
