# Native Adapter MCP Contract v1

This fixture turns the generated Codex and Claude Code adapter guidance into concrete MCP calls.
The integration test renders each adapter through a live in-memory Engram daemon, checks the
required guidance fragments, validates every call argument against that daemon's `tools/list`
schema, executes the calls, and checks the scoped response contract.

The fixture intentionally covers the lean default journey only:

1. `orient` with the harness identity and a canonical project.
2. Related-scope `search` when orientation is insufficient.
3. Related-scope `memory(action="procedure_match")`, which must safely abstain when no verified
   procedure matches.

The test creates the fixture project only inside the daemon's in-memory store. It does not install
adapters, invoke Codex or Claude Code, restart the user's daemon, or touch persistent Engram data.

