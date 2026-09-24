# Native-pilot isolated file-cache authentication support — 2026-08-31

## Outcome

The evaluator now supports an explicit `chatgpt_file_cache` Codex authentication mode in addition
to the existing browser-Keychain and device-Keychain modes. Preparation remains provider-free and
does not access a real credential. On explicit operator approval, the separate guarded provisioning
command created three protected lane-local copies without printing, hashing, or storing their
contents in evidence.

Official Codex documentation states that local credentials may be stored in `auth.json`, that this
file contains access tokens and must be treated like a password, and that copying the cache is a
supported fallback when browser or device authentication is unavailable:
<https://learn.chatgpt.com/docs/auth>.

## Frozen contract

A file-cache protocol freezes all Codex login, status, teaching, activation, and evaluation argv
with:

```text
cli_auth_credentials_store="file"
```

The runner removes `CODEX_ACCESS_TOKEN` and `OPENAI_API_KEY` from every Codex child process and
requires the status command to report exactly `Logged in using ChatGPT`. It reports a missing cache
as `not_logged_in` and fails closed on an invalid cache before provider execution.

On Unix, `CODEX_HOME` must be a real `0700` directory. Its `auth.json`, when present, must be a
regular non-symlink file with mode `0600`, the same owner as the home, exactly one hard link, valid
JSON-object shape, and a size from 1 byte through 1 MiB. Keyring plans continue to reject any
lane-local `auth.json`.

Preparation never performs the credential copy. Provisioning remains a separate operator-approved
step against a fresh file-cache successor. Existing frozen plans and their credential-store modes
are unchanged.

The evaluator now exposes `provision-native-memory-pilot-auth-cache`. It refuses to access the
source unless `--confirm-plaintext-cache-copies` is present, revalidates the plan digest, frozen
binaries, effective Engram runtime contract, recovery absence, and all-prepared zero-failure audit,
and accepts only `chatgpt_file_cache` Codex lanes. The source must pass the same private regular-file
checks and have the same owner as every destination home. All destinations are preflighted as
absent; the command never overwrites or reuses a cache, never returns source contents or a content
digest, and emits only the existing sanitized authentication-readiness report. If a copy fails,
best-effort rollback removes only destination files created by that invocation; any remaining file
causes a retry to fail closed. The source bytes are held in a `Zeroizing<Vec<u8>>`, so the
provisioning buffer is zeroized on every success and error return. After every destination is
preflighted, the source is opened once with symlink following disabled, validated and owner-checked
from the opened file metadata, and read through that same handle with a post-open 1 MiB cap. This
removes the prior validate-then-reopen time-of-check/time-of-use window.

A canary-only transaction test now exercises the public provisioning function end to end with a
frozen synthetic plan, binary and runtime attestations, a pristine prepared-lane audit, protected
copy creation, exact sanitized ChatGPT readiness, and a second-call no-overwrite rejection. The
synthetic cache value does not appear in the report or rejection.

## Validation

- `cargo fmt --all --check`: passed.
- `CARGO_TARGET_DIR=/private/tmp/engram-forward-evidence-closure-target cargo test -p engram-eval`:
  142 passed, 0 failed.
- `CARGO_TARGET_DIR=/private/tmp/engram-forward-evidence-closure-target cargo clippy -p engram-eval
  --all-targets -- -D warnings`: passed.
- Repository `target/debug`: absent.
- Free space on the repository volume: 90 GiB.

Exact SHA-256 evidence:

