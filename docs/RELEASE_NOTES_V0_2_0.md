# engram v0.2.0 Release Notes

v0.2.0 is the planned GA release for Engram's local Brain OS and brain-harness
workflow. It promotes the v0.2.0 beta line into a release intended for daily
local agent use, with conservative support claims and explicit limits where
live cross-harness proof is still pending.

## What Changed Since v0.1.0

- Added the Memory OS hot path for agents: compact `orient`, current-plan
  retrieval, used-memory candidate IDs, obligation summaries, and evidence-backed
  memory continuity.
- Added generated Markdown vault support for durable local inspection of
  MemoryItems, knowledge commits, repositories, entities, and projects.
- Added review-gated Memory OS flows for migration, digest extraction, orphan
  cleanup, quarantine review, and lifecycle-sensitive memory updates.
- Added `engram setup`, a dry-run-first setup path for Claude Code, Codex, and
  Cursor harness adapters.
- Added first-run knowledge onboarding: when no documents are indexed yet,
  `orient` asks the agent to collect approved docs, runbooks, notes, ADRs, or
  knowledge folders before ingestion.
- Added `engram warmup embeddings` to prepare and verify the local fastembed
  ONNX model cache before first use or offline work.
- Improved daemon provenance reporting, stale-runtime diagnostics, RocksDB lock
  recovery guidance, and daemon startup error output.
- Added release packaging with pre-archive manifest validation, package install
  smoke tests with structured manifest checks, Homebrew formula rendering,
  published-release install verification with tag/version, tag-commit, and
  draft/prerelease checks, hosted-CI verifier evidence, beta/GA readiness
  reports, and native Claude preflight evidence scripts.

## Install

Homebrew is the preferred path for Apple Silicon macOS, Intel macOS, and Linux
x64 after the v0.2.0 formula and matching release artifacts are published:

```bash
brew tap ymeiri/engram
brew install engram
engram init
engram setup
```

Release tarballs will be available from GitHub Releases after v0.2.0 is
published for users who do not use Homebrew:

```bash
version=0.2.0
case "$(uname -s)-$(uname -m)" in
  Darwin-arm64) target="aarch64-apple-darwin" ;;
  Darwin-x86_64) target="x86_64-apple-darwin" ;;
  Linux-x86_64) target="x86_64-unknown-linux-gnu" ;;
  *) echo "unsupported platform for published tarballs" >&2; exit 1 ;;
esac
archive="engram-${version}-${target}.tar.gz"

curl -LO "https://github.com/ymeiri/engram/releases/download/v${version}/${archive}"
curl -LO "https://github.com/ymeiri/engram/releases/download/v${version}/${archive}.sha256"
if command -v shasum >/dev/null 2>&1; then
  shasum -a 256 -c "${archive}.sha256"
else
  sha256sum -c "${archive}.sha256"
fi
tar -xzf "${archive}"
install -m 755 "engram-${version}-${target}/engram" "$HOME/.local/bin/engram"
engram init
engram setup
```

## Upgrade From v0.2.0 Beta

1. Install the v0.2.0 binary through Homebrew or a release tarball.
2. Restart the daemon so the running service matches the installed binary:

   ```bash
   engram daemon stop
   engram daemon start
   engram daemon status
   ```

3. Run `engram warmup embeddings` if the embedding model cache has not already
   been prepared.
4. Re-run `engram setup --agent <claude-code|codex|cursor>` and review the
   dry-run. Use `--write` only after approving the planned files.
5. Start a fresh agent session and ask it to run `orient`.

Engram stores local data under `~/.engram/` by default. The v0.2.0 release does
not intentionally delete or rewrite that data during install or setup, but users
who rely on the database for production work should keep their normal backups.

## First Run

1. Run `engram init`.
2. Optionally run `engram warmup embeddings` before offline or sandboxed work.
3. Run `engram setup` or `engram setup --agent <agent>`.
4. Review the dry-run output before applying setup writes.
5. Restart the agent and ask it to run `orient`.
6. If no documents are indexed, approve the docs or folders Engram should
   ingest before relying on document search.

## Known Limitations

- Native Claude prompt-bearing proof, live `/hooks` effective-hook visibility,
  and live Claude host-label proof must not be claimed unless the final release
  gate records a clean native Claude preflight/proof run.
- Setup writes are approval-gated. Dry-run is the default, and Engram does not
  overwrite user-owned harness files without explicit adoption.
- Native Windows packaging is not part of v0.2.0. The v0.2.0 release targets
  Apple Silicon macOS, Intel macOS, and Linux x64 artifacts.
- The first embedding use may download `all-MiniLM-L6-v2` from Hugging Face
  unless the cache has already been warmed.
- Legacy Entity, Session, Document, Tool, Coordination, Knowledge, and Work
  layers remain supported substrate and evidence sources. v0.2.0 does not claim
  broad legacy deprecation, destructive cleanup, or unrestricted automated
  lifecycle mutation.

<!--
Release gate: publish these notes only after the final v0.2.0 head has exact-head
CI, local package/install smoke, rendered Homebrew formula evidence, vault and
obligations checks, and a documented decision on native Claude proof scope.
-->
