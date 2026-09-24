# Native stale-safety isolation successor foundations — 2026-09-05

## Status and authority

This is a provider-free, standalone foundation for a fresh stale/missing-source successor. It did
not execute, repair, or mutate an existing lane. No model-provider request or login command ran, no credential
bytes were read or copied, no live Engram adapter/daemon/settings were changed, and no VM was
created. Repository `target/debug` remained absent; Rust artifacts used the external target
`/private/tmp/engram-stale-isolation-foundation-target-20260905-a`.

Every launch and production-relay object in this module has
`authorizes_provider_execution=false`. The current same-tree runner is not authorized by these
types. Shared runner integration and a fresh protocol remain blocked on a separate audit. Local
Claude is additionally restricted to `confounded_exploratory` evidence because effective
server-managed policy is opaque; the flagship Claude proof must run in the mount-free VM with an
in-guest login.

The implementation is isolated in:

- `engram-eval/src/native_isolation.rs`
- `engram-eval/tests/native_isolation_foundation.rs`

No shared CLI/library hook has been added.

## Verified provider-free host facts

The final host checks ran on macOS 26.4.1 (25E253), arm64. Exact executable identities were:

- Codex `codex-cli 0.153.3`, SHA-256
  `e57b3081ef7a33014e9afa1f7cd44f472d2d72b4a2084bdc8bc16cd261159302`;
- Claude Code `2.1.260 (Claude Code)`, SHA-256
  `3c269f66801028823e24a63ced9fdd3988cb86cf85fccd9f03f87e463b9d3e3c`;
- `/usr/bin/sandbox-exec`, SHA-256
  `d1ee30dbde955aaa75c7f801fdfea4df05b10129454d7982eb6453f771436d42`.

The focused suite passed 25 tests with four explicit host gates ignored in the ordinary run. Each
host gate then passed individually:

- real Codex `codex sandbox` matrix: one allowed checkout read; sibling traversal, direct source
  read, child-shell source read, checkout write, and network denied for policy reasons; a separate
  unsandboxed `nc` control to the same live listener succeeded;
- real `/usr/bin/sandbox-exec` matrix: generated profile compiled; evaluation canary read and the
  two exact allowed loopback transports succeeded; source/run/teaching reads, source/native/Engram
  writes, adjacent loopback, external network, and an unexpected bind were denied;
- runner-owned fake CONNECT relay: missing and wrong per-invocation proxy secrets were rejected,
  the one correct secret was accepted, and a bidirectional fake-upstream exchange completed;
- macOS managed-policy inventory: all locally enumerable base/MDM/drop-in surfaces were re-read
  and hash-bound while the unobservable server-managed tier remained explicitly unclosed.

The focused Clippy gate passed with `-D warnings`. No provider executable was invoked beyond the
provider-free `--version` observation and Codex's local sandbox subcommand; Claude itself was not
started for a model request.

## Source-to-enclave transition

Each lane has canonical, disjoint sibling `source/` and `evaluation-enclave/` roots. Teaching
checkout, traces, plan, receipts, and contracts remain source-only. The retention transition copies
only an explicit hash/size/mode/class manifest into a fresh enclave:

- fresh evaluation checkout;
- marker-free schema, settings, MCP configuration, and instructions;
- exactly `$CODEX_HOME/memories` or `$CLAUDE_CONFIG_DIR/auto-memory` when native memory is enabled;
- exactly `$ENGRAM_HOME/projects/<frozen-project>` when Engram is enabled.

Native and Engram source/destination zone prefixes must be exact mirrors of those launch-state
roots. A same-basename dummy tree such as `other/memories` or `engram-home/state` cannot satisfy the
contract. Every required zone has at least one manifest entry and every lane has a nonempty exact
marker set.

The copy fails on root overlap, traversal, duplicate or renamed mappings, missing/unlisted files,
symlinks (including ancestor symlinks), hard links, sockets/nonregular nodes, oversized content,
credential/session-semantic names, mode/size/hash drift, source-path bytes, and marker bytes.
Destinations use no-overwrite creation. Owner-controlled enclave directories are mode 0700; copied
files cannot be group/other writable. This path traversal is pathname-based and does not claim
same-UID TOCTOU resistance; the later VM/separate-UID execution is the stronger boundary.

The exact immutable native/config set is hashed before and after. Engram may update operational
RocksDB/WAL metadata on a read, so raw database equality is not falsely treated as semantic
equality. Caller-created semantic evidence is now rejected: the standalone module has no live,
execution-owned semantic export collector. That missing collector is a deliberate fail-closed P0
for shared integration, not a boolean a caller can set.

## Exact host launch contracts

