# Schema 7 no-result evidence closure: resource budgets prepared

Date: 2026-08-31

## Outcome

The next native-memory comparison now preregisters host-visible packet, token, and runner-duration
budgets. The evaluator keeps outcome Pareto dominance separate, but a treatment cannot support
portable incremental value unless every matched pair provides complete resource telemetry and stays
within every frozen limit. Missing telemetry fails closed.

No authentication or provider execution occurred while preparing this successor. The existing
default Codex ChatGPT credential cache was inspected only for existence/status and was not read,
copied, hashed, or placed in a lane.

## Frozen budgets

| Measurement | Per matched treatment lane |
| --- | ---: |
| Largest serialized Engram result | 8,192 bytes |
| Total serialized Engram results | 16,384 bytes |
| Treatment-minus-native input + output tokens | 50,000 tokens |
| Treatment-minus-native runner provider duration | 30,000 ms |

These are forward-only gates, not post-result thresholds:

- the completed topology example returned at most 6,164 bytes in one Engram result and 11,878 bytes
  across its progressive interaction, establishing the observed packet envelope;
- in the completed repeated wrong-scope run, the largest matched average total-token increase was
  32,026 tokens (`113,285 - 81,259`) and the largest runner-duration increase was 16,784 ms
  (`43,187 - 26,403`);
- 50,000 tokens and 30 seconds preserve room for the newly required bounded local evidence-closure
  read while still rejecting clearly token-heavy behavior such as the earlier combined topology
  arms (Claude added at least 76,317 input tokens and Codex added 154,547 input tokens before output
  tokens).

The source measurements remain descriptive historical evidence. Only the new plan below can apply
these limits as acceptance gates.

## Evaluator behavior

`NativePilotProtocol` and `PreparedNativePilot` carry an optional `resource_budgets` contract so
historical plans remain readable. For each host/case/repetition treatment/native pair, the report
now records:

- native and treatment input-plus-output totals and their signed delta;
- native and treatment runner-observed provider duration and their signed delta;
- treatment Engram total-result and maximum-result bytes;
- exact telemetry failures and exact budget violations;
- a fail-closed `within_resource_budgets` result.

The top-level report publishes the frozen budgets and `all_resource_budgets_passed`. Outcome Pareto
dominance remains visible, but `portable_incremental_value_observed` and the cross-host signal require
the same treatment to satisfy both outcome and resource gates on both hosts.

Focused tests cover clean budget evidence, packet/token violations, missing duration telemetry,
backward-compatible historical plans, and protocol validation. Validation results:

- `cargo test -p engram-eval`: 130 passed, 0 failed;
- `cargo clippy -p engram-eval --all-targets -- -D warnings`: passed;
- `cargo fmt --all --check`: passed;
- all builds used `CARGO_TARGET_DIR=/private/tmp/engram-forward-evidence-closure-target`;
- repository `target/debug`: absent.

## Current immutable successor

| Artifact | Path | SHA-256 |
| --- | --- | --- |
| Protocol | `evals/native_memory_pilot_v1/protocol-schema-7-no-result-evidence-closure-forward-device-auth-bounded.json` | `d8ed1b727dd8a128e260e0364c9f5bdd4a9c1f1c17b7d1583e60468e1d135436` |
| Run plan | `/Users/yuval.meiri/.engram/evals/native-memory-pilot/no-result-evidence-closure-forward-device-auth-bounded-20260831-02/run/run-plan.json` | `b8c371f6e6044034f97f697e590fcab671b5db1e508c417acdabf25ad7ed939f` |
| Evaluator | `/Users/yuval.meiri/.engram/evals/native-memory-pilot/no-result-evidence-closure-forward-device-auth-bounded-20260831-02/bin/engram-eval` | `f2d4d712adf5b625f7c2b58d97ec9a9768579f87d1d7ff583f60e7718625ca4f` |
| Engram | `/Users/yuval.meiri/.engram/evals/native-memory-pilot/no-result-evidence-closure-forward-device-auth-bounded-20260831-02/bin/engram` | `905f9bad0c964d95957c6e7be717c432329f166636f84893e5c67ef11cf26018` |
| Codex | `/Users/yuval.meiri/.engram/evals/native-memory-pilot/no-result-evidence-closure-forward-device-auth-bounded-20260831-02/bin/codex` | `a6042937174f72112dbd2d554a4af36936422e0c5ac69e353dc68994458996e9` |
| Claude Code | `/Users/yuval.meiri/.engram/evals/native-memory-pilot/no-result-evidence-closure-forward-device-auth-bounded-20260831-02/bin/claude` | `5086b9b64d8bb842e1f599cdd3767ab08c6b2266e462fcc5686ae4b019cca8f7` |

Provider-free state is pristine:

- attestation: `verified=true`; real agent-profile MCP runtime verified six tools and rejected both
  restricted administration and caller-supplied reviewer authority;
- audit: `invalid=false`, zero failures, six `prepared` lanes;
- authentication status: all three isolated Codex lanes are `not_logged_in`;
- teaching, activation, evaluation, agent-output, procedure-verification, and runner reports: absent;
- lane-local `auth.json` files: absent;
- frozen plan still uses `chatgpt_device_keyring`; no login was attempted.

The `20260831-01` candidate is preserved but must never run. It was superseded pre-provider after its
incomplete report exposed a misleading limitation sentence; its frozen plan and evaluator were not
modified.

## Next decision

The default Codex home is already authenticated with ChatGPT and has an owner-only file cache.
Official Codex guidance allows copying that cache when browser/device login is unavailable. Reusing
it would avoid repeated login attempts, but it changes the approved Keychain-only security contract
because each isolated lane would contain a plaintext `0600` token cache. Provider-free evaluator
support for a separate file-cache mode and confirmation-gated provisioning command now exists. It
fails closed unless the plan is pristine and every lane cache is a valid single-link JSON file,
owned like its `0700` home and set to `0600`; it never overwrites a destination or emits credential
contents or a content digest, opens and reads the validated source through one no-follow bounded
file handle, and zeroizes the in-memory source buffer on return. No cache has been copied and no
successor has been prepared. If
approved, prepare another fresh successor and provision only that successor; do not copy
credentials into or repair this Keychain plan. Otherwise, retain this exact plan and authenticate
the three lanes one at a time with the frozen device-code commands after the login rate limit cools
down.
