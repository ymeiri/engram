# T306 Rustdoc Graph Node Example Cleanup

Date: 2026-06-07
Status: completed narrow docs-CI hardening slice

## Question

Can the current `cargo doc --no-deps` warnings be removed without changing runtime behavior,
public MCP shape, graph semantics, or release scope?

## Hypotheses

| Hypothesis | Result |
| --- | --- |
| Preferred | The warnings are caused by Rustdoc parsing literal `memory:<...>` examples as HTML tags, and can be fixed by marking those examples as inline code. | Supported. |
| Null | The warnings indicate a deeper schema or graph API problem. | Not supported. |
| Failure | The slice changes runtime behavior or schema text beyond the warning-producing doc comments. | Avoided. |

## Evidence

Initial `cargo doc --no-deps` produced three `rustdoc::invalid_html_tags` warnings:

- `engram-core/src/graph.rs`: `memory:<uuid>`
- `engram-mcp/src/tools.rs`: `memory:<id>`
- `engram-cli/src/main.rs`: `memory:<id>`

T306 changes only those Rust doc comments, wrapping the graph node examples in backticks.

## Validation

These commands passed after the change:

```bash
cargo fmt --all --check
cargo doc --no-deps
git diff --check
```

## Boundary

T306 does not change graph behavior, MCP request/response structure, JSON schema fields, harness
adapters, installed runtime, native Claude behavior, effective-hook visibility, host labels, M6,
lifecycle state, release approval, merge, tag, or publish state.
