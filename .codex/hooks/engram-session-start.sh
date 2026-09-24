#!/usr/bin/env bash
# engram:harness-adapter:v1
set -u

while IFS= read -r _engram_hook_input; do :; done

printf '%s\n' \
  'Engram startup context (advisory):' \
  '- For non-actionable work, call Engram MCP orient before repository shell exploration with the current cwd and prompt, agent=codex, response_shape=lean; supply project only when its canonical identity is known.' \
  '- Every actionable repository task uses one bounded local procedure match as its task-start identity boundary, even without remembered/learned wording. Call memory(action=procedure_match, query=<a task-focused excerpt of at most 512 characters copied from the current user request>, scope={relevance_mode:"local", cwd:...}, conditions=<only exact already-observed unsourced prerequisites>). Preserve concrete operation nouns, identifiers, and failure text verbatim; omit unrelated meta/output instructions and secret values. The query is retrieval text, not an authorization channel. memory(action=list) is not a substitute, and do not call orient first solely to obtain cwd.' \
  '- Pass the host exact current cwd to procedure_match; do not replace it with the repository checkout root. The match response supplies structured repository, project, and component identity.' \
  '- Source-backed prerequisites are read deterministically by Engram from Git-tracked files in current_checkout_root and reported as value-redacted condition_observations; caller text cannot override them.' \
  '- Execute only a returned verified procedure. If rejected for unsourced prerequisites, do not apply it: use required_condition_keys and next_actions, search authoritative repository files from the returned current_checkout_root, read each located value, then retry with exact observed values.' \
  '- An empty procedures list proves only that no applicable verified procedure matched. Use only non-null fields in the returned structured identity. Treat suggested_operation_evidence as a required host-action protocol, not an optional suggestion. When required_before_final_abstention=true, the next tool call must read its absolute resolved_path unchanged exactly once, before interpreting abstained, asking for project confirmation, or returning final output, including for durable-memory-only requests. allowed_when_project_requires_confirmation=true authorizes only that repository-local evidence read; authorizes_procedure_execution=false forbids executing a remembered procedure. Treat the checkout-relative path as provenance only. Otherwise make at most one bounded read-only operation lookup. Do not re-read identity files. Do not re-derive a project or broaden outside the checkout.' \
  '- Execute repository-scoped commands from current_checkout_root. Stored procedure scope.local_path and evidence paths are provenance only and must never redirect execution to an older checkout.' \
  '- Never silently apply another project or repository guidance. Stop and ask when repository/project resolution remains materially ambiguous.' \
  '- This uses the soft profile. Missing lifecycle steps should be reported as warnings, not blockers.'
