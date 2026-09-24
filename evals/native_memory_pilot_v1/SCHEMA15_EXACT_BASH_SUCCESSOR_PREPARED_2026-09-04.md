# Schema 15 exact-Bash successor — prepared 2026-09-04

> Completion note (2026-09-05): the host-drift `-04` successor completed the retention gate,
> activation, and evaluation exactly once. The valid positive result is recorded in
> `SCHEMA15_EXACT_BASH_COMPLETED_2026-09-05.md`. This file remains the historical preparation and
> forward-repair record.

## Status and claim boundary

The schema-15 `-02` plan is preserved and permanently invalid. Its one authorized authentication
provision created exactly nine protected Codex cache copies, and its teaching phase ran exactly
once. The runner stopped before lane 6 when Claude lane 5 used a visible shell redirect to create
native memory after a `Write` call to an obsolete path was denied. No completed lane was replayed
or repaired.

The replacement `-03` plan is provider-free and pristine. All 18 lanes are `prepared`; the audit
returns `invalid=false`, zero failures, and zero teaching, activation, evaluation, verification, or
agent-output artifacts. It contains zero `auth.json` files, so all nine isolated Codex homes report
`not_logged_in`. This document is preparation evidence, not outcome evidence.

## Preserved failure evidence

```text
602ecd270feb89acbd17109e430f97620a25b38e4934f7171fd75d9d4af162e6  /Users/yuval.meiri/.engram/evals/native-memory-pilot/required-host-action-bounded-query-repeated-20260904-02/run/run-plan.json
9ee4cda592de55c9b182d6ef17aef7de140323837e921d63d7ef68df7e613e5a  /Users/yuval.meiri/.engram/evals/native-memory-pilot/required-host-action-bounded-query-repeated-20260904-02/run/lanes/05-learned_procedure_wrong_repository_scope-claude_native_memory-r1/teaching-trace.jsonl
```

Lane 5 completed both fixture commands and wrote one native-memory detail file with the host
`Write` tool. It then hallucinated an obsolete `MEMORY.md` path; the host denied that `Write`, but
the plan's bare `Bash` allowance let the model fall back to `cat > .../claude-memory/MEMORY.md`.
The preregistered auditor correctly invalidated the lane as shell-manufactured native memory.

## Exact repair

Claude teaching now exposes Bash but does not pre-approve the bare `Bash` permission. Every Claude
lane freezes exactly two full commands:

1. `cd <that lane's teaching checkout root> && ./bin/context-probe --channel amber`
2. `cd <that lane's teaching checkout root> && ./bin/context-probe --channel cobalt`

The teaching prompt requires those exact invocations. Evaluation has zero Bash permission rules.
Native-memory arms retain normal `Write`/`Edit` access to their isolated memory directory. The
runtime validator rejects a Claude plan with missing, wildcarded, non-rooted, or drifted teaching
commands and re-derives the exact teaching/evaluation `--allowed-tools` values before any provider
call.

Claude Code's documented permission semantics make `dontAsk` deny unapproved write-capable tool
calls, while exact `Bash(command)` rules pre-approve only the named command. A direct host smoke
confirmed the full compound-command form; the shorter subcommand-only rule was not accepted by the
installed host and was not used.

## Validation evidence

The evaluator suite passes 153 tests. Formatting and strict all-target Clippy pass with an external
target directory, leaving repository `target/debug` absent.

The final forward Claude diagnostic used the exact successor permissions with native memory
enabled. It made exactly two Bash calls—the failed and successful fixture commands—and then used
`Write`, `Read`, and `Write` for two native-memory files. No Bash call mentioned the memory path.

```text
0fdb55677a1fd0bfe1435eb77cf580a71d45d73a1c5a91279bcc6afddf19cf9b  /private/tmp/engram-claude-shell-guard-smoke-20260904.OaVoEW/trace-full-command-allow.jsonl
30b25e9b378ed3f52ae988e9bc707ba03f0981612e2d2d56c833eef6cb4c14a6  /private/tmp/engram-claude-shell-guard-smoke-20260904.OaVoEW/trace-forward-native.jsonl
```

The forward diagnostic completed in 22,870 ms, used six turns, and cost $0.03994295. Across the
three completed Claude lanes in the invalid plan and all seven permission diagnostics, new spend
was $0.27127065. The successor conservatively records 7,092,490 micro-USD of total prior spend,
retains a 150-cent allocation, and uses a 1,000-cent authorized cumulative ceiling under the
standing user budget authorization.

## Authoritative successor artifacts

