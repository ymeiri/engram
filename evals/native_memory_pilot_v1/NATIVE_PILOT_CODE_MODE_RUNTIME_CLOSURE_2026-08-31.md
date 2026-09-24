# Native-pilot Codex code-mode runtime closure — 2026-08-31

## Observed failure

The first real bounded file-cache candidate authenticated all three isolated Codex homes, then its
single teaching invocation exposed a missing runtime dependency. Frozen Codex
`0.151.0-alpha.7.2` looked for `codex-code-mode-host` beside its own executable and failed closed
because only `codex` had been frozen. Codex nevertheless exited 0 after explaining the failure, so
the old generic JSONL audit labeled its trace passed. Lane 4 then stopped at trusted Engram
verification with zero procedure candidates. No Codex probe command or Engram write occurred.

The candidate at
`/Users/yuval.meiri/.engram/evals/native-memory-pilot/no-result-evidence-closure-forward-file-cache-bounded-20260831-01`
is retired. Preserve its plan, protected caches, traces, and stderr; never resume or repair it.

## Repair

For `chatgpt_file_cache` plans, preparation now requires a regular non-symlink sibling named
`codex-code-mode-host`. Because the companion does not implement `--version`, the evaluator runs
its `--help`, requires the exact `Usage: codex-code-mode-host` identity, ties its compatibility
label to the attested Codex version, and independently records its SHA-256. Every later plan
attestation re-runs that identity check and byte hash. Recovery metadata retains or relocates the
companion only when its version identity and bytes match. Older plans without file-cache auth remain
unchanged.

The runner also validates teaching evidence immediately after every provider call. Each teaching
trace must contain the frozen failed-command attempt when one is declared and one correlated
successful required command with the expected exit status and marker. An agent message explaining a
tool failure can no longer advance a lane.

Validation passed:

```text
142 evaluator tests; 0 failed
cargo clippy -p engram-eval --all-targets -- -D warnings
cargo fmt --all --check
cargo build -p engram-eval
repository target/debug absent
```

Exact implementation and binary hashes:

```text
ad21126afe56de6c82c497ac0b1d14acb27cebc30c097d80ae68f086b7af199e  engram-eval/src/pilot.rs
bef87d358a32054ecb1a5d9caf9677f6136944b031c5cb191bbe7200f6927ad4  engram-eval/src/native_pilot.rs
a4f6f989375365249a2705ddfc38e80a1cbb8c574dc9e639f31fcc85bcea8398  engram-eval/src/native_runner.rs
c992f8e1fcaafe28ee847fa2fd089feda036a6af9e399bec5de490b659b11b1f  engram-eval/src/native_audit.rs
e1d2ac600fd4e0a41efdeb3d3f03563eeab483fc8d026563767b33076eae5ae6  current repaired engram-eval
a6042937174f72112dbd2d554a4af36936422e0c5ac69e353dc68994458996e9  frozen codex
aa3c08e84921acf61c180cfaf4db6733607a37a82ecad6d43776b31dc011a4ef  frozen codex-code-mode-host
```

## Fresh runtime-complete successor

The fresh root is:

```text
/Users/yuval.meiri/.engram/evals/native-memory-pilot/no-result-evidence-closure-forward-file-cache-runtime-complete-bounded-20260831-01
```

Frozen protocol SHA-256:
`ffaaf5a91111bbe5fbddba12ba4735ca27e202ecd04bd580fe8c92660ea32f2a`.
Frozen run-plan SHA-256:
`9e864c2b3e29c42afbf66509824697b7c5a01d4798e9c7bb0c3c43cefebba5ac`.
The successor froze evaluator SHA-256
`3d260140e0e73ccade39b32eb12c03e29b365e6229488e33e5778bd50bde527d` before a final test-only
negative assertion rebuilt the current evaluator. Its production companion and teaching-evidence
gates are unchanged; the successor remains immutable and attests against its own frozen bytes.
The plan carries forward prior Claude spend as 5,333,380 microusd, including the two completed calls
from the retired candidate.

