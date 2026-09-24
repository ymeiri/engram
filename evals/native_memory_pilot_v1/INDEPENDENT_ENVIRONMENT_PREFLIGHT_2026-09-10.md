# Independent-environment preflight — 2026-09-10

Status: provider-free, read-only preflight; this is not portability or host-journey evidence

## Host

- Time observed: `2026-09-10T22:37:25+03:00`.
- OS/architecture: Darwin 25.6.0, arm64.
- Codex: `/Applications/ChatGPT.app/Contents/Resources/codex`, `codex-cli 0.153.4`.
- Claude Code: `/Users/yuval.meiri/.local/bin/claude`, `2.1.267`.
- Data-volume free space: 253,993,264 KiB.
- Repository `target/debug`: absent.

## Independent VM

`colima status` reported a running VM backed by macOS Virtualization.Framework with aarch64
architecture and virtiofs mounts. A read-only `colima ssh` probe reported:

- kernel: Linux 6.8.0-100-generic, aarch64;
- distribution: Ubuntu 24.04.4 LTS (Noble);
- root filesystem free space: 18,050,472 KiB;
- `codex`: absent from `PATH`;
- `claude`: absent from `PATH`;
- `cargo`: absent from `PATH`.

## Disposition

The VM is a viable genuinely separate OS environment, but it is not ready for the required
host-visible replication. Toolchain installation, exact executable/configuration attestation, and
isolated Codex and Claude authentication remain future gates. The shared virtiofs mount must not be
used as independent state evidence; the replication must create and attest VM-local state roots.

No VM package, file, daemon, credential, or provider state was changed by this preflight.

## Official installation and authentication path

Current official documentation removes the need to invent a VM-specific credential mechanism:

- OpenAI documents a standalone macOS/Linux Codex installer and supports either headless device
  login or copying a protected `~/.codex/auth.json` cache from an authenticated browser-capable
  machine. It explicitly treats that cache like a password. Sources:
  <https://learn.chatgpt.com/docs/codex/cli> and <https://learn.chatgpt.com/docs/auth>.
- Anthropic documents a native Linux installer, browser/code login for SSH or container contexts,
  and Linux credential storage at mode-`0600` `~/.claude/.credentials.json` (or beneath
  `CLAUDE_CONFIG_DIR`). A fresh interactive login or separately generated OAuth token is required;
  the macOS Keychain entry is not itself a portable Linux credential file. Sources:
  <https://code.claude.com/docs/en/terminal-guide> and
  <https://code.claude.com/docs/en/authentication>.

These sources establish feasibility only. Before installation or authentication, the replication
must freeze exact installer artifacts or resulting executable hashes, a VM-local private home, the
credential-copy/login count, and the cleanup boundary. No official-doc statement is evidence that
the controlled journeys have run.

## Same-day revalidation

At `2026-09-10T23:58:44+03:00`, read-only host probes confirmed:

- Colima `0.10.1` at canonical path
  `/opt/homebrew/Cellar/colima/0.10.1/bin/colima`, SHA-256
  `37e632654ed2c7e2901927bf752a307140348ba9c22cce9a02a86a89b5565a3b`;
- `limactl 2.0.3` at canonical path
  `/opt/homebrew/Cellar/lima/2.0.3/bin/limactl`, SHA-256
  `6455e484927c8d873d4eacca4a5a1610090d62e6ea65d86ce06ee6397780d57e`;
- the running `default` Colima profile still uses `virtiofs`, Docker, and host-visible socket paths,
  so it cannot satisfy the mount-free independent-state contract;
- the only other listed profile, `devc`, is stopped and is not the required fresh
  `engram-vm-*` profile;
- the data volume has `222,317,668` KiB available and repository `target/debug` remains absent.

This revalidation did not create, start, stop, or modify a VM or profile. The required independent
run therefore still needs a fresh non-default, mount-free profile and a VM-local deterministic
bundle; neither existing profile may be adopted as completion evidence.
