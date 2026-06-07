# T304 Beta Release Metadata And Runtime Refresh Gate

Date: 2026-06-07
Branch: `yuval.meiri/memory-os-phase1`
Previous head: `6c86012b08126d02f33be1865fd78d55e2939a4e`
PR: <https://github.com/ymeiri/engram/pull/3>

## Question

Can phase 1 move toward a shippable beta without installing/adopting local harness files or
restarting user runtime state?

## Answer

Yes. T304 prepares repo-owned beta release metadata and records the installed-runtime refresh gate.
It does not install adapters, update `/Users/yuval.meiri/.local/bin/engram`, restart the daemon, or
touch user-owned `AGENTS.md`.

## Evidence

- PR #3 was open/draft and merge-clean at
  `6c86012b08126d02f33be1865fd78d55e2939a4e`.
- Exact-head CI run `27080135030` passed Format, Docs, Check, Clippy, and Test on that head.
- `cargo build -p engram-cli` builds the local candidate binary as `engram-cli v0.2.0-beta.1`,
  and `target/debug/engram --version` returns:

```text
engram 0.2.0-beta.1
```

- Source-rendered Codex guidance from `target/debug/engram harness render --harness codex
  --adapter codex-memory-session-skill` includes:

```text
obligations(action=doctor, project=..., cwd=...)
```

- The installed global Codex skill at
  `/Users/yuval.meiri/.codex/skills/engram-memory-session/SKILL.md` still contains:

```text
obligations(action=doctor)
```

- The installed global binary render path also still emits the unscoped line:

```bash
/Users/yuval.meiri/.local/bin/engram harness render --harness codex \
  --adapter codex-memory-session-skill | rg -n "obligations\\(action=doctor"
```

returned:

```text
28:  `obligations(action=doctor)`; resolve or explicitly skip open obligations. If a document changes
```

## Release Metadata Prepared

- Workspace version moves to `0.2.0-beta.1`.
- `CHANGELOG.md` gains a `0.2.0-beta.1` pre-release entry with supported scope and known
  limitations.
- `docs/RELEASE_NOTES_V0_2_0_BETA_1.md` defines the beta-supported path, deferrals, and release
  gates.
- README and MCP setup docs tighten host-scope caveats so connection examples are not confused with
  host behavioral parity proof.

## Not Authorized By T304

- Installing or adopting Codex, Claude, Gemini, Cursor, or generic harness files.
- Replacing `/Users/yuval.meiri/.local/bin/engram`.
- Restarting or signaling the running daemon.
- Running native Claude prompt-bearing validation.
- Marking PR #3 ready, merging it, tagging a release, or publishing a GitHub release.
- Editing, staging, or committing root `AGENTS.md`.

## Next Closure Condition

After explicit approval for runtime refresh, install the candidate build/adapters from the exact
release head, rerun the local/Codex smoke, verify installed render output includes scoped
`obligations(action=doctor, project=..., cwd=...)`, and then require fresh exact-head CI before
marking PR #3 ready or tagging `v0.2.0-beta.1`.
