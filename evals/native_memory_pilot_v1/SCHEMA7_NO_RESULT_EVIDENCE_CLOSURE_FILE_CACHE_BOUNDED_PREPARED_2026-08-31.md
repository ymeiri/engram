# Schema 7 no-result evidence-closure file-cache bounded successor

## Status

Historical pre-teaching checkpoint. This candidate was subsequently retired after its one permitted
teaching invocation exposed an incomplete frozen Codex runtime. Never resume, repair, or replay it.

This is a fresh forward-only diagnostic. Never modify or run the earlier device-Keychain candidate,
never repair this successor after a provider failure, and never replay a completed lane.

## Frozen evidence

- Protocol:
  `evals/native_memory_pilot_v1/protocol-schema-7-no-result-evidence-closure-forward-file-cache-bounded.json`
  - SHA-256: `ea7e61043b54cd5017c7bb60d5e77300503860937e6c6cf7d93a5a41c6308bdd`
- Run plan:
  `/Users/yuval.meiri/.engram/evals/native-memory-pilot/no-result-evidence-closure-forward-file-cache-bounded-20260831-01/run/run-plan.json`
  - SHA-256: `32fad8f59b07e5eb0da4a9cc3539541204a334605cc76e44d496e027ff7735bc`
- Frozen evaluator: `engram-eval 0.2.3`
  - SHA-256: `69c7840e3f0b0681812ccce5919c8dd71be1f6e53b31a90cdffd941fadecb95e`
- Frozen Engram: `engram 0.2.3`
  - SHA-256: `905f9bad0c964d95957c6e7be717c432329f166636f84893e5c67ef11cf26018`
- Frozen Codex: `codex-cli 0.151.0-alpha.7.2`
  - SHA-256: `a6042937174f72112dbd2d554a4af36936422e0c5ac69e353dc68994458996e9`
- Frozen Claude Code: `2.1.247`
  - SHA-256: `5086b9b64d8bb842e1f599cdd3767ab08c6b2266e462fcc5686ae4b019cca8f7`

The protocol is structurally identical to the bounded device-auth source except for the new pilot
ID and `codex_authentication_mode=chatgpt_file_cache`. Every Codex login, status, teaching,
activation, and evaluation argv freezes `cli_auth_credentials_store="file"`. The run-plan digest
matches its SHA-256 file. Provider-free attestation returned `verified=true`, including the exact
binary hashes and effective six-tool Engram runtime contract.

## Protected cache provisioning

Before provisioning, the lifecycle audit returned `invalid=false`, all six lanes were `prepared`,
and there were zero failures or provider outputs. All three destination files were absent. The
operator's source cache passed the private regular-file gate: owner UID 502, mode `0600`, one link,
4,657 bytes, and exact frozen status `Logged in using ChatGPT`.

The operator explicitly approved three protected plaintext copies. The confirmation-gated
provisioning command ran exactly once and returned only a sanitized readiness report. Exactly three
lane-local `auth.json` files now exist. Each is a regular non-symlink file with owner UID 502, mode
`0600`, one link, and 4,657 bytes inside a `0700` isolated `CODEX_HOME`. Credential contents and
content digests were never printed, inspected, or recorded.

The post-provision check returned:

```text
provider_free=true
ready=true
lane 2 codex_native_memory: ready
lane 4 codex_engram_plus_native: ready
lane 6 codex_lean_engram: ready
```

Re-attestation remained verified. The plan hash remained
`32fad8f59b07e5eb0da4a9cc3539541204a334605cc76e44d496e027ff7735bc`; the audit remained
`invalid=false`, six `prepared` lanes, and zero failures. All provider outputs remained absent,
repository `target/debug` remained absent, and free disk reserve remained positive. The earlier
device-Keychain candidate retained plan SHA-256
`b8c371f6e6044034f97f697e590fcab671b5db1e508c417acdabf25ad7ed939f` and zero lane-local
`auth.json` files.

## Forward-only next boundary

Before every provider phase, revalidate the exact plan and binary hashes, effective runtime
attestation, lifecycle audit, all three cache metadata contracts, exact sanitized readiness, disk
reserve, and absent outputs for that phase. If pristine, teaching may run exactly once with the
already-authorized 50-cent Claude aggregate allocation. Then wait the frozen one-hour Codex idle
interval before activation. Any cache-refresh conflict or provider failure invalidates this
candidate; do not repair, refresh one lane, replay, or continue later lanes.

## Subsequent one-shot teaching result

The frozen teaching command ran once. Lanes 1 through 4 produced traces; lanes 5 and 6 never ran.
Both Codex traces reported that `codex-code-mode-host` was missing beside the frozen `codex`
executable. Neither Codex lane executed the failed probe, the successful probe, or an Engram write.
The lane-4 trusted selector then stopped the phase because it found zero matching procedure
candidates. The exact terminal error was:

```text
invalid evaluation data: trusted evaluator expected exactly one matching procedure candidate, found 0
```

The generic JSONL lifecycle audit still labeled the two Codex teaching traces `passed` because the
Codex process exited 0 after explaining its tool-host failure. That classification is not accepted
as success. The candidate is operationally retired even though the old audit reports
`invalid=false`. Activation and evaluation are forbidden.

Preserved trace SHA-256 values:

```text
5f66160758dd0bbdd90d3707d13c45f29a8d6cc4612a2226fc6984d415cebf31  lane 1 teaching
9ab27598a4bc05b30410345818219ab244d66b65aabe7c208eda7cd85407bdba  lane 2 teaching
36c83604ce9b0d93574217298f53989f0f74e7853b35a94f9c67c9382bd8f191  lane 3 teaching
eeac4e1e2ca98bc68e242091aa5d816cf52874da335b8072c4742e48ceaeba88  lane 4 teaching
6a9bf55ed7adeb65b0cb0e9454723874df9f000b404db83650da17484e7be55d  lane 4 stderr
```

Claude lanes 1 and 3 reported `$0.0464325` and `$0.0412393`. The successor ledger rounds those
independently to 46,433 and 41,240 microusd and carries the 87,673-microusd total forward.
