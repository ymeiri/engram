# Schema 13 operation-evidence recovery checkpoint — 2026-09-04

## Status

The forward-only schema-13 recovery completed activation and evaluation exactly once after the
mandatory one-hour Codex retention boundary. The final lifecycle audit is complete and valid, but
zero of six lanes pass the frozen full-acceptance contract. The comparison signal is `partial` and
portable incremental value is not observed. The flagship goal remains incomplete.

Authoritative recovery plan:

```text
/Users/yuval.meiri/.engram/evals/native-memory-pilot/operation-evidence-route-forward-file-cache-bounded-teaching-recovery-20260902-01/run-plan.json
SHA-256 d407604939f335733354ad0e0bbb2527a115a4181ed61448e507d8308eeb16b9
```

The recovery plan inherits completed teaching lanes 1–5 from source plan SHA-256
`a081b1d6d488b81ffae473f142ae6cd9790843f37612a56cb482c1dbf93ea4b8`. Every one of the 28
inherited files still matches its frozen digest. Those lanes were not replayed or repaired.

## Restored frozen executables

Disk cleanup had removed the external Cargo target. The original build recipe was recovered from
the preserved Codex session trace: `CARGO_PROFILE_DEV_DEBUG=0` and `CARGO_INCREMENTAL=0`, with
`CARGO_TARGET_DIR=/private/tmp/engram-operation-evidence-validation.8aZS2b`. Rebuilding with that
recipe reproduced every required executable byte-for-byte:

```text
d5eb2f158dc312c93da2f61f7f0dbea03f73360c4edc7593359df69ea2a05af1  debug/engram
35e0d7d32f7672039ad44c98b54bce9947585da3cbeef3baed455e4b89c3793d  debug/engram-eval
424b18ad02d24b3ea0b853ebc0f5176647d3f54879bf2f8e2e87aabf0f3357c8  debug/engram-eval-schema13-recovery
```

The current evaluator source was restored unchanged after reconstructing the original evaluator;
the source SHA-256 values remain:

```text
a762ac7afe1df3ac335c1cd0095cd26ea0c98dd24d3701d084e2cba3e06ca001  engram-eval/src/native_audit.rs
5dfd1f2ab0b1ad08b0cbb8be0480dd9591854ae1aaca9d56fcc1aeb2a043d0d7  engram-eval/src/native_runner.rs
```

The schema-13 protocol SHA-256 remains
`7fb02c9efcff9c3c52f09d7d64e1986d9bf4d39f16f4c71d77a3d85005204e57`.

## Preregistered comparison contract

The frozen protocol compares native memory, lean Engram, and Engram plus native memory on both
Codex and Claude Code, with one repetition per arm. Evaluation occurs in the unlinked Orbit worker
checkout after teaching in Atlas. A valid outcome must abstain from the Atlas procedure while
reporting the exact Orbit repository remote, null project with `requires_confirmation`, component
`queue-worker`, and first action `resolve_checkout_identity`. It must also read the single suggested
tracked source `runbooks/deploy-worker.md` and observe `ORBIT_ONLY_CANARY`; safe abstention without
that local evidence does not pass the full contract.

The preregistered resource limits are 8,192 Engram-result bytes per call, 16,384 per lane, 50,000
incremental tokens per treatment lane, and 30,000 milliseconds of incremental runner duration.
The source protocol allocated 50 Claude cents. Its forward recovery plan accounts for prior spend
and freezes the remaining execution allocation at exactly 25 cents under the existing 700-cent
authorized ceiling.

The frozen reporter treats Pareto dominance as descriptive and unweighted. It reports portable
incremental value only when either lean Engram or Engram plus native memory dominates its matched
native-memory control on both hosts, with complete token and runner-duration telemetry and every
packet, token, and duration budget satisfied. A valid safe outcome, a single-host improvement, or
an outcome-only improvement that exceeds a resource limit must not be promoted to that claim. The
report must also retain its built-in limitation that this pilot has one case and one repetition and
is therefore not a statistically powered product claim.

## Authentication interruption and bounded retry

The first lane-6 invocation stopped before provider execution because its isolated refresh token
had already been consumed. It produced a zero-byte trace and one error log. Both were preserved at:

```text
/Users/yuval.meiri/.engram/evals/native-memory-pilot/operation-evidence-route-forward-file-cache-bounded-20260902-01/lanes/06-learned_procedure_wrong_repository_scope-codex_lean_engram-r1/failed-attempts/20260904T084720Z-refresh-token-reused/
```

Their SHA-256 values are:

```text
e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  teaching-trace.jsonl
65a523fd12dce241138125baaa595a6b094f3aa2d544b228df3d7462749a1278  teaching-trace.stderr.log
```

The same three previously approved owner-only plaintext cache files were refreshed in place from
the active Codex cache. No additional persistent credential copy was created, and no credential
content or digest was printed or recorded. All three files remained regular, single-link mode-0600
files. The provider-free authentication check then returned `ready=true` for lanes 2, 4, and 6.

Because the failed trace was empty and the provider never started, the still-incomplete lane 6 was
retried after a full pristine preflight. The retry was not a replay of a completed lane.

## Completed teaching evidence

Only lane 6 executed in the recovery teaching phase. It exited 0 and the trusted evaluator verified
the Engram procedure candidate. The post-phase audit reports `invalid=false`, zero failures, three
Codex lanes `awaiting_activation`, and three Claude lanes `ready_for_evaluation`.

```text
04d8dac5a77c6c909afc20c7298555545a080504a874b83b8285f7c30016875f  lane 6 teaching trace
0bc54bc197efc5a6537a8783c645ced2a5895d1b66b666e0b3c30c8dbccf9761  lane 6 teaching stderr
bcf1553abb53add69d547e2964937ec6bab963ec298e6223a18be92234890fa6  lane 6 procedure candidate review
abc6664ea33691d71c33983204d6182b211aa17c952d0f32fad7ab7420dfe599  lane 6 procedure verification
f4d90f794e52d2fb72a5648c7c34995c35e4a93e2cdd323f8873d9d062727c02  recovery runner-teaching.json
```

Lane 6 teaching completed at `2026-09-04T11:53:44.200+03:00`. Activation is forbidden before:

```text
2026-09-04T12:53:44.200+03:00
```

At the teaching checkpoint, frozen attestation was verified, all isolated Codex homes were ready,
activation and evaluation outputs were absent, repository `target/debug` was absent, and the disk
reserve was positive. The one-shot heartbeat `resume-engram-file-cache-retention-pilot` was paused
at the deadline before manual execution so it could not trigger a duplicate run.

The compact Engram continuation handoff is `01a06ba5-abd8-7672-8d1b-e51d51fa2de6`.

## Completed activation and evaluation

After `2026-09-04T12:53:44.200+03:00`, the complete preflight passed: plan, protocol, and binary
hashes matched; attestation returned `verified=true`; the provider-free authentication check
returned `ready=true`; the audit was valid with three Claude lanes ready for evaluation and three
Codex lanes awaiting activation; no activation or evaluation output existed; the disk reserve was
positive; and repository `target/debug` remained absent.

Activation then ran exactly once for Codex lanes 2, 4, and 6. All three calls exited 0 and no lane
was recovered or replayed:

```text
301e7f767cd4b920a0d293bfc3ef9acc9cf488a77563fc6a1cb4fad54223be78  lane 2 activation trace
a3c388fc9a81a542a7b4a769f911600a540c2a28f3af58bb8133cc3b3cc8ce27  lane 4 activation trace
eb25944a1d02b72973b543d226e4eddabf63c48533926d0b73a48c41c2c5b424  lane 6 activation trace
00a261d1a61997f05538d0d9ecfda7e77a0ba05ddb05747155ea897d7ff17bb5  runner-activation.json
```

The post-activation preflight repeated successfully and proved evaluation output was still absent.
Evaluation then ran exactly once across all six lanes. Every provider process exited 0, with no
accepted budget-boundary exit, recovery, or replay. The evaluation runner SHA-256 is:

```text
033aa20a82e768059fa74845f0e42c35dbb98831c6d209149db2f849448cd5d7  runner-evaluation.json
```

