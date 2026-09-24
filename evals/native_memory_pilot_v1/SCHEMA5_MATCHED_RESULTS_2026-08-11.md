# Native-memory pilot v1 schema-5 matched result — 2026-08-11

## Verdict

The immutable source plan completed teaching, retention, Codex activation, and all six provider
evaluation calls exactly once. It remains **incomplete and invalid**, so that source provides no
admissible matched comparison. A forward-only, evaluation-only successor then reran all six
matched evaluation arms into new destinations with the corrected Codex terminal-response
boundary. That successor is **complete and valid** and reports a descriptive portable incremental
value signal for Engram and combined memory across both hosts. It does not establish a powered
product claim, and zero lanes passed the full task because every lane failed the exact identity
tuple.

- source plan:
  `/private/tmp/engram-native-memory-pilot-v1-schema5-lifecycle-accounted.GNpz9q/run-plan.json`
- run-plan SHA-256:
  `aad54051adc52f0c44deccd6e26c6fd0fa05c635353999ea9c78b6a360b7e6a1`
- teaching report SHA-256:
  `ddce5e6033867a5066e25a5f4d37fd6ce7ad5b74a78eb76e236fe01eb35fedd0`
- activation report SHA-256:
  `bb32f172ecb89e75d546455a6f69eac6f33c2304de30beb5cbaff868d38403d4`
- final source audit: `complete=false`, `invalid=true`
- provider-free report classification: `invalid`
- evaluation phase report: absent, as required after the runner rejected lane 6

Successor identity:

- successor plan:
  `/private/tmp/engram-native-memory-pilot-v1-codex-terminal-recovery-v2.RP7KHm/run-plan.json`
- successor run-plan SHA-256:
  `3d67c97a2637953c6ab40546bd1c456e8dd8d3d3e7f6439b4d769aa3939e578d`
- successor evaluator:
  `/private/tmp/engram-codex-terminal-recovery-bin-v2.qZ5mZf/engram-eval`
- successor evaluator SHA-256:
  `e39415086fef1a43907bc01a51d24ddf475a895d9c3635d9bc067ccb7ea5bad6`
- final successor audit: `complete=true`, `invalid=false`, `all_acceptance_passed=false`
- strict comparison gate: passed
- comparison signal: `engram_and_combined_across_hosts`
- portable incremental value observed: `true`

No source lane, trace, structured output, or predecessor was repaired, replaced, selected post
hoc, or replayed. The source plan SHA-256, rejected lane-6 trace SHA-256, and original evaluator
SHA-256 remained unchanged after successor completion.

## Completed lifecycle evidence

Immediately before activation, the frozen plan digest, evaluator and host executable hashes,
effective six-tool Engram MCP contract, isolated Keychain-backed ChatGPT authentication, and
lifecycle audit were revalidated. Attestation initially found the project Engram daemon occupying
the runtime-probe port; the non-pilot daemon was stopped, the port was proved free, and attestation
then passed before any provider call. All three Codex activation calls exited 0, with trace hashes:

1. Codex native:
   `5192405e1cfd875eb1e1d6be16ae2d2a8f514f387d7faec20d735b87e0a8617e`
2. Codex Engram plus native:
   `a5293351d4d9e376d3ba6820463ea92bc43ad7711c4aec5c611046adb937207e`
3. Codex lean Engram:
   `e76863c3df766ec26a650ccf357cc13acc76ecc69feaa229812898c77f99de3c`

The post-activation audit was pristine: all six lanes were `ready_for_evaluation`, with no
integrity failures. The approved evaluation command then invoked all lanes once, in frozen order.
Evaluation trace hashes are:

1. Claude Engram plus native:
   `afe2a9dbbf3b3b557450a9e72d81dc7e236e795f90ab421fc02fd5f3da84e423`
2. Codex native:
   `0eaeec574cd54269d35c3f0cc28320a842dfb9452d1a9ba9345acc86692a2c56`
3. Claude lean Engram:
   `9d10d924cc40a77f2e50cf2c1a07cf9d1be56ccce72dc4cec2b02b7491bde2f8`
4. Codex Engram plus native:
   `088009f2b8ad65452ac9fae90fd6698e34e18b1b109f2136b14c11dc6360ac89`
5. Claude native:
   `72e962973cc885f913a79f3ae5aba23a616083df8342b31e6afe10285352e7e3`
