# Brain Harness T85 Claude Bridge Project Tool Exposure Recheck

Status: Complete; tools unavailable in Claude Bridge project harness
Date: 2026-06-01
Scope: One read-only Claude Bridge project-harness Engram tool-exposure check

This slice is an evidence-quality diagnostic only. It must not run M6 inspection, migration status,
prioritize, review apply, candidate decisions, deletion, lifecycle mutation, document indexing,
harness writes, schema/storage/index changes, ranking changes, public MCP changes, or `orient`
payload changes.

## Research Question

Does the Claude Bridge project harness still fail to expose `mcp__engram__orient` and
`mcp__engram__search` under the same constrained conditions that made T79
`HARNESS_INCONCLUSIVE`?

This checks tool exposure only. It does not test `orient` quality, `search` quality, task
completion, Claude Code hooks, personal-harness behavior, or any write-capable path.

## Hypotheses

| Type | Hypothesis |
| --- | --- |
| Preferred | The project harness still reports the allowed Engram tools as unavailable, matching T79 and making the caveat stable until configuration changes. |
| Null | The tools are exposed and callable in the project harness, so T79 no longer describes the sampled environment. |
| Simpler alternative | Treat T79 as sufficient and do no recheck until a harness configuration change is made. |
| Failure | The run retries, falls back to personal harness, scores task outcome, or treats tool exposure as evidence about Engram behavior or any gated operation. |

## Consultation Summary

AI Council recall found prior guidance to keep Brain Harness slices narrow and avoid hot-path or
lifecycle expansion. A fresh AI Council broadcast supported T85 as a low-cost diagnostic only if it
is pre-registered literally, run once, and classified only for tool exposure. The Council cautioned
that a repeat negative result should close this line of inquiry until the bridge or harness
configuration changes, and that no result may authorize M6, lifecycle, harness, ranking, schema,
storage, public MCP, document-index, or `orient` changes.

## Measurement

Run exactly one Claude Bridge call:

```text
claude_ask(
  harness="project",
  write=false,
  cwd="/Users/yuval.meiri/projects/engram",
  allowBash=[],
  allowTool=[
    "mcp__engram__orient",
    "mcp__engram__search"
  ]
)
```

Claude must attempt exactly two read-only tool calls, then stop:

```text
mcp__engram__orient(
  project="engram",
  cwd="/Users/yuval.meiri/projects/engram",
  intent="verify_decision",
  response_shape="lean",
  prompt="T85 project-harness tool exposure check: can Claude Bridge call orient?"
)

mcp__engram__search(
  project="engram",
  cwd="/Users/yuval.meiri/projects/engram",
  intent="verify_decision",
  query="T85 project-harness tool exposure check Engram orient search",
  limit=3
)
```

The measured object is only whether those two tool names are exposed and callable through Claude
Bridge project harness under the constrained setup.

## Classification

| Class | Meaning |
| --- | --- |
| `TOOLS_EXPOSED` | At least one of the two allowed Engram tools returns a real Engram result or trace ID. |
| `TOOLS_UNAVAILABLE` | Claude Bridge reports both allowed Engram tools unavailable or not found. |
| `CONFIGURATION_BLOCKED` | The bridge cannot establish the requested project-harness/no-Bash/allowlist setup. |
| `HARNESS_INCONCLUSIVE` | Timeout, transport error, ambiguous output, or insufficient raw bridge evidence prevents classification. |
| `CONTAMINATED` | The run uses Bash, write mode, personal harness fallback, replacement tools, extra probes, or task-outcome scoring. |

Only `TOOLS_EXPOSED` may justify follow-up retrieval feedback, and only for the Engram traces that
actually exist. All other outcomes submit no T85 telemetry feedback.

## Stop Rules

- Run exactly one Claude Bridge call and stop.
- Do not retry on failure, timeout, missing trace IDs, or ambiguous output.
- Do not fall back to personal harness or native Claude Code.
- Do not add tools, use Bash, or change `write=false`.
- Do not classify task success, preference adherence, or retrieval quality.
- If T85 repeats T79 with `TOOLS_UNAVAILABLE` or `HARNESS_INCONCLUSIVE`, close this exact
  project-harness tool-exposure recheck line until the bridge or harness configuration changes.
- Do not use T85 to authorize M6, lifecycle writes, harness writes, schema/storage/index changes,
  public MCP changes, document indexing, ranking changes, or `orient` expansion.

## Raw Evidence Rule

Because the classification depends on bridge/tool-call output, the execution result must preserve
the exact Claude Bridge parameters above and a short raw excerpt of the returned bridge output. That
excerpt is author-captured evidence, not independent or cryptographic proof.

## Execution Result

Pre-registration commit: `90828f5` (`Pre-register T85 Claude Bridge tool exposure recheck`)

The single Claude Bridge call used:

```text
harness="project"
write=false
cwd="/Users/yuval.meiri/projects/engram"
allowBash=[]
allowTool=[
  "mcp__engram__orient",
  "mcp__engram__search"
]
```

Claude returned this raw bridge output excerpt:

```text
| `mcp__engram__orient` | FAILED | none | `No such tool available: mcp__engram__orient` | - |
| `mcp__engram__search` | FAILED | none | `No such tool available: mcp__engram__search` | - |

Final classification: TOOLS_UNAVAILABLE

Neither Engram MCP tool is registered in this harness context. The errors are identical
namespace-resolution failures (`No such tool available`), not permission or configuration
rejections - the MCP server is simply not connected.
```

## Interpretation

T85 repeats T79's Claude Bridge project-harness exposure caveat under the same constrained setup.
The project harness did not expose `mcp__engram__orient` or `mcp__engram__search`, and no Engram
trace IDs were produced.

Per the pre-registered stop rules:

- no retry or personal-harness fallback was run;
- no telemetry feedback was submitted because there were no Engram traces;
- this exact project-harness tool-exposure recheck line is closed until the bridge or harness
  configuration changes.

This result is not evidence against Engram `orient` or `search` behavior. It does not authorize
M6, lifecycle writes, harness writes, schema/storage/index changes, public MCP changes, document
indexing, ranking changes, or `orient` expansion.
