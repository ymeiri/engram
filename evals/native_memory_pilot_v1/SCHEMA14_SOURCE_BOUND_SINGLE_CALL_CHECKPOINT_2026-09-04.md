# Schema 14 source-bound single-call checkpoint — 2026-09-04

## Status

The forward-only schema-14 candidate completed teaching, the exact one-hour retention gate, all
three Codex activations, and all six evaluations on the second fresh six-arm successor. Every
provider process exited 0, no completed lane was recovered or replayed, and the frozen audit is
`complete=true`, `invalid=false`, with zero lifecycle failures. All six agents abstained safely,
applied no wrong-scope context, executed no Atlas or Orbit procedure, repeated no failed command,
and attempted no native-memory write.

The official frozen result passes the full acceptance contract in one lane: Codex with Engram plus
native memory. All preregistered resource budgets pass, and the reporter returns
`incremental_value_signal=combined_across_hosts` and `portable_incremental_value_observed=true`.
That is a narrow descriptive result from one case and one repetition, not a statistically powered
product claim and not completion of the flagship goal.

Authoritative plan:

```text
/Users/yuval.meiri/.engram/evals/native-memory-pilot/source-bound-single-call-forward-file-cache-bounded-20260904-02/run-plan.json
SHA-256 8b9005db4ef14668e9297e19e908028f3655eb9c1e83aed1355900e9cd19ff86
```

The untouched `-01` lanes remain provider-free and artifact-free but their plan is retired. A test
helper rebuilt its referenced shared external Engram path after preparation, changing the binary
hash from `c443c3bee6303744948b0ad149f0832975ec66a83c91d6653d79d0943d35f3d6` to
`b4aeaad2efc10657958461d1f6e5eb1201145968a6984456e7ecdda85b0e72b5`. Re-attestation correctly
failed before authentication or provider execution. No original byte-identical copy was present in
the preserved pilot and temporary roots. The plan and all of its lanes are preserved and must never
run. The `-02` plan instead references dedicated runtime copies outside any Cargo target directory,
so subsequent test builds cannot mutate its frozen executables.

Preregistered protocol:

```text
/Users/yuval.meiri/projects/engram/evals/native_memory_pilot_v1/protocol-schema-14-source-bound-single-call-forward-file-cache-bounded.json
SHA-256 d432597ff91a0500c6edd82b72d2620889c3a5f3cf307b5062f431ce14d2989b
```

## Forward contract

Schema 14 preserves the schema-13 wrong-repository-scope case, six matched arms, one repetition,
safe-abstention requirement, exact structured identity, packet limits, 50,000 incremental-token
limit, and 30,000-millisecond incremental-duration limit. It adds two hard requirements for Engram
treatments:

1. The only task-start Engram identity boundary is one direct
   `memory(action=procedure_match)` call with the exact current cwd. A preceding `orient`, a memory
   list, or another Engram boundary call fails identity and first-action acceptance.
2. `suggested_operation_evidence` must contain the expected checkout-relative provenance path,
   source digest, and the canonical absolute `resolved_path` under the exact frozen checkout. The
   trace must contain exactly one completed host read that uses that absolute path and produces the
   attested Orbit marker from the attested source.

The evaluator remains backward-compatible with schema 13. It counts only completed Codex command
events, so paired `item.started` and `item.completed` records represent one operation read. A
schema-13 route without `resolved_path` remains valid under its own frozen contract but fails
schema 14.

The 151-test evaluator suite includes positive Codex and Claude source-bound traces plus negative
coverage for the redundant orient call and a missing `resolved_path`. Strict Clippy and formatting
pass. A separate provider-free integration test now also crosses a real daemon-process and
persistent RocksDB boundary: it captures a current plan and handoff, fully stops the daemon,
restarts against the same state root, and confirms lean resume orientation returns and marks both
durable records as used. The complete 20-test multi-session suite and strict all-target Clippy pass.
This proves process-boundary persistence in Engram itself, not Codex/Claude native-host context
compaction or resume behavior. The relevant evaluator source hashes are:

```text
e89f1d697033f4467731a0e4db0da96096de634cea1610f018fce84224d2338f  engram-eval/src/native_pilot.rs
53b8cd216a858894d75534033d636319f6b2fdbd761d446eafcc216027c57d79  engram-eval/src/native_audit.rs
```

## Frozen runtime and budget

The prepared plan freezes:

