# Learned native-memory pilot v1 replacement freeze — 2026-08-09

## Outcome

The first Keychain-backed teaching plan stopped safely on its first Claude lane. The provider trace
proved that `dontAsk` denied required tools unless they were explicitly pre-approved and that one
API request could exceed the per-call `--max-budget-usd` value before Claude Code stopped. The
invalid plan and trace remain immutable evidence.

Protocol schema v3 corrects the provider controls without changing the case, six-arm matrix,
acceptance contract, memory layers, Codex lifecycle, or scoring:

- every Claude lane pins `claude-haiku-4-5` and at most 12 turns;
- `dontAsk` explicitly pre-approves `Read`, `Bash`, and the six Engram agent-profile MCP tools;
- `Write`, `Edit`, web, notebook, and subagent tools remain denied;
- each of the six Claude calls has a `$0.050` runner allocation;
- the replacement allocation is `$0.30`;
- the predecessor's provider-reported `$0.0918771` is conservatively frozen as 91,878 micro-USD;
- the plan rejects budget accounting above the already authorized cumulative `$0.50` ceiling.

The cumulative nominal amount is therefore `$0.391878`, leaving `$0.108122` of cushion for the
observed turn-boundary overshoot behavior. This is accounting and risk control, not a claim that
Claude Code's `--max-budget-usd` is a strict billing ceiling.

## Fresh frozen plan

The fresh provider-free plan is:

`/private/tmp/engram-native-memory-pilot-v1-haiku-accounted.zc64hn/run-plan.json`

| Artifact | SHA-256 |
| --- | --- |
| `run-plan.json` | `2967e5ec63765453e4a3ffa8d5e764dea6aea040298de2ae93172ec83a8571a7` |
| `protocol.snapshot.json` | `192cc18374d65c4ba950b34ea9d684cd09be838822eca12e6aaea595484adcb7` |
| source `protocol.json` | `192cc18374d65c4ba950b34ea9d684cd09be838822eca12e6aaea595484adcb7` |
| `target/debug/engram-eval` | `06b51c1b4e0a8203697303c75cef06fa516b4663d4589a08864a5d8f9fd87fcd` |

Provider-free re-attestation passes, and all six lanes audit as pristine `prepared`. The three new
Codex homes correctly report `not_logged_in`; no browser login or provider call has run against the
replacement plan.

## Verification

```text
cargo fmt --all --check
passed

cargo test -p engram-eval
65 passed; 0 failed

cargo clippy -p engram-eval --all-targets -- -D warnings
passed

attest-native-memory-pilot
verified: true

audit-native-memory-pilot
6 prepared; 0 invalid
```

## Remaining gates

Before retrying teaching:

1. Complete ordinary Keychain-backed ChatGPT browser login for the three replacement Codex homes.
2. Confirm the replacement plan's exact `$0.30` Claude runner allocation with the documented
   turn-boundary overshoot caveat.
3. Re-run digest, attestation, pristine audit, and all-lane authentication checks.

No provider retry is authorized merely by preparing this replacement plan.

## Superseded after the second teaching attempt

After all three replacement Codex homes passed Keychain-backed ChatGPT login and the operator
approved the exact `$0.30` allocation with the turn-boundary overshoot caveat, teaching stopped
safely on Claude lane 1 again. The model ran both frozen commands correctly, but the generic
teaching instruction made it reconstruct Engram's conditional write contract. Its first memory
call omitted `scope_type`; its retry omitted `local_path` and also mistyped the receipt path. The
fifth turn ended at `error_max_budget_usd` with provider-reported cost `$0.0524307` against the
`$0.050` per-call allocation. No later lane ran.

The uncontaminated exact-write replacement is documented in
[`EXACT_WRITE_REPLACEMENT_2026-08-09.md`](./EXACT_WRITE_REPLACEMENT_2026-08-09.md). The failed plan
and trace remain immutable diagnostic evidence.