Provider-free attestation returned `verified=true` for Engram, Codex, the code-mode companion,
Claude Code, the evaluator, and the effective six-tool Engram runtime. Lifecycle audit returned
`invalid=false`, six `prepared` lanes, and zero failures. All teaching, activation, evaluation,
agent-output, and runner-report targets are absent. The three Codex lanes report `not_logged_in`
because the fresh successor deliberately contains zero `auth.json` files.

## Approved provisioning and teaching

The operator explicitly approved the fresh successor and three protected plaintext authentication
cache copies. The frozen confirmation-gated provisioner ran once. It created exactly three regular,
single-link, owner-only `auth.json` files: one in each isolated Codex home, all mode `0600`, owner
`yuval.meiri`, and 4,657 bytes. Credential contents and content digests were never printed,
inspected, or recorded. The frozen provider-free checker then returned `ready=true` with all three
Codex lanes `ready`.

Before teaching, the plan, protocol, five executable hashes, six-tool runtime contract, lifecycle
audit, cache metadata, disk reserve, absent provider outputs, untouched device candidate, and
retired-candidate trace hashes all passed. The exact 50-cent teaching invocation ran once. All six
provider calls exited 0; no lane was recovered or replayed. The three Claude calls reported 64,257,
41,388, and 41,905 micro-USD, respectively, for 147,550 micro-USD total. The runner report is
`runner-teaching.json`, SHA-256
`6fb011a853eba826748933237f6480915ee7434ab7ba66fb65b08c3c4616dd35`.

Teaching trace SHA-256 values are:

```text
ee18a2ec65528d207a1fd494a7036812d5b71cb4d3527375b366341853d7400b  lane 1
da0d918917b2ed6c9b8c76e4f10765896afb77aa3a9c0e04711b1ffcea43e33c  lane 2
e5d8616330c5a04af818cccdc98e6c57d318258c36454cc19e1a6701c7d6d2a0  lane 3
e2e471854248ac2a081f9bc7db6a3dd5409bb932369498f0a93b4d76a416e019  lane 4
f7f7035c28e15cb01afb2cf6d90a5ccc3b4aff55cec948ca85e24498a7440a91  lane 5
27c77bc82f225f4951c3e53353cfa2f3ab251deb120a768cf0e6ddfa1146704f  lane 6
```

The post-teaching audit is `invalid=false` with zero failures: three Claude lanes are
`ready_for_evaluation`, and three Codex lanes are `awaiting_activation`. Every required teaching
command-evidence check passed. All four Engram procedure-verification artifacts passed; all three
Engram stores were observed, all three Codex native-memory artifacts were observed, and neither
Claude native-memory treatment produced its optional auto-memory artifact. Re-attestation and the
three-lane authentication check remained pristine. No activation or evaluation output exists.

## Retention, activation, and evaluation

The last Codex teaching trace was modified at Unix millisecond `1788179470408`. The frozen one-hour
gate opened at `2026-08-31T16:31:10.408+03:00` (`2026-08-31T13:31:10.408Z`). A live local timer
waited until that exact boundary. The fallback heartbeat was then paused before activation and
remains paused.

The plan, protocol, five executable hashes, runtime contract, lifecycle state, three protected
cache files, disk reserve, and absent future-phase outputs all revalidated after the deadline.
Activation ran exactly once. All three Codex calls exited 0 with no recovery or replay. Its report
is `runner-activation.json`, SHA-256
`c2c7ae5c1c8693f6df02f6136cbb1e10a4086cb5c37fbdedffbb3f78614d9be7`.
Activation trace SHA-256 values are:

```text
065f7074c951f860e5502a3d4e5cbd3b229fe632e47001b18defa3b927b1a603  lane 2
82c0e835605a90ccc635a723fdd7b744d466eebc1f952c95000cddf7c9680c79  lane 4
122b72999ec1a9640fd59272ee923fc931dd221dcaf55d1c01da1be3378f3587  lane 6
```

The complete between-phase attestation was again pristine. Evaluation then ran exactly once across
all six lanes. Every provider process exited 0, and no trace was recovered or replayed. Its report
is `runner-evaluation.json`, SHA-256
`bd667b5e3103f51ac346807b8d5ff5bd934b8e909f0acb560caea60b09cd60b6`.
Evaluation trace SHA-256 values are:

