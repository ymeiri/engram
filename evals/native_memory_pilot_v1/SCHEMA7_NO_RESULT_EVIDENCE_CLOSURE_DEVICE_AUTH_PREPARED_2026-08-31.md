# Schema 7 no-result evidence-closure device-auth successor

## Status

Prepared and provider-free attested on 2026-08-31. No provider call or login completed. All six
lanes remain in `prepared`; the three isolated Codex lanes report `not_logged_in`. Teaching,
activation, evaluation, agent-output, and runner-report targets are absent.

This is a fresh forward-only diagnostic. Never run or repair the invalidated August 27 or August 31
browser-auth plans, and never replay a completed pilot lane.

## Why this successor exists

The callback-based login attempt failed after the CLI opened its OAuth URL and the same URL was
opened a second time, producing a CSRF mismatch. Codex also supports ChatGPT device-code OAuth,
which avoids the localhost callback. The evaluator now freezes that path as
`chatgpt_device_keyring`; it continues to require isolated `CODEX_HOME` directories, macOS
Keychain storage, the exact `Logged in using ChatGPT` status, and removal of token/API-key
environment variables.

The Codex executable is copied into the durable private artifact root so a ChatGPT application
update cannot change the attested bytes before the pilot runs.

## Frozen evidence

- Protocol:
  `evals/native_memory_pilot_v1/protocol-schema-7-no-result-evidence-closure-forward-device-auth.json`
  - SHA-256: `5720fa352d2cb69c4e06ac35f62b6fb71d70018ccc9f9a0f8ae1a6768f9cd102`
- Run plan:
  `/Users/yuval.meiri/.engram/evals/native-memory-pilot/no-result-evidence-closure-forward-device-auth-20260831-01/run/run-plan.json`
  - SHA-256: `d579c8af4a92d3ed404081f902122e81debe14e8ea85bae7f7e2786b1135786b`
- Frozen evaluator: `engram-eval 0.2.3`
  - SHA-256: `f8af6ce2c2f35b27283e9bb584cb1ef7bf22fd1cbeb2353cb874acc32ff23c8b`
- Frozen Engram: `engram 0.2.3`
  - SHA-256: `905f9bad0c964d95957c6e7be717c432329f166636f84893e5c67ef11cf26018`
- Frozen Codex: `codex-cli 0.151.0-alpha.7.2`
  - SHA-256: `a6042937174f72112dbd2d554a4af36936422e0c5ac69e353dc68994458996e9`
- Frozen Claude Code: `2.1.247`
  - SHA-256: `5086b9b64d8bb842e1f599cdd3767ab08c6b2266e462fcc5686ae4b019cca8f7`

Provider-free attestation returned `verified=true`. The effective Engram MCP contract is runtime
verified with schema 3, the `agent` profile, six tools, and the frozen Engram executable hash.
Lifecycle audit returned `invalid=false`, zero lane failures, and six `prepared` lanes. Disk reserve
was 106 GiB, and repository `target/debug` was absent.

## Validation

The following commands passed against the external Cargo target
`/private/tmp/engram-forward-evidence-closure-target`:

```text
cargo test -p engram-eval --all-targets
125 passed; 0 failed

cargo clippy -p engram-eval --all-targets -- -D warnings
passed

cargo fmt --all --check
passed
```

## Exact next action

Retain the user-corrected light cadence: attempt only one login after an explicit user request,
then stop for that turn. Lane 2 is first. Run its frozen command with only its isolated home:

```bash
CODEX_HOME=/Users/yuval.meiri/.engram/evals/native-memory-pilot/no-result-evidence-closure-forward-device-auth-20260831-01/run/lanes/02-learned_procedure_wrong_repository_scope-codex_native_memory-r1/codex-home \
  /Users/yuval.meiri/.engram/evals/native-memory-pilot/no-result-evidence-closure-forward-device-auth-20260831-01/bin/codex \
  login --config 'cli_auth_credentials_store="keyring"' --device-auth
```

Do not open a second login, copy credentials, use an API key/token, execute a provider phase, or
attempt another lane in the same user-triggered retry. After the user completes the device flow,
run the frozen provider-free status command for lane 2 and stop.
