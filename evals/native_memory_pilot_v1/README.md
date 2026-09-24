# Learned Native-Memory Pilot v1

This preregistered pilot measures transfer across genuinely separate provider sessions. It does not
present hand-authored files as native memory. A teaching session first experiences a known failed
command and its verified replacement. A fresh evaluation session then runs in a previously unseen
checkout of the same repository without receiving the command, context key, success marker, or
acceptance contract.

The six-arm matrix is evaluated within each host:

| Host | Native only | Engram only | Engram + native |
| --- | --- | --- | --- |
| Codex | `codex_native_memory` | `codex_lean_engram` | `codex_engram_plus_native` |
| Claude Code | `claude_native_memory` | `claude_lean_engram` | `claude_engram_plus_native` |

Every lane receives one matched teaching call and one evaluation call. Every Codex lane also
receives one matched activation call after the frozen one-hour idle interval. This extra phase is
necessary because Codex extracts memories asynchronously from eligible idle conversations. Claude
Code writes auto memory during a session, so it has no synthetic activation call.

Preparation creates isolated `CODEX_HOME`, `CLAUDE_CONFIG_DIR`, Claude auto-memory, and `ENGRAM_HOME`
roots per lane. It records safe environment values but never reads or copies provider credentials.
Each Codex lane freezes one credential-store mode: ChatGPT browser or device-code login with
`cli_auth_credentials_store="keyring"`, or the explicit `chatgpt_file_cache` fallback with
`cli_auth_credentials_store="file"`. Keyring plans reject lane-local `auth.json`; file-cache plans
accept it only as a regular, single-link, valid JSON file of at most 1 MiB, owned like its `0700`
isolated home and set to mode `0600`. The pilot never requires `CODEX_ACCESS_TOKEN` or
`OPENAI_API_KEY`, rejects either requirement in every ChatGPT lane, and strips both variables from
every Codex process. Claude Code may use its normal platform authentication or an explicitly
supplied API key. On Unix, the pilot output root and each mutable host-state root default to mode
`0700`; runner-produced traces and reports use private files.

Prepare without calling a provider:

```bash
cargo run -p engram-eval -- prepare-native-memory-pilot \
  --protocol evals/native_memory_pilot_v1/protocol.json \
  --output /path/to/new-or-empty-native-memory-pilot \
  --engram-bin /absolute/path/to/engram
```

Preparation always emits `execution_approved: false`. Before any evaluation call, the executor
must enforce the generated artifact gates. The plan also freezes the exact Engram executable hash
and the output of `engram contract --profile agent --verify-runtime --json`, including the embedded
six-tool schema and profile-instruction hashes plus the values observed through a real isolated
stdio MCP handshake. The probe also proves that the compact profile rejects an administrative tool
and caller-supplied `manual_review` authority.
The plan also freezes the exact `engram-eval` binary that prepared and enforces it. The runner
re-attests itself, Engram, and both hosts before starting any provider process, so a rebuilt
evaluator or product binary, changed host binary, or changed host-facing MCP profile invalidates
the prepared plan. Historical protocol-schema-4 plans retain their original three-binary contract;
schema-5 plans retain their trusted host-trace identity semantics unchanged.

Authenticate the generated isolated Codex lanes only after preparation. Each lane's
`codex_authentication.login_argv` is a frozen, secret-free `codex login` command. In keyring plans,
browser mode uses the normal callback flow and device mode adds `--device-auth`; complete the frozen
command separately for every Codex lane. Never copy `auth.json` into a keyring plan.

File-cache plans are a distinct opt-in fallback for an already authenticated local Codex account.
After the plan is frozen and independently approved for plaintext credential copies, use the
frozen evaluator's confirmation-gated provisioning command:

```bash
engram-eval provision-native-memory-pilot-auth-cache \
  --plan /path/to/native-memory-pilot/run-plan.json \
  --source-cache ~/.codex/auth.json \
  --confirm-plaintext-cache-copies
```

The command revalidates the frozen plan, binaries, runtime contract, pristine lifecycle, and
file-cache mode before reading the source. It requires every destination to be absent, never
overwrites or reuses a cache, and returns only the existing sanitized readiness report. Treat every
source and destination cache like a password: never print, hash into the plan, commit, or attach it
to an issue or chat. The provisioning process zeroizes its in-memory source buffer on every return.
It opens the source once without following symlinks, validates and owner-checks that open file, and
reads the bounded contents through the same handle. Preparation never performs this copy. The
provisioning, readiness, and
provider gates reject a missing, symlinked, hard-linked, malformed, oversized, overly permissive,
or wrong-owner cache. A copy failure triggers best-effort removal only of destination files created
by that invocation. If any destination remains, the no-overwrite gate rejects a retry; preserve and
supersede that candidate rather than repairing it.