6. Codex lean Engram:
   `944375aa65acb171315c5df0c8e43e8719aaa01d0f5d3c6c57890e6b3a284cc1`

Claude reported `$0.0805593`, `$0.0671319`, and `$0.0396165` for evaluation, or `$0.1873077`
before conservative micro-dollar rounding. These calls are retained in successor accounting.

## Integrity failure

Lane 6 produced a complete Codex turn and exited 0. Its 19-line JSONL trace contains one
`turn.started`, one `turn.completed`, the expected Engram calls and command execution, and two
distinct schema-shaped `agent_message` events:

1. Before tool use, Codex emitted a progress message saying that it was using the Engram
   memory-session procedure. Because `--output-schema` was active, that message had the output
   schema's fields with null or empty evidence.
2. Immediately before `turn.completed`, Codex emitted the real final response. It reported the
   Atlas remote, the cobalt command, the durable procedure key, evidence targets, and
   `ATLAS_CONTEXT_PROBE_OK`.

The frozen evaluator recursively collected every schema-shaped object and required exactly one
distinct value. It therefore stopped with:

```text
evaluation trace must contain exactly one distinct structured agent output, found 2
```

The lane-6 stderr SHA-256 is
`34a9c10d7e030634182cfb796e83240a90a4bb40afa7fcba92cc65b39e0d0dcd` and contains only host
warnings, not a provider failure. Codex 0.147.0-alpha.6.5's own local `exec --help` defines
`--output-schema` as the shape of the model's **final response** and separately defines
`--output-last-message` as the last agent message. The recursive evaluator was therefore using the
wrong host lifecycle boundary. Selecting the final message inside the frozen source would still be
an unregistered post-hoc reinterpretation, so the source remains invalid.

## Descriptive source observations

These observations explain the traces but are not a valid matched-comparison claim:

- Claude Engram plus native, Claude lean Engram, and Codex Engram plus native all retrieved the
  verified procedure, resolved `tool.version=3`, executed cobalt first, observed exit 0 and the
  marker, and cited the procedure evidence.
- Codex native abstained and did not execute the procedure.
- Claude native executed cobalt successfully but did not cite the preregistered evidence.
- Every completed lane reported component `services/worker`; the frozen identity contract expected
  the authoritative `queue-worker` value in `services/worker/component.json`. Consequently every
  completed outcome failed the identity metric. This is a real adapter/orientation gap, not an
  evaluator integrity failure.
- No lane executed the known amber failure, so repeated failures remained zero.

The provider-free report exposes descriptive deltas for the five completed lanes, but explicitly
labels the experiment `invalid`, reports no Codex lean-Engram pair, and states that outcome
comparisons are not valid. Those deltas must not be quoted as incremental-value evidence.

## Forward-only recovery and valid successor

The evaluator now has a narrow recovery contract for this exact failure class:

- it requires exactly one invalid Codex lane, every other source lane complete, no source
  evaluation report, more than one schema-shaped message, and exactly one complete Codex turn;
- it checks the attested Codex binary's local help for the final-response contract;
- it treats only the terminal `agent_message` immediately before `turn.completed` as Codex's final
  structured response, while retaining unique-output extraction for Claude;
- it freezes a new evaluator executable and requires the source evaluator bytes to remain intact;
- it accounts for predecessor, source teaching, and source evaluation Claude spend;
- it creates new evaluation destinations and reruns all six matched evaluation arms. It does not
  write into or reinterpret the source plan.

The first recovery build exposed that the schema rewrite still assumed the earlier Claude
draft-2020-12 rejection. No provider call ran and no plan was frozen. The recovery contract was
then split by exact host-boundary failure kind: the Claude recovery still replaces only the
rejected dialect, while this Codex-terminal recovery requires the already-portable source schema
and changes only Codex's output-schema path. The completed evaluator passes 87 of 87 engram-eval
tests, including a source-shaped two-message Codex trace and the portable-schema recovery case.
Engram-eval Clippy passes with warnings denied; formatting and diff checks pass.

Before execution, the successor re-attested every frozen executable and the live six-tool Engram
contract, proved all three isolated Keychain-backed Codex homes ready, and passed the provider-free
ready audit with `invalid=false`. It then made exactly three Claude and three Codex evaluation
calls. It did not run teaching or activation. Every call exited 0, no trace was recovered or
replayed, and the complete audit passed.