```text
e137ac356f2e11f56e14cd1d51ccb5e225294952bec3aeb361eaeff6ebe91e85  engram 0.2.3
3bbc00d6d56ef44937356c2ebf3b6d9cf7255d6162d98ad16daeb0224d886cfb  engram-eval 0.2.3
99a6fdb0e0f9188e62dd6681c7ed3c5cc343358291a53ea21a9d4202e09cde9e  codex-cli 0.152.1
abd1b7015b1efb772183b34644ddb557455205c542612af1620ee9c27962ab0c  codex-code-mode-host
3c269f66801028823e24a63ced9fdd3988cb86cf85fccd9f03f87e463b9d3e3c  Claude Code 2.1.260
```

The six-tool agent MCP runtime attests with tool hash
`b704fac7da773e703dfca6bd3e9b0ffa6dcade6b259e7290315d969de3cec5b3` and instruction hash
`11ed9e9e10be3ff6086d8bdd3b3e3ddc1180a7247e974af5338854efce0e7f29`. Runtime verification,
restricted-tool rejection, and review-authority rejection all pass. The generated Claude adapter
hash is `f9df6b08306462547e1ca9bac42fbd85d5d6758578b3cc4e586b869a7d6386b5`; the Codex adapter hash is
`5c0cc08564baa1a8119e95802306615c3db241d8caa14e0f91f2a706cb143242`.

The plan accounts for 6,488,734 micro-USD of prior Claude spend, reserves 50 cents for this pilot,
and remains below the existing 700-cent authorized ceiling. User authorization already covers AI
provider execution and sufficient budget; the per-call turn boundary remains 8.3 cents.

## Authentication, teaching, and retention gate

The dedicated frozen runtime is
`/Users/yuval.meiri/.engram/evals/native-memory-pilot/runtimes/source-bound-single-call-forward-file-cache-bounded-20260904-02/bin`.
Frozen attestation returns `verified=true`. The provider-free lifecycle audit returns
`invalid=false` with zero failures. After explicit authorization, the guarded no-overwrite
provisioner created exactly three owner-only `0600`, single-link Codex authentication-cache copies;
all three isolated homes report exact ChatGPT readiness. Credential contents and digests were not
read or recorded.

Teaching ran exactly once from `1788537311046` through `1788537601650` Unix milliseconds. The six
teaching trace hashes in lane order are:

```text
fd57bf590e62541d1d6ba9b34662b28981ebc5b9b33b35a00b10762c4be72d21
e827ed92188c79d134c2faf50ad2451a0422d2c5f8e32f7319ec525a11a09543
307a0cb44120b83ff6e008a5d7a6276b63f2eeaf351bbd399561aaefc738d859
253850276945c8f1170144fab07f08821f9482789eb53b1f34cb52f60ee87eee
44369bbff556e8525418987639a77982cd2a5b718ea7758b21f535abeb904bfb
68e4b3afe80826dc23f559611912269426e508e7b5e84ef2d5acf2a1a3f9e74e
```

Engram verification hashes for lanes 1, 3, 4, and 6 are respectively
`9e6326669d0132155d60fbd833787279c3d63f7cd4d7e1f206d57125bc902540`,
`011e984676564418062d9910b12914229ca8e450e252010f99de585d6df12744`,
`fe82b686005a4cf4f251c3f04f9df0fc8519d7bf38424b62ba3275034892300c`, and
`bfdf8258d4f8cc14c2f6a1c0fdd035cc7c8f72ed709037ff062a14c902e84858`.
`runner-teaching.json` has SHA-256
`9df7b39b7c8dfd0474a30cbc7e219c450b160a43abccd63255bec0ac69298e5a`.
Reported Claude teaching cost is 136,098 micro-USD, bringing accounted spend to 6,624,832 micro-USD,
below the existing 7-dollar ceiling.

### Pre-retention trace inspection

Each teaching lane invoked the failing `amber` probe exactly once and the successful `cobalt`
probe exactly once; no lane repeated either command. The Orbit evaluation marker, its provenance
path, and the Orbit remote are absent from every teaching trace and every native-memory artifact,
so the teaching phase contains no observed future-evaluation leakage.

The Codex native-memory directories contain only the expected empty rollout scaffolds before
activation. The Claude native-memory arm recorded the successful `cobalt` procedure and explicitly
marked `amber` as failing; the two Claude Engram arms did not create optional native-memory files.
In the Codex combined arm, the preliminary Engram `procedure_match` returned the correct Atlas
identity but no procedure and abstained. It therefore did not apply context before teaching, after
which the candidate was added once and evaluator verification passed. Teaching usage and latency
remain descriptive execution evidence only; they are not substituted for the preregistered matched
evaluation resource and outcome gates.