The evaluator test suite also runs a full canary-only provisioning transaction through a frozen
synthetic plan and verifies sanitized readiness plus no-overwrite behavior. This is infrastructure
evidence only; it does not prove real multi-lane token refresh across the retention boundary.

Check login readiness without opening a browser or invoking a model provider:

```bash
engram-eval check-native-memory-pilot-auth \
  --plan /path/to/native-memory-pilot/run-plan.json \
  --require-ready
```

The check persists no raw command output. A lane is ready only when its frozen status command exits
successfully and reports exactly `Logged in using ChatGPT`; API-key authentication is rejected.
For file-cache plans, the private-file checks must also pass before the status command runs.

Claude Code lanes pin an explicit model and turn limit. In `dontAsk` mode they pre-approve `Read`,
`Bash`, and the six Engram agent-profile MCP tools. Native and combined lanes also expose
`Write`/`Edit`, but pre-approve edits only inside that lane's isolated auto-memory directory;
repository, web, notebook, and subagent writes remain denied. Each provider argv freezes its own
`--max-budget-usd` allocation. Claude Code checks
that value between API calls, so an individual final request can exceed the stated allocation; the
plan records prior spend and requires prior spend plus the replacement allocation to remain below
the separately authorized cumulative ceiling.

The artifact observations and required gates are:

- native-only and combined Codex lanes record whether non-empty, provider-generated artifacts
  appear under the isolated `CODEX_HOME/memories` after the idle and activation phases;
- native-only and combined Claude lanes record whether a non-empty, auto-generated `MEMORY.md`
  appears in the isolated auto-memory directory;
- Engram and combined lanes must show exactly one structured procedure-candidate `memory` add in
  the teaching trace. A trusted evaluator substitutes its returned ID into the generated
  `post_teaching_verification_argv`, verifies the pre-attested receipt through the confirmation-
  gated CLI, and the fresh evaluation session must use `procedure_match` to revalidate the
  unchanged receipt at use time;
- absence of a host-native artifact is a valid measured outcome, not an infrastructure failure.
  Missing required Engram state, contamination, stale artifacts, and shell-manufactured native
  memory still invalidate the lane. The executor must never repair provider output.

The attested Codex host may initialize or populate a Git-backed memory working tree during teaching.
The audit excludes `.git` bookkeeping from the content digest. State freshly created after the
attested teaching call remains pending until the frozen activation gate, whether it is an empty
scaffold or substantive provider state; its existence alone is not contamination. Exact known empty
scaffolds report `observed: false`, while unknown or substantive fresh state reports `observed: true`
without becoming evaluation-ready. Files predating preparation, visible shell writes, failed or
early activation, and evaluation before every gate passes remain integrity failures. Artifact
freshness uses the oldest included file, so a new file cannot conceal stale state.

Audit the generated state at any point without invoking a provider:

```bash
cargo run -p engram-eval -- audit-native-memory-pilot \
  --plan /path/to/native-memory-pilot/run-plan.json
```

The audit reports each lane as `prepared`, `awaiting_activation`, `awaiting_verification`,
`awaiting_artifacts`, `ready_for_evaluation`, `evaluation_complete`, or `invalid`. Add
`--require-ready` immediately before evaluation and `--require-complete` before scoring or making
any comparison claim. `evaluation_complete` means a structurally valid task outcome was captured;
it does not mean that the arm passed acceptance. Use `--require-all-passed` only when that stronger
gate is desired. A valid failed task outcome remains comparison evidence, while malformed traces,
forged or missing required artifacts, early evaluation, and other protocol-integrity failures
remain `invalid`. The audit records native artifacts as observed or absent, hashes every generated
artifact and trace, and excludes only the Codex memory repository's internal `.git` directory
before recursive traversal so transient Git locks cannot make provider-free postprocessing race;
all memory content remains included in the digest. The audit enforces the Codex idle
interval, verifies that Engram activated the exact candidate returned by teaching with the frozen
receipt hash, and rejects visible shell attempts to manufacture native memory files. Completed
lanes must also prove identity, first action, returned/applied context key, evidence citation,
Engram `procedure_match` use where applicable, and zero repetitions of the known failed command.
Command acceptance is host-correlated rather than substring-based: Codex must emit one completed
`command_execution` item containing the exact learned command, expected exit code, and success
marker; Claude Code must emit the exact Bash `tool_use` and a single matching non-error
`tool_result` containing the marker. The audit separately records whether the learned command was
the first relevant procedure attempt, and a marker repeated only in the final answer does not count.