Successor evaluation trace SHA-256 values, in lane order, are:

1. `34cda454be1d6649a7b39cfde89786b12b773bd727b29185e717a28b066ef716`
2. `4240adb6706b5bad1a753da93a87e2dbc9820e4203b5dc0ef0b567bf97837250`
3. `d4cb9ebc5376e578b6e42bbf1e514e6afa45e61f085e73c4c6a47772615c0a76`
4. `4c933d1be683e12552df19cfc853e01f29d49b2b817340348e9480e809e5d004`
5. `49c619f011112e542639a621c38b97aa5f6f48665ca95974bf1619caf5c71cc3`
6. `758a8d33b1c611f8be7d70a857f76928b0b0518151a30b2ad1ae731625126167`

Structured agent-output SHA-256 values, in lane order, are:

1. `fb58c865807fb2553e45ca1231c9f00ae442db3ae2fb263148419676ee10a254`
2. `0fdfd9b8c806fd110c3784285d4a3abdde8c2e06c8426b4e6e7ef2f8320ba506`
3. `32b6c371f9fc8f389402c8019d225a3c53ca41012c5da7e4e89653e2c61be4ed`
4. `72d8f15e5b327151a62e66e61b49e4b694986f3c47f068016bddaddb5e3f19d8`
5. `1f82327a1b2b3f1df5a3bda799710ab2c4007e02a318a35a8bff3c199d378ed0`
6. `e2f79f618b8d096ef51d17ee7fd429e171ddf8bb8930993c46c74252a4b1a94f`

Claude reported 72,609, 79,463, and 41,451 micro-USD for the successor, totaling 193,523
micro-USD. With 433,569 micro-USD of predecessor and source spend, total accounted Claude spend is
627,092 micro-USD under the frozen 1,000,000 micro-USD ceiling.

## Successor comparison

The preregistered identity tuple was remote `https://github.com/acme/atlas`, project `null`, and
component `queue-worker`. Every lane missed at least one element:

- Claude combined and Claude lean Engram returned the remote but no component.
- Codex combined returned the remote but no component.
- Codex lean Engram returned component `services/worker` instead of `queue-worker`.
- Codex native correctly found the remote and `queue-worker` but invented project `atlas`, then
  abstained because it had no applicable durable procedure.
- Claude native returned remote `unknown`, project `atlas`, and component `context-probe`.

Trace and source inspection localize the Engram-side cause. Native-pilot preparation materializes
the fixture but starts each Engram lane with an empty isolated store and teaches only the
repository-scoped procedure. It does not run the general fixture seeder that registers the
`queue-worker` component. Consequently Engram correctly matched the moved checkout by remote but
returned `resolution.component_names: []`; the hosts then inconsistently inferred component and
project fields from paths or local files. This is not a failure to match an already-registered
component. The next slice must register or derive topology through a product-realistic,
source-grounded flow—or make the host inspect authoritative component metadata with trusted trace
evidence—without hardcoding the benchmark value.

All four Engram-bearing lanes nevertheless retrieved the verified procedure, resolved
`tool.version=3`, executed cobalt first from the current checkout, observed exit 0 and
`ATLAS_CONTEXT_PROBE_OK`, cited evidence, and avoided amber. Claude native also ran cobalt and
observed success but did not cite the preregistered evidence. Codex native abstained without
executing a command.

Against native memory on Claude Code, both Engram and combined memory improved evidence citation
by one and regressed on no preregistered metric. Against native memory on Codex, both treatments
improved first action, context application, evidence citation, first procedure attempt, successful
execution, expected exit, and success marker by one and avoided one abstention, again with no
regression. The strict report therefore marks both treatments as Pareto-dominating native memory
on both hosts and reports portable incremental value.

That result is deliberately bounded: it is one case with one repetition, every full-task pass
delta is zero, and every identity delta is zero. It supports continuing Engram's verified
engineering-context direction; it does not prove that Engram is broadly better than either host's
native memory or that the flagship goal is complete.

## Post-pilot topology repair matched result — 2026-08-12

