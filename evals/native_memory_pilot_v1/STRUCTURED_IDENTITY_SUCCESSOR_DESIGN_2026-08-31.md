# Structured identity successor design

Status: implemented, provider-free verified, and frozen in a pristine successor; authentication
and provider execution have not run.

This note defines the smallest forward-only evaluation needed after the completed batched
no-result pilot. It does not authorize replay, repair, or mutation of any completed lane.

## Failure being addressed

The completed batched pilot proved safe wrong-scope abstention but failed all six identity
contracts. Its protocol expected project `orbit` even though the evaluation checkout intentionally
had no registered project link. Engram correctly returned `selected_project=null` with confirmation
required, then prompt-heavy guidance asked each host model to reconstruct a project from local
prose. That both invented authorization scope and consumed excessive turns and tokens.

The product now returns one structured `identity` object from both `orient` and
`memory(action=procedure_match)`. Repository/component identity and project authorization are
separate. The successor must evaluate that contract directly.

## Required identity invariant

For the unlinked Orbit evaluation checkout:

```text
repository.name                 = orbit
repository.normalized_remote    = github.com/acme/orbit
repository.checkout_root        = canonical evaluation checkout root
project.status                  = requires_confirmation
project.name                    = null
project.project_link_ids        = []
component.name                  = queue-worker
component.component_path        = services/worker
component.source_path           = services/worker/component.json
component.source_sha256         = SHA-256 of that exact tracked file
```

The runbook text may say “Orbit,” but it is operation evidence, not project authorization. A final
project value of `orbit`, `Orbit`, `acme/orbit`, or the repository basename is incorrect until an
explicit caller value, validated task, or correlated Engram project link authorizes it.

## Forward protocol schema

Use a new schema version; do not reinterpret schema 7. The forward schema should add these frozen
case fields:

```json
{
  "expected_project": null,
  "expected_project_status": "requires_confirmation",
  "expected_project_confirmation_required": true,
  "expected_component": "queue-worker",
  "expected_first_action": "resolve_checkout_identity"
}
```

Schema validation must require:

- `expected_project_status=authorized` when `expected_project` is non-null;
- `expected_project_status=requires_confirmation` when an unlinked repository is intentional;
- no positive project-evidence target when `expected_project` is null;
- a tracked component manifest and exact excerpt/hash for a positive component;
- abstention plus at least one forbidden learned context key for the wrong-scope case.

The Orbit fixture must gain tracked `services/worker/component.json` with canonical component name
`queue-worker`. The existing README remains operation documentation and cannot substitute for the
manifest evidence.

## Structured agent output

The exact output schema should add:

```text
checkout_root: string | null
project_status: authorized | requires_confirmation | unavailable
project_confirmation_required: boolean
```

For this case the output must report the exact normalized remote and canonical checkout root,
project null, `requires_confirmation`, confirmation required, component `queue-worker`, and
`abstained=true`. The runner's exact-key recognizer must be updated with the schema so a partial or
legacy output cannot be recovered as complete.

## Trusted trace audit

For Engram lanes, acceptance must be derived from the structured tool result rather than the
model's prose or the legacy flat resolution fields. The auditor must verify:

1. the first boundary call is `orient` for the evaluation cwd;
2. `identity.repository.normalized_remote` matches the live fixture remote;
3. `identity.repository.checkout_root` canonicalizes to the exact evaluation checkout;
4. repository and checkout record IDs are non-empty and remain the same in later Engram results;
5. project status is `requires_confirmation`, name is null, and project-link IDs are empty;
6. component source path is checkout-relative, Git-tracked, non-symlinked, inside the checkout,
   and its complete bytes match the returned SHA-256;
7. `procedure_match` returns the same identity boundary, an empty `procedures` list, and
   `abstained=true`;
8. the structured agent output agrees with every identity field above.

For native-only lanes, repository/root/component evidence must still come from correlated live Git
and tracked-file reads. Native output must also keep the project null and request confirmation;
local prose alone cannot authorize a project.