```text
71e015ec15721ae1522a12553e3dc596f38cc69dd93ddf378a7b03d2692a1f01  lane 1
54f39df767208b28714b35594b70764277851db55c8b0513682c6cf404035fa8  lane 2
9268a28afd4a52c887ab523f6b62e637efb9a45a1c499e443d23067d361b669f  lane 3
9e4b6f4eaa7e04a8550498f79f22838429933dbf112b770bf31a0aa70a6ea1bd  lane 4
ece5c7a0b0d8b95077b16dc2c10e5f6f69e1f45aa5b535bf67454b014b728768  lane 5
6161ab7d9c62f92f053ae7cd0b4875b06f0d45ec430eec58eb92307a14cbe18e  lane 6
```

The frozen completion audit reports `complete=true`, `invalid=false`, six
`evaluation_complete` lanes, zero lifecycle failures, and `all_acceptance_passed=false`. Every lane
correctly abstained, attempted neither the Atlas nor Orbit command, returned no forbidden Atlas
procedure, revalidated its procedure boundary when required, and had zero repeated failures.
However, the frozen acceptance result is 0/6. Each lane records the same three failures: scoped
identity did not match, the trusted first action was not proven, and the current condition evidence
was not proven through a recognized host tool call.

The strict comparison reporter exits 1 with `incremental_value_signal=not_observed`,
`portable_incremental_value_observed=false`, and `all_resource_budgets_passed=false`. All Engram
packets stayed inside the preregistered 8 KiB per-call and 16 KiB per-lane bounds. The matched
resource deltas were:

| Host | Treatment | Token delta | Runner delta | Budget result |
| --- | --- | ---: | ---: | --- |
| Claude Code | lean Engram | +132,117 | +24,371 ms | token violation |
| Claude Code | Engram + native | +15,531 | -1,195 ms | within bounds |
| Codex | lean Engram | +51,536 | +20,292 ms | token violation |
| Codex | Engram + native | +100,994 | +66,090 ms | token and duration violations |

No treatment Pareto-dominates native memory because all task-outcome deltas are zero and every lane
fails the frozen acceptance contract. Claude evaluation cost was 144,502 micro-USD; teaching plus
evaluation cost for this successor was 292,052 micro-USD. Total accounted Claude spend is therefore
5,625,432 micro-USD, below the frozen 7,000,000 micro-USD ceiling.

## Trace diagnosis and forward-only audit repair

The valid negative result exposes both product behavior and two evaluator defects; it must not be
rewritten or re-audited with repaired bytes.

- Lanes 1, 4, and 6 performed only part of the required bounded closure. They inspected at most the
  repository remote and component README, then skipped the operation runbook. Lane 2 likewise did
  not read the runbook. Lane 5 relied only on native Atlas memory. These are genuine host-journey
  failures.
- Lane 3 did inspect the exact tracked `runbooks/deploy-worker.md` through Claude's `Read` tool and
  observed both `Orbit` and `ORBIT_ONLY_CANARY`. Its successful tool result omitted `is_error`, as
  real Claude `Read` results do. The frozen auditor required `is_error: false`, so its condition,
  evidence, and first-action failures are parser false negatives.
- For every Engram lane, the frozen scoped-identity audit accepts only `orient`'s initial project
  selection. In this deliberately fresh Orbit evaluation store, orientation correctly returned an
  unresolved project and instructed the host to close trusted local evidence. The audit then
  ignored preregistered, correlated project/component reads even though the schema-7 contract
  explicitly freezes those evidence sources. This makes the evidence-closure acceptance objective
  internally inconsistent.

The forward source now treats a correlated Claude result as successful unless it explicitly reports
`is_error: true`. For schema-7-or-newer abstention only, an unresolved Engram orientation may be
closed by exact structured identity plus correlated reads of the preregistered, Git-tracked project
and component evidence. A conflicting resolved Engram project cannot be overridden. Execution
cases and older schemas retain the prior behavior.

