# Native strict successor Stage C2B2a — completion-launcher boundary review

## Disposition

The exact completion-launcher boundary amendment below is accepted as a design-boundary repair:

```text
path    evals/native_memory_pilot_v1/NATIVE_STRICT_SUCCESSOR_STAGE_C2B2A_COMPLETION_LAUNCHER_BOUNDARY_FROZEN_2026-09-07.md
SHA-256 8bed5246e2969511002ebff76fdd898abbf4316f3036393714c8a89f822f83ab
lines   513
bytes   34249
```

Three independent exact-SHA reviews each verified that identity from byte zero and returned
`P0=0/P1=0`. Their final verdicts are external reviewer observations summarized here; neither the
amendment nor this record manufactures reviewer authority. This record is itself only an accepted
scanner trust anchor after independent review of its final exact SHA returns `P0=0/P1=0`.

No launcher source exists yet. No controller, provisioner, Cargo, compiler, build, provider, VM,
adapter, pilot daemon, pilot datastore, payload or runtime command ran during this design review.
Normal Engram context retrieval did run: the primary agent called repository-local
`procedure_match` and lean `orient` (trace `01a07c5a-0003-7892-a1ee-675ccdf92e43`), and at least one
independent reviewer called repository-local `procedure_match`. Those calls can persist ordinary
Engram telemetry and checkout refreshes, including a datastore checkout UPSERT. The checkout
refresh and explicit read-only status checks can invoke `git status --porcelain`, which may refresh
Git-index metadata. None changed the amendment bytes, launcher/support/payload artifacts or pilot
state and none satisfies an acceptance gate. The exact preflight-only static-test exception cannot
run until the amendment and this review are accepted, the launcher and combined implementation-
preflight candidate are constructed through the finite semantic projection, and the required non-
executing source-safety inspection is clean.

## Review-record ratchet

The prior exact review-record candidate is rejected:

```text
SHA-256 55ae37b36397ad12bac88b14035bcde5d5c0ad844109d1eb6cb0aeb638ac3d31
lines   112
bytes   7413
```

Its three independent exact-SHA reviews returned `P0=0/P1=0`, `P0=0/P1=0` and `P0=0/P1=1`.
The sole defect was its overbroad no-daemon/no-datastore claim, which omitted the ordinary Engram
context effects disclosed above. That record cannot serve as the scanner trust anchor.

## Controlling accepted inputs

The review revalidated these exact controlling artifacts:

```text
path    evals/native_memory_pilot_v1/NATIVE_STRICT_SUCCESSOR_STAGE_C2B2A_ENTROPY_BUILD_ID_REPAIR_FROZEN_2026-09-07.md
SHA-256 4a4dec13b99c88694625b52538b246323f69465741300c7cb9b8394cbda83e02
lines   2653
bytes   179776

path    evals/native_memory_pilot_v1/NATIVE_STRICT_SUCCESSOR_STAGE_C2B2A_DARWIN_METADATA_ADDENDUM_FROZEN_2026-09-07.md
SHA-256 34ded5a82bba22ea741b8e896b15779a90893d858ed2826522b7a2b5df7d27d6
lines   893
bytes   56876

path    evals/native_memory_pilot_v1/NATIVE_STRICT_SUCCESSOR_STAGE_C2B2A_DARWIN_METADATA_ADDENDUM_REVIEW_2026-09-07.md
SHA-256 58b5e5324803cf8989a3368d111278defc4c828c67d9794e75b7660688bff3f5
lines   187
bytes   11527
```

Everything those artifacts require remains authoritative except the amendment's exact, narrow
supersessions.

## Rejected-candidate ratchet

Every predecessor below remains rejected and cannot authorize implementation or execution:

| SHA-256 | Lines | Bytes | Independent P0/P1 outcomes | Union defect |
| --- | ---: | ---: | --- | --- |
| `983c2bc0da52e46caeb82cfe51e8de50a935e03debc83269cfe5ca913f2c66e4` | 196 | 12184 | `0/2`, `0/3`, `0/4` | Outer entry unsuperseded; mutual-hash loop; no legal static checker; no outer completion witness. |
| `c471e20422a52609bb5d21e3730d9569b3c8ef9e2767df3505cdbd6907056a27` | 366 | 24030 | `0/1`, `0/2`, `0/2` | Final non-runnable clause unsuperseded; static capture lacked status/bounds; claimed impossible cross-file same-function testing. |
| `c0df59c3e3353f196e0781fd2f69594c677aa2049512920b83ac83702700eec5` | 418 | 27785 | `0/1`, `0/2`, `0/2` | Pure-AST execution not expressly excepted; no exclusive validated-data dependence; production outer observer unbounded. |
| `70ee6d152168397f47de5cfffe58cf984cfd5c52809d2e3a85bb45c89382604b` | 471 | 31854 | `0/1`, `0/0`, `0/0` | Pre-execution inspection omitted module initialization before dispatcher reachability. |
| `47e930320a15bc0a8fd6222d3f2dfb98c2d88546c4d0297b368f7ce72eb6d76b` | 495 | 33316 | `0/1`, `0/1`, `0/1` | Bootstrap allowlist inspected but did not permit the sole `__main__` comparison and entry call needed to reach validation. |

The final `8bed5246...` bytes repair the last bootstrap defect by permitting exactly one reviewed
`__name__ == "__main__"` comparison and one direct frozen zero-argument entry call, requiring full
entry validation as that function's first reachable work, and allowing only direct `SystemExit`
propagation after it returns.

## Final independent findings

All three final reviewers independently found the following boundaries closed:

1. **Edit and entry authority.** Exactly one host-only source path and the two audit records are
   appended outside the payload. The payload's accepted 29-object and virtual 30-object sidecars
   remain unchanged. The top-level launcher and static-test entries, including the addendum's final
   non-runnable-support clause, are superseded only as expressly listed.
2. **Finite preflight authority.** The combined record has one launcher-source hash field. Its
   fixed-width zero projection produces the semantic digest embedded in the launcher; the final
   launcher hash then fills only that projected field. Stable launcher metadata is preserved,
   timestamps are not equality witnesses, source-dependent evidence is excluded, and final clean
   reviewer verdicts remain external.
3. **Bootstrap and early execution.** Complete module/import/native initialization is reviewed.
   Before validation only inert imports, immutable literals, function definitions, the one exact
   `__main__` comparison and direct entry call may execute. The static-test mode is the sole early
   support execution and cannot reach production adapters.
4. **Actual production-function testing.** Launcher-local vectors call loaded production pure
   functions. Exact allowlisted controller/provisioner `FunctionDef` ASTs and literal constants are
   purity-checked, compiled only into an isolated minimal namespace and exercised without importing
   or executing either module, mode, top level or adapter. Control-flow and backward def-use proofs
   require exclusive canonical-return dependence at every acceptance branch and effect adapter;
   raw aliases, rereads, reparses, bypasses and descriptor substitution are terminal.
5. **Static-test observation.** At least three final implementation reviewers must directly parent
   bounded static-test executions, drain through EOF, enforce the 120-second deadline and stream
   ceilings, observe normal zero exit and complete reaping, and compare the entire canonical output.
   Output alone or a second-hand transcript cannot pass.
6. **Production capture and cleanup.** The scanner, launcher and outer reviewer use nested exact
   900-, 1,100- and 1,200-second deadlines. The outer reviewer owns a fresh session, bounds both
   streams, enumerates process identities and must terminate, drain and prove the outer and scanner
   groups absent on every failure. A valid pair cannot pass without direct normal outer completion.
7. **Publication and recursion.** The scanner remains read-only and childless. Only validated fd-1
   bytes may become the create-new TSV/digest pair. The launcher cannot write its review record;
   completion review directly binds outer status; final reviewer observations are not appended to
   the files they review; and no recursive metadata audit follows.

## Remaining gates

This design acceptance does not accept implementation or authorize production execution. The next
legal steps are to construct the one launcher source and bounded controller/provisioner edits,
construct the finite combined implementation-preflight candidate, perform the exact source-safety
inspection and bounded static-test reviews, and obtain final exact-SHA P0=0/P1=0 implementation
reviews. Build/source identity, production completion audit, Linux runner and runtime qualification
remain separate later gates.

Repository `target/debug` was absent at review time. User-owned worktree changes remain preserved;
nothing was staged or committed.