Protocol schema 6 adds a forward-only `expected_outcome` contract for prerequisite-mismatch cases.
`execute_procedure` preserves the schema-5 rules. `abstain` instead requires the host to inspect a
preregistered checkout-local condition source through a successful correlated Read/Bash/command
tool call whose actual path argument canonicalizes to the exact Git-tracked, hash-frozen source.
The result must contain the frozen excerpt. The agent must withhold the inapplicable context key,
avoid every learned and failed procedure command, and report the preregistered
prerequisite-inspection first action plus clean abstention. Model text, diagnostics, failed reads,
shell echoes, comments, or same-named outside files do not count. Engram-bearing lanes must still
call `procedure_match`, so the audit measures use-time applicability rather than a generic refusal.
A schema-6 protocol cannot be frozen until predecessor spend from the active topology plan is known
exactly.

The provider-free product candidate after the completed schema-6 repair removes the unreliable
host read from the normal source-backed path: procedure cards may declare a bounded Git-tracked
TOML scalar source, and `procedure_match` reads it directly from the resolved current checkout.
Caller conditions cannot override a source-backed observation, and the response reports only a
value-redacted matched/mismatched/unavailable observation. This does not revise schema 6 or any
completed lane. A future protocol must exercise this candidate forward-only; until then, the
native prerequisite-source result remains unproven.

Protocol schema 7 hardens positive project and component identity for future native safety cases.
Each non-null `expected_project` or `expected_component` must declare a corresponding
checkout-relative evidence target and excerpt that contains the expected identity. Preparation
accepts only regular, non-symlinked, Git-tracked sources, verifies the excerpt, and freezes the
source SHA-256 into the acceptance contract. For native-only lanes, the audit requires both the
exact structured identity and a correlated successful host read of the frozen source with matching
current bytes; model restatement alone cannot pass. A null expected identity declares a safe
non-claim and cannot carry positive identity evidence. Schemas 4 through 6 retain their frozen
semantics.

Protocol schema 8 evaluates the deterministic source boundary forward-only. Every case must map
each procedure prerequisite to one safe Git-tracked TOML scalar. Preparation verifies the teaching
value, freezes the evaluation source hash and exact native-read excerpt, and records the source
descriptor in the acceptance contract. Engram lanes must prove one completed, cwd-scoped
`procedure_match` call whose correlated result contains the exact value-redacted
`condition_observations` entry. Native-only lanes must prove an equivalent correlated direct read
of the frozen source. The audit rejects missing or uncorrelated results, source drift, wrong cwd,
caller-value disclosure, extra observation fields, and any execute/abstain result inconsistent with
the frozen matched/mismatched status. Schemas 4 through 7 retain their frozen semantics.

After evaluation, generate the unweighted matched comparison report:

```bash
engram-eval report-native-memory-pilot \
  --plan /path/to/native-memory-pilot/run-plan.json
```

The report keeps integrity failures separate from task-outcome failures, publishes raw per-layer
totals and matched Engram-minus-native deltas, and marks Pareto dominance only when every matched
pair avoids regression on every preregistered metric and at least one metric strictly improves.
Schema-6 comparisons normalize execution and abstention to the case's frozen expected outcome:
command/exit/marker and avoided-abstention metrics apply only to execution cases, while correct
abstention applies only to mismatch cases. Raw command and abstention counts remain visible, but a
safe required abstention is not scored as a command-success regression.
`--require-portable-incremental-value` exits nonzero unless the same Engram treatment dominates
native memory on both Codex and Claude Code. A forward protocol may also freeze
`resource_budgets`. When present, every matched pair must supply host input/output tokens and
runner-observed provider duration, every Engram result must remain within the per-call and per-lane
byte limits, and treatment-minus-native tokens and duration must remain within their incremental
limits. Missing telemetry fails closed. Outcome Pareto dominance remains visible, but it cannot
support portable incremental value when a declared resource gate fails. Historical plans without
this optional contract retain their original outcome-only interpretation. This one-case,
one-repetition pipeline pilot remains a descriptive signal rather than a statistically powered
product claim.

