# Native Host Calibration — 2026-08-07

This calibration is evidence about the frozen v1 protocol, not a modification of its scenarios or
budgets. Commands used fresh, project-isolated Engram daemons and stopped the exact daemon after
each run.

## MCP surface

| Profile | Tools advertised | Serialized `tools/list` declarations |
|---|---:|---:|
| `full` | 32 | 60,732 bytes |
| `agent` | 6 | 6,755 bytes |

The agent profile reduces advertised tools by 81% and serialized declarations by 89%. It exposes
`orient`, `memory`, `repo`, `search`, and the action-restricted `harness` and `obligations` paths
needed by generated Claude hooks. Guessed hidden tools and disallowed actions are rejected by the
stdio proxy.

## Claude Code smoke

Host: Claude Code 2.1.224, Claude Sonnet 5, `--bare`, native MCP, one lean `orient` call.

| Configuration | Cache creation | Cache read | Cost | Result |
|---|---:|---:|---:|---|
| Bare host, no Engram | 1,043 tokens | 0 | $0.008748 | `baseline` |
| Agent profile + generated workflow | 6,012 tokens | 5,144 | $0.0434682 | selected `engram`; no confirmation |
| Earlier full Engram surface | 27,810 tokens | 26,967 | $0.180557 | selected `engram`; no confirmation |

Against the earlier full-surface smoke, the agent profile reduced cache-creation tokens by 78%,
cache-read tokens by 81%, and cost by 76% while preserving the orientation result.

## Codex smoke

Host: native `codex exec`, user config/plugins/apps/memories disabled, read-only sandbox, one lean
`orient` call.

| Configuration | Host-reported cumulative input | Cached input | Result |
|---|---:|---:|---|
| Bare host, one model turn, no Engram | 16,546 tokens | 0 | `baseline` |
| Agent profile, tool call plus answer turn | 57,206 tokens | 19,200 | selected `engram`; no confirmation |
| Earlier full Engram surface | 89,368 tokens | 58,368 | selected `engram`; no confirmation |

Codex reports cumulative input across the tool-call and answer turns, so the one-turn bare result is
not a direct incremental-cost subtraction. The like-for-like Engram smokes still show a 36%
reduction from the earlier full surface. A future runner should retain per-request token usage or
pair Engram with a minimal two-turn MCP control.

## Consequence for frozen v1

The frozen scenarios declare 8k–12k absolute total-token budgets. The current Codex bare-host
baseline already exceeds those limits before Engram is present. Therefore:

- keep v1 unchanged and report those budget violations honestly;
- do not treat absolute-token failure as Engram-specific when the matching host baseline fails;
- publish a v2 protocol before broad execution, with both absolute host cost and preregistered
  incremental-over-matched-baseline budgets;
- continue enforcing the existing packet-size budgets, because Engram directly controls those.

The full 432-run matrix should not be launched until the user approves the expected provider spend.
A small randomized pilot should validate fixture materialization, trace extraction, and paired
baseline accounting first.

## Deterministic fixture pipeline

`engram-eval materialize` now creates the portable filesystem/Git fixture and emits
`fixture-map.json`; `engram-eval seed` applies its semantic plan to a fresh isolated RocksDB store,
verifies three procedure receipts, and emits stable context keys mapped to generated IDs. Both
commands refuse non-empty targets. The current path-independent fixture revision is
`26c5a4277603bb114dc8f5e1f2807bf3045fbb29e214c6c24171d78954d215fe`. The revision includes
the same neutral, authoritative-source-first checked-in guidance for Codex and Claude Code so the
instructions-only arms receive equivalent repository policy.

A local end-to-end run resolved the previously unseen moved checkout to the existing Atlas remote,
selected the `queue-worker` component, returned the current JetStream decision with file evidence,
and matched only the verified v3 integration-test procedure. That run exposed and fixed a weak
lexical match that had also returned the related-but-wrong deploy procedure.