The exposed Engram component gap now has a source-grounded candidate repair. From the current cwd
to the repository root, orientation discovers `component.json` only when Git tracks it, requires a
small regular non-symlink JSON file with bounded `name` and `kind` values, and derives the result
live rather than persisting a second copy. A same-path checked-in manifest overrides a manual
registry entry in the returned context; deleting or renaming the manifest changes the next
orientation immediately. Untracked manifests are ignored and malformed tracked manifests make
resolution fail closed. Manual `engram repo component-add` registration remains the user-local
fallback when a repository has no checked-in component manifest.

The lean orientation response now exposes the component's checkout-relative manifest path and
SHA-256. A provider-free runtime smoke used the exact moved-checkout lane-6 fixture and cwd that
exposed the gap. Candidate Engram binary
`/private/tmp/engram-topology-candidate-bin.cgPgKR/engram`, SHA-256
`d2f36fcc7a05a43e2383a5e0e0bee906a68fa8c0d9d81e9ab90b7c40b6d7e2f0`
returned trace `019ff0f0-6d77-7f90-815b-fdc1b83cdb68` with:

- repository `atlas`;
- selected project `null`, `requires_confirmation=true`;
- component `queue-worker`;
- source `services/worker/component.json`;
- source SHA-256
  `975a5bb285038b1f75fa8c3242b685be04945b81a64e653ad6dced871d629d74`.

That source hash independently matches the Git-tracked fixture file. Focused repository tests
cover live rename/removal, untracked rejection, and malformed-source abstention; focused and MCP
integration orientation tests cover the returned evidence. The complete `engram-index` suite
passes 268 tests with one model-download test ignored, and the affected MCP integration suites pass
55 tests.

The evaluator candidate now derives project/component acceptance for Engram-bearing schema-5
lanes from the correlated `engram.orient` tool result for the exact evaluation cwd. It independently
requires the component manifest to remain tracked, regular, inside the resolved component path,
and byte-identical to the returned SHA-256. Tests cover both Codex and Claude trace formats, reject
an agent-message spoof, reject changed source bytes, and prove that deliberately wrong identity
fields in the final model JSON cannot override trusted orientation evidence. Candidate evaluator
is `/private/tmp/engram-topology-candidate-bin.cgPgKR/engram-eval`, SHA-256
`2e01da82edfc2e16b7c922dc4439c271b9c0b03f9ff2e8510137f67d007dfe50`.
All 90 evaluator tests pass, including the persistent-store seed test that had previously been
blocked by the free-space guard. Clippy for core, index, MCP, evaluator, CLI, and integration tests
passes with warnings denied; formatting and diff checks pass. The exact no-override
agent-profile runtime regression also passes against the selected candidate executable.

The candidate's exact agent-profile runtime attestation also passes without a persistent store.
`contract --profile agent --verify-runtime --json` now launches the exact executable as an
in-memory HTTP daemon and as its real stdio proxy, then verifies the six-tool hash
`cb48eb1bc9a6d21e8aafb38a4bf012a39aceb37f63a5d7684d30d56987a2836e`, instruction hash
`a608ff0614525387b6cebd89b14fc73f4c9e93b5ab00c7d5b9d260e5850037a7`, restricted-tool
rejection, and reviewer-authority rejection. Persistent user and pilot stores still retain the
normal disk-headroom gate.

This forward-only result does not reinterpret the completed successor. The new six-arm run used
fresh destinations and the frozen candidate hashes above. Its preregistered source is
`evals/native_memory_pilot_v1/protocol-schema-5-topology.json`, SHA-256
`990c582781f92aa4220666095373824ff9635f017ca55b378f2dbf1f89b2baab`. It preserves the exact case
and arm order, accounts for 627,092 micro-USD of prior Claude spend, and reserves $0.60 under a
new $1.30 cumulative ceiling authorized by the operator's standing provider-execution approval.
The initial compile was gated by local filesystem headroom: the volume had about 17 GiB available
while Engram's persistent-store policy required about 18.5 GiB. No prior trace, lane, or user-owned
build cache was deleted to bypass that guard. The operator subsequently approved removal of the
isolated generated Cargo target at
`/private/tmp/engram-codex-terminal-recovery-build-v2`; `cargo clean` removed 32.5 GiB and restored
about 46 GiB of free space without touching any lane, trace, candidate binary, or repository
worktree file. The fresh six-arm plan is now frozen at
`/private/tmp/engram-native-memory-pilot-v1-topology-repair.BtznKt/run-plan.json`, SHA-256
`643edf269ddd3c0aeb02f086710e464909b829af1f8b39c98039d0067e7fe58a`. The frozen structured-output
schema SHA-256 is `c0b3f7ee79caa802af0fb13d92b55f342f7c244438095f90acb7bc09a1a26f60`.
Provider-free audit reported all six lanes in `prepared`, `invalid=false`; binary and effective-
runtime attestation passed with the candidate hashes above. Ordinary Keychain-backed ChatGPT
browser login then became ready in all three isolated Codex homes.

