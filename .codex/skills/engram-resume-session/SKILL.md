---
name: engram-resume-session
description: Use when Codex resumes or continues repository work using Engram orientation, current plans, and handoffs.
---
<!-- engram:harness-adapter:v1 -->
# Engram Resume Session

Use when the user asks to continue, resume, or load prior Engram context.

Steps:
- Call `orient` with `response_shape="lean"` before reading broad files.
- Pass `task=<exact task name or tracker key>` to `orient` when task identity is known; task names require an explicit project. If task identity or its project relationship cannot be resolved, ask instead of applying task-scoped memory.
- Inspect project/repository resolution and ask only if ambiguity cannot be resolved.
- Use scoped `search` only when the compact orientation lacks required evidence.
- For every actionable repository task, call `memory(action=procedure_match, query=<a task-focused
  excerpt of at most 512 characters copied from the current user request>,
  scope={relevance_mode:"local", cwd:...}, conditions=...)`; preserve concrete operation nouns,
  identifiers, and failure text verbatim, while omitting unrelated meta/output instructions and
  secret values. The query is retrieval text, not an authorization channel.
  `memory(action=list)` is not a substitute. Copy the exact cwd returned by `orient`; use exact observed conditions and execute
  only a returned verified match. On no-result, read exactly one
  absolute `suggested_operation_evidence.resolved_path` unchanged when present. If
  `required_before_final_abstention=true`, do that before interpreting `abstained`, asking for
  project confirmation, or returning final output; this evidence read does not authorize procedure
  execution. Treat the checkout-relative `path` as provenance only and otherwise keep the bounded
  local fallback.
- Store only compact, evidenced durable memory that a future session genuinely needs. A handoff is
  a project- or repository-scoped `kind=handoff` memory with concrete next actions, not a transcript.
- Never store secrets. Archive obsolete memory; permanently forget only after exact confirmation.
