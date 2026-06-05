# Brain Harness T272 Canonical Vault Count Drift Report

Date: 2026-06-05
Status: docs-only drift report and gate-management note. Not executed.

## Scope

This report records fresh read-only evidence that T267's canonical vault approval packet is stale
as an execution packet because live Memory OS source counts naturally changed after the T266 temp
vault baseline.

T272 does not edit T267, initialize or compile the canonical vault, mutate Memory OS data, archive
memory, run `lint apply_safe`, run M6/migration/quarantine actions, run native Claude or bridge
writes, edit harness files, publish branches, change ranking or `orient`, change public MCP,
schema/storage/index, or document-index behavior, delete data, roll back, or touch user-owned
files.

## Research Question

When a future durable vault approval packet uses fixed source-count parity, and normal current-plan
captures make those counts drift before execution, what is the safest repo-local step that preserves
the exact approval boundary?

## Hypotheses

| Type | Hypothesis |
| --- | --- |
| Preferred | A dated drift report can preserve T267 as immutable historical evidence while marking it non-executable under current counts and requiring a fresh successor packet before any canonical write. |
| Null | T267's hard stop is sufficient; documenting count drift adds no useful operational signal. |
| Simpler alternative | Do nothing and rely on future preflight failure. |
| Failure | The drift report is mistaken for approval to use live counts, initialize the canonical vault, or bypass T267's hard stop. |

## Measurement

T267 used the T266 temp-vault baseline:

| Source count | T266/T267 baseline |
| --- | ---: |
| MemoryItems | 1,585 |
| KnowledgeCommits | 536 |
| Repositories | 9 |
| Entities | 32 |
| Projects | 79 |
| Expected generated files | 2,245 |

Fresh read-only `vault(action="status", vault_path="/Users/yuval.meiri/.engram/vault")` during
T272 returned:

| Source count | Fresh T272 status | Delta |
| --- | ---: | ---: |
| MemoryItems | 1,591 | +6 |
| KnowledgeCommits | 542 | +6 |
| Repositories | 9 | 0 |
| Entities | 32 | 0 |
| Projects | 79 | 0 |
| Expected generated files | 2,257 | +12 |

The canonical vault path remains absent:

- `exists=false`
- `initialized=false`
- `total_file_count=0`
- `generated_file_count=0`
- `user_file_count=0`

The recent memory commit log explains the drift: six normal current-plan captures occurred after
the T266 temp status baseline, from "Current plan after T266 temp vault compile validation" through
"Current plan after T271 branch publication gate". Each capture records a new MemoryItem and a
KnowledgeCommit, so a later generated vault projection should produce more files than the T266
temp projection.

## Consultation Synthesis

AI Council recall plus a fresh three-model broadcast agreed on the low-risk direction:

- do not edit T267 in place;
- do not create another fixed-count approval packet unless canonical execution is imminent;
- write an append-only drift report so future agents do not silently rediscover the mismatch;
- keep the canonical write gate default-deny.

Claude Bridge agreed that preserving T267 immutability is useful, but flagged a gate-management
gap: a stale hard-stop should not remain the operative execution packet without a transition rule.
This report adopts that critique.

Model consensus is not proof. The decision below follows from the fresh source counts, memory log,
and exact-gate requirements.

## Decision

T267 remains the historical approval packet for the T266 source-count baseline, but it is not an
executable canonical-vault packet under the current live counts. If the user gives the exact T267
wording now, the correct behavior is to stop on source-count mismatch and refer to this T272 drift
report.

Future canonical vault execution needs a fresh exact successor packet or approval that:

1. explicitly supersedes T267 for the canonical vault execution gate;
2. names `/Users/yuval.meiri/.engram/vault`;
3. captures live source counts immediately before execution approval or launch;
4. states whether any count drift after that snapshot is a hard stop;
5. preserves the no-M6, no-lifecycle, no-native-Claude, no-harness-write, no-branch-publication,
   no-schema/public-MCP/ranking/index-behavior, no-deletion, and no-user-owned-file boundaries.

Do not mutate T267 to retrofit moving counts. If execution is not imminent, do not create another
fixed-count packet that will likely drift before use.

## Completion Impact

T272 does not close the canonical vault gate. It narrows the gate by making the current blocker
explicit:

- T266 proves temp-path compileability for its point-in-time data.
- T267 prepared an exact canonical-vault packet for those point-in-time counts.
- T272 shows the live source counts have drifted through normal Memory OS writes, so T267 is now
  stale as an execution packet.

The canonical vault remains absent and uninitialized. The broader Brain Harness goal remains open
on M6 dispositions/deferral, lifecycle cleanup/deferral, prompt-bearing native Claude,
effective-hook visibility, live Claude host-label proof, canonical vault execution, and branch
publication/upstream if remote publication is desired.