Both hosts use `env_clear` and an exact allowlist: clean enclave `HOME`/`TMPDIR`, one isolated host
home, fixed locale/PATH, telemetry/update/nonessential-traffic controls, and `ENGRAM_HOME` only for
Engram arms. API-key/access-token variables, lowercase/alternate proxy variables, and all ambient
extras fail. Exact argv, environment, prompt, and non-secret config bytes are scanned for source,
run, teaching, and marker literals. Prompts beginning with `-`, equals-form flags, duplicates, and
extra switches fail byte-for-byte validation. Provider binaries are hashed and their exact
`--version` output is observed under the cleared environment on every validation; the later frozen
plan must independently bind those paths and hashes.

The boundary names actual lane auth-cache files rather than dummy directories:

- Codex: `$CODEX_HOME/auth.json`, owner-only 0600, regular, single-link;
- Claude: `$CLAUDE_CONFIG_DIR/.credentials.json`, owner-only 0600, regular, single-link.

Credential bytes are never opened or hashed. Codex freezes `cli_auth_credentials_store="file"`.
Before any caller-supplied executable, schema, settings, MCP, Seatbelt, skill, Engram, or daemon
receipt path is opened, hashed, or executed, a metadata-only guard rejects lexical ancestry,
canonical aliases, symlinks, hard links, and filesystem-object aliases to the exact credential
file. The copy classifier also treats Claude's leading-dot `.credentials.json` as credential
material. Adversarial tests bind the rejection to this early guard rather than a later parse or
hash mismatch.
The whole Claude process must read/refresh its own credential material, and macOS Seatbelt cannot
distinguish trusted client reads from model-tool reads. The profile therefore does **not** claim raw
Claude credential read/write denial: the exact credential path is absent from both generated
Seatbelt deny sets, and adding it invalidates the typed boundary. The claim is instead restricted CLI tool confinement plus
source/state Seatbelt controls. Keychain/helper/server-managed authentication lies outside the
local exact-auth claim; local Claude cannot be labeled subscription-pure from this foundation.
Non-native memory roots, the unused provider's home, and non-Engram homes must be absent.

### Codex 0.153.3

The only accepted real evaluation shape is `codex exec --profile isolation
--ignore-user-config --ignore-rules --strict-config --ephemeral --cd <checkout> --json
--output-schema <schema> <prompt>`, with no legacy `--sandbox` and no outer Seatbelt. The selected
`isolation.config.toml` freezes:

- `approval_policy="never"` and file credential storage;
- `default_permissions="engram-eval-read"`;
- `:root=deny`, `:minimal=read`, `:tmpdir=deny`, `:slash_tmp=deny`;
- read-only exact evaluation checkout and network disabled;
- exact native-memory booleans and no memory generation;
- exact Engram MCP/skill/instruction surface only for Engram arms.

The proven boundary is the model-command sandbox. It does not claim that the trusted Codex client
or trusted MCP server is itself confined.

### Claude Code 2.1.260

The exact argv freezes stream-json plus required `--verbose`, `--restricted`, permission mode
`dontAsk`, prompts `none`, strict MCP config, no persistence/Chrome/slash commands, built-in
`--tools Read`, exact six Engram names only via `--allowed-tools`, explicit write/web/Bash/task
denials, and protocol-supplied model/turn/micro-USD ceilings. `StructuredOutput` is expected because
the launch uses `--json-schema`.

Trace validation freezes the provider-observed 2.1.260 init surface: exact CLI version/cwd/model,
`Read,StructuredOutput` plus six Engram tools, Engram status `connected` when applicable, permission
`dontAsk`, no plugins/skills/slash commands, exact built-in agent list
`claude,Explore,general-purpose,Plan,statusline-setup`, and exact three lifecycle capabilities. It
records `apiKeySource`, helper attempt/success, analytics/product-feedback fields, and rejects a
local subscription-pure claim when `apiKeySource=apiKeyHelper`. Terminal subtype is restricted to
exact `success` or `error_max_budget_usd`; arbitrary nonempty failures do not pass.

The generated Seatbelt profile denies network by default and allows only the exact loopback ports
for a prestarted lane-private Engram daemon and runner-owned provider proxy. It denies source,
run, and teaching reads and denies source/native/Engram writes. Engram evidence includes live
PID/executable observation, owner-private port/PID/metadata receipts, and a bounded token-free
`GET /health` requiring the exact service/version/PID/schema-3/MCP-contract-4/tool-digest/protocol,
auth-required, storage-ready, and positive-reserve contract. Receipt freshness is bounded; shared
execution must re-observe health immediately at phase launch.

The serialized launch contains only
`HTTPS_PROXY=http://engram-proxy:REDACTED_PER_INVOCATION_SECRET@127.0.0.1:<port>`. A 256-bit secret
comes from `/dev/urandom`, remains in zeroizing non-serializable memory, and is injected only into
the exact child environment. The proxy requires one constant-time exact Basic header before DNS or
dial, rejects missing/duplicate/wrong headers, and retains only a salted invocation digest. This is
possession attribution, not kernel socket-to-PID attribution; malicious same-UID processes are an
explicit exclusion. Session, resolution, and accepted-relay timestamps are checked against the
current clock and the single frozen wall-clock deadline, so replayed or future-dated receipts fail.

