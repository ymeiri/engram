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

T315 adds source-level coverage for the future T314 repair path. The new harness test proves that
`HarnessSettingsTarget::SnippetOnly` can repair generated Claude Code adapters without rewriting an
existing `settings.json`, `settings.local.json`, or `engram-settings-snippet.json`.

This is safety evidence for the approval-gated T314 command, not execution of that command. Calling
the Claude Code harness beta-ready still requires explicit approval for T314 or an explicit beta
decision that Claude Code adapter repair is deferred from the supported path.