The matcher response and generated host guidance now also turn local evidence closure into three
mandatory evidence requirements: canonical Git identity, a tracked project/component read, and an
operation-runbook/config/source read. After `current_checkout_root` is known, independent identity
and bounded-discovery calls may be batched or parallelized, and hosts that support it should inspect
the exact tracked targets in one direct multi-target read. Each result must remain correlated to its
source. Checkout-relative paths resolve from `current_checkout_root`; answering after only one or
two requirements is prohibited, and a project value is permitted only when tracked evidence names
it unambiguously.

Forward validation passed without touching the completed candidate:

```text
143 evaluator tests; 0 failed
271 Engram-index tests passed; 1 model-download test ignored
cargo clippy -p engram-index -p engram-eval --all-targets -- -D warnings
cargo fmt --all --check
cargo build -p engram-eval
repository target/debug absent
```

The ordered interim implementation hashes were:

```text
f98f03fb856aa51675489e625c7c7e48ba3aaaeb8f6c572969483b80693bbf38  engram-index/src/harness.rs
580c54cf58274ae91267c1e04b924274c09f74366bcecb6e8da9db7a5f50a8b8  engram-index/src/memory.rs
09031fa52a55f8fa36a6fb0271980376458f7c636b7febf65f8fc0b92ba17fb2  engram-eval/src/native_audit.rs
aff8f61389cf2a4750d1cc65db2eb739b5103b7cd876918281e81933d54b6eed  /private/tmp/engram-forward-evidence-closure-target/debug/engram-eval
```

This ordered implementation remained provider-free. Trace-level resource analysis showed that a
sequential three-call route would preserve the main cost failure: the completed Claude lean lane
used ten model turns and 184,141 total tokens, while the four-turn Claude combined and native lanes
used 67,555 and 52,024. The adapter was therefore optimized forward before any new credentials or
provider outputs existed. The completed candidate remains immutable.

## Ordered evidence-closure successor

A first provider-free draft at
`/Users/yuval.meiri/.engram/evals/native-memory-pilot/no-result-ordered-evidence-closure-file-cache-bounded-20260831-01`
is superseded and must never run. It referenced the mutable external build paths directly. It has
zero authentication caches and zero provider outputs; no lane was repaired or replayed.

The runtime-complete ordered replacement was:

```text
/Users/yuval.meiri/.engram/evals/native-memory-pilot/no-result-ordered-evidence-closure-file-cache-runtime-complete-bounded-20260831-01
```

Its root, `bin`, and `run` directories are owner-only. Exact frozen hashes are:

```text
6a37b38435beb7a42b5ff52232fe0327bf441ebd87e147decf59d83b2ba72500  evals/native_memory_pilot_v1/protocol-schema-7-no-result-ordered-evidence-closure-forward-file-cache-bounded.json
d51afa820f6bba118641e88f8a753a69bf9d343970aeac3ab729f14dddc1d258  run/run-plan.json
11aaa2b4ff715fd94cfc8558088946a5f18f06514629151425da5b10d1956102  bin/engram
aff8f61389cf2a4750d1cc65db2eb739b5103b7cd876918281e81933d54b6eed  bin/engram-eval
a6042937174f72112dbd2d554a4af36936422e0c5ac69e353dc68994458996e9  bin/codex
aa3c08e84921acf61c180cfaf4db6733607a37a82ecad6d43776b31dc011a4ef  bin/codex-code-mode-host
5086b9b64d8bb842e1f599cdd3767ab08c6b2266e462fcc5686ae4b019cca8f7  bin/claude
```

Re-attestation returned `verified=true`, including the exact six-tool effective runtime and adapter
instruction hash. Lifecycle audit returns `invalid=false`, six `prepared` lanes, and zero failures.
There are zero teaching, activation, evaluation, agent-output, or procedure-verification artifacts.
There are also zero `auth.json` files, so the provider-free authentication check correctly returns
`ready=false` and all three Codex lanes `not_logged_in`.

The plan carries forward 5,625,432 micro-USD of accounted Claude spend, retains the exact 50-cent
allocation and 7-dollar ceiling, and preserves the same preregistered packet/token/duration bounds.
It was superseded before authentication or provider execution because the sequential route was not
resource-aligned. It must never be provisioned, run, repaired, or replayed.

