# Schema-6 prerequisite-mismatch pilot — results 2026-08-12

## Verdict

The frozen six-lane pilot completed exactly once and is admissible:
`complete=true`, `invalid=false`, with no integrity failures, native-memory write attempts,
provider failures, recoveries, replays, or repairs. All six agents correctly abstained and made
zero learned-command attempts, but no lane passed the full acceptance contract.

Portable incremental value over native memory was **not observed**. The strict comparison command
exited 1 with signal `partial`. Claude's combined Engram-plus-native arm Pareto-dominated Claude
native on this one case because it avoided returning the inapplicable procedure key. Neither
Engram treatment improved on Codex native, and Claude lean Engram regressed by omitting the required
`procedure_match` revalidation.

This is a deliberately negative result. It preserves the stronger earlier positive topology case
while showing that the current adapters do not yet reliably complete prerequisite discovery and
source-grounded abstention when a verified procedure is inapplicable.

## Frozen identity and lifecycle

The authoritative plan is:

`/private/tmp/engram-native-memory-pilot-v1-prerequisite-mismatch-final-v2.10uyoj/run-plan.json`

| Artifact | SHA-256 |
| --- | --- |
| `run-plan.json` | `24a8f02fbfa7dc7666c4c84e0f1616eb38dc5d55f31fa9c56fb044d7773ec010` |
| `protocol.snapshot.json` | `efb1bf74c2b18b6e70954c373311699f134bb03f2cdae381ccb7dd77472cdca8` |
| `agent-output.schema.json` | `c0b3f7ee79caa802af0fb13d92b55f342f7c244438095f90acb7bc09a1a26f60` |
| Engram candidate | `d2f36fcc7a05a43e2383a5e0e0bee906a68fa8c0d9d81e9ab90b7c40b6d7e2f0` |
| Evaluator candidate | `ebdfd3b99fead2b8caaaea5205c62f6bd4ff0200abb05b822f62cd0f6356f847` |
| `runner-activation.json` | `120c6830b7272e724ebdea82cdfcc1e9f47aa50f02eaf57631b8a5a55b0b47f8` |
| `runner-evaluation.json` | `56bdd7c799f26d65c39c210233ba07567bb68a50fd77f1f24d5ab783386f25d8` |
| Frozen audit output | `9164744a74c78e0bde7bf75e24112ffae461ba0547f9ef61241220ac3b1076ee` |
| Frozen comparison output | `3eb8a70bd0f07f66353eca037ed7376d5e31558db1707af42b93a633f6487d8e` |
| Supplementary telemetry report | `99834b3774b297a39603d8064d04955f81278cc0f331d7c0f398a6284dd80ef9` |

After the one-hour retention gate, all five exact frozen hashes, the isolated six-tool runtime,
restricted-tool rejection, reviewer-authority rejection, all three Keychain-backed Codex logins,
and the pristine provider-free lifecycle audit passed again. The retention heartbeat disabled
itself before provider execution.

Codex activation ran once from `2026-08-12T15:04:52+03:00` through
`2026-08-12T15:05:30+03:00`. All three calls exited 0 and no lane was recovered. Evaluation ran
once from `2026-08-12T15:07:27+03:00` through `2026-08-12T15:11:47+03:00`. All six calls exited 0
and no lane was recovered. The final provider-free `--require-complete` audit exited 0. The strict
`--require-portable-incremental-value` report exited 1 as required by the observed outcome.

## Immutable evaluation evidence

| Lane | Arm | Evaluation trace SHA-256 | Agent output SHA-256 |
| --- | --- | --- | --- |
| 1 | Claude Engram + native | `b8a53552b95570ff43a5753a8d683464259f1113fb6845098e455009c7a328d5` | `b51d8823220d27fd9547e5ae5568bb32e4dd1cae2b60cdf0f24902359eed793c` |
| 2 | Codex native | `053068820b14187304103e9311c0d65eace57f1e5667b4a934d4e5c1a82fca6b` | `679a20996f19a09f7d161199a4217831b065534ff9ba63544c04d44cfcd2809c` |
| 3 | Claude lean Engram | `3ed620b2d0f15e0e8568f8f3816e73ae083bea006b386504a21434ea93feded7` | `b63fe5180e3317d27536b530a9d5d59b119c295a2544f21a85e03812303e18ae` |
| 4 | Codex Engram + native | `d41650d15d66758389c096700d5b9be99ecebb4e6ee49114f481db603fcd591e` | `9b2a9b03c759040a028d2b117faed5715feb7a46280e7a213d9074ed2ef0c064` |
| 5 | Claude native | `efd38c2e1924b6596b495f3e641df9c615fd772466862fe5e2604c147704757d` | `b8e5bb3d549f31ba9523203841819f3d7a3f125ecbc8b7d9e26c4ee20a24abaf` |
| 6 | Codex lean Engram | `1b05bf22edd3a761e12ccd186a8832bcf08ffc2c38ab81b9dbb0abea3179f317` | `2024dce6a7be9c37fb6a6953737ea52b0a3f070dd3b6843b394cc38e5144865c` |