The approved teaching phase completed all six lanes exactly once with exit code 0. The fresh audit
reports `invalid=false`, no lane failures, no native-memory write attempts, Claude lanes 1, 3, and
5 in `ready_for_evaluation`, and Codex lanes 2, 4, and 6 in `awaiting_activation`. Teaching-trace
SHA-256 values, in frozen lane order, are:

1. `abd2fcf1f875828846e778b79af8e684d6ac93b4c241b382929602bfcaee71de`
2. `133747f196d31c694f010e8f043c6dd49f24fdea4b4dac4f123e871d2090dfac`
3. `0e5b89d062b8b26e7db9ec212c6e90cc04d1ee7ae4a7c63a3a68204eb8bb56d6`
4. `77eafa06a2fb8614a80fbf9de84cddbdc2f0f2bb855406cc1952f279aa6a7440`
5. `4212f88818de2d3ad4826d97607c1d514b533aebe0d013160ff4b9d3ee5943a1`
6. `94882e4616d7df5e151a4213369dfd52bf00c8281df263dce58223bd4cdf816b`

Claude reported 145,003 micro-USD for the three teaching calls; the authenticated ChatGPT Codex
lanes reported no API spend. Together with the frozen predecessor accounting, cumulative Claude
spend through teaching was 772,095 micro-USD, below the $1.30 ceiling. The latest Codex teaching
trace fixed the one-hour retention deadline at `2026-08-11T18:24:58.013+03:00`.

After that deadline, the run-plan digest, protocol and schema digests, candidate executable hashes,
effective six-tool runtime contract, isolated Keychain-backed authentication, and provider-free
lifecycle audit were all revalidated. The approved activation phase then ran Codex lanes 2, 4, and
6 exactly once. All exited 0, with trace SHA-256 values:

1. Codex native: `d7c28be49b57631b2471446f869c9f76c90f55ac57bfdcfa9abc351606941e70`
2. Codex Engram plus native: `cb2fc51de862ba98e5caf871b325e60cb1c229408c5d60592d85262e6863bc18`
3. Codex lean Engram: `b427501f7bb29e5943322dac6ba2f7249c2e9564472a974f71247b91326955c0`

The post-activation audit, attestation, authentication, and frozen hashes remained pristine. The
approved evaluation phase then ran all six lanes exactly once. No lane was recovered, repaired, or
replayed. Evaluation trace and structured-output SHA-256 values are:

| Lane | Arm | Evaluation trace | Structured output |
| ---: | --- | --- | --- |
| 1 | Claude Engram plus native | `2f3db3b580fcaf95a36feb3841f8543babe36c793793718b1482ee1adc8633d7` | `b1fa111a6ef3cab7b478b243fc841133ca703030d191ef90d3086a48e961fe85` |
| 2 | Codex native | `36f7cf8c7bd770490f7af3b59c4fa64bfe7fc2fe0575b27f3ec12ba81ecd98f3` | `43b6977becab4b818ecd0d21498fd418cc14da9576f08904549cded020216dce` |
| 3 | Claude lean Engram | `23f550f348ad853e4912ff6b8fd849d59bd39e7e590e6f1b882dbf2aa73426ad` | `ea3c92030b8484d1392399704334cbb613e147e00b6385f7b2145170b730eb39` |
| 4 | Codex Engram plus native | `6ba147b3a13627f20d29dd853f28667da4970d2992f55242ba9afcdc5000d955` | `e9c03fbd9416e1cf9847911b4e022e780933195810f5955aa01ecfff7755a478` |
| 5 | Claude native | `ee6cae173367a2b055779a4a298a37f239eecbea15d0245ee40e7d4e8cecda7b` | `fe88f77343f918e4094bde58eba11095966ab10ae40ebc46c27be7ff7260b939` |
| 6 | Codex lean Engram | `f1e8cc2fa56cb0ae39519ef790e93edc65e3f70059f37dc6beed3d029d0e931d` | `069da9e2448d83d9ea6f654a019205284af4d163cf69c18955c247edba6f1e94` |