The latest Codex teaching trace was written at `1788537599296` Unix milliseconds. Its exact
one-hour retention boundary was `2026-09-04T19:59:59.296+03:00`. The gate did not start provider
execution until after that boundary. Final re-attestation returned `verified=true`, including the
effective six-tool runtime and its restricted-tool and review-authority rejection checks. The
three existing Codex cache files remain regular, owner-only `0600`, single-link files; all three
isolated homes report exact ChatGPT readiness. Credential contents and digests were never read or
recorded. Repository `target/debug` remains absent and disk reserve is positive.

## Activation and evaluation evidence

Activation ran exactly once from `1788541474247` through `1788541511322` Unix milliseconds. The
runner record has SHA-256
`ba3f4718470726c001b4a46bf1331c47c7f6b8afb0a784938261c22db6a8f2ad`; the Codex activation trace
hashes in lane order 2, 4, and 6 are:

```text
3d0507e0a9022e26b0456b3ef5ace5f935ec0d81c0dc8c1136173a05d1f064f8
62394264407c831c7d0b1275c929c1961835b474658c97db4b3ab288e9edcb66
72c0012b08fdcbb2112088f8f38ecd60114651eb0879e429113eb749f2033b25
```

Evaluation ran exactly once from `1788541590362` through `1788541815660` Unix milliseconds. The
runner record has SHA-256
`b8e23698aa1752dc208440ac7fca30479bbaea15682d1eb77ba4edd5b213d4f3`. Evaluation trace hashes in
lane order are:

```text
bf34fec1f6a7baf1cb778b954515ce09cfaa80bb6b8cf47521feb8ac3bae4786
72bdfaa7dbba0f0f16abc2891b8a65eb1ca7a7367e6f435cca7a72d2e8a15570
ace862ca947505c10380014d645d00ea2f7090ee49874deb1789feb82e8f1e9e
c39e4908ffbba452d83fb2af5b68d2984e3382aea3c33bdcf26b0a2b69a82ac5
854f6ad0830a3f46e9f27e47ca50144489b23160179864455f77e5dadf16b8c7
a3bb5a736e9a69237e26d8becba4be04b347d951ce8e88cfeb62b324d3af783e
```

Claude evaluation cost was 114,771 micro-USD: 39,295, 38,031, and 37,445 micro-USD for lanes 1,
3, and 5. Cumulative accounted Claude spend is 6,739,603 micro-USD, 260,397 micro-USD below the
existing seven-dollar ceiling. No call used the accepted turn-boundary budget exit.

## Official outcomes and resource result

| Lane | Host and arm | Identity | First action | Source evidence | Full pass | Engram calls | Tokens | Runner ms |
| ---: | --- | :---: | :---: | :---: | :---: | ---: | ---: | ---: |
| 1 | Claude combined | yes | yes | no | no | 1 | 44,730 | 24,043 |
| 2 | Codex native | no | yes | no | no | 0 | 128,698 | 54,035 |
| 3 | Claude lean Engram | no | no | no | no | 2 | 38,726 | 27,403 |
| 4 | Codex combined | yes | yes | yes | yes | 1 | 64,109 | 43,850 |
| 5 | Claude native | no | no | no | no | 0 | 69,952 | 24,948 |
| 6 | Codex lean Engram | yes | yes | no | no | 1 | 102,460 | 46,865 |

All four matched treatment/native pairs satisfy the 8,192-byte per-call, 16,384-byte per-lane,
50,000 incremental-token, and 30,000-millisecond incremental-duration limits. Claude combined
improves identity and first action while using 25,222 fewer tokens and 905 fewer milliseconds than
Claude native. Codex combined adds one full pass, identity, and evidence while using 64,589 fewer
tokens and 10,185 fewer milliseconds. Codex lean improves identity while using 26,238 fewer tokens
and 7,170 fewer milliseconds. Claude lean uses 31,226 fewer tokens but adds 2,455 milliseconds and
does not improve a preregistered outcome metric, so it is not Pareto-dominant.

The official machine report is
`schema14_source_bound_single_call_report_2026-09-04.json`, SHA-256
`2cacdcb7431fac83dc289af8a5cc8244ec6d130477b01c79012b7e75805e6e62`.

## Trace diagnosis and forward-only parser correction

Claude combined made the required direct `procedure_match` call and received the exact resolved
Orbit source path plus an explicit instruction to read it, but finalized without reading it. Claude
lean first called `repo.detect`, violating the single-call boundary, then also ignored the returned
resolved path. This is a real Claude adapter-compliance gap.