```text
c8340f3309df199b2a7e974bc2aaf8d25cd34099ec8331194bdd9c285873be96  /Users/yuval.meiri/projects/engram/evals/native_memory_pilot_v1/protocol-schema-15-required-host-action-bounded-query-repeated-file-cache-exact-bash.json
d870f0a646a1c95023a20de2690bed73d97fe811d01222284e1b1a744fb04742  /Users/yuval.meiri/.engram/evals/native-memory-pilot/required-host-action-bounded-query-repeated-exact-bash-20260904-03/run/run-plan.json
e12bced683bcfb5ff1accb01a58819a0784d5e6ece3214f4312ff96d5c789772  /Users/yuval.meiri/.engram/evals/native-memory-pilot/runtimes/required-host-action-bounded-query-repeated-exact-bash-20260904-03/bin/engram-eval
3f3710b876f72fb5023e4084e242541308e39d866a242dacb18c205143304e2a  /Users/yuval.meiri/.engram/evals/native-memory-pilot/runtimes/required-host-action-bounded-query-repeated-exact-bash-20260904-03/bin/engram
2e901735fabdc8c8ff6c0952f4014f52d6f0549ab5da8605df3108c92354db6e  /Users/yuval.meiri/projects/engram/engram-eval/src/native_pilot.rs
4ca9c90ebe13bbe2f103914aac554e2126edfe3a2dfcf7a30827750b26950278  /Users/yuval.meiri/projects/engram/engram-eval/src/native_runner.rs
```

The frozen Engram agent profile still exposes six tools. Its tool declaration SHA-256 is
`69c86ebfba6576696e24904f7004c35fe10067c2c5ebcb741c086ae168859e6d`; its initialization
instruction SHA-256 is
`db459df6f9a4af55b7a76ecf1ec10d6a557005434befc0a7ac48ad830dfbcddb`.
Claude and Codex adapter hashes remain
`645efd01426349489b00505d88dc8016180e02ae1ffdb7237688b97ff229c6a5` and
`ecc10b4de005d25a4cc68f1f8729ff6f4d4683411be51a0df2674935c8785980`.

Re-attestation returns `verified=true`, including exact host bytes, the live six-tool MCP
handshake, restricted-tool rejection, and forged-review-authority rejection. Disk reserve exceeds
219 GiB.

## 2026-09-05 host-drift successor and continuation boundary

Before the approved `-03` provisioning ran, exact-host re-attestation found that ChatGPT had
updated Codex from `0.152.1` to `0.153.3`. The Codex SHA-256 changed from
`99a6fdb0e0f9188e62dd6681c7ed3c5cc343358291a53ea21a9d4202e09cde9e` to
`e57b3081ef7a33014e9afa1f7cd44f472d2d72b4a2084bdc8bc16cd261159302`; its companion now hashes
to `aa40b20a60448c208a99f5efbf889bdd234f2be653c5a0984b1088c987e5c58a`. The runner rejected the
drift before inspecting lane login state. `-03` is preserved untouched with zero cache copies and
zero provider outputs and must never run.

The fresh `-04` successor freezes dedicated copies of the unchanged evaluator and Engram binary,
re-attests the current Codex host, and uses the unchanged schema-15 protocol:

```text
c8340f3309df199b2a7e974bc2aaf8d25cd34099ec8331194bdd9c285873be96  /Users/yuval.meiri/projects/engram/evals/native_memory_pilot_v1/protocol-schema-15-required-host-action-bounded-query-repeated-file-cache-exact-bash.json
d7495a2915f3b2b4bc43e665af162b4cc95741240fecb57a22b1d1bd9e1646ea  /Users/yuval.meiri/.engram/evals/native-memory-pilot/required-host-action-bounded-query-repeated-exact-bash-20260905-04/run/run-plan.json
e12bced683bcfb5ff1accb01a58819a0784d5e6ece3214f4312ff96d5c789772  /Users/yuval.meiri/.engram/evals/native-memory-pilot/runtimes/required-host-action-bounded-query-repeated-exact-bash-20260905-04/bin/engram-eval
3f3710b876f72fb5023e4084e242541308e39d866a242dacb18c205143304e2a  /Users/yuval.meiri/.engram/evals/native-memory-pilot/runtimes/required-host-action-bounded-query-repeated-exact-bash-20260905-04/bin/engram
```

After pristine provider-free gates, the guarded no-overwrite provisioner created exactly nine
protected cache copies. Each is a 0600, owner-matching, single-link regular file and not a symlink;
credential contents and digests were not inspected or recorded. All nine isolated homes pass the
frozen ChatGPT readiness check.

Teaching then ran exactly once with the frozen 150-cent allocation. All 18 provider processes
exited 0, zero lanes were recovered, and reported Claude teaching cost was 383,577 micro-USD. The
audit is `invalid=false` with zero failures: nine Claude lanes are `ready_for_evaluation`, nine
Codex lanes are `awaiting_activation`, all required Engram procedure verifications pass, and no
native-memory shell-write attempt was recorded. The teaching receipt is:

```text
0c3073584efd08421749c8d4e4162e5f4019144331920f4b7f72629efd896283  /Users/yuval.meiri/.engram/evals/native-memory-pilot/required-host-action-bounded-query-repeated-exact-bash-20260905-04/run/runner-teaching.json
```

Teaching was not rerun and no completed lane was replayed or repaired. The latest Codex teaching trace was
modified at Unix ms `1788599246667`; the exact one-hour retention deadline is therefore
`1788602846667`, or `2026-09-05T13:07:26+03:00`. The gate passed; every hash, live-host
attestation, authentication, lifecycle, disk, and repository-cache check remained pristine.
Activation and evaluation then ran once with the same exact 150-cent allocation. See
`SCHEMA15_EXACT_BASH_COMPLETED_2026-09-05.md` for the immutable outcome and exact receipts.
