# Native-memory pilot v1 schema-5 matched plan — prepared 2026-08-10

## Status

The forward-only matched plan was prepared with pristine provider-free checks and subsequently
executed. Teaching and activation completed, and all six evaluation provider calls ran exactly
once. The frozen evaluator rejected lane 6 because Codex emitted both a schema-shaped progress
message and a schema-shaped terminal response. The source therefore remains immutable,
`complete=false`, and `invalid=true`; it is not an admissible matched comparison.

An evaluation-only successor later completed all six new evaluation destinations without
replaying teaching, activation, or any source lane. Its audit is `complete=true`, `invalid=false`;
the strict report observes descriptive portable incremental value for Engram and combined memory
across both hosts, while zero lanes pass the exact identity contract. The successor plan is
`/private/tmp/engram-native-memory-pilot-v1-codex-terminal-recovery-v2.RP7KHm/run-plan.json`,
SHA-256 `3d67c97a2637953c6ab40546bd1c456e8dd8d3d3e7f6439b4d769aa3939e578d`.
Exact traces, hashes, outcomes, costs, limitations, and the forward-only recovery contract are
recorded in `SCHEMA5_MATCHED_RESULTS_2026-08-11.md`.

Frozen preparation identity:

- protocol schema: `5`
- pilot: `native-memory-pilot-v1-schema5-matched`
- run plan: `/private/tmp/engram-native-memory-pilot-v1-schema5-lifecycle-accounted.GNpz9q/run-plan.json`
- run-plan SHA-256: `aad54051adc52f0c44deccd6e26c6fd0fa05c635353999ea9c78b6a360b7e6a1`
- protocol snapshot SHA-256: `fd7eae5c9cf4cc6f4c70910c42fdb0cbb7c556e855d1ce41ee3f1ea6099b350c`
- portable agent-output schema SHA-256:
  `c0b3f7ee79caa802af0fb13d92b55f342f7c244438095f90acb7bc09a1a26f60`
- preparation attestation: verified
- initial audit: `invalid=false`; all six lanes were `prepared`
- provider calls made at preparation time: none

Three predecessors are preserved but forbidden:

- `/private/tmp/engram-native-memory-pilot-v1-schema5-matched.cdRbSt` was never executed. Its
  preparation exposed that the protocol attested Engram and both hosts but not the evaluator binary.
- `/private/tmp/engram-native-memory-pilot-v1-schema5-runner-attested.7DSwqe/run-plan.json` was
  partially executed. Lane 1 teaching completed, then lane 2 teaching produced Codex 0.147's new
  exact evidence-empty scaffold. Its frozen evaluator did not recognize that scaffold variant and
  marked the lane invalid; the runner stopped before lane 3. The traces remain immutable. This was
  an evaluator compatibility defect, not evidence-bearing native memory.
- `/private/tmp/engram-native-memory-pilot-v1-schema5-scaffold-compat.sr5jGL/run-plan.json` was
  partially executed. Lane 1 teaching completed, then the same attested Codex binary emitted a
  different evidence-empty scaffold in lane 2. Exact template enumeration again marked the lane
  invalid, and the runner stopped before lane 3.

The replacement fixes the lifecycle invariant rather than enumerating templates: fresh Codex state
created by the attested teaching call remains pending until activation, while pre-preparation state,
visible shell writes, failed activation, stale files, and early evaluation remain invalid. Known
exact empty scaffolds still report `observed: false`; unknown fresh state reports `observed: true`
without becoming valid for evaluation before activation. The replacement freezes and re-attests the
corrected `engram-eval` and carries over no provider evidence.

Schema 5 represents the case's correct identity boundary explicitly: the stable Atlas repository
remote is expected, the component is `queue-worker`, and project identity is `null` because the
fixture intentionally has no registered repository-to-project link. Schema 4 still requires its
original concrete project/component strings and retains exact historical semantics.

The frozen Claude allocation is `$0.60` across six calls (`$0.100` per call), with a `$1.00`
authorized ceiling. The protocol accounts for `$0.097046` of provider-reported spend from the two
preserved predecessor lane-1 teaching calls, so accounted allocation remains below the ceiling. The
operator's standing approval covers all in-scope AI execution, so no further budget approval is
required. The runner invocation must still carry its mechanical
`--approve-provider-execution --confirm-claude-budget-cents 60` flags.

## Frozen executable identities

| Executable | Version | SHA-256 |
| --- | --- | --- |
| Engram | `0.2.3` | `92fb1c072c23d62d3ab44a8ce13aab1e6f1127cba6938b6b218997d41be5616b` |
| Engram evaluator | `0.2.3` | `6275da115743033efea7d83a1495a8a15e9408aff853bb126b5d18d7f2237d9c` |
| Codex | `0.147.0-alpha.6.5` | `e4432c0c085e4a2e5b9cf982e4dd2ebdb44ed33c422827b6e6c64353778e773b` |
| Claude Code | `2.1.226` | `013a1cf17df5ff1dcc189d5d6fd3fdd5f097ddc3cd41aa9992e99805574febbe` |

The attested agent profile exposes six tools, rejects administrative access and manufactured
reviewer authority, and has tool-contract SHA-256
`cb48eb1bc9a6d21e8aafb38a4bf012a39aceb37f63a5d7684d30d56987a2836e`.

## Teaching completed — 2026-08-11