## Batched evidence-closure successor

The authoritative runtime-complete successor is:

```text
/Users/yuval.meiri/.engram/evals/native-memory-pilot/no-result-batched-evidence-closure-file-cache-runtime-complete-bounded-20260831-01
```

Its exact frozen hashes are:

```text
c2730efb70c9738012f2c1bcbd4ea0c95ffac185c34c04291b75d798dc18a6a3  evals/native_memory_pilot_v1/protocol-schema-7-no-result-batched-evidence-closure-forward-file-cache-bounded.json
4deab07baf3bc9b114466ec06177e1f9f232ccb66dd2efaa14859b89759cc27b  run/run-plan.json
cee7cdf3fdaa68a32eb69a9a81ab871b6f00477b2fc6caa96bb7f96c1d20e5f2  bin/engram
37081797306ea3b2c424a1e9a826acb44ce0e0f8cc0b93d0f4035224333d82cd  bin/engram-eval
a6042937174f72112dbd2d554a4af36936422e0c5ac69e353dc68994458996e9  bin/codex
aa3c08e84921acf61c180cfaf4db6733607a37a82ecad6d43776b31dc011a4ef  bin/codex-code-mode-host
5086b9b64d8bb842e1f599cdd3767ab08c6b2266e462fcc5686ae4b019cca8f7  bin/claude
```

The frozen forward source hashes are:

```text
b34adf84c389fcc7a3fc8641929d3709237f2b26318878a3c2e1c8e8324c193e  engram-index/src/harness.rs
e90bfb582cad2d8e8943afae0de4b16e637b25348f3feb9277216a8a0926aee5  engram-index/src/memory.rs
a53305b607541ca817dcab43d2be1c19ee96f1976b9b00ab0ef1484448861a68  engram-eval/src/native_audit.rs
```

The generated external Cargo target was intentionally removed after explicit approval when it
grew to 36.5 GiB and Engram's disk-reserve gate shut down the global daemon. The frozen binaries
inside the candidate root remain intact and continue to match the hashes above.

Before execution, attestation returned `verified=true`; lifecycle audit returned `invalid=false`,
six `prepared` lanes, and zero failures; and no provider output existed. The user then authorized
exactly three additional protected plaintext Codex cache copies. The frozen confirmation-gated
provisioner created exactly three owner-only `0600` lane-local files and returned `ready=true` for
all three Codex lanes without printing or hashing credential contents.

Teaching ran exactly once from 2026-08-31T17:58:15+03:00 through
2026-08-31T18:06:36+03:00. All six provider processes exited 0; no lane was recovered, retried, or
replayed. All six teaching traces passed, all three Engram procedure verifications passed, the three
Codex lanes are `awaiting_activation`, and the three Claude lanes are `ready_for_evaluation`.
Teaching consumed 157,505 micro-USD of reported Claude cost, bringing accounted spend to 5,782,937
micro-USD under the 7,000,000 ceiling. Exact teaching trace hashes are:

```text
b834461abe3d824cadd2f66de7b14d10fe00c3fc93f4e5fdfe65c01afdb8e3cf  lane 1
10578221f1ce70dcb873f60e57b2ffa2394a7b6eb9ceed414520c24bfd143034  lane 2
48cd95b0cc456ec4692f20467892fe1888773ecddcd5c50b04e2081d61d1e988  lane 3
beccc52b94c564c903869794035ddd749ea42211c8e3257b7cf5d2314d4975dc  lane 4
43e654d890b287b531b14c54e448d2ff4fab4783c36f0d5228d59a26942e1b1a  lane 5
a6a1c80d7fd14beed4ec41a442cb1bf0ca3be2de217ebf5646246f8ab8afaf64  lane 6
```

The mandatory one-hour retention deadline was 2026-08-31T19:06:36+03:00. The one-shot automation
paused itself at the post-deadline wake. Its first gate correctly refused provider execution because
available disk was 141,266,534 bytes below Engram's reserve. After explicit approval, only
`/private/tmp/engram-forward-evidence-closure-target` was cleaned; 61,900 generated files and
36.5 GiB were removed. The protected caches and every pilot artifact were preserved. Free-space
reserve then became positive, the global daemon restarted writable, and the full frozen gate passed
again.

