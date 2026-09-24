<!-- engram:harness-adapter:v1 -->
# Resume Engram Session

1. Call `orient` with current cwd, `agent=claude_code`, and `response_shape="lean"`; supply
   project only when its canonical identity is known.
   Pass `task=<exact task name or tracker key>` to `orient` when task identity is known; task names require an explicit project. If task identity or its project relationship cannot be resolved, ask instead of applying task-scoped memory.
2. Read the selected scope, compact context, ambiguities, and candidate IDs. Ask the user before
   project/task-scoped memory if that scope cannot be resolved safely; project ambiguity alone does
   not block the repository-local procedure route in step 4.
3. Use scoped `search` only when the compact orientation lacks required evidence.
4. For every actionable repository task, even when the project is unresolved and even when the
   user does not say remembered/learned, call
   `memory(action=procedure_match, query=<a bounded task-focused excerpt from the current user request>,
   scope={relevance_mode:"local", cwd:...}, conditions=...)`. The query is required and must be at most
   512 characters. Preserve concrete operation nouns, identifiers, and failure text verbatim;
   omit unrelated meta/output instructions and secret values. The query is retrieval text, not an
   authorization channel. Project ambiguity
   blocks project/task memory only, not repository-local procedure matching. `memory(action=list)`
   is not a substitute. Copy the exact cwd returned by `orient`; do not replace it with the
   checkout root. Execute only a returned verified match. On no-result, read exactly one
   absolute `suggested_operation_evidence.resolved_path` unchanged when present. When
   `required_before_final_abstention=true`, perform that read before interpreting `abstained`,
   asking for project confirmation, or returning final output. The read is repository-local
   evidence collection, not procedure execution. Otherwise keep the bounded local fallback. Do
   not substitute the checkout-relative `path` for the host read.
5. Store only compact, evidenced durable memory that a future session genuinely needs. Never
   store secrets.
