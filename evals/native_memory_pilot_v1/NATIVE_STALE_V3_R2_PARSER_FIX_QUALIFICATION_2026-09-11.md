# Native stale-safety v3-r2 parser-fix qualification checkpoint

Date: 2026-09-11

Status: pre-teaching gates passed; no provider admission has been consumed in the
current successor. This document is a continuation checkpoint, not a completion
claim.

## Rejected predecessor

The immutable predecessor is:

`/Users/yuval.meiri/.engram/evals/native-memory-pilot/native-stale-v3-r2-qualification-20260911.kHDIMa`

Admission 1 of 34 was consumed and sealed ambiguous. It must never be replayed or
advanced. The evaluator rejected a valid Codex 0.153.4 rollout because a later
`payload.full=false` world-state delta omitted `model`:

`Error: invalid evaluation data: Codex world_state record omitted its model`

The preserved rollout SHA-256 is
`3691031eb78efdd91f3a3a7555b8c356aa7213d5284b4a2f7147fa44e5ef4384`.

## Parser correction

`engram-eval/src/native_runner.rs` now distinguishes full world states from
explicit partial deltas. Full and legacy states still require `model` and
`agents_md`; explicit `full=false` deltas may omit them. Any model or instruction
object that is present remains strictly typed and participates in concordance.
Conflicting delta models, malformed model fields, missing full-state instructions,
and divergent delta instructions fail closed.

Verification for the corrected source completed with 674 tests passed, zero
failed, and six ignored; formatting and clippy were clean. The following four
provider-free ignored host-boundary tests were then run explicitly and passed:

- `executes_real_provider_free_claude_seatbelt_matrix`
- `executes_real_provider_free_codex_sandbox_matrix`
- `executes_runner_owned_fake_upstream_through_the_exact_claude_seatbelt`
- `observes_every_local_claude_managed_policy_source_without_claiming_remote_policy_closure`

## Current immutable successor

Root:

`/Users/yuval.meiri/.engram/evals/native-memory-pilot/native-stale-v3-r2-parser-fix-qualification-20260911.C3mt8R`

Frozen identities:

- Treatment plan SHA-256: `6fa9d2d44dbce57e01d6289be55d5c8dbab3d3c58adf1860600a7bdcc70192c1`
- Control plan SHA-256: `df1f2231257ad8b2787b025bcc1f0db3e390cc826f9d5ff922592f1a8d9ed5b6`
- Treatment protocol SHA-256: `54ea93445bcf4b62b78568bc9804697bb95f9e9013fee91c00e880f353ea60cb`
- Control protocol SHA-256: `625908973d0202c1890c0be9624c2dd64d5ebedc53fbbb4ad9d82062ade3fe80`
- Evaluator SHA-256: `39842d489d853584490ab18d2eabb5cd414244ac8a46f0f4d63f1f210ca9fa0b`
- Engram SHA-256: `b9e7d2606966519299685bdb821432620fdf6cefeab2033187cd3e56696afb95`
- Codex SHA-256: `a30ec314bbd0e3721632234d07db7c99855db3b9f1e32dbe8c791947f07e7629`
- Codex code-mode host SHA-256: `fdd977821def000939dd48da48b39d581845470671135bd4642584eeb0762a6b`
- Claude Code SHA-256: `a681f3008f0050029aeebcab3af51bb6a55ddeb625a3af3141a4416d43cd2558`
- ONNX Runtime SHA-256: `d8be733cb8dd097cfe2b21e069a7462b5ff561625141d9c4b98d866f15bfb852`
- Prelaunch guard SHA-256: `90ebe57e7fb1aa41384ebe684f64b6e45d75d60b049b634fe6d729532ead9b95`

The control plan contains four instructions-only lanes at admission ordinals
31-34. The immutable provider bundle contains exactly 34 contiguous admissions.

## Pre-teaching evidence

The latest guarded checks prove:

- all twelve treatment lanes remain in `prepared` state;
- generic audit `invalid=false` with no lane failures;
- stale-safety setup integrity, plan digest, no-replay lifecycle, and disk reserve
  all pass;
- the admission bundle has no admission, terminal, ambiguous, or journal receipt;
- six isolated Codex homes are ready with owner-only, link-count-one file caches;
- all six Claude lane configurations report logged-in first-party
  `api_key_helper` status;
- the repository `target/debug` directory is absent.

The sanitized Claude status receipt is
`evidence/016-claude-auth-status-pre-teaching.json` under the successor root,
SHA-256 `f22a0548dfe12a46b775e8713843267d1d8fdd5bd7eda95bdeb49478e3c7b0e4`.
This proves first-party helper readiness, not a Claude subscription-login claim.

Manifest 014 mistakenly invoked the preparation-only auditor after auth
provisioning. It failed closed before emitting a report or consuming any provider
admission and left a private zero-byte stdout file. It is retained as diagnostic
evidence. The correct post-provision stale audit is manifest/evidence 015 and was
repeated as 020; both pass with zero observed admissions.

## Exact continuation

If the bundle still has zero admissions and all latest checks remain pristine,
run teaching exactly once through a fresh guard manifest:

```text
<root>/runtime/bin/engram-eval run-native-memory-pilot \
  --plan <root>/treatment/run-plan.json \
  --phase teaching \
  --provider-bundle <root>/provider-bundle-envelope/native-stale-provider-bundle-v1 \
  --approve-provider-execution \
  --confirm-claude-budget-cents 50
```

After all twelve teaching lanes complete, calculate the retention deadline from
the latest valid teaching completion. Do not run activation before the shared
one-hour gate. Never delete, overwrite, replay, or repair a completed or ambiguous
admission.
