# Schema-8 source-backed prerequisite pilot completed — 2026-08-18

The forward-only schema-8 pilot is frozen at:

`/private/tmp/engram-native-memory-pilot-v1-schema8-final.awlRFa/run-plan.json`

Run-plan SHA-256:

`0d833424e04fe2501a4c35ec97dd37387cbc2edbdc107149eb2594dfe27c41b0`

No provider phase ran during preparation. The plan retained an explicit execution gate, allocated
at most $0.50 of additional Claude runner spend, records 1,496,461 micro-USD of prior spend, and
stays below the existing $2.00 cumulative ceiling. The user's standing provider authorization can
satisfy the runner flag; it does not weaken any frozen hash, authentication, lifecycle, or outcome
gate.

## Case and acceptance boundary

The plan repeats the completed schema-6 prerequisite-mismatch topology across the same six native,
Engram, and combined arms. A verified procedure learned with `tool.version = 3` is evaluated in a
moved checkout whose tracked `toolchain.toml` contains version 2. The expected outcome is clean
abstention, zero learned or failed command attempts, and first action
`inspect_procedure_prerequisites`.

Schema 8 changes the evidence boundary rather than the expected task outcome:

- The procedure card maps `tool.version` to TOML path `toolchain.toml` and key path
  `tools.version`.
- Preparation verified the teaching scalar, independently classified the evaluation scalar as a
  mismatch, and froze source SHA-256
  `206d57028c48a0e855e5d59b63d0c653be7b7f23b619aac4af37ed86df2ca4f3`.
- Engram lanes must make exactly one completed, evaluation-cwd-scoped `procedure_match` call and
  prove its correlated result contains the exact five-field, value-redacted observation: condition
  key, three-field source descriptor, `mismatched` status, and source hash. An observed scalar or
  any extra observation field fails acceptance.
- Native-only lanes must instead directly read the exact tracked source and produce the frozen
  excerpt. Model text, uncorrelated tool output, failed reads, same-named outside files, and stale
  source bytes do not count.
- The structured abstention result must agree with the trusted observation. No result may apply the
  rejected context key or run cobalt or the known failed amber command.

The evaluator keeps schema-6 and schema-7 behavior unchanged. Its new correlation regressions cover
both Codex completed MCP calls and Claude `tool_use`/`tool_result` IDs, including missing results,
wrong cwd, source drift, disclosed values, and extra fields.

## Frozen identities

- Codex: `codex-cli 0.148.0-alpha.9`, SHA-256
  `6170ff5578170ee9b74ad92bfcff96e6186f41d02b60815a7c2b01ad424c754f`
- Claude Code: `2.1.233`, SHA-256
  `bc466b6cde63edafc773f471a1fb98787fabb31f52240c8616ce7e1f587b212d`
- Engram: `0.2.3`, SHA-256
  `745a6c564227d26d3365e45ae87424ee3ea95c193d736a258b72ba1fab0e30f6`
- Evaluator: `0.2.3`, SHA-256
  `120cb8318a12d761a6d107a5303dabdbc4516567c1cfe36c2b1da9458157910b`
- Effective agent profile: six MCP tools, tool-schema SHA-256
  `7633baf9d364b74908c92a3b65f308080fc4925aab3dcbd75a50dd6961f2f179`, instructions SHA-256
  `2d0493af502412c6264e8d6c74f104fc39dcfa65874c80e4f1149a4050e55581`

The effective runtime attestation passed, including restricted-tool and manufactured-reviewer
rejection. Generated adapter hashes are
`35767a1908f8e9c03ea03fc7bb3c47f4b575e819c9c18e8e364f2196690b933a` for Claude and
`07f297aa902ea54a0594dc17e293246817d5c69d79d50895bb91af18eeb5fb3a` for Codex.

## Execution and frozen outcome

Preparation, frozen-plan re-attestation, the prepared lifecycle audit, all 103 evaluator tests,
Clippy for all evaluator targets with warnings denied, formatting, and `git diff --check` pass. The
three fresh isolated Codex homes initially reported `not_logged_in`; this is expected because
Keychain entries are scoped by the isolated home. On 2026-08-18, all three completed ordinary
ChatGPT browser login and passed `check-native-memory-pilot-auth --require-ready`. Frozen binary,
runtime, and prepared-lifecycle gates passed immediately before provider execution.

Teaching then ran exactly once across all six lanes. Every provider process exited 0; no lane was
recovered, repaired, or replayed. Runner report
`/private/tmp/engram-native-memory-pilot-v1-schema8-final.awlRFa/runner-teaching.json` has SHA-256
`a5b45d664112f0786bde7b0c9abd2c73494dbce73c219e841ffd3bd1d2aad66f`. Claude teaching cost was
161,788 micro-USD, bringing cumulative accounted spend to 1,658,249 micro-USD under the frozen
2,000,000-micro-USD ceiling.

All six teaching traces pass standalone audit. Trusted procedure verification passes for Engram
lanes 1, 3, 4, and 6. Claude combined generated no optional native-memory file, while Claude
native-only generated one non-empty `MEMORY.md`; both are valid measured outcomes. Codex native and
combined generated fresh native-memory state. The audit reports `invalid=false`, zero failures,
and zero native-memory write attempts. Teaching trace SHA-256 values by lane are:

