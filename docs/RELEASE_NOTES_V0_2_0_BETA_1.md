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

Before tagging this beta, the candidate commit must have:

- exact-head CI green for Format, Docs, Check, Clippy, and Test,
- `cargo fmt --all --check` passing locally,
- a focused local/Codex smoke confirming current source-rendered harness guidance,
- canonical generated vault status count-aligned,
- `obligations(action=doctor, project=engram, cwd=/Users/yuval.meiri/projects/engram)` clean,
- a refreshed installed runtime/adapters check.

Current PR #3 head `8f228ecacd436fb4f6c0078e59fb385eacc800eb` has strong local validation:
`cargo fmt --all --check`, `git diff --check`, `cargo test -p engram-index harness::tests`,
`cargo clippy --all-targets -- -D warnings`, and full `cargo test` all pass. Hosted CI run
`27090842423` failed before running workflow steps because GitHub Actions reported an account
billing/spending-limit block. That external account gate does not contradict the local validation,
but it means the normal exact-head hosted-CI release proof is still missing until Actions can rerun.

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