Activation ran exactly once from Unix milliseconds 1788194043038 through 1788194085008. All three
Codex lanes exited 0 with no recovery, replay, or accepted budget-boundary exit. Evaluation ran
exactly once from 1788194222902 through 1788194614166. All six lanes exited 0 with no recovery or
replay. The immutable runner hashes are:

```text
dae9daa2bbde1222c90270b9eb3b42f7d96e6ebe19d3d0524c2e77990800e3b0  run/runner-teaching.json
1c8d06b07670910dc0eec53df8f166aee98c243d507ca55729c6bd796456d46f  run/runner-activation.json
97e1cab9b9d553e9ec03dd593459f21c0e2467ddf89f08f915ee5237dca75428  run/runner-evaluation.json
```

The immutable post-teaching lane artifacts are:

```text
70af57ad54bd5574f29707e1fb7af8d3728ee5c2bcad0fc9bc3fe66c5b72d018  lane 2 activation trace
7ecfe2ae431b28359709d0046230c169310b797e64e82ccd21a896709c013527  lane 4 activation trace
7a12169607edf562a8ce5bd28e4ab00568d9877c1f4b39fb42c3a4bce74a026c  lane 6 activation trace
210443381c598411360a5439dd8a65b27241c97d5d777e80319ada416f5011c5  lane 1 evaluation trace
16bbca2d614c4407b1ac3e97dcfedf3e6831acc8b82cdc628c94f27112e6bd1b  lane 2 evaluation trace
d41c7cdfbedb31c85cf7e954edd97e280aae13e8f2e2e2c44bc6d9abdfb77ba8  lane 3 evaluation trace
8d5ef5c782c47ca19b8dd2fa78c5e322b9841bc8b7dc89f1923fe8863badc87a  lane 4 evaluation trace
606c85ffeb42815609729dcb76fb76a62bf3b4d6e61aaa7fb05f0478f0f2793c  lane 5 evaluation trace
31ed03e25b62f63e2987217daedeab2c68d445ebe19a944aa6af42ef6a1ab79d  lane 6 evaluation trace
f734550098f481265400e35b1c5023ef0c6043289f66dc9ef87906306d7040e4  lane 1 agent output
dad8d5e476e789044ef4f5d642d93699d7646d4eb6779f2138ed12056752fbb0  lane 2 agent output
721f531f98889be3102d09f9c2dd4f40b244505cd953c70d15fb37fa01b875f9  lane 3 agent output
65c6cdeee92b31b070b5731aaf4cfcb6278fa3c2eca961d0962b1fb05562abdf  lane 4 agent output
bc22c84ad2a186384e8338d1f2ef6ea0e10865f2ad143455603fea21a30111cf  lane 5 agent output
91eaa22496de2f4bfe18ee7021796ec2da2a51a384285eaea1bc626b29a62e8a  lane 6 agent output
```

The three protected `auth.json` files remain owner-only `0600`; their contents and hashes were
never inspected or recorded.

The completion audit is `complete=true`, `invalid=false`, with six valid evaluation outcomes, zero
lifecycle failures, and zero native-memory write attempts. The honest acceptance result is 0/6.
Every lane made the expected safe abstention, applied no wrong-scope procedure, attempted neither
the Atlas command nor the local Orbit runbook command, and had zero repeated failures. All six
failed scoped identity. Engram orientation correctly detected repository `orbit` and normalized
remote `github.com/acme/orbit`, but returned `selected_project=null` with confirmation required
because no project link was registered. The agents did not close that unresolved identity with the
exact preregistered structured value from local tracked evidence: lanes 1, 3, and 4 returned a null
project, while lane 6 returned case-mismatched `Orbit`. Native lane 2 returned `acme/orbit`; native
lane 5 omitted the remote and returned component `worker service`. Claude's two Engram treatments
did inspect the required current evidence and improved first-action/evidence metrics over Claude
native memory. The two Codex treatments did not improve those metrics.