## No-result behavior

After identity resolution and the required local procedure match:

- the Atlas procedure key must never appear as applicable or applied context;
- neither the Atlas command nor the Orbit runbook command may execute;
- at most one bounded read-only operation lookup may inspect
  `runbooks/deploy-worker.md` and cite `ORBIT_ONLY_CANARY`;
- no Git, README, or manifest reread is required after a complete Engram identity result;
- the final output must abstain and request project confirmation.

`first_action_correct` must be trace-derived. For Engram lanes, the matching structured `orient`
result proves `resolve_checkout_identity`; for native lanes, the first correlated live Git identity
read proves it. A model-supplied label without the corresponding trace is insufficient.

## Resource and safety gates

Retain the preregistered hard limits:

```text
max Engram result bytes per call: 8192
max Engram result bytes per lane: 16384
max incremental total tokens over native: 50000
max incremental runner duration over native: 30000 ms
```

Also report, without hiding failures:

```text
Engram identity-boundary calls
host identity rereads after a complete Engram identity
operation-specific file reads
project-confirmation correctness
```

All existing leakage, repeated-failure, command, native-write, authentication, attestation, and
disk-reserve gates remain mandatory.

## Implementation order after disk recovery

1. Add the forward schema constant and validation without changing historical schema behavior.
2. Extend the output schema, exact-key recovery, acceptance contract, and trace parser.
3. Add provider-free Codex and Claude trace fixtures for accepted ambiguity and rejection of an
   invented project, wrong root, stale/mismatched identity between calls, untracked component,
   changed component hash, and extra identity rereads.
4. Add the tracked Orbit component manifest and verify the fixture revision changes intentionally.
5. Run evaluator tests, full relevant tests, Clippy, formatting, packet/resource probes, and disk
   reserve checks in a fresh exact external target.
6. Only then prepare and attest a self-contained successor with zero provider outputs. Freeze its
   hashes before authentication or execution.

The first provider run should remain one case, six arms, and one repetition. Expand repetitions or
additional linked-project cases only if this exact boundary passes on both hosts within the frozen
resource limits. The flagship goal remains incomplete until those native results exist.

## Implementation and provider-free verification

Forward-only schema 12 now implements this design without reinterpreting any historical schema.
It freezes the explicit project-authorization status, confirmation requirement, canonical checkout
root, exact structured output keys, and the unchanged packet/token/duration limits. The runner
rejects recovery of a legacy or partial output as schema-12 completion.

The trusted audit correlates `orient` and `procedure_match` identities, independently checks the
live Git remote and canonical root, requires stable repository/checkout IDs, verifies the tracked
non-symlinked component manifest and exact SHA-256, requires the empty matcher result with
`abstained=true`, and rejects host identity rereads after Engram already returned the complete
boundary. Native-only lanes must derive the same facts from correlated live Git and tracked-file
reads. Both routes require exactly one operation-specific read for this abstention case.

The Orbit fixture now tracks `services/worker/component.json` with canonical name `queue-worker`.
Provider-free trace tests accept the intended Codex and Claude shapes and reject an invented
project, wrong root, cross-call identity mismatch, extra orientation, untracked component,
component-hash drift, duplicate operation reads, and host Git rereads.

Validation on the exact external target
`/private/tmp/engram-structured-identity-evaluator-target` passed:

```text
schema-12 focused tests: 5 passed; 0 failed
engram-eval tests: 148 passed; 0 failed
cargo clippy -p engram-eval --all-targets -- -D warnings: passed
cargo fmt --all --check: passed
cargo build -p engram-eval -p engram-cli: passed
repository target/debug: absent
```

Exact forward source and protocol hashes are:

```text
d0fd306c539032a5bb6fa897a1d27b5aee82579d0425d432b0deb454adeafbe3  engram-eval/src/native_pilot.rs
3dfffbcfb3e0cc8c954120dc8522dbedc770a932a5cbccff0d5ec4cbeded2e89  engram-eval/src/native_audit.rs
e87678f99c2ccf0558aca6734a9d8cc8e2a1f14881058ae1c6e77ce2ef861ebc  engram-eval/src/native_runner.rs
60f4d305e7209d3dc7a2016dfbff6e31de1b3bd2b6993e412084fa6429dd66cb  engram-eval/src/fixture.rs
d3e1bfa99874c433310f1807f611481919a1b1bfca692462da1fba4bf889c4d7  protocol-schema-12-structured-identity-no-result-forward-file-cache-bounded.json
```

The successful debug build produced evaluator SHA-256
`444ef5bcbaf5c522a8eff2c424046b1f773433f046adae1b0dccda5d609a7bb1` and Engram SHA-256
`6ee349135c875d16e749a9015566f3578fbe606b8de97c02688de651319ae860`, but these are not frozen
candidate artifacts yet. The external Cargo target occupies 15,322,800 KiB. Immediately after the
build, available space was 19,548,086,272 bytes against Engram's mandatory 19,893,251,686-byte
reserve, a deficit of 345,165,414 bytes. No preparation, authentication copy, or provider call was
attempted after that gate closed. Freezing must wait for explicit authorization to clean the exact
generated external target or another user-approved cache target.

The two verified build outputs were preserved without rebuilding in the owner-only staging root
`/private/tmp/engram-structured-identity-freeze-inputs-20260831-01`; their hashes remain the two
binary hashes above. This is an input staging area, not a candidate or run plan. The intended fresh
candidate root
`/Users/yuval.meiri/.engram/evals/native-memory-pilot/structured-identity-no-result-forward-file-cache-bounded-20260831-01`
is absent.

Read-only host preflight observed:

```text
a6042937174f72112dbd2d554a4af36936422e0c5ac69e353dc68994458996e9  Codex 0.151.0-alpha.7.2
aa3c08e84921acf61c180cfaf4db6733607a37a82ecad6d43776b31dc011a4ef  codex-code-mode-host
625869b01e0050f260b2980fac248fd9cef9e462612bded4ec9d3d49ff8969a5  Claude Code 2.1.251 launcher target
```

The source ChatGPT cache remains a regular, single-link, owner-only `0600` file of 4,657 bytes;
its contents and digest were not inspected. No lane-local copy exists because no plan exists. The
installed daemon was not responding while the reserve was negative and its installed executable
did not match the new staging binary. This is recorded as installed/source-runtime separation, not
as adoption of the candidate. At this preflight the reserve deficit was 362,597,990 bytes. No live
adapter, setting, installation, or daemon state was changed.

## Prepared successor after approved cleanup

On 2026-09-01 the operator authorized deletion of the exact generated Cargo target. Cargo removed
24,521 files (15.3 GiB), restoring a positive disk-reserve margin without touching the preserved
binary inputs, repository `target/debug`, or any completed pilot.

The fresh self-contained successor is:

```text
/Users/yuval.meiri/.engram/evals/native-memory-pilot/structured-identity-no-result-forward-file-cache-bounded-20260901-01
```

Its run-plan SHA-256 is
`52c366344844421a58d1245881c686b9b5663e41a48b8f0c3800eaccd0f9feae`. The frozen protocol and
binary hashes remain those documented above except that the current self-contained Claude Code
2.1.252 binary is
`b661c6a094fcc32656bf7c0071c5b45bf900b34d4f0a1ab3d78fd59aeba2c2c7`.

Provider-free re-attestation returned `verified=true`; the lifecycle audit returned
`invalid=false`, six `prepared` lanes, and zero failures. All provider-phase outputs are absent.
The expected pre-provisioning authentication check returns three `not_logged_in` Codex lanes and
there are zero lane-local `auth.json` files. Available disk was 53,672,968,192 bytes against the
19,893,251,686-byte reserve. Exact evidence is in
`SCHEMA12_STRUCTURED_IDENTITY_PREPARED_2026-09-01.md`.
