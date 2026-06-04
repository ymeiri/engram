# Brain Harness T264 Claude Code Session ID Fallback

Date: 2026-06-04
Status: completed source-level implementation. This slice adds a guarded Claude Code
`CLAUDE_CODE_SESSION_ID` fallback for existing external-session telemetry labels in CLI and MCP
source. It does not add public MCP parameters, change response payload shape, change
schema/storage/index/document-index behavior, edit hooks/settings/adapters, refresh the installed
runtime, run native Claude, mutate lifecycle or M6 state, push or set upstream, delete data, or
touch user-owned files.

## Research Question

Can Engram reduce Claude Code null `external_session_id` traces by adopting a documented
host-native session signal without violating the prior rule against synthesizing labels from
unrelated metadata?

## Evidence Inputs

- Official Claude Code environment-variable documentation says `CLAUDECODE` is set to `1` in
  subprocesses Claude Code spawns, including stdio MCP server subprocesses, and
  `CLAUDE_CODE_SESSION_ID` is set automatically in Bash, PowerShell, hook, and stdio MCP server
  subprocesses. It also warns that an MCP server subprocess can retain the startup ID after
  resume/continue.
- Official Gemini CLI documentation exposes resume/list-session CLI behavior and configuration
  environment variables, but no documented MCP-subprocess session-id environment variable was found
  in the checked docs.
- AI Council recall resurfaced the prior host-label caution: do not infer telemetry labels from
  generic transport/process metadata without a host-session contract.

Sources:

- <https://code.claude.com/docs/en/env-vars>
- <https://github.com/google-gemini/gemini-cli/blob/main/docs/cli/cli-reference.md>
- <https://github.com/google-gemini/gemini-cli/blob/main/docs/reference/configuration.md>

## Hypotheses

- Preferred: a private edge resolver can use `CLAUDE_CODE_SESSION_ID` only when `CLAUDECODE=1`,
  after explicit labels and `ENGRAM_EXTERNAL_SESSION_ID`, and before Codex fallback. This prevents
  inherited Codex env from mislabeling Claude-spawned MCP/CLI work.
- Null: Engram should keep Claude host labels caller-supplied only because MCP subprocess env can be
  stale after resume/continue.
- Simpler alternative: document `ENGRAM_EXTERNAL_SESSION_ID=claude-code://sessions/$CLAUDE_CODE_SESSION_ID`
  as manual setup. Rejected for this source slice because Claude Code already provides a documented
  subprocess session signal and because manual setup would not prevent inherited Codex fallback from
  winning in Claude-spawned subprocesses.
- Failure: ambient or malformed env labels non-Claude runs, overrides user intent, or broadens the
  telemetry/public API surface.

## Consultation

AI Council recall found the T217/T262 host-label guidance. A fresh three-model broadcast agreed the
slice is safe if it stays host-specific, guarded, telemetry-only, and lower precedence than explicit
labels and `ENGRAM_EXTERNAL_SESSION_ID`. All three responses specifically recommended that Claude
fallback outrank Codex fallback when both host envs are present, because a Claude subprocess can
inherit Codex parent env while Claude is the immediate execution host.

Claude Bridge read-only critique timed out twice. This is a consultation confound, not evidence for
or against the change. No material AI Council disagreement required pausing.

## Change

The resolver precedence is now:

1. Explicit MCP request or CLI flag value.
2. `ENGRAM_EXTERNAL_SESSION_ID`.
3. Guarded `CLAUDE_CODE_SESSION_ID` as `claude-code://sessions/{trimmed_id}`.
4. Guarded `CODEX_THREAD_ID` as `codex://threads/{trimmed_id}`.
5. No label.

`CLAUDE_CODE_SESSION_ID` is ignored unless `CLAUDECODE` is exactly `1`. Session IDs longer than 128
bytes or containing characters outside ASCII alphanumeric, `-`, and `_` are ignored. Invalid
Claude values fall through to Codex only when Codex is independently guarded and valid. CLI
whitespace explicit values retain the existing behavior: they normalize to no label and do not
fall through to inferred host labels.

Gemini remains unimplemented for source-level host fallback in this slice. There is no safe
documented session-id environment contract to use, and the resume/list-session CLI surface is not
an MCP subprocess label source.

## Validation

Passed:

```text
cargo test -p engram-mcp external_session_id
cargo test -p engram-cli external_session_id
cargo test -p engram-tests --test telemetry_tests
cargo fmt --all --check
cargo check -p engram-cli
cargo clippy --all-targets -- -D warnings
git diff --check
```

Focused coverage proves:

- explicit values and `ENGRAM_EXTERNAL_SESSION_ID` still win;
- guarded Claude labels are emitted as `claude-code://sessions/{id}`;
- `CLAUDE_CODE_SESSION_ID` without `CLAUDECODE=1` is ignored;
- unsafe Claude session values are rejected;
- Claude wins over Codex when both guarded host signals are present;
- Codex remains available when Claude is missing or invalid;
- runtime MCP env tests isolate and verify env behavior; and
- telemetry record-trace still uses `ENGRAM_EXTERNAL_SESSION_ID` when present.

Telemetry integration, formatting, CLI check, full clippy, and diff whitespace checks also passed.

## Remaining Gap

This is source-level validation only. The installed daemon/runtime has not been refreshed for T264,
so live Claude Code MCP traces will not use this fallback until a separate runtime refresh is run.
T264 also does not prove prompt-bearing native Claude behavior, effective hook visibility, Gemini
host-label adoption, lifecycle cleanup, M6 completion, or remote publication/upstream policy.

The Claude docs note that stdio MCP subprocesses can retain their startup session ID after
resume/continue. T264 accepts that as a bounded label-staleness limitation rather than adding a
polling, hook, or lifecycle mechanism.

## Decision

T264 narrows Claude Code source-level host-label adoption without expanding Engram's public surface.
It also reduces the risk that inherited Codex env mislabels Claude-spawned MCP/CLI traces. Gemini
host-label adoption remains unproved and intentionally untouched.