All Engram packet limits passed: each result was below 8,192 bytes and each lane stayed below
16,384 bytes. All four matched treatment/native pairs exceeded the 50,000 incremental-token limit:
Claude lean +53,802, Claude combined +72,010, Codex lean +112,076, and Codex combined +158,136.
The Claude pairs stayed within the 30,000 ms duration limit (+28,776 and +25,640 ms); both Codex
pairs exceeded it (+69,274 and +82,211 ms). Reported Claude evaluation cost was 153,350 micro-USD;
teaching plus evaluation cost 310,855 micro-USD, bringing cumulative accounted spend to 5,936,287
micro-USD under the 7,000,000 ceiling. The strict report therefore returns
`incremental_value_signal=not_observed`, `portable_incremental_value_observed=false`, and
`all_resource_budgets_passed=false`. This one-case, one-repetition negative result proves safe
wrong-scope abstention under the strengthened batched route, but it does not prove identity closure
or portable incremental value.

## Provider-free deterministic identity boundary

The next slice did not replay, repair, or mutate the completed pilot. It replaced the failed
prompt-only identity reconstruction with one structured boundary returned by both `orient` and
`memory(action=procedure_match)`. The compact `identity` object now separates three concerns:

- `identity.repository` returns the correlated Engram repository and checkout record IDs,
  canonical name, credential-free normalized remote, exact checkout root, and last recorded HEAD;
- `identity.project` returns `authorized`, `requires_confirmation`, or `unavailable`, the selected
  project only when authorized, the exact resolution source and candidates, and supporting
  project-link IDs;
- `identity.components` returns the matched component name/path plus the Git-tracked manifest path
  and SHA-256 when live component evidence exists.

An unlinked `orbit` checkout therefore returns exact repository, remote, root, and tracked
`queue-worker` evidence while keeping the project name null with
`status=requires_confirmation`. Repository or directory basenames never become project
authorization. Existing flat `resolution` fields remain for compatibility, but the lean response
now includes the structured boundary that agents previously had to reconstruct from prose.

The generated Codex and Claude guidance no longer requires a three-read identity ritual after an
empty procedure result. It directs the host to trust the returned structured identity, perform at
most one bounded read-only operation-specific lookup from the returned checkout root, remain
inside that checkout, avoid candidate execution, and ask when project confirmation is required.
The core guidance constant is explicitly bounded below 5,000 bytes.

Provider-free validation used only
`/private/tmp/engram-deterministic-identity-closure-target`; repository `target/debug` remained
absent. Results were:

```text
cargo test -p engram-index
271 passed; 0 failed; 1 model-download test ignored; 1 doc test passed

cargo test -p engram-tests --test repo_tests
11 passed; 0 failed

cargo test -p engram-tests --test memory_tests
46 passed; 2 persistent-store tests refused setup on the disk-reserve gate

cargo clippy -p engram-index -p engram-mcp --lib -- -D warnings
passed

cargo fmt --all --check
passed
```

The two memory-suite failures were not assertion or product failures. Their RocksDB fixtures
required 19,893,251,686 available bytes, while only 16,012,677,120 remained during their setup.
The focused moved-checkout procedure test, all 271 runnable index tests, all 11 repository tests,
and the other 46 memory integration tests passed. After Clippy, the isolated target occupied
13,607,872 KiB and filesystem availability was 12,899,676 KiB. It was preserved pending explicit
cleanup authorization; no additional build or provider run was attempted. A provider-free Engram
handoff dry run then failed closed before any write because only 15,638,237,184 bytes were
available against the same 19,893,251,686-byte reserve. The repository evidence documents remain
the continuation source until the exact external target can be cleaned and the handoff stored.

Current source hashes for this slice are:

```text
9bd2da386cc8b13c8afb9e2b2357761666881916e6c54a7fbc03e629d2952ca4  engram-index/src/memory.rs
1b21ed0b850663d932a1a3b266ec4b9921fa9e07234b47619be8450ae99c2a30  engram-index/src/harness.rs
d9ec0b2fa85f3739201f39b3d6f8af2bf12aa2f1407e690f47bed5831f9ac581  engram-index/src/lib.rs
93842520f82e353f016ddfdb1a4ecb3615729001de112c8ad4c8e623edca096f  engram-mcp/src/tools.rs
34795b5bb8069388637af6f3c51c266c1a679de1d46408aae2c3f5481527f68c  engram-mcp/src/server.rs
4da9fb556001e29be3fcd0ce4b4a825fe43385e1d6065007b09811013c992e85  engram-tests/tests/repo_tests.rs
a787bfa928f6b909d6f241644079e0b827233e99c363acf0021b57b6e9b79dd6  engram-tests/tests/memory_tests.rs
```