1. `f754abdf580d716d33ab9c1d6524cfc913513dce0c8d7a4fcb748e0d72c831f8`
2. `0dc0a6fdae529de7c5b63d43e6045ae727e14eaf08514f08e79f0f9773495636`
3. `fad5593d6b18f11923b995477924ae75fd96f6f29037b4cbd97ce49cdaab69d6`
4. `bebdb74de0222fe1eea52e38c7c74c1b72c8e5221d14c76427e9f0fde5a0a1f8`
5. `db061da590227f98436381b08ef42ab411d0d981e94a3231f96b937a9c752e36`
6. `952c901698cf345361073677665c179c961a67a394e3aa19c13f2b88f1b2224c`

The latest retention deadline passed at `2026-08-18T12:30:55+03:00`. The fallback automation was
paused before it could duplicate the manual continuation. Frozen hashes, attestation,
Keychain-backed auth, and lifecycle audit passed again. Activation then ran exactly once for Codex
lanes 2, 4, and 6, all exited 0, and none was recovered or replayed. Runner report
`runner-activation.json` has SHA-256
`bd9353ed661898dd4f05ac84264f03edb0241499f4bb1af3dd4e7091305c4a07`; activation trace hashes are
`d4838402d63492ad35789d8b7a4dff6c643309155080b3757544dd21185ae8d4`,
`31a69ace56fde4bab49ede2453036efdc71cb94b6b7bfac9f88a86c0999b935b`, and
`8d42a31c7b33b234e584dad567840f384563e8039ea3b9c3b5d059e742d53801`.

The post-activation audit reported every lane ready with no failure or native-memory write attempt.
Evaluation then ran exactly once across all six lanes. Every provider process exited 0; no lane was
recovered, repaired, or replayed. `runner-evaluation.json` has SHA-256
`6bbebac09626b9a22a61b7da302e8084698eaa102d66f8d04fefb2af88ac292c`. Evaluation trace SHA-256
values by lane are:

1. `57cffcbb43cf131899cabf830610b68240e1889bc4d6f3995fa863e39d544165`
2. `845d17d51ae3dadc3723be17456894cf76f38d7a347540db039bf8439e83d374`
3. `6ab974dee06eafe0be43c076dec3da1d2456dc1f1d351a81bf4766e35e128378`
4. `845dceef415a422c30604f188e642fc5686679dc61e92daff7136171b20a6ba3`
5. `3690294c5f257118ebb367e819ffb730f1193ceb620065b1098cd5b7975d627a`
6. `47cf31f46ae8aae71f29ea332b4924d91d705bc2ee47d4c0474ddf90cf6e111b`

Claude evaluation cost was 94,663 micro-USD. Combined with teaching, the schema-8 run cost 256,451
micro-USD and brought cumulative accounted Claude spend to 1,752,912 micro-USD, below the frozen
2,000,000-micro-USD ceiling.

The frozen completion audit is valid and complete but not all-passing. The frozen reporter's
deterministic stdout has SHA-256
`8bd34226d996cc9976e7dded90aa40a1ef6e39bb323105615b3b390ea42a1c91`; it reports
`portable_incremental_value_observed=false`, `incremental_value_signal=partial`, and the strict
portable-value command exits 1.

| Lane | Frozen acceptance | Exact result |
| --- | --- | --- |
| Claude combined | Fail | Oriented and abstained, but never called `procedure_match`; therefore it had no correlated current-condition evidence. |
| Codex native | Fail | Abstained safely, but did not prove the frozen identity and direct tracked-source inspection boundary. |
| Claude lean Engram | Fail in the frozen evaluator | Oriented, called cwd-scoped `procedure_match`, received the exact value-redacted mismatch observation, and abstained; the frozen parser rejected the successful result because Claude omitted the optional `is_error` field. |
| Codex combined | Pass | Revalidated the source-backed mismatch and abstained with the required trusted evidence. |
| Claude native | Fail | Read native memory, attempted its remembered cobalt command despite the mismatch, then abstained only after that command failed. |
| Codex lean Engram | Pass | Revalidated the source-backed mismatch and abstained with the required trusted evidence. |

Under the frozen scorer, Codex lean Engram and combined each Pareto-dominate Codex native memory;
neither Claude treatment passes, so the preregistered cross-host claim remains rejected.

## Post-run evaluator finding

Claude Code 2.1.233 emits a successful MCP `tool_result` without an `is_error` member. The frozen
schema-8 evaluator required `is_error == false`, even though the correlated lane-3 result contains
the exact expected five-field observation and source hash. This is a measurement-contract defect,
not a provider or Engram result failure.

A forward-only source fix accepts a missing success flag while continuing to reject explicit
`is_error: true`. The focused regression, all 103 evaluator tests, Clippy with warnings denied,
formatting, and `git diff --check` pass. The isolated corrected evaluator has SHA-256
`2be9232039bdc66b83edb7703424006e3aa124b857201b126c38ea8180079f75`. A post-hoc read-only audit
of the immutable traces makes Claude lean Engram pass and yields the descriptive signal
`engram_across_hosts`; it is diagnostic only and does not replace or reclassify the frozen result.
The next forward pilot must freeze the corrected parser before execution, then repeat this case and
the matched, stale, wrong-scope, missing-source, and no-result variants across both hosts.