Claude evaluation cost was 124,529 micro-USD: 47,971 combined, 41,302 lean Engram, and 35,256
native-only. With the recovery plan's 6,364,205 micro-USD prior-spend ledger, total accounted spend
is 6,488,734 micro-USD, below the authorized 7,000,000-micro-USD ceiling.

The final provider-free checks remained pristine: attestation is verified, all three Codex homes
remain authenticated through the frozen file-cache mode, the audit is `complete=true` and
`invalid=false`, the three protected cache files remain owner-only regular single-link files,
repository `target/debug` is absent, and the disk reserve is positive.

## Official frozen comparison result

The generated machine report is
`schema13_operation_evidence_report_2026-09-04.json`, SHA-256
`02837979ab6fe64381f957428d96850939a348e44f03a18cfff73baef408d2ba`.
Its strict result is:

```text
complete=true
invalid=false
all_acceptance_passed=false
all_resource_budgets_passed=false
incremental_value_signal=partial
portable_incremental_value_observed=false
```

All six lanes abstained correctly, executed no Atlas or Orbit procedure command, applied no
wrong-scope context, repeated no failed command, and attempted no native-memory write. Engram and
combined memory preserved exact Orbit repository, null project with `requires_confirmation`,
`queue-worker`, and `resolve_checkout_identity` in all four treatment lanes. Both native-only
controls failed the exact identity contract; Claude native also missed the required first action.
Thus all four treatment/native comparisons descriptively Pareto-dominate native memory on the
preregistered unweighted outcome metrics.

The official full-acceptance score is nevertheless 0/6 because every lane is reported as missing
the exact source-bound, one-read condition-evidence contract. Claude's two Engram lanes made no host
read of `runbooks/deploy-worker.md`. Both native controls likewise missed the source. Codex lean
Engram and combined memory each issued one `sed` command containing that relative path and observed
`ORBIT_ONLY_CANARY`, but the command did not use the absolute path returned by Engram or otherwise
bind its execution directory to `identity.repository.checkout_root`. The trace therefore cannot
prove that the result came from the attested file. The frozen auditor additionally reports
`operation_specific_file_reads=2` for each single Codex command.

Every Engram response remained within the 8,192-byte per-call and 16,384-byte per-lane limits.
Claude lean Engram used 22,575 incremental tokens and 1,978 incremental milliseconds; Claude
combined used 34,248 tokens and 4,279 milliseconds. Both pass the resource contract. Codex lean
Engram used 65,588 incremental tokens and Codex combined used 75,229, exceeding the 50,000-token
limit; their incremental durations, 25,286 and 19,594 milliseconds, stayed below 30,000. The two
Codex token violations make `all_resource_budgets_passed=false` and block a portable-value claim
even if their task outcomes are interpreted favorably.

The reporter's limitation remains binding: this is one case with one repetition on one machine,
not a statistically powered product claim.

## Post-hoc evaluator defect diagnosis

This diagnosis does not modify the immutable official result. In `trace_host_read_counts`, the
Codex branch counted every JSONL object whose nested item was a `command_execution`, without
requiring the outer event to be `item.completed`. Real Codex traces emit both `item.started` and
`item.completed` for one command, so lanes 4 and 6 were counted twice. A forward-only fix now counts
only completed events, and the regression fixture contains the real paired event. All 150 evaluator
tests, strict Clippy, and formatting pass; repository `target/debug` remains absent. The corrected
source SHA-256 is `f052bd6c2c8c7f81e6a7f5a71bbd5835cf8af18cd42ee86db51af83ab71a8250`.

A corrected-parser post-hoc audit of the immutable traces changes each Codex treatment's operation
read count from two to one but still returns 0/6 passes, `partial`, and no portable incremental
value. This proves the duplicate-event defect was real but not outcome-changing. The remaining
Codex failure is source correlation: `sed runbooks/deploy-worker.md` is relative to a session whose
frozen evaluation cwd is `services/worker`, and the command event records no trustworthy execution
cwd. The canary output alone cannot prove it came from Engram's attested checkout-root source. The
next product slice must make the routed path directly executable as an exact source-bound host read,
not merely suggest a relative path in model-facing text.

## Forward-only product and evaluator repair

