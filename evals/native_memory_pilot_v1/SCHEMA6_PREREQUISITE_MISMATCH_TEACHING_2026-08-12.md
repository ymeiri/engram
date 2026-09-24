# Schema-6 prerequisite-mismatch pilot — teaching complete 2026-08-12

## Frozen identity

The only authoritative plan is:

`/private/tmp/engram-native-memory-pilot-v1-prerequisite-mismatch-final-v2.10uyoj/run-plan.json`

| Artifact | SHA-256 |
| --- | --- |
| `run-plan.json` | `24a8f02fbfa7dc7666c4c84e0f1616eb38dc5d55f31fa9c56fb044d7773ec010` |
| `protocol.snapshot.json` | `efb1bf74c2b18b6e70954c373311699f134bb03f2cdae381ccb7dd77472cdca8` |
| `agent-output.schema.json` | `c0b3f7ee79caa802af0fb13d92b55f342f7c244438095f90acb7bc09a1a26f60` |
| Engram candidate | `d2f36fcc7a05a43e2383a5e0e0bee906a68fa8c0d9d81e9ab90b7c40b6d7e2f0` |
| Evaluator candidate | `ebdfd3b99fead2b8caaaea5205c62f6bd4ff0200abb05b822f62cd0f6356f847` |

Every hash, the isolated six-tool agent-profile runtime, restricted-tool rejection, reviewer-
authority rejection, and all three Keychain-backed Codex logins passed immediately before
teaching. The provider-free pre-teaching audit found six `prepared` lanes, `invalid=false`, no
failures, no artifacts, and no native-memory write attempts.

## Teaching outcome

Teaching ran exactly once in frozen order from `2026-08-12T13:57:39+03:00` through
`2026-08-12T14:01:57+03:00`. All six host calls exited 0; no lane was recovered, replayed, or
repaired. All six traces passed. Each Engram-bearing lane added exactly one procedure candidate,
and the trusted evaluator verified it against the frozen receipt. The post-teaching audit is
`invalid=false` with no lane failures or native-memory write attempts.

| Lane | Arm | Teaching trace SHA-256 | Verification SHA-256 | State |
| --- | --- | --- | --- | --- |
| 1 | Claude Engram + native | `b39bd09ca58d79604a25fff3d06de8249d9e8fd01188a77d8226fbbf67ca592c` | `f38d1599636b1a6f6659ba50049ccf741a439ece2f0c8dca2caac20086f2c710` | `ready_for_evaluation` |
| 2 | Codex native | `4253f4759a2732016b4d67ebd662c82fe404b3e53ddbb06bef88dbc8e7a94489` | — | `awaiting_activation` |
| 3 | Claude lean Engram | `0a6d96574298e70ac4eb9723a4fe6a90681b141f83813c3b39173f79a71b9ef9` | `fd8fdb2f51b62e8aeb9e3e803f6bdd613fcf88c796b421cca4ec862026b52a4e` | `ready_for_evaluation` |
| 4 | Codex Engram + native | `ae8a47b99724a3ee5cd016ba3584c7f6466eabc2dc6923c6db88dce702ff24f5` | `fd1e05617cfeb080b96ded27a92f08ad96d4511d74fa34a37c99039638d515a3` | `awaiting_activation` |
| 5 | Claude native | `6119ea8d91ee21657f7cd77c7ab6da6b87763a557319702d661c588c5e95ec80` | — | `ready_for_evaluation` |
| 6 | Codex lean Engram | `8fdd6b487253088d72114e30cdf3407e10495006b32f63ba33e57aeb6673a059` | `f189fabe955fc0566ef9fbd03d3ad3e4d1574bdb0405af30c1672e096f13ea45` | `awaiting_activation` |

Codex generated fresh native-memory state in lanes 2 and 4. Claude generated native `MEMORY.md`
only in lane 5; absence in the combined lane is a valid measured outcome. Engram data artifacts are
present and passed in lanes 1, 3, 4, and 6.