```text
bef87d358a32054ecb1a5d9caf9677f6136944b031c5cb191bbe7200f6927ad4  engram-eval/src/native_pilot.rs
a4f6f989375365249a2705ddfc38e80a1cbb8c574dc9e639f31fcc85bcea8398  engram-eval/src/native_runner.rs
ad21126afe56de6c82c497ac0b1d14acb27cebc30c097d80ae68f086b7af199e  engram-eval/src/pilot.rs
c992f8e1fcaafe28ee847fa2fd089feda036a6af9e399bec5de490b659b11b1f  engram-eval/src/native_audit.rs
b74207fa430547efb3443831d8f20d6c9f9de9238609087438e7664344b70de1  engram-eval/src/main.rs
5238c63cb31eb420ad27ae50c44e92cfbb39bdab1c895dade1cf066770c480c6  engram-eval/Cargo.toml
bcb0dbbde422fb5afb60babf6ccbefc4071bdfe7b7e9af0ac72c637c2b2c2fbd  Cargo.lock
1b78dbc49dab1391dccdf392d3e95542697edc319a6b21222719692b74819e18  evals/native_memory_pilot_v1/README.md
72a2fdb44e7d352b430af643a98c17838f3c306f0533e9258b23aea3c94c41ef  evals/native_memory_pilot_v1/SCHEMA7_NO_RESULT_EVIDENCE_CLOSURE_RESOURCE_BUDGETS_PREPARED_2026-08-31.md
e1d2ac600fd4e0a41efdeb3d3f03563eeab483fc8d026563767b33076eae5ae6  /private/tmp/engram-forward-evidence-closure-target/debug/engram-eval
```

## Real bounded-successor checkpoint

The operator approved a fresh bounded file-cache successor and three protected plaintext copies.
The successor is:

```text
/Users/yuval.meiri/.engram/evals/native-memory-pilot/no-result-evidence-closure-forward-file-cache-bounded-20260831-01
```

Exact frozen evidence:

```text
ea7e61043b54cd5017c7bb60d5e77300503860937e6c6cf7d93a5a41c6308bdd  evals/native_memory_pilot_v1/protocol-schema-7-no-result-evidence-closure-forward-file-cache-bounded.json
32fad8f59b07e5eb0da4a9cc3539541204a334605cc76e44d496e027ff7735bc  run/run-plan.json
905f9bad0c964d95957c6e7be717c432329f166636f84893e5c67ef11cf26018  bin/engram
a6042937174f72112dbd2d554a4af36936422e0c5ac69e353dc68994458996e9  bin/codex
5086b9b64d8bb842e1f599cdd3767ab08c6b2266e462fcc5686ae4b019cca8f7  bin/claude
69c7840e3f0b0681812ccce5919c8dd71be1f6e53b31a90cdffd941fadecb95e  bin/engram-eval
```

The run-plan digest matched, frozen-binary and effective-runtime attestation returned
`verified=true`, and the lifecycle audit returned `invalid=false`, six `prepared` lanes, and zero
failures. Before provisioning, all three destinations were absent and all three isolated Codex
lanes reported `not_logged_in`. The source was a regular non-symlink file owned by the operator with
mode `0600`, one link, a 4,657-byte size, and exact file-store status
`Logged in using ChatGPT`.

The guarded command ran exactly once. It created exactly three lane-local `auth.json` files. Each
is a regular non-symlink file owned by the operator with mode `0600`, one link, and a 4,657-byte
size; their contents and content digests were not inspected or recorded. The three containing
`CODEX_HOME` directories remain mode `0700`. The sanitized post-provision readiness check returned
`provider_free=true`, `ready=true`, and `state=ready` for Codex lane orders 2, 4, and 6.
Re-attestation remained verified, the run-plan hash was unchanged, all provider outputs remained
absent, repository `target/debug` remained absent, and the older device-Keychain candidate retained
its exact plan hash with zero lane-local caches.

## Remaining limitation

The first real file-cache candidate proved authentication readiness but then exposed a separate
runtime-packaging defect: Codex 0.151 resolved a required `codex-code-mode-host` beside its frozen
binary, while the evaluator had copied only `codex`. The one teaching invocation is preserved and
that candidate is retired.

The repaired evaluator now requires file-cache plans to freeze and independently hash the sibling
host, proves its `--help` identity, preserves it in recovery metadata, and rejects a missing
companion before provider execution. It also requires every teaching trace to correlate the frozen
failed probe plus the successful command, exit status, and output marker before advancing. A fresh
runtime-complete successor is prepared and attested but intentionally has zero authentication
copies pending separate operator authorization. Exact evidence is in
`native_memory_pilot_v1/NATIVE_PILOT_CODE_MODE_RUNTIME_CLOSURE_2026-08-31.md`.

Token refresh behavior across multiple independent cache copies and the one-hour retention boundary
is still not proven. Readiness must be rechecked before every provider phase, and any observed
refresh conflict must invalidate the candidate rather than trigger replay or repair.