The final audit is `complete=true`, `invalid=false`, with no integrity failures, lane failures,
native-memory write attempts, or turn-boundary budget exits. All four Engram-bearing lanes passed
every acceptance metric: exact source-grounded identity, correct first action, context application,
evidence citation, procedure revalidation, first-attempt cobalt execution, expected exit, and
`ATLAS_CONTEXT_PROBE_OK`. Both native-only controls failed the exact identity contract. Claude
native otherwise executed the procedure successfully; Codex native abstained despite the
applicable context and failed the associated first-action, context, evidence, and execution
metrics.

The preregistered strict report passed with `portable_incremental_value_observed=true` and signal
`engram_and_combined_across_hosts`. Engram and combined memory Pareto-dominate native memory on the
frozen metrics on both hosts; every treatment-versus-native pair has `passed_delta=1` and
`identity_correct_delta=1`, with no regression. Claude reported 181,477 micro-USD for evaluation,
bringing this plan's teaching-plus-evaluation spend to 326,480 micro-USD and cumulative accounted
Claude spend to 953,572 micro-USD, below the $1.30 ceiling. Codex used the isolated subscription
login and reported no API spend.

This is a valid, identity-complete native matched result for one learned-procedure moved-checkout
case with one repetition. It proves a portable incremental-value example and closes the measured
topology gap. It is descriptive and unweighted, not statistically powered, and does not prove that
Engram is broadly better than native memory or that the flagship goal is complete. Wrong-scope,
stale-prerequisite, no-result, compaction/resume, host-visible packet bounds, and repeated native
reliability remain outside this result.

## Supplementary host-boundary telemetry — 2026-08-12

A forward-only provider-free reporter parsed the immutable completed topology traces without
changing their acceptance or comparison. Input totals preserve each host's own accounting; ratios
are therefore matched only within a host. Engram result bytes are exact JSON-serialized MCP
`content` arrays returned to the host.

| Host | Treatment | Input tokens | Native input | Treatment/native | Engram result bytes | Largest result | Orient result |
| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: |
| Claude Code | Engram + native | 127,019 | 50,702 | 2.51× | 11,794 | 6,129 | 2,821 |
| Claude Code | Lean Engram | 100,300 | 50,702 | 1.98× | 11,691 | 6,077 | 2,801 |
| Codex | Engram + native | 223,440 | 68,893 | 3.24× | 11,878 | 6,164 | 2,813 |
| Codex | Lean Engram | 273,874 | 68,893 | 3.98× | 11,795 | 6,123 | 2,792 |

Every individual Engram response remained below 8,192 bytes, and orientation itself was about
2.8 KB. The three-call progressive interaction totaled about 11.7 KB. More importantly, both
Engram treatments substantially increased host input tokens; even the lean arm added 97.8% on
Claude and 297.5% on Codex versus native-only. The combined arm added 150.5% and 224.3%,
respectively. This shows that the successful outcome is not yet token-lean at the actual host
boundary and makes adapter/system-prompt/tool-schema overhead a measured optimization target.
The traces localize 8.89–9.07 KB of the Engram traffic to two `procedure_match` results: one
condition-discovery abstention and one applicable full-record result after inspecting
`tool.version`. A forward opt-in compact-projection prototype preserved executable procedure,
scope, evidence, verification/freshness, applicability, and abstention fields, but reduced a
real-shaped applicable raw response only from 3,963 to 3,419 bytes (544 bytes, 13.7%) and could not
shrink the procedure-empty condition-discovery response. That product/schema change was removed.
The stronger optimization target is eliminating duplicated resolution/call cycles and reducing
host adapter, tool-schema, and multi-turn replay overhead without weakening authoritative condition
inspection or safe abstention.

This is supplementary evidence from one case and one repetition, not a preregistered budget gate.
The frozen runner report predates per-lane provider timestamps, so Codex latency cannot be recovered
honestly. Forward runner reports now record the exact provider-process interval for both hosts;
Claude traces additionally retain their host-reported duration.
