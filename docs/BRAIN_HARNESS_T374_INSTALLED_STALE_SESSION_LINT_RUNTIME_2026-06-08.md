# Brain Harness T374 Installed Stale Session Lint Runtime

Date: 2026-06-08
Status: installed and validated the T373 lint context in the live local/Codex runtime.

## Research Question

Does the installed Engram runtime that agents use now include the T373 stale-session lint context,
or is the source change still only present in the repository/package candidate?

Preferred hypothesis: installing the freshly rebuilt `0.2.0-beta.1` binary and restarting the
daemon makes live `engram lint run --scope-project engram --json` output include project, agent,
`started_at`, and `age_hours` for `stale_active_session` findings.

Null hypothesis: the installed binary or daemon remains on an older hash and live lint output still
lacks the T373 context.

Failure hypothesis: rebuild, install, daemon restart, or live lint validation fails, leaving the
runtime drift unresolved.

## Preflight

Before install, the repository release binary and installed runtime both reported
`engram 0.2.0-beta.1`, but their hashes differed:

- installed `/Users/yuval.meiri/.local/bin/engram`:
  `77a08e895614bea3b02816e67bafd64087ea0634f4b0ca58b8199a9ef7855633`
- repository `./target/release/engram`:
  `7a41409f4ac63565d3b7e2f31056212c38799c69ea5f53e0ca1cf9652c979b00`

The running daemon was PID `47577`, spawned by `/Users/yuval.meiri/.local/bin/engram`, with spawn
version `0.2.0-beta.1`.

## Action

T374 rebuilt and installed the release binary:

- `cargo build --release`
- `install -m 755 ./target/release/engram /Users/yuval.meiri/.local/bin/engram`

After install, installed and repository binary hashes matched:

`2446fe249b0b24745f47fafd356eec62fde2ca585b16c4f865f56d5e7c4c6a6c`

The daemon was restarted cleanly:

- `engram daemon stop`
- `engram daemon start`

Post-restart daemon status reported PID `30394`, spawned by
`/Users/yuval.meiri/.local/bin/engram`, spawn version `0.2.0-beta.1`. HTTP health returned
`{"status":"ok","service":"engram","version":"0.2.0-beta.1"}`.

## Validation

Live installed runtime validation passed with:

- `engram lint run --scope-project engram --limit 20 --json`
- `engram harness status --harness codex --json`
- `engram harness doctor --harness codex --json`
- `cd dist && shasum -a 256 -c engram-0.2.0-beta.1-aarch64-apple-darwin.tar.gz.sha256`

The live lint output now includes T373 context in installed-runtime findings, for example:

```text
Session has been active for more than one day (project: engram, agent: codex,
started_at: 2026-04-27T19:21:43.247812Z, age_hours: 1003); consider ending or abandoning it.
```

Codex harness status/doctor remained `ready=true`. The doctor still reports the known advisory
warning that lifecycle compliance is soft and depends on the agent following policy.

## Gate Impact

T374 closes installed-runtime drift for the T373 stale-session lint context. Agents using the local
daemon can now see the enriched stale-session review fields without rebuilding from source.

This does not end or abandon sessions, archive memory, run `lint apply_safe`, mutate M6/migration
state, change ranking or `orient`, launch native Claude, run `/hooks`, signal processes, mutate
settings/adapters, mark PR #3 ready, merge, tag, publish, or change beta scope.