Claude teaching cost was 119,702 micro-USD: 39,904 for lane 1, 43,667 for lane 3, and 36,131 for
lane 5. With the frozen predecessor spend of 953,572 micro-USD, cumulative accounted spend is
1,073,274 micro-USD, below the authorized 2,000,000-micro-USD ceiling.

## Retention gate

The latest teaching completion timestamp was `2026-08-12T14:01:57+03:00`; the one-hour retention
deadline passed at `2026-08-12T15:01:57+03:00`. The `resume-engram-pilot` heartbeat disabled itself
before provider execution. All exact hashes, runtime attestation,
`check-native-memory-pilot-auth --require-ready`, and the provider-free lifecycle audit passed
again. Activation and evaluation then ran exactly once with the standing 50-cent allocation.

The final audit is `complete=true`, `invalid=false`, but no lane passed the full task and the strict
portable-value gate failed. Complete evidence and interpretation are in
`SCHEMA6_PREREQUISITE_MISMATCH_RESULTS_2026-08-12.md`.

## Supplementary trace telemetry

During the retention interval, the forward source reporter gained provider-free extraction of
host-reported input, cached-input, cache-creation, output, and reasoning-token fields plus the exact
serialized byte count of Engram MCP result `content` arrays returned to each host. Claude's direct,
cache-creation, and cache-read input fields are summed for its comparable total; Codex's reported
`input_tokens` is retained as its total and cached input remains a subset. Claude's reported
`duration_ms` is retained. Codex evaluation JSONL does not expose per-run wall time, so its
host-reported duration remains explicitly null instead of being inferred from filesystem
timestamps. The forward runner now records the exact wall interval around every provider process
for both hosts; legacy and frozen reports without those timestamps remain compatible.

This telemetry is supplementary: it does not alter frozen acceptance, Pareto dominance, the plan,
or either attested candidate binary. All 97 `engram-eval` tests pass, including exact trace-shape
and legacy-runner compatibility tests; evaluator Clippy, formatting, and diff checks pass.

## Forward-only prerequisite-evidence repair lifecycle — 2026-08-16

The completed 2026-08-12 plan above remains immutable. After its negative result, the documented
forward repair was rebuilt into new candidate binaries and frozen in a new destination. The only
authoritative repair-rerun plan is:

`/private/tmp/engram-native-memory-pilot-v1-prerequisite-mismatch-repair-retry.6R8oRp/run-plan.json`

| Artifact | SHA-256 |
| --- | --- |
| `run-plan.json` | `c682b1335f0f1e0362cb6225e01de733435cd9707a215ceb0aa1a16e1b92734a` |
| `protocol.snapshot.json` | `dbfb4d7f5b73f5724ccf5c124deed7c37cf18b57e189a0661f55ca45486e6bcd` |
| `agent-output.schema.json` | `27d023fca5096c090f3d1cff9e5b02f9b9cbc67fc9d531ab0edcf34272e133ae` |
| Engram candidate | `ffd079fc2f256797273c8c69edb3c6202a3d2087c9a4924817eeb668706b50fe` |
| Evaluator candidate | `e2ad7361c11369c1b6588739c9ae96ffa2eef02d95f8b10d00d2d4d4d7004b03` |

The plan attests Claude Code 2.1.233 at
`bc466b6cde63edafc773f471a1fb98787fabb31f52240c8616ce7e1f587b212d` and Codex
0.148.0-alpha.9 at
`6170ff5578170ee9b74ad92bfcff96e6186f41d02b60815a7c2b01ad424c754f`.
The protocol snapshot exactly matches the repository repair protocol. The repaired output schema
exposes only `inspect_procedure_prerequisites` or `null` for `first_action`, and documents the
trusted repository, project, and component identity fields. All candidate hashes, effective
six-tool runtime checks, restricted-tool and reviewer-authority rejections, and isolated
Keychain-backed Codex logins passed before teaching.

