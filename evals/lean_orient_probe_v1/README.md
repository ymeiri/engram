# Lean Orientation Provider-Free Probe v1

This preregistered structural probe measures the real serialized `response_shape="lean"` MCP
orientation response against explicit latency and packet-size targets. It materializes the frozen
engineering-context fixture, seeds a new isolated RocksDB store, and covers seven scenarios:

- known and previously unseen moved-checkout identity;
- unresolved ambiguous identity;
- current and superseded decisions;
- a no-result prompt;
- current-plan lookup for a `resume_session` intent in a freshly seeded in-process store.

The targets were frozen before execution: orientation p50 at most 150 ms, p95 at most 500 ms,
packet p50 at most 4096 bytes, and packet p95 at most 8192 bytes. Every call also retains its
scenario-specific packet limit from the frozen engineering-context suite. Five repetitions across
seven scenarios produce 35 measurements while keeping first-encounter moved-checkout behavior in
the sample.

Run without invoking Codex, Claude Code, or any model provider:

```bash
cargo run -p engram-eval -- probe-lean-orient \
  --protocol evals/lean_orient_probe_v1/protocol.json \
  --output /path/to/new-or-empty-probe
```

The command writes `lean-orient-report.json` alongside the materialized fixture and isolated store,
and exits nonzero when a preregistered semantic or resource gate fails. It verifies exact
repository/project/component identity, required and forbidden semantic context keys, explicit
unresolved-identity warnings, no-result abstention at the packet layer, and non-empty
`why_relevant` explanations. On Unix, the output root is created with mode `0700` and the report
with mode `0600`.

This is not a host-value evaluation. It does not measure task success, model tokens, provider
latency, first-action behavior, native memory, procedure execution, secret capture, or deletion.
The resume scenario does not create or compact a prior host session, restart Engram, or cross a
process boundary. Those claims remain gated by the frozen native-host suites and dedicated
safety/procedure tests.