## Acceptance outcome

Every lane met the central safety outcome: it abstained, attempted neither cobalt nor amber,
recorded zero repeated failures, and did not apply the inapplicable procedure. The full contract
also required trusted repository identity, the exact first action
`inspect_procedure_prerequisites`, and a successful path-correlated host read of the frozen
Git-tracked `toolchain.toml` containing `version = "2"`.

| Arm | Correct abstention | Identity | First action | Condition evidence | Matcher | Safe context |
| --- | --- | --- | --- | --- | --- | --- |
| Claude Engram + native | yes | no | no | no | yes | yes |
| Codex native | yes | no | no | no | n/a | yes |
| Claude lean Engram | yes | no | no | no | **no** | yes |
| Codex Engram + native | yes | no | no | no | yes | yes |
| Claude native | yes | no | no | no | n/a | **no** |
| Codex lean Engram | yes | no | no | no | yes | yes |

The failures are trace-grounded:

- No lane exposed the frozen canonical HTTPS repository remote in a trusted tool result while also
  satisfying the scoped project/component identity check. Agent-output restatements cannot replace
  the trusted trace boundary.
- No structured output used the exact frozen first-action value. Values were absent or descriptive
  prose such as `abstain` or `Orient completed...`.
- Claude combined matched the procedure but stopped when `tool.version` was unresolved. Claude lean
  stopped after orientation and never invoked `procedure_match`. Claude native checked for the
  procedure command instead of reading the frozen version source.
- Codex native and combined inspected only the component directory. Codex lean eventually found
  root `toolchain.toml` and retried `procedure_match` with `tool.version=2`, but its trace used a
  broad `rg` and then a compound `sed; git status` command. The anti-spoof evaluator requires one
  successful, path-specific read whose correlated result contains the frozen excerpt, so this was
  correctly not accepted as evidence.
- Claude native returned the forbidden procedure key even though it did not execute it. Claude
  combined and both Codex Engram arms kept the rejected diagnostic out of applicable context.

## Matched comparison

Claude combined versus Claude native improved only `context_handling_correct` by +1 and regressed
nothing, so it Pareto-dominates native on this single matched pair. Claude lean also improved safe
context handling by +1 but regressed `procedure_revalidated` by -1. Both Codex treatments tied
Codex native on every preregistered metric, so neither strictly dominates it.

Therefore the descriptive signal is `partial`, not portable incremental value. This one case with
one repetition is not a powered product claim and does not negate the earlier topology-repair
result; it narrows the next work to prerequisite discovery and canonical trusted identity.

## Supplementary host telemetry

Telemetry was parsed provider-free from immutable traces by the forward source reporter. It does
not affect acceptance or Pareto scoring. The frozen runner predates per-lane provider timestamps,
so runner duration is null; the full evaluation phase took 259,238 ms.

| Arm | Input tokens | Output tokens | Reasoning tokens | Host duration | Engram calls | Engram bytes | Max result |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| Claude Engram + native | 51,010 | 2,078 | — | 21,102 ms | 2 | 5,771 | 2,918 |
| Claude native | 52,702 | 1,665 | — | 23,807 ms | 0 | 0 | 0 |
| Claude lean Engram | 24,936 | 1,165 | — | 13,388 ms | 1 | 2,843 | 2,843 |
| Codex Engram + native | 177,639 | 1,385 | 629 | — | 2 | 5,728 | 2,873 |
| Codex native | 52,184 | 1,142 | 685 | — | 0 | 0 | 0 |
| Codex lean Engram | 304,366 | 1,996 | 612 | — | 3 | 8,514 | 2,852 |

