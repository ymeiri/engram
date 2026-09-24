# Schema-12 structured-identity successor prepared — 2026-09-01

Status: completed and valid; safe negative result with no portable incremental value.

The authoritative forward-only successor is:

```text
/Users/yuval.meiri/.engram/evals/native-memory-pilot/structured-identity-no-result-forward-file-cache-bounded-20260901-01
```

It was prepared only after the operator authorized deletion of the exact rebuildable Cargo target
`/private/tmp/engram-structured-identity-evaluator-target`. Cargo removed 24,521 generated files
(15.3 GiB). The preserved evaluator and Engram binaries were copied into the owner-only candidate;
no source rebuild, completed-lane replay, live adapter write, installation change, or provider call
occurred.

## Frozen contract

```text
d3e1bfa99874c433310f1807f611481919a1b1bfca692462da1fba4bf889c4d7  protocol source and protocol.snapshot.json
52c366344844421a58d1245881c686b9b5663e41a48b8f0c3800eaccd0f9feae  run/run-plan.json
6ee349135c875d16e749a9015566f3578fbe606b8de97c02688de651319ae860  bin/engram
444ef5bcbaf5c522a8eff2c424046b1f773433f046adae1b0dccda5d609a7bb1  bin/engram-eval
a6042937174f72112dbd2d554a4af36936422e0c5ac69e353dc68994458996e9  bin/codex
aa3c08e84921acf61c180cfaf4db6733607a37a82ecad6d43776b31dc011a4ef  bin/codex-code-mode-host
b661c6a094fcc32656bf7c0071c5b45bf900b34d4f0a1ab3d78fd59aeba2c2c7  bin/claude
```

Frozen host versions are Codex `0.151.0-alpha.7.2`, its independently attested companion, and
Claude Code `2.1.252`. The candidate root, `bin`, and `run` directories are owner-only `0700`; all
five binary files are regular, single-link, owner-only executables.

The run plan freezes:

- schema 12 and one wrong-repository-scope case across six arms and one repetition;
- fixture revision `99662dadc9e5bde6c6bab915bfd926e166433ad5a8a4066d4d651dc3264c10f8`;
- an unlinked Orbit project with null project, `requires_confirmation`, exact checkout root, and
  tracked component `queue-worker`;
- one-hour Codex retention before activation;
- 8,192 bytes per Engram result, 16,384 bytes per Engram lane, 50,000 incremental tokens, and
  30,000 incremental runner milliseconds;
- Claude Haiku 4.5, 12 turns, a 50-cent aggregate allocation, 5,936,287 micro-USD prior spend, and
  the unchanged 7-dollar cumulative ceiling.

The effective restricted Engram runtime is verified with six tools, tool-set SHA-256
`b704fac7da773e703dfca6bd3e9b0ffa6dcade6b259e7290315d969de3cec5b3`, and instruction SHA-256
`09be365fafaf2542d137eb69384b1db1b1ac2166c1d6a71a6995cf84a27c3eac`. Restricted-tool and
review-authority rejection both pass.

## Provider-free gate

Fresh re-attestation returned `verified=true`. Lifecycle audit returned `invalid=false`, six
`prepared` lanes, zero failures, and `complete=false`, as required before teaching. Every teaching,
activation, evaluation, verification, agent-output, and runner-report target is absent. No
`auth.json` exists in any lane.

The provider-free authentication checker therefore returns the expected `ready=false` with all
three Codex lanes `not_logged_in`. The source ChatGPT cache remains an owner-only `0600`, regular,
single-link 4,657-byte file; its contents and digest were not inspected. No credential copy has
been created for this successor.

Available disk was 53,672,968,192 bytes against the mandatory 19,893,251,686-byte reserve, a
positive margin of 33,779,716,506 bytes. Repository `target/debug` remains absent.

An independent cross-lane freeze check then verified all six acceptance contracts directly against
their materialized Orbit Git checkouts. For every lane, the canonical checkout root and evaluation
cwd agree; origin is the expected equivalent SSH remote; project is null with
`requires_confirmation`; component is `queue-worker`; first action is
`resolve_checkout_identity`; and outcome is abstention. The component manifest and operation
runbook are Git-tracked, non-symlinked, contain their frozen markers, and match their exact contract
SHA-256 values. Every lane forbids the Atlas context key. The shared output schema requires exactly
the twelve schema-12 keys, including checkout root and project authorization fields, and rejects
additional properties.

## Next gate

Before teaching, obtain explicit authorization for exactly three protected plaintext cache copies
for this exact successor. Then run the frozen confirmation-gated provisioner once, require all
three Codex lanes ready, and repeat attestation, lifecycle, output-absence, permission, hash, and
disk gates. Only a pristine result may proceed to the already-authorized provider teaching phase.
Never modify or replay a completed predecessor.

## Provisioning preflight disk stop

The operator later authorized the three protected copies. The resume/orientation gate stopped
before opening or copying the source cache because available disk had independently fallen to
14,533,808,128 bytes, below the mandatory 19,893,251,686-byte reserve. No authentication copy or
provider call occurred, and the frozen plan remains unchanged.

Read-only disk attribution found that the new Engram successor occupies only 880,944 KiB. The
material new pressure is dominated by four unrelated `dd-source` worktrees under `/private/tmp`,
each about 4.3 GiB. Two contain user changes and must be preserved. Two are clean and together are
large enough to restore reserve:

```text
4,343,876 KiB  /private/tmp/ideai721-retention.DNUU1U
4,343,840 KiB  /private/tmp/ideai-724-row4-probe-dd-source
```

Both clean worktrees have zero tracked changes and zero untracked files, no process references their
path, and their exact commits are retained by local and remote branches:

```text
6dcd9a2de801af694a0051135e8c104c8255a801  yuval.meiri/ideai-721-datadog-tags-retention
9596ba188b78cc5683c92c0e5f88579ff35b7e02  yuval.meiri/ideai-724-row4-signal-probe
```

Removing these registered worktrees remains a separate destructive action and requires explicit
operator approval. If approved, use `git worktree remove` for only those two exact clean paths,
then revalidate branch refs, disk reserve, plan hash, absent auth/output files, and runtime state
before running the already-approved provisioner.

## Teaching checkpoint — 2026-09-02

The operator approved removal of only the two clean registered worktrees above. Both paths are now
absent, while their exact local branch refs still resolve to the recorded commits. The unrelated
`/private/tmp/ideai248-runtime.jyIlcd` and `/private/tmp/ideai751-v2.IyMMJq` worktrees remain. Disk
reserve recovered before any credential copy or provider execution.

Live Codex integration was independently checked without changing settings. Codex reports the
`engram` MCP server enabled over stdio with command `/Users/yuval.meiri/.local/bin/engram` and args
`serve --profile agent`. The installed `0.2.3` binary and daemon share SHA-256
`0a28565a768b6e11db026e4d5638ae491798df070affbcaada1ebae351946487`; daemon storage is writable,
and a real initialize/tools-list exchange returned the six restricted agent tools. The live
installed contract is distinct from the frozen pilot runtime and was not substituted into the
candidate.

The already-approved provisioner then created exactly three isolated `auth.json` copies. Each is a
regular, non-symlinked, single-link, owner-only `0600` file of the same 4,657-byte size as the
source. No content or credential digest was printed or computed. The authentication checker now
returns `ready=true` for Codex lanes 2, 4, and 6.

Fresh attestation, output-absence checks, lifecycle audit, target/debug absence, and disk reserve
all passed before provider execution. Teaching then ran exactly once across all six lanes; every
provider process exited 0, no lane was recovered or replayed, and the final audit is
`invalid=false` with zero failures. Claude teaching spend was 135,778 micro-USD in total:
46,313 for lane 1, 44,733 for lane 3, and 44,732 for lane 5.

The immutable teaching trace hashes are:

```text
26880c4415ba17b237290a840c39309f1072e850149ea098e6bdcd08ee7cd2f4  lane 1
9dad8f0689c2799cf7d732c019e0a35aa5a706084a240e48ca595a7cb6c62342  lane 2
74c38cf45a2ccd32860453e635a654f1eae0480cf422eb81ba5bfa35e214ad53  lane 3
941e0b5c2a6696d16aaa1bdd3ce3a8be80ec35570b49969f368d5807194806c2  lane 4
5e998571e4e17f2a5b8ab275bef5cee99de11b186d89a08df2091f0adeac8a9b  lane 5
aa4b878eafe95b0d1a70ed65f96abb015c65b02425d412cfad64b2e010a9e2b4  lane 6
```

All required Engram and procedure-verification artifacts passed. Codex created fresh native-memory
state in lanes 2 and 4; the evaluator records it as pending until activation. Claude native lane 5
created a non-empty native `MEMORY.md`; absence in Claude combined lane 1 remains a valid measured
outcome. Activation, evaluation, and agent-output targets are all still absent.

An independent post-teaching inspection also rehashed each trusted success receipt. All four Engram
treatments contain an `active` Atlas repository-scoped procedure, written by the expected host,
with an October 2 expiry and receipt SHA-256
`2c79b3a3ce417cd9062b2b29d227fb53f152553daaa737f15392cfab02a8912d`. Their procedure IDs are
`01a06148-2c5c-7771-8d20-28ccc665e470`, `01a06149-b15d-7322-ac78-08fcd6ac944a`,
`01a0614c-6ed4-7b62-9139-2e3c956b8496`, and
`01a0614f-0848-7ee1-9554-e13e03154431`. This preserves the intended wrong-scope challenge: the
later Orbit checkout must not receive or execute the Atlas procedure.

The frozen plan sets `codex_min_idle_hours=1` and `activation_wait_hours=1` for every Codex lane.
The newest Codex teaching trace was modified at Unix millisecond `1788338971028`, so activation
must not begin before `2026-09-02T12:49:31+03:00`. A one-shot heartbeat is scheduled for 12:51
local time. It must disable itself first, repeat every pristine gate, run activation exactly once,
then evaluate exactly once only if activation leaves the candidate valid and ready. Teaching must
never be rerun.

## Final outcome — 2026-09-02

The retention deadline passed and the one-shot heartbeat was disabled before execution. Fresh
hash, authentication, lifecycle, output-absence, disk, and repository-target gates remained
pristine. Activation ran exactly once for Codex lanes 2, 4, and 6; evaluation then ran exactly once
for all six lanes. Every provider process exited 0, no lane was recovered or replayed, and the final
audit reports `complete=true`, `invalid=false`, and zero integrity failures.

The result is a valid negative: all six lanes safely abstained with no wrong-scope context,
repository command, native-memory write, or repeated failure, but zero lanes passed the full
contract because none read `runbooks/deploy-worker.md`. Only Claude lean Engram preserved the exact
structured identity. Every resource budget passed, yet only that one treatment descriptively
Pareto-dominated native memory; the strict report therefore rejects portable incremental value.

The immutable machine report is
`SCHEMA12_STRUCTURED_IDENTITY_RESULTS_2026-09-02.json`, SHA-256
`6897557e6ddf49c7c67eae5e4a9ef818eb3243c9395751907a8ed9006ed61c8b`. The human-readable result,
exact activation/evaluation trace hashes, cost accounting, per-lane outcome, and product-boundary
diagnosis are in `SCHEMA12_STRUCTURED_IDENTITY_RESULTS_2026-09-02.md`. No completed lane may be
replayed or repaired.
