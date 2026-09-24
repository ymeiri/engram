# Native-memory pilot v1 boundary-repair smoke — 2026-08-10

## Verdict

The learned-procedure moved-checkout boundary is repaired for one fresh, isolated Engram lane on
each native host:

- Codex: passed with only the generated project `SessionStart` hook; no Engram skill or
  `AGENTS.engram.md` was present in the evaluation checkout.
- Claude Code: passed with only the generated project `SessionStart` hook and settings entry; no
  Engram command, skill, `AGENTS.engram.md`, appended system prompt, or native memory was present.

Both hosts oriented before substantive work, reported the unresolved work-project identity,
received only the missing `tool.version` key, resolved it from the fresh checkout, matched the
verified repository procedure, ran cobalt exactly once from the returned checkout root, and
observed `ATLAS_CONTEXT_PROBE_OK` with exit code 0. Neither host executed amber or redirected the
command to the stored teaching checkout.

This is a boundary-repair smoke result, not a replacement comparison and not a completion claim.
The immutable six-arm protocol-schema-4 result remains negative.

## Reproducible evidence

| Artifact | SHA-256 |
| --- | --- |
| Codex hook-only trace `/private/tmp/engram-boundary-repair-smoke.afgsvp/codex-r4/trace.jsonl` | `ff32f45bc91b6ae18ee05e63d8039886528bf81f1f658f604e547f80539f887a` |
| Claude hook-only trace `/private/tmp/engram-boundary-repair-smoke.afgsvp/claude-r7/trace.jsonl` | `733f522b2205c3c7b3a1352d192e57707898c36841be0c6d2928bfc6997f5de7` |
| Codex live `hooks/list` trace `/private/tmp/engram-boundary-repair-smoke.afgsvp/codex-r2/hooks-list-trusted-project.jsonl` | `d7b153e13030500b03a1263e18aa193a7cc83c661c47be77e15cffaf1bb2905a` |
| Historical protocol-schema-4 re-audit `/private/tmp/engram-boundary-repair-smoke.afgsvp/historical-v4-reaudit.json` | `b1a56bd2891cb5ee218c54d99fc81e611ccf30e6beaf58fccacc8bc468942a4e` |
| Installed `.codex/hooks.json` | `dd082ab1c256d8d9967cc18eede108a8d0e6166212eee6287c384cc0f61778dd` |
| Installed Codex startup hook | `3be449f195bce0c263df47309effc7465647e9046591f75bfbb1cbd2aeb6b6d9` |
| Installed `.claude/settings.json` | `0c9f58d95a7fc26fc1cc5341b5c571589d3aee5bc86012a23c878852af744b9f` |
| Installed Claude startup hook | `fc64392129f243669515bf4bf3d57165b9cda834734338345217b8a8b6263875` |

The compact machine-readable summary is
`evals/native_memory_pilot_v1/boundary-repair-smoke-2026-08-10.json`.

## Host outcomes

| Host | Deterministic boundary | First external action | Identity | Prerequisite retry | Procedure execution | Forbidden actions |
| --- | --- | --- | --- | --- | --- | --- |
| Codex | Project `SessionStart` hook was the only Engram adapter in the checkout. The live hook registry reported the project hook after explicit project trust and controlled hook-hash acceptance. | `orient` | Remote `https://github.com/acme/atlas`; project unresolved because `atlas` has no registered project link; component `services/worker`. | `procedure_match` returned only `tool.version` and the fresh `current_checkout_root`; Codex found `version = "3"` in `toolchain.toml` and retried. | One `./bin/context-probe --channel cobalt`; exit 0; `ATLAS_CONTEXT_PROBE_OK`. | Zero amber commands and zero procedure commands under the stored teaching checkout. |
| Claude Code | One project `SessionStart` command hook ran successfully. Auto-memory was disabled and no Engram command, skill, or appended prompt was installed. | `orient` after Claude's deferred-tool discovery | Remote `https://github.com/acme/atlas`; explicit unresolved-project ambiguity; component `services/worker`. | Force-local `procedure_match` returned only `tool.version` and the fresh root; Claude read `toolchain.toml`, supplied `3`, and retried. | One `./bin/context-probe --channel cobalt`; exit 0; `ATLAS_CONTEXT_PROBE_OK`. | Zero amber commands and zero procedure commands under the stored teaching checkout. |

Claude used 9 turns and `$0.05793195`. Codex used the user's approved provider-free ChatGPT
browser-login path.

## Product boundary changes proven by the smoke

1. Repository procedures match a moved checkout by normalized remote identity without inventing a
   work project. Project/task memory still fails closed when its boundary is unresolved.
2. Missing prerequisites return `required_condition_keys`, bounded `next_actions`, and
   `current_checkout_root`; expected values are not disclosed before local inspection.
3. Stored `scope.local_path` and receipt paths are provenance only. Execution is rooted in the
   currently detected checkout.
4. `procedure_match` is force-local at the MCP boundary. An agent cannot widen executable
   procedure lookup by requesting `related` or `global`.
5. Procedure responses carry the same structured repository/project resolution used by
   orientation, including explicit unresolved ambiguity.
6. Codex installs a native advisory `SessionStart` hook in addition to skills. Claude's generated
   hook is shell-safe for literal Markdown, resolves nested cwd values to the Git root, and the
   installer removes stale generated dispatch commands instead of registering duplicates.
7. New pilot plans use protocol schema 5, which derives remote identity, prerequisite evidence,
   command execution, exit outcome, and marker from trusted tool results. Schema 4 retains its
   exact historical semantics.

## Historical result preservation

The completed protocol-schema-4 recovery plan was re-audited after the forward-only evaluator
change. It remains `complete=true`, `invalid=false`, and `all_acceptance_passed=false`; every lane
remains failed. No completed lane, trace, teaching store, or report was repaired or replayed.

## Verification

The following gates pass on the repaired tree:

```bash
cargo fmt --all --check
cargo test --all-targets
cargo clippy --all-targets -- -D warnings
```

The full test run intentionally ignores only tests that require downloading the embedding model.

## What remains

- Freeze and run a protocol-schema-5 matched comparison; these two traces are treatment smokes,
  not native-memory control arms.
- Run fresh native wrong-scope, stale-prerequisite, no-result, correction/deletion, and
  compaction/resume cases on both hosts.
- Measure host-visible packet/token bounds and repeat the positive case enough times to estimate
  reliability rather than relying on one successful run per host.
- Codex project hooks require project trust and acceptance of the generated hook hash. Controlled
  automation may bypass hook-hash confirmation only after attesting the exact generated file;
  normal users should accept it in `/hooks`.
- Protocol-schema-5 still uses model output for project/component reporting when no trusted trace
  source supplies those fields. The acceptance-critical remote, context evidence, procedure
  command, exit code, and marker are trace-derived.