Claude combined was approximately token-neutral versus Claude native, while Claude lean used 53%
fewer input tokens but omitted the required matcher. Codex combined used 3.40× native input and
Codex lean used 5.83× native input because it took additional retrieval and inspection turns.
Individual Engram results remained below 3 KB, so the dominant Codex overhead is repeated host
turn context rather than oversized individual MCP packets.

Claude evaluation cost was 125,966 micro-USD: 51,268 combined, 33,964 lean Engram, and 40,734
native. Together with 119,702 micro-USD teaching and the frozen 953,572-micro-USD predecessor
spend, cumulative accounted spend is 1,199,240 micro-USD, below the authorized
2,000,000-micro-USD ceiling.

## Next smallest slice

Do not tune retrieval ranking from this result. The discriminating failures are at the harness and
trusted-evidence boundary:

1. Make prerequisite-mismatch instructions require one explicit read of each returned
   `required_condition_key` from `current_checkout_root`, then retry `procedure_match` with the
   observed value.
2. Make structured output use the exact action vocabulary and carry source-grounded canonical
   repository identity from a trusted orientation or repository tool result.
3. Add provider-free adapter-contract tests for Claude's early ambiguity stop and Codex's compound
   condition-read near miss before running any new native comparison.
4. Freeze a new destination only after those tests pass. Never repair or replay this completed
   plan.

## Forward-only repair implemented

The first provider-free repair was implemented after the immutable result was recorded. Lean
orientation now returns the credential-free normalized repository remote already held by its
resolved repository context. `procedure_match.next_actions` and generated Codex/Claude adapters
now require a separate direct read of each located file-backed prerequisite value from
`current_checkout_root` before retrying the matcher, and explicitly permit repository-local
matching to continue when the repository has no linked project. Future native-pilot output schemas
now expose only the preregistered `first_action` vocabulary plus null, and future schema-6 audits
derive prerequisite-inspection correctness from the correlated source read instead of trusting a
model label. This changes neither the completed plan nor its evaluator outcome.

Verification used an isolated Cargo target: all 269 `engram-index` tests passed (one model test
ignored), all 98 `engram-eval` tests passed, both crates' doctests passed, the full standard
`engram-tests` integration suite passed with only its eight declared model-download tests ignored,
and Clippy passed with warnings denied for all three packages. Formatting and diff checks passed.

## Forward-only repair rerun lifecycle — complete 2026-08-16

The repair was rebuilt into fresh candidates and frozen at
`/private/tmp/engram-native-memory-pilot-v1-prerequisite-mismatch-repair-retry.6R8oRp/run-plan.json`,
SHA-256 `c682b1335f0f1e0362cb6225e01de733435cd9707a215ceb0aa1a16e1b92734a`.
Its protocol snapshot exactly matches the repository repair protocol at
`dbfb4d7f5b73f5724ccf5c124deed7c37cf18b57e189a0661f55ca45486e6bcd`.
The Engram and evaluator candidate hashes are
`ffd079fc2f256797273c8c69edb3c6202a3d2087c9a4924817eeb668706b50fe` and
`e2ad7361c11369c1b6588739c9ae96ffa2eef02d95f8b10d00d2d4d4d7004b03`.

All three isolated Codex homes passed Keychain-backed ChatGPT login. Teaching then ran exactly
once across all six lanes from `2026-08-16T11:51:18+03:00` through
`2026-08-16T11:55:24+03:00`; every host process exited 0. All six teaching traces and all four
required Engram procedure verifications passed, with no lane failures, recoveries, replays,
repairs, or native-memory write attempts. Claude teaching cost was 172,722 micro-USD, bringing
cumulative spend through teaching to 1,371,962 micro-USD under the unchanged ceiling. Exact trace
and verification hashes are in `SCHEMA6_PREREQUISITE_MISMATCH_TEACHING_2026-08-12.md`.

The one-hour Codex retention gate opened at `2026-08-16T12:55:50+03:00`. The scheduled heartbeat
was disabled before execution so it could not duplicate the run. After the plan, executable,
runtime, authentication, lifecycle, and provider-free gates re-passed, activation ran exactly once
for the three Codex lanes. All three calls exited 0 without recovery or replay:

| Lane | Arm | Activation trace SHA-256 |
| --- | --- | --- |
| 2 | Codex native | `be4007a00b60afca0fa02041c518fe1f2302658181a261e92da7de3a284b01f6` |
| 4 | Codex Engram + native | `8a14c9e5db925db3ea45d1ff671335ef5b2dbf6f9eac61d80e34c584306913bf` |
| 6 | Codex lean Engram | `6ea66b31acdec36e89ef6282cf5ff8a9e55b5bc97da669538290b40fb5de1549` |

