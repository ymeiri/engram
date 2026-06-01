# Brain Harness T84 Raw Terminal Evidence Rule

Status: Complete; doc-only evidence-method decision
Date: 2026-06-01
Scope: Artifact-quality rule for terminal-dependent controlled outcome rows

This slice changes no source behavior, telemetry schema, storage, indexes, public MCP request
parameters, ranking, harness adapters/hooks, migration, lifecycle state, document indexing, or the
`orient` payload. It does not run a standalone raw-output pilot and does not submit telemetry
feedback for sampled T82 rows.

## Research Question

After T83 flagged that T82-4's staging-discipline subclaim relied on an authored doc summary, what
is the smallest evidence-quality change that improves future terminal-dependent artifact rows
without adding process ceremony or implementation surface?

## Hypotheses

| Type | Hypothesis |
| --- | --- |
| Preferred | Codify a narrow rule: when a controlled artifact row's outcome depends on terminal state, preserve exact scoped raw output in a durable artifact or keep the subclaim indirect. |
| Null | Updating the method rule adds no useful discipline beyond T83's existing note. |
| Simpler alternative | Run a standalone `git status --short` capture pilot and commit the raw output. |
| Failure | Treat copied terminal output as independent proof, create a standing process requirement, or introduce a parallel artifact schema. |

## Consultation Summary

AI Council recall found no prior decision for this exact raw-terminal evidence rule. AI Council
broadcast agreed that raw output can improve auditability only if the claim is tightly bounded:
the command, timestamp, scope, raw output, interpretation, and limitations must be explicit; the
artifact must not claim independent attestation; and the pattern must not become automation,
hooks, or a universal process requirement.

Claude Bridge critique identified the decisive blind spot: a standalone `git status` capture now
would not retroactively strengthen T82-4 because the T78-era working tree state no longer exists.
It would demonstrate that terminal output can be pasted into Markdown, not answer an outcome-link
question. Claude recommended updating the research method rule now and applying it naturally in
the next real controlled row whose outcome depends on terminal evidence.

## Decision

Do not run a standalone raw terminal output pilot.

Codify the rule in the research method:

- if a controlled artifact row's classification or subclaim depends on git status, staged diff,
  test output, command output, or another terminal state, pre-register the exact command and scope;
- preserve the raw output in a committed artifact when it is short enough to review and materially
  reduces ambiguity;
- include interpretation and limitations next to the raw output;
- state that copied terminal output is still author-captured, not cryptographic or independent
  proof;
- if raw output is not preserved, keep the terminal-dependent subclaim indirect or downgrade the
  evidence strength.

This makes T83's caveat actionable without adding new code, schema, tooling, hooks, indexing, or
hot-path behavior.

## Acceptance Criteria

T84 is complete when:

- the research method explicitly names the terminal-output preservation rule;
- governing docs record that T84 is a method decision, not an implementation approval;
- no standalone terminal-output capture is presented as outcome evidence;
- no schema/storage/public MCP/harness/ranking/lifecycle/migration/document-index/`orient` change
  is made.

## Next Non-Gated Slice

The next controlled outcome artifact row that genuinely depends on terminal state should apply the
T84 rule prospectively. If no such row exists, do not manufacture one; continue with the highest
evidence-value non-gated Brain Harness gap.
