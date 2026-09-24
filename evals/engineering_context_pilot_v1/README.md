# Engineering Context Native-Host Pilot v1

This preregistered eight-run pilot validates the paid-run pipeline and the two flagship mechanics
before any larger matrix: stable identity for a previously unseen moved checkout, and exact
retrieval/execution of a verified condition-aware procedure. It compares checked-in instructions
with the lean Engram profile on Codex and Claude Code, once per scenario, in a fixed mixed order.

This pilot is not sufficient to claim incremental value over host-native memory. That remains a
required later comparison. Codex native memories are asynchronously generated state and the
official manual says not to rely on editing their files as the primary control surface. Claude
Code auto memory is directly auditable and editable, but is stored and loaded differently. A fair
native-memory protocol therefore needs a separate teaching phase, proof of effective memory
loading, and matched provider spend rather than hand-authored files presented as learned memory.

Prepare the pilot without calling either provider:

```bash
cargo run -p engram-eval -- prepare-pilot \
  --protocol evals/engineering_context_pilot_v1/protocol.json \
  --output /path/to/new-or-empty-pilot \
  --engram-bin /absolute/path/to/engram
```

Preparation:

- validates the selected scenarios and arms against frozen v1;
- attests the exact Engram, Codex, and Claude executables by path, version, and SHA-256;
- creates a fresh deterministic Git fixture for every run;
- renders the current native Engram boundary adapter only for lean Engram arms;
- seeds each Engram arm into its own `ENGRAM_HOME` and project RocksDB store;
- writes a derived scorer manifest, structured-output schema, exact argv arrays, cleanup commands,
  and future trace paths to `run-plan.json`;
- always records `execution_approved: false` and never starts Codex or Claude Code.

The aggregate Claude hard cap is $0.25. Preparation divides it across the four Claude runs and
places a per-run `--max-budget-usd` limit in each command preview. Codex provider execution has no
equivalent CLI dollar cap, so it remains separately approval-gated.

Native memory references:

- [Codex memories](https://learn.chatgpt.com/docs/customization/memories)
- [Claude Code memory](https://code.claude.com/docs/en/memory)