This closes the smallest product-side identity boundary provider-free. It does not change the
immutable 0/6 pilot result and does not yet prove host behavior or incremental value. A future
successor must preregister the structured ambiguity as a correct outcome for an unlinked fixture,
consume `identity` directly in both host adapters, retain the 8,192-byte packet gate, and compare
turn/token overhead without replaying any completed lane. The exact forward-only acceptance and
resource design is recorded in
`STRUCTURED_IDENTITY_SUCCESSOR_DESIGN_2026-08-31.md`; at that point it was design-only and had no
provider output. The next section records its later provider-free implementation.

## Schema-12 evaluator closure and disk gate

That forward successor is now implemented provider-free as schema 12. Historical schemas retain
their prior parsing and acceptance behavior. The new case contract treats the unlinked Orbit
project as explicit structured ambiguity: repository and component identity must be exact, project
must remain null with `requires_confirmation`, and the host must abstain after one bounded
operation-specific read. The evaluator correlates the complete `identity` values returned by
`orient` and `procedure_match`, independently verifies the live Git root/remote and tracked
component bytes, and rejects output-only claims, mismatched identities, extra boundary calls, and
host identity rereads.

The preregistered protocol is
`protocol-schema-12-structured-identity-no-result-forward-file-cache-bounded.json`, SHA-256
`d3e1bfa99874c433310f1807f611481919a1b1bfca692462da1fba4bf889c4d7`. It carries forward exactly
5,936,287 micro-USD of reported Claude spend, retains a 50-cent allocation under the unchanged
7-dollar ceiling, uses one case/six arms/one repetition, and keeps the exact 8,192-byte per-call,
16,384-byte per-lane, 50,000-token, and 30,000-ms resource limits.

Provider-free verification passed all five schema-12 discriminating tests, all 148 evaluator tests,
strict Clippy, formatting, and debug builds of both frozen-binary inputs. Repository
`target/debug` remained absent. Exact source hashes and the discriminating rejection matrix are in
`STRUCTURED_IDENTITY_SUCCESSOR_DESIGN_2026-08-31.md`.

No successor was prepared or authenticated. Linking the CLI grew the exact external Cargo target
`/private/tmp/engram-structured-identity-evaluator-target` to 15,322,800 KiB and left
19,548,086,272 bytes available against the mandatory 19,893,251,686-byte Engram reserve. The gate
therefore closed by 345,165,414 bytes before preparation. No provider was called, no protected cache
was copied, and no completed candidate was changed or replayed. The external target is generated
and rebuildable but must remain until its exact deletion is explicitly authorized.

On 2026-09-01 the operator granted that exact authorization. Cargo removed 24,521 generated files
(15.3 GiB), and the installed daemon returned writable with matched installed/live identity. The
separately preserved evaluator and Engram binaries were then used without rebuilding to prepare the
self-contained schema-12 successor at
`/Users/yuval.meiri/.engram/evals/native-memory-pilot/structured-identity-no-result-forward-file-cache-bounded-20260901-01`.
Its run-plan SHA-256 is
`52c366344844421a58d1245881c686b9b5663e41a48b8f0c3800eaccd0f9feae`.

Fresh provider-free attestation returns `verified=true`, and lifecycle audit returns
`invalid=false`, six `prepared` lanes, and zero failures. Every provider-phase output is absent.
All three Codex lanes correctly report `not_logged_in` because no protected lane cache has been
created. No provider call, installation change, live adapter write, or completed-candidate replay
occurred. Exact frozen hashes, resource accounting, permissions, and the next authorization gate
are in `SCHEMA12_STRUCTURED_IDENTITY_PREPARED_2026-09-01.md`.