After all three isolated Codex homes passed the Keychain-backed ChatGPT login check, the frozen
hashes, evaluator attestation, authentication check, and untouched audit were revalidated. The
approved teaching command completed all six lanes once with exit code 0 and no recovered or
replayed trace. The post-teaching audit reports `invalid=false`, no lane failures, three Claude
lanes `ready_for_evaluation`, and three Codex lanes `awaiting_activation`.

The teaching-trace SHA-256 values, in frozen lane order, are:

1. `975b432c9c70e8733fc84ed51a774b47ef4f666f08fd08041aa668082267633a`
2. `bbf1ab330227971f7d86818d6fd09afda32ea74bb60988b0439c59ba84658da2`
3. `87408be5e0051ec74215062927f0f380d49309bd9238c19e5d63f51502100200`
4. `0cbf0b9dfbeb8dda84696eb54d86499303f5dcd2d136dc99f1b90009da426b9b`
5. `e24ad4a941975db6ed927cc565aa024965709c7e67cf42c4d564e032a295c652`
6. `56cd19351743ebad206144f93b74fcd7f1613fe4c344ab381accebc4616d6a95`

Claude reported `$0.149214` total for the three teaching calls. Codex calls used the authenticated
ChatGPT lanes and reported no API spend. The latest teaching trace was written at
`2026-08-11T10:32:42.257+03:00`, so the frozen one-hour retention gate permits activation only at
or after `2026-08-11T11:32:42.257+03:00`. The `resume-engram-pilot` heartbeat was scheduled for
`11:34+03:00`. It was paused before manual execution to prevent a concurrent duplicate;
activation and evaluation then ran under the same frozen gates. It remains paused.

During the retention interval, `cargo test --workspace --all-targets` ran against the current
worktree with `CARGO_TARGET_DIR=/private/tmp/engram-retention-test.18FTDm`. It passed 1,035 tests
with zero failures; 12 tests requiring an embedding-model download were explicitly ignored. A
post-test hash check proved that the isolated build did not alter any frozen pilot identity, and a
fresh audit preserved `invalid=false`, the three Claude-ready lanes, the three Codex
`awaiting_activation` lanes, and zero failures.

The same isolated target also passed `cargo clippy --workspace --all-targets -- -D warnings`.
`cargo fmt --all --check` and `git diff --check` passed against the worktree.

All 12 model-dependent tests were then run explicitly. The three embedder tests and the index
pipeline test passed. Seven of eight semantic integration tests initially passed; the full-pipeline
case exposed a stale assertion that still expected heading-section chunks even though the current
documented default preserves each short document as one whole-document chunk. The assertion was
corrected to require exactly two chunks for its two short fixtures, after which all eight semantic
tests passed. The explicit run also exposed an unrelated ignored doctest whose `?` expressions were
not enclosed in a `Result`-returning function; the example is now a compiling `no_run` doctest.
Focused index regression passed 263 tests, the doctest passed, affected-crate Clippy passed with
warnings denied, and formatting plus diff checks remained green.

The full workspace doctest suite then exposed two more ignored examples. The MCP example compiled
but failed when the doctest executed a stdio server without a client; it is now a compile-only
`no_run` example. The Store example used `await` outside an async function; it is now a `no_run`
example with a hidden async `Result` main. The workspace doctest suite now passes all three examples
with none ignored, and affected-crate Clippy, formatting, and diff checks pass.

After the live-daemon integration run, the project `engram` daemon metadata pointed to a PID that
no longer existed. Starting that exact project daemon cleaned only its stale runtime files and
restored port 8765. Status now reports version 0.2.3, a ready and writable datastore, bearer auth,
spawn/current SHA-256
`92fb1c072c23d62d3ab44a8ce13aab1e6f1127cba6938b6b218997d41be5616b`, and
`Runtime attestation: matched`. The already-running in-app stdio proxy retains the former bearer
token, so the retention heartbeat records that it must use a fresh local stdio session if that
proxy returns 401. This project-daemon recovery is outside every isolated pilot lane.

## Manual Keychain login gate

Each new isolated `CODEX_HOME` must complete the normal ChatGPT browser login separately. Run the
following commands exactly; do not paste an API key or copy an `auth.json` file:

```bash
CODEX_HOME='/private/tmp/engram-native-memory-pilot-v1-schema5-lifecycle-accounted.GNpz9q/lanes/02-learned_procedure_moved_checkout-codex_native_memory-r1/codex-home' \
  /Applications/ChatGPT.app/Contents/Resources/codex login \
  --config 'cli_auth_credentials_store="keyring"'

CODEX_HOME='/private/tmp/engram-native-memory-pilot-v1-schema5-lifecycle-accounted.GNpz9q/lanes/04-learned_procedure_moved_checkout-codex_engram_plus_native-r1/codex-home' \
  /Applications/ChatGPT.app/Contents/Resources/codex login \
  --config 'cli_auth_credentials_store="keyring"'

CODEX_HOME='/private/tmp/engram-native-memory-pilot-v1-schema5-lifecycle-accounted.GNpz9q/lanes/06-learned_procedure_moved_checkout-codex_lean_engram-r1/codex-home' \
  /Applications/ChatGPT.app/Contents/Resources/codex login \
  --config 'cli_auth_credentials_store="keyring"'
```

Then verify without calling a provider:

```bash
target/debug/engram-eval check-native-memory-pilot-auth \
  --plan /private/tmp/engram-native-memory-pilot-v1-schema5-lifecycle-accounted.GNpz9q/run-plan.json \
  --require-ready
```

All three lanes must report `state: ready`. After that, re-attest and re-audit before teaching; do
not mutate the plan or replay any completed lane.
