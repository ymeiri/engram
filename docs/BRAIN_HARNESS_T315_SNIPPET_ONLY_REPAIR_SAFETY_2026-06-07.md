# T315 Snippet-Only Claude Adapter Repair Safety

Date: 2026-06-07
Status: completed source-level safety hardening slice

## Question

Can the future T314 Claude Code adapter repair be made safer by proving that
`--settings-target snippet-only` can repair generated adapters without rewriting existing Claude
settings or an existing Engram settings snippet?

## Hypotheses

| Hypothesis | Result |
| --- | --- |
| Preferred | A focused source test can prove that snippet-only write mode repairs generated adapter drift while leaving existing `settings.json`, `settings.local.json`, and `engram-settings-snippet.json` unchanged. | Supported. |
| Null | Existing tests already cover this real-user safety case. | Rejected. |
| Failure | The slice executes T314, mutates real user harness files, or weakens the approval gate. | Avoided. |

## Evidence

Commit `8f228ecacd436fb4f6c0078e59fb385eacc800eb` adds the test
`claude_install_snippet_only_repairs_adapters_without_rewriting_existing_settings` in
`engram-index/src/harness.rs`.

The test creates a temporary Claude root with:

- existing `.claude/settings.json`,
- existing `.claude/settings.local.json`,
- existing `.claude/engram-settings-snippet.json`,
- a stale generated `commands/engram-memory-session.md` adapter.

It then runs `HarnessService::install_with_options` with:

```rust
HarnessInstallOptions {
    write: true,
    adopt_user_owned: false,
    settings_target: HarnessSettingsTarget::SnippetOnly,
}
```

The assertions prove that:

- the stale generated adapter is rewritten,
- no settings file or existing snippet path appears in the written-file report,
- the settings merge is skipped with a snippet-only message,
- all three existing settings/snippet files remain byte-for-byte unchanged,
- the repaired adapter contains the current scoped obligation guidance.

## Validation

These commands passed locally on head `8f228ecacd436fb4f6c0078e59fb385eacc800eb`:

```bash
cargo fmt --all --check
git diff --check
cargo test -p engram-index harness::tests
cargo clippy --all-targets -- -D warnings
cargo test
```

PR #3 hosted CI run `27090842423` did not run the workflow steps. GitHub check-run annotations say:

```text
The job was not started because recent account payments have failed or your spending limit needs to be increased.
```

Treat that as an external GitHub Actions account gate, not as evidence that the T315 code failed.
It still blocks normal exact-head hosted-CI release proof until the account issue is fixed and the
checks are rerun.

## Boundary

T315 does not execute T314, run `harness install --write` against the real user home, edit
`~/.claude`, merge `settings.json`, change `settings.local.json`, rewrite the existing Engram
settings snippet, launch native Claude, run `/hooks`, send process signals, mark PR #3 ready, merge,
tag, publish, or change beta scope.

The future T314 write gate remains unchanged: it requires explicit user approval for the exact
`snippet-only --write` command and must verify that only the generated adapter paths changed.