The production relay remains non-authorizing. Its resolver is a killable/reaped, output-bounded
`/usr/bin/dscacheutil` child under the same absolute invocation deadline—there is no detached DNS
thread. Raw resolver cardinality is capped before normalization and duplicates/local/special ranges
fail. TLS stays end-to-end to Claude, so certificate verification, not a false complete IANA
classifier claim, authenticates the provider. Accept, header, DNS, all connect attempts, and both
tunnel directions share one deadline and byte ceilings. Shared integration must additionally own
the Claude child, kill/reap it on every failure, and causally bind the relay/trace/phase receipt.

### Managed Claude policy

The local inventory uses the current macOS paths:

- `/Library/Application Support/ClaudeCode/managed-settings.json`;
- `/Library/Application Support/ClaudeCode/managed-mcp.json`;
- every sorted regular `managed-settings.d/*.json` drop-in;
- `/Library/Managed Preferences/com.anthropic.claudecode.plist`.

It rejects symlink/unbounded/non-root-owned/writable local sources and detects security-relevant
`apiKeyHelper`, `policyHelper`, hook, MCP, and plugin keys without retaining raw policy bytes.
Server-managed effective policy remains unobservable and highest precedence. A provider-free exact
2.1.260 observation by the independent auditor showed the managed `apiKeyHelper` attempting
`ddtool auth token rapid-ai-platform --datacenter us1.ddbuild.io` before init. Therefore local
Claude is a paired but confounded exploratory option only, never flagship isolation evidence.

## Phase and process gates

Only `/dev/null` and distinct canonical owner-only single-link stdout/stderr files may be inherited.
On macOS the runner uses the kernel `PROC_PIDLISTFDS` inventory rather than a capped numeric scan,
then applies `F_GETFD` and `fstat` immediately before spawn. The negative gate duplicates an
inheritable descriptor above 70,000 and proves it is detected and blocks preparation. Auth
status execution accepts only the already-validated launch plus its exact boundary and derives the
wrapper, provider, profile, argv, environment, and limits internally. A serialized or mutated auth
contract has no execution authority; standalone attestation validation re-derives it and compares
byte-for-byte before consulting caller fields. Credential-alias negatives cover both Codex and
Claude auth executables and Claude's profile path. Output is bounded and only digests/status are
retained. This does not override the Claude helper/managed-policy nonclaim.

Each activation/evaluation uses a dedicated owner-private output root. The phase contract derives
four required outputs, but preflight also requires the **entire root to be empty**, so partial,
intent, temporary, proxy, auth, semantic, or unrecognized residue blocks retry. Disk reserve must be
positive and available. `target/debug` absence uses `symlink_metadata`, so a dangling symlink also
fails.

## Mount-free Colima model

The VM model is deliberately non-authorizing until observed. It requires a new disposable
non-default VZ/aarch64 Colima profile, Rosetta and SSH-agent forwarding off, `--mount none`, and at
least a 40-GiB root disk. A hash-manifested bundle is streamed over SSH into VM-owned ext4 and
rehashed; no bind mount is permitted. Zero/unrecognized mounts, host paths, VirtioFS/9p/SSHFS,
Docker socket, agent socket, or Rosetta invalidate the attestation.

Exact distinct Linux/aarch64 Codex, Claude, Engram, evaluator, Node, rustc, Cargo, bubblewrap, and
AppArmor runtime paths/hashes plus command/profile receipts are mandatory. Teaching and evaluation
use distinct unprivileged UIDs. Authentication occurs in the guest through subscription login;
silent API-key fallback fails. The new 12-lane pilot must teach and evaluate fresh VM-native state,
not import macOS memories. The honest claim is bounded separate OS/filesystem/runtime on the same
physical Mac/account/network—not an independent site.

## Remaining P0s before any real pilot

The standalone foundation intentionally cannot authorize execution. Shared integration must still:

1. bind the exact frozen plan/protocol/executable hashes and invoke only execution-owned probe,
   auth, FD, daemon, proxy, trace, output, and state collectors (never deserialize caller booleans);
2. implement the live Engram semantic export collector with immediate before/after chronology;
3. own and kill/reap every provider/proxy/daemon child on timeout or failure;
4. re-attest all current paths, versions, policies, daemon health, phase emptiness, disk, and
   `target/debug` immediately before launch;
5. complete independent adversarial review; and
6. run Claude flagship lanes only in the attested mount-free VM with in-guest login.

No real stale protocol should be frozen or provider-run until those conditions are proven.