Use `OUTCOME_DECISION_MATRIX_2026-08-09.md`, frozen before activation and evaluation, to interpret
the aggregate signal and select the next experiment without post-result threshold changes.

Re-attest the frozen digest, the evaluator plus all three executed products, every provider argv,
and the live isolated
Engram stdio profile without authorizing or invoking a provider:

```bash
engram-eval attest-native-memory-pilot \
  --plan /path/to/native-memory-pilot/run-plan.json
```

After explicit approval, execute one lifecycle phase at a time. The runner refuses to start unless
the transient approval flag is present, the confirmed aggregate Claude runner allocation exactly
matches the frozen plan, accounted predecessor spend plus that allocation remains within the
authorized cumulative ceiling, every Codex lane passes the provider-free Keychain/ChatGPT status
preflight, the run-plan digest and all executable attestations still match, the phase starts from
the expected audit state, and no output would be overwritten:

```bash
engram-eval run-native-memory-pilot \
  --plan /path/to/native-memory-pilot/run-plan.json \
  --phase teaching \
  --approve-provider-execution \
  --confirm-claude-budget-cents 30
```

Phase execution is restart-safe across already completed lanes. If Claude finishes a final turn and
then exits with its exact `error_max_budget_usd` / `budget_exhausted` / `end_turn` envelope, the
runner records the nonzero exit and provider-reported cost, validates the captured trace through the
normal audit, and continues local post-processing. During evaluation it also accepts the exact
`error_max_budget_usd` / `budget_exhausted` / `tool_use` envelope only when the immutable trace
contains exactly one distinct schema-valid completed `StructuredOutput` tool input. A retry can
extract that existing output without replaying the provider call. No arbitrary, missing, malformed,
or ambiguous tool-use exit is accepted. If a prior teaching invocation stopped at the end-turn
boundary after a valid Engram trace, a retry finalizes the existing candidate and skips completed
lanes; it never deletes, overwrites, or reruns provider output.

Repeat the separately authorized invocation for `activation` only after the frozen Codex idle
interval, then for `evaluation` only after the audit reports every lane ready. The runner executes
argv directly without a shell, captures private stdout/stderr artifacts, stops isolated Engram
daemons before trusted CLI review, activates exactly one matching procedure candidate, extracts one
schema-shaped result from each evaluation trace, and emits `runner-<phase>.json` only when the whole
phase reaches its required postcondition.

If and only if a frozen evaluation is rejected locally before any provider result because Claude
cannot load the optional JSON Schema meta-schema, preserve the source plan and all artifacts as an
invalid run. Prepare a new evaluation-only successor without invoking a provider:

```bash
engram-eval prepare-native-memory-pilot-evaluation-recovery \
  --source-plan /path/to/rejected-native-memory-pilot/run-plan.json \
  --output /path/to/new-or-empty-evaluation-recovery
```

The command accepts only the exact single-lane Claude schema-rejection shape, records hashes of the
source plan and rejected trace/stderr, reuses completed teaching and activation evidence by
reference, creates new evaluation outputs, and replaces only the host schema arguments with a
portable subset. The resulting plan authorizes evaluation only and requires its own exact provider
allocation approval. It cannot repair, replay, overwrite, or chain recovery plans. Re-run
attestation, Keychain authentication, and `audit-native-memory-pilot --require-ready` against the
successor before requesting approval.

The initial case deliberately tests a high-value coding-agent pattern: avoiding a recently failed
tool invocation, recalling the verified command with exact prerequisites, and carrying it to a moved
checkout using stable repository identity. This is a pipeline and signal pilot, not a statistically
powered product claim; repetitions and additional decision, stale-memory, ambiguity, and safety
cases come only after the lifecycle proof succeeds.

Lifecycle references:

- [Codex authentication and credential storage](https://learn.chatgpt.com/docs/auth)
- [Codex memories](https://learn.chatgpt.com/docs/customization/memories)
- [Claude Code auto memory](https://code.claude.com/docs/en/memory)
- [Claude Code environment isolation](https://code.claude.com/docs/en/env-vars)
- [Claude Code CLI isolation controls](https://code.claude.com/docs/en/cli-usage)