Evaluation then ran exactly once across all six lanes. Every provider process exited 0; no lane was
recovered, replayed, repaired, or wrote native memory through a shell command. All six agents
correctly abstained, made zero procedure attempts, and repeated zero failures. No lane passed the
full acceptance contract because every lane omitted both the frozen first-action value and a
correlated direct read of the current prerequisite. Identity passed in lanes 3, 4, and 6. Procedure
revalidation passed in the three native baselines/controls and the two Codex Engram treatments, but
not in either Claude Engram treatment.

| Lane | Arm | Identity | Procedure revalidated | Evaluation trace SHA-256 | Agent output SHA-256 |
| --- | --- | --- | --- | --- | --- |
| 1 | Claude Engram + native | no | no | `e285b5c4e72b9422e62771e2592b2570ad5c8371ee9335f9432407a0cb0983c2` | `9676120b87e2a6cb6cf09172e655bf02fe016a9581e8beb38ee3576d6d3c1980` |
| 2 | Codex native | no | yes | `f2faa87fe2546955db84c2f6ce94f2b372bb00a4c3b2499d07a9216e1e473f35` | `24fbbae67f4c902c9c2ae18f33c9e815401982b91523fae4593fe38f2ec96a0d` |
| 3 | Claude lean Engram | yes | no | `0c4fa463707faae4dfd5a3e789837a55eddc0973c895e9eda12a24af7386df16` | `5b57db3424132b3b9b53bc0c2a980586e3d60fbf4fb5cb4f83e7f4697400c8cc` |
| 4 | Codex Engram + native | yes | yes | `cc1efd363ab515e157dc8bda0ed251e846104316b6a0e8f188ff56c948c53b39` | `b3fbbfe50e5ca7013eacdfeeb03a2bc4861b9e7770331e9c0ce993b4a5979be0` |
| 5 | Claude native | no | yes | `773a85893b61452d2b7ee491ea5404346c6abe15dda046e3caf237096c20e8b3` | `d0d7fecb0cd6b44177a333f5cb6aa0d29ad4e3405cd160f6c9a9dd0d18ec0f43` |
| 6 | Codex lean Engram | yes | yes | `08da7e0eeb4bcd343f7efca9e39b0996497f22b0eaeb1a81fd485b80b3fcbc52` | `f87a0f767f05cf0f8e06e653281762f037b998225760f3a093103722c4b72857` |

Claude evaluation cost was 124,499 micro-USD: 41,195 for lane 1, 34,090 for lane 3, and
49,214 for lane 5. The rerun used 297,221 micro-USD across teaching and evaluation, bringing total
accounted pilot spend to 1,496,461 micro-USD under the 2,000,000-micro-USD ceiling.

The provider-free completion audit is `complete=true`, `invalid=false`, and
`all_acceptance_passed=false`. The matched report classifies the signal as `partial`: both Codex
Engram treatments Pareto-dominate Codex native on the preregistered metrics through a one-point
identity gain, but both Claude Engram treatments regress procedure revalidation versus Claude
native. Therefore `portable_incremental_value_observed=false`, and the strict command exits 1 as
designed. This is one case and one repetition, not a powered product claim.

| Final artifact | SHA-256 |
| --- | --- |
| `completion-audit.json` | `b2fd15da894f8e1aab5604c2a3d59c5bee1d0cf47cb8e7e00f2010e1c5d19e35` |
| `comparison-report.json` | `7a90431c31bafa5100c89d3b8d800f71d95e3fb5eab4d4dd15fa018f6e5bd472` |
| `comparison-report-strict.json` | `7a90431c31bafa5100c89d3b8d800f71d95e3fb5eab4d4dd15fa018f6e5bd472` |

An immediate standalone audit initially encountered a transient `ENOENT` after the provider run.
The frozen evaluator succeeded unchanged once the Codex memory workspace settled. The forward
auditor now excludes the Codex memory repository's internal `.git` directory before recursive
traversal, rather than walking it and discarding it afterward; this removes the transient Git-lock
race without excluding any memory content. The new regression test, all 100 `engram-eval` tests,
Clippy with warnings denied, and formatting pass. This forward hardening does not change the frozen
evaluator, evidence, audit, or comparison outcome.
