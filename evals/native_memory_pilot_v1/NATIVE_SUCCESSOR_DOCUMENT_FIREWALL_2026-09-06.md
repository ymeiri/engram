# Native successor document firewall — accepted foundation

Date: 2026-09-06 (Asia/Jerusalem)

## Outcome

The provider-free native-document boundary and historical-runner firewall are independently
accepted. The accepted slice does not authorize provider execution and does not make the
historical native runner suitable for a successor pilot.

## Accepted behavior

- Native JSON documents are read through a 4 MiB bounded, stable, owner-controlled,
  single-link, `O_NOFOLLOW` loader.
- Duplicate object keys are rejected recursively before typed deserialization.
- A successor document is classified once from the exact outer envelope
  `{family, schema_version, payload}`. Malformed successor markers never fall back to a
  historical parser.
- Historical protocol, stale-protocol, and run-plan roots are checked against explicit key
  allowlists before typed deserialization.
- All production historical prepare, recovery, attest, authentication, provisioning, execution,
  audit, report, and stale-safety plan/protocol loads cross this boundary.
- The four execution/evaluation recovery routes load the caller's original, uncanonicalized
  source-plan path before digest, binary, referenced-output, or destination handling. They carry
  the accepted bytes' canonical identity and SHA-256 together and validate a separately bounded,
  stable digest sidecar without rereading the plan unbounded.

## Frozen evidence

- `engram-eval/src/native_document.rs`:
  `2d55bb175a1b26baf4e035b8ccc311651aa99bc446ceba77ef20e19db9c94fe8`
- `engram-eval/src/native_runner.rs`:
  `e59faf8301ccbf36a33cd23b7a4f6aba76e0a0e2acc6e7281cf6f1c4e76fb5b5`
- `engram-eval/src/lib.rs`:
  `f210b5a6bc9f79ea127a960d0736a6d9570233850b837297b0b4d6fce8217e7a`
- `engram-eval/src/native_pilot.rs`:
  `79db7c622fca2357e4cbb915ecc3533638f5fa21e893df98a17956da7dce67cc`
- `engram-eval/src/native_stale.rs`:
  `37ef2031c6e2207cac5b27eefe83f5f4c2ff0e5fd1fdabc76869499953d555e4`
- `engram-eval/src/native_audit.rs`:
  `7cd7f1455e06b3ae5b4e2b766f54613c5a0a5f7ce192a4adf05b9706440947a4`
- `engram-eval/src/native_report.rs`:
  `07b4702b1f4f2093c2765bdf38a458ef9f8cb1dd4199a68d5a74262f9ea0196c`

## Verification

- Full provider-free `engram-eval` library suite: 196 passed, 0 failed.
- Native-document firewall tests: 8 passed, 0 failed.
- Public recovery-route firewall test: 1 passed, 0 failed.
- Embedded recovery-validation firewall test: 1 passed, 0 failed.
- Native stale-preparation integration test: 1 passed, 0 failed.
- Strict Clippy: passed with warnings denied.
- Formatting and whitespace checks: passed.
- Repository `target/debug`: absent before and after verification; an external target was used.
- No provider, authentication-copy, VM, live-settings, staging, or commit action occurred.

## Deliberate boundary

This slice proves a parser and ordering firewall, not immutable execution authority against a
same-UID concurrent actor. Ordinary historical paths can still perform later pathname-based
digest or canonicalization work, and the historical runner still accepts serialized argv and
inherits ambient environment. A successor must never route through that runner. It requires a
strict sibling family, internally derived launches, execution-owned process authority, live
semantic evidence, no-replay receipts, and fresh VM authority where applicable.