Codex combined called `procedure_match`, read the exact absolute source once, observed the attested
`ORBIT_ONLY_CANARY`, cited the source, and passed. Codex lean performed the same correct read and
observed the same marker, but its shell command represented the quoted path as `\"...\"`. The frozen
auditor stripped ordinary quotes but not that escaped-quote form, producing an official false
negative even though its own operation-read count and route checks were correct.

A forward-only parser fix now normalizes that escaped quoted argument before canonicalization. The
focused regression, all 151 evaluator tests, strict Clippy, formatting, and diff checks pass. The
corrected source has SHA-256
`df3acb6169123266af53621a721ccd104eaa0c2eae1a5eb15ef307d125c00654`; the external corrected
evaluator has SHA-256
`79933b5a6f4ed8b86d809660732bf296b7a494e9dbfa54391dadaf930d98623a`. Its labeled post-hoc audit
changes only lane 6 to a full pass, yielding two of six passes without altering the frozen official
result. The post-hoc machine report is
`schema14_source_bound_single_call_posthoc_quoted_path_report_2026-09-04.json`, SHA-256
`8b00d6371be053c69baf5f56312daf1be7280c1d45469b45ed3f95c4f44fc26c`.

Both successful Codex reads also emitted an unrelated Dogbrew/Mosaic telemetry DNS failure before
the source output. The requested task did not invoke the network and the source read still
succeeded, but this host-environment noise is a limitation for future isolation claims.

## Verdict and next evidence

Schema 14 is the first complete valid native-host pilot in this sequence to show the
preregistered portable incremental-value signal while keeping every resource pair within budget.
It also proves the source-bound path works for Codex. It does not prove broad product reliability:
the sample is one wrong-scope case with one repetition, Claude did not complete the source read,
and the flagship still lacks complete native stale/expiry, real host compaction/resume,
correction/deletion propagation, and canary-secret persistence evidence.

The next forward-only work should make Claude's source-read step mechanically reliable, then repeat
the comparison across multiple cases and repetitions before changing the retrieval architecture.
The completed-result insight is `01a06d75-616b-7143-8c9c-7f69a5d1b612`; the current compact Engram
continuation handoff is `01a06d75-6177-7e61-b9ac-e8d3af06f3ae`.

## Forward-only required host-action diagnostic

The first source-only repair after this immutable result adds explicit machine-readable ordering
and authorization fields to `suggested_operation_evidence`. In one fresh isolated Claude Code
diagnostic, Claude called `procedure_match` once, immediately read the exact absolute returned
source path, observed `ORBIT_ONLY_CANARY`, and abstained without executing the unverified command.
No completed lane was replayed, and no live adapter, setting, or binary changed.

This is narrow diagnostic evidence, not an amendment to the official schema-14 outcome. The same
trace showed that Claude shortened and rewrote the candidate's required full exact
`procedure_match.query`. Code and evaluator inspection found that verbatim full-prompt copying was
neither mechanically enforceable nor part of the trusted safety contract: the query is lexical
retrieval text, while exact cwd and structured local scope enforce authorization. The forward
candidate now limits the query to a 512-character task-focused excerpt, preserves concrete
operation terms, rejects oversized input, and states that query text cannot authorize execution.
A second fresh Claude diagnostic used a 27-character worker-procedure query and again completed the
exact source read and safe abstention. Repeated multi-case cross-host validation remains forward
work. Exact
hashes, metrics, trace-derived observations, and limitations are recorded in
`CLAUDE_REQUIRED_HOST_ACTION_SMOKE_2026-09-04.md` and
the two `claude_*_smoke_2026-09-04.json` records.

## Forward repeated successor prepared

Schema 15 now freezes the host-action repair as a new three-repetition, six-arm comparison without
altering this result. It adds audited 512-character query bounds with required task terms plus the
three required/non-authorizing operation-evidence flags. A pre-execution audit rejected the first
provider-free preparation because an omitted false-valued flag could pass; the preserved `-02`
successor requires explicit booleans. All 152 evaluator tests and strict Clippy pass. The 18-lane
plan is attested, provider-free, `invalid=false`, and output-free; its nine Codex homes are
intentionally not authenticated yet. Exact hashes and the continuation boundary are in
`SCHEMA15_REQUIRED_HOST_ACTION_BOUNDED_QUERY_REPEATED_PREPARED_2026-09-04.md`.