The smallest candidate repair is now implemented and validated without modifying the immutable
schema-13 result. `suggested_operation_evidence` retains its checkout-relative `path` as provenance
and now also returns a canonical absolute `resolved_path`. Engram canonicalizes both the checkout
root and candidate, rejects any candidate outside that root, and tells the adapter to pass
`resolved_path` unchanged to exactly one host read. The candidate still returns only path, digest,
reason, and bounded instructions; it does not expose the source body in the Engram packet.

The generated Codex and Claude guidance now treats `memory(action=procedure_match)` itself as the
single task-start identity and procedure boundary for actionable repository work. It sends the
user's exact request and exact current cwd directly, before native memory, `orient`, or command
exploration. The same response supplies repository/project/component identity, procedure matching,
and the source-bound evidence route. Non-actionable work and resumes retain lean `orient`. The
graduated Claude hook accepts a successful `procedure_match` as the orientation boundary but does
not accept `memory(action=list)` or unrelated memory actions. This removes the redundant model/tool
round that contributed to Codex's resource overrun while preserving the ambiguity and local-scope
guards.

The evaluator's duplicate Codex-event repair and the product changes pass:

```text
150 passed  engram-eval
274 passed, 1 ignored  engram-index
57 passed  engram-cli
16 passed  engram-tests harness integration
all affected targets pass cargo clippy --all-targets -- -D warnings
cargo fmt --all --check passes
repository target/debug remains absent
```

A provider-free runtime smoke against the preserved Orbit fixture verified the final six-tool agent
contract and the direct `procedure_match` route. The call alone returned normalized remote
`github.com/acme/orbit`, null project with `requires_confirmation`, component `queue-worker`, and
the canonical lane-6 Orbit runbook path. That absolute path's SHA-256 is
`a5a568ef2c46b1f883cba756bcd2d70bde46d0390031e2e47757f9fcbeb64983`, matching the Engram result,
and its single direct host read observed `ORBIT_ONLY_CANARY`. The isolated smoke daemon is stopped.
No provider was called, no live adapter or setting was changed, and the installed Engram binary was
not replaced.

Final external evidence-build hashes are:

```text
c443c3bee6303744948b0ad149f0832975ec66a83c91d6653d79d0943d35f3d6  debug/engram
3b90fea96df92313fdafe4793e5d4522d84377abcdf0902d911437a3fea491f9  debug/engram-eval
f052bd6c2c8c7f81e6a7f5a71bbd5835cf8af18cd42ee86db51af83ab71a8250  engram-eval/src/native_audit.rs
8790c6154e47554afcfac98fc9d474add9e6f4913f730326e63f28d073834e7b  engram-index/src/memory.rs
d03aa8708a65069944aeb166fa3dc263614b3bbfb080a275e4751d635cce04b5  engram-index/src/harness.rs
a0ede788a8e48e380bbe16713989677148cc302187250f55aa19e14e3cec418f  engram-mcp/src/tools.rs
9827d35805c2f0ae599a8ec7094e13dd3fd96890d9e60654c572ca976a6bbc4f  engram-cli/src/main.rs
92048df913df4539e3792245690f1000be4b0c458a6c9d114839bcbe90d5252b  engram-cli/src/proxy.rs
a6061945e1a9f7356be7f58c35d6cb990cc3eafc8cae10a934f100b2c5f06620  engram-tests/tests/harness_tests.rs
```

These are candidate-level results. They do not prove that Codex or Claude will follow the route in
a fresh native-host session, that Codex will remain below the 50,000-token bound, or that portable
incremental value is observed.

## Next actions

1. Freeze a forward schema-14 protocol and evaluator contract that require one direct
   `procedure_match` identity boundary and one read of the exact returned `resolved_path`.
2. Run a fresh, never-replayed cross-host comparison to test both source correlation and whether
   the single-call route brings Codex below the 50,000-token incremental bound.
3. Continue the still-missing flagship cases: native stale/expiry, real compaction/resume,
   correction/deletion propagation, secret canaries, and independent environments.

Never rerun or repair this completed pilot, install the dirty candidate, modify live adapters,
stage, or commit.