Teaching ran exactly once from `2026-08-16T11:51:18+03:00` through
`2026-08-16T11:55:24+03:00`. All six host calls exited 0; no lane was recovered, replayed, or
repaired. Each Engram-bearing lane added one procedure candidate and the trusted evaluator
verified the returned ID against the frozen receipt. The audit is `invalid=false`, with no lane
failures or native-memory write attempts.

| Lane | Arm | Teaching trace SHA-256 | Verification SHA-256 | State |
| --- | --- | --- | --- | --- |
| 1 | Claude Engram + native | `0f082a43f54c54c332daddcba2671314d3b4541dfe420c2d4eb81860a875f6d3` | `e7a1ee312b27f9c0fb4f8099ca15150341fefd06bf06e9ca70fa0f877fbea401` | `ready_for_evaluation` |
| 2 | Codex native | `89c5c2d71d73d9d288c66f82da18fae82d4c9d5645235745c6f987985fd3dd11` | — | `awaiting_activation` |
| 3 | Claude lean Engram | `b46b559c1f02fe7445018c613a97e0e01f5e02c8e8c792c3652c854b1eea1725` | `8ec46e23024bef4fbf0bc20a66721a02d860085ab93da28ef15ac52e637a25f1` | `ready_for_evaluation` |
| 4 | Codex Engram + native | `c55b405d66b69d911259c3e183e3e356016b4de46ebea4e6a3eda7bf3523234e` | `5110a9d109c4535328db2865af1ac3121d0319337545760c69532c6be78ccf9a` | `awaiting_activation` |
| 5 | Claude native | `5cad1bec22d80e9a09c9ddb93f3fd6bd8425b5a5f77869d44a6ad7a7856f7214` | — | `ready_for_evaluation` |
| 6 | Codex lean Engram | `a7114b4abb2b7b611d6a075aff8ac318e50ceaa3f23d6316cd3e3e9f51bbccff` | `672830bc63ff3ff090365519dc89980a97ce03199a1ea4c8f2729862b0aee1d5` | `awaiting_activation` |

Claude teaching cost was 172,722 micro-USD: 79,071 for lane 1, 47,368 for lane 3, and
46,283 for lane 5. With 1,199,240 micro-USD of prior accounted spend, cumulative spend through
teaching is 1,371,962 micro-USD under the unchanged 2,000,000-micro-USD ceiling.

The one-hour Codex retention gate opened at `2026-08-16T12:55:50+03:00`. The
`resume-engram-pilot` heartbeat was disabled before execution to prevent a duplicate run. Every
frozen hash, runtime attestation, Keychain login, teaching trace, procedure verification, failure
list, native-memory write-attempt list, and provider-free audit re-passed.

Activation then ran exactly once in Codex lanes 2, 4, and 6; all three calls exited 0 without
recovery or replay. Their trace hashes are respectively
`be4007a00b60afca0fa02041c518fe1f2302658181a261e92da7de3a284b01f6`,
`8a14c9e5db925db3ea45d1ff671335ef5b2dbf6f9eac61d80e34c584306913bf`, and
`6ea66b31acdec36e89ef6282cf5ff8a9e55b5bc97da669538290b40fb5de1549`.

Evaluation subsequently completed all six lanes exactly once. Every provider call exited 0; no
lane was recovered, replayed, repaired, or wrote native memory through a shell command. Claude
evaluation cost was 124,499 micro-USD, bringing cumulative accounted spend to 1,496,461 micro-USD.
The final audit is `complete=true`, `invalid=false`, and `all_acceptance_passed=false`: all six
agents abstained safely with zero procedure attempts and zero repeated failures, but none performed
the required direct prerequisite read or reported the frozen first action. The matched report is
`partial`, with Codex-only identity gains and no portable cross-host incremental value. Exact
evaluation/output/report hashes and the per-lane acceptance result are recorded in
`SCHEMA6_PREREQUISITE_MISMATCH_RESULTS_2026-08-12.md`.
