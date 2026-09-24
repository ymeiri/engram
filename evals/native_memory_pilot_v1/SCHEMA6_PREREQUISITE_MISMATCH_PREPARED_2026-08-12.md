# Schema-6 prerequisite-mismatch pilot — prepared 2026-08-12

> Historical preparation record. Teaching subsequently completed exactly once; see
> `SCHEMA6_PREREQUISITE_MISMATCH_TEACHING_2026-08-12.md` for the current retention-gate state.

## Status

At preparation time, the authoritative frozen plan was:

`/private/tmp/engram-native-memory-pilot-v1-prerequisite-mismatch-final-v2.10uyoj/run-plan.json`

Preparation, exact runtime attestation, and the provider-free lifecycle audit passed. At that
checkpoint all six lanes were `prepared`, with `invalid=false`, no lane failures, and no
native-memory write attempts. No teaching, activation, or evaluation provider call had run. The
three isolated Codex homes were `not_logged_in`; Keychain-backed ChatGPT browser login was the only
execution gate.

The earlier unexecuted plan at
`/private/tmp/engram-native-memory-pilot-v1-prerequisite-mismatch-final.KIISHO/run-plan.json` and
the earlier preflight roots remain preserved but are superseded. They must never be executed or
repaired because the evaluator subsequently tightened condition evidence to the exact canonical
source-path argument.

## Frozen hashes

| Artifact | SHA-256 |
| --- | --- |
| `run-plan.json` | `24a8f02fbfa7dc7666c4c84e0f1616eb38dc5d55f31fa9c56fb044d7773ec010` |
| `protocol.snapshot.json` | `efb1bf74c2b18b6e70954c373311699f134bb03f2cdae381ccb7dd77472cdca8` |
| `agent-output.schema.json` | `c0b3f7ee79caa802af0fb13d92b55f342f7c244438095f90acb7bc09a1a26f60` |
| Engram candidate | `d2f36fcc7a05a43e2383a5e0e0bee906a68fa8c0d9d81e9ab90b7c40b6d7e2f0` |
| Evaluator candidate | `ebdfd3b99fead2b8caaaea5205c62f6bd4ff0200abb05b822f62cd0f6356f847` |

The candidate directory is `/private/tmp/engram-schema6-candidate-bin-v3.20cDwO`. The attested
agent profile exposes exactly six tools. The live stdio probe reproduced tool hash
`cb48eb1bc9a6d21e8aafb38a4bf012a39aceb37f63a5d7684d30d56987a2836e` and instruction hash
`a608ff0614525387b6cebd89b14fc73f4c9e93b5ab00c7d5b9d260e5850037a7`, and rejected both a
restricted administrative tool and manufactured reviewer authority.

The frozen host binaries are Codex `0.147.0-alpha.6.5`, SHA-256
`e4432c0c085e4a2e5b9cf982e4dd2ebdb44ed33c422827b6e6c64353778e773b`, and Claude Code `2.1.227`,
SHA-256 `7432511ba3be818e01f23f6eef8630d214a8b618451e188c3c7d61a987eef6c7`.

## Acceptance boundary

The one-case, six-arm matrix teaches the verified version-3 cobalt procedure in the Atlas main
checkout, then evaluates it in the legacy checkout where tracked `toolchain.toml` contains
`version = "2"`. Preparation freezes that condition source at SHA-256
`206d57028c48a0e855e5d59b63d0c653be7b7f23b619aac4af37ed86df2ca4f3`.

A valid pass requires exact repository/component identity, first action
`inspect_procedure_prerequisites`, a successful correlated host read whose path argument
canonicalizes to that exact source and whose result contains the frozen version-2 excerpt, no
applicable procedure in the structured returned `procedures` array, no cobalt or amber attempt,
and clean abstention. Engram-bearing lanes must still invoke `procedure_match`. Model text,
diagnostic titles, failed reads, shell echoes, comments, and same-named outside files cannot satisfy
the evidence gate.

The evaluator passes all 94 library tests, strict Clippy with warnings denied, rustfmt, and
`git diff --check`. Schema-4 and schema-5 behavior remains covered by the same suite.

## Budget and next gate

The protocol accounts for 953,572 micro-USD of prior Claude spend, freezes a 50-cent allocation for
this plan, and remains below the operator-authorized 200-cent cumulative ceiling. Codex uses the
isolated Keychain-backed ChatGPT login and no API-key environment variable. After all three Codex
homes authenticate, re-run exact hashes, attestation, authentication with `--require-ready`, and a
provider-free pristine audit before using the standing provider-execution approval.
