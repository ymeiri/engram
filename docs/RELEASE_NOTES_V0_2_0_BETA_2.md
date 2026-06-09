# engram v0.2.0-beta.2 Release Notes

v0.2.0-beta.2 is the first beta candidate focused on installability and first-run onboarding after
the v0.2.0-beta.1 review.

## What Changed

- Added `engram setup`, a guided dry-run-first setup path for Claude Code, Codex, and Cursor.
- Added first-orient knowledge onboarding: when no documents are indexed yet, the agent is told to
  ask which existing docs, runbooks, notes, or knowledge folders should be ingested.
- Added `engram warmup embeddings` for explicit local embedding-cache preparation before first use
  or offline work.
- Improved cold embedding-cache warnings, RocksDB lock recovery messages, and daemon startup
  failure diagnostics.
- Prepared Homebrew packaging for macOS Apple Silicon beta installs.
- Expanded README, MCP setup, security, and contribution guidance for an open-source beta.

## Install

Homebrew is the preferred macOS Apple Silicon path once the tap is published:

```bash
brew tap ymeiri/engram
brew install engram
engram init
engram setup
```

Release tarballs remain available from GitHub Releases for users who do not use Homebrew.

## First Run

1. Run `engram warmup embeddings` if you want to prepare the local embedding model cache up front.
2. Run `engram init`.
3. Run `engram setup --agent <claude-code|codex|cursor>` and review the dry-run.
4. Re-run setup with `--write` after approving the planned files.
5. Restart the agent and ask it to run `orient`.
6. If no documents are indexed yet, the orient packet asks the agent to collect approved knowledge
   paths from the user before ingestion.

## Known Limitations

- Homebrew packaging is scoped to macOS Apple Silicon for this beta.
- Setup writes are approval-gated. Dry-run is the default.
- Engram stores data locally under `~/.engram/` unless a project-specific or explicit data path is
  used.
- The first embedding use may download `all-MiniLM-L6-v2` from Hugging Face unless the cache has
  already been warmed.
