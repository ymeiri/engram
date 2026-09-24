# Mount-free native VM collector foundation — 2026-09-05

Status: **provider-free foundation only; no VM or provider run is authorized**.

This schema-2 slice replaces the earlier caller-supplied Colima observation model with a
runner-owned collection boundary in `engram-eval/src/native_vm.rs`. It does not join the shared
evaluator or CLI yet. No VM was created, started, stopped, or deleted while implementing or testing
this foundation. No Codex or Claude authentication file was read or copied, no provider was called,
and no live Colima or agent setting was changed.

## Frozen host and VM shape

The contract accepts only a named non-default `engram-vm-*` profile and pins both host executables:

- Colima `0.10.1`, official commit `ed905203afdbc6fd4eae6cc301918099ff31e86e`;
- `limactl` `2.0.3`;
- both canonical executable paths, SHA-256 digests, exact version commands, version markers, and
  raw version-output digests are preregistered and rechecked. Host executable basenames, all seven
  bundled runtime locations, and all 19 guest collector locations are exact rather than
  caller-selected.

The derived start command is byte-for-byte fixed. It explicitly selects VZ, aarch64, containerd,
at least 40 GiB for both VM and root disks, and `--mount none`. It explicitly disables mount
inotify, Rosetta, binfmt, SSH-agent forwarding, host activation, host SSH-config modification,
Kubernetes, reachable/host network addressing, preferred host routing, templates, and container
port forwarding. It does not rely on Colima's mount or boolean defaults. Containerd is used so the
guest does not need a Docker socket.

All host commands run with a cleared environment and a fresh, canonical, mode-0700 synthetic
`HOME` which initially contains only an empty mode-0700 `.colima` directory. The exact Colima home
is that direct child; the profile path must not exist. This prevents the collector from inheriting
the operator's normal Codex, Claude, SSH, or other home state. The prepared intent durably records
that precondition, and the final audit rejects an authentication-home name appearing afterward.

Colima necessarily creates and uses its own private SSH transport material and generated profile
SSH configuration. `--ssh-config=false` means it must not edit the user's `~/.ssh/config`; it does
not claim that Colima has no runner-private SSH configuration. `--port-forwarder none` disables
container/service forwarding; it does not claim that the management SSH channel ceases to exist.

## Bounded stream, filesystem, and identity evidence

The input is a deterministic POSIX ustar archive plus an independently derived sorted hash
manifest. Before any SSH stream, the collector binds the archive by canonical path, owner, inode,
single-link count, size, mode, and SHA-256. Its internal ustar parser rejects unsafe paths,
duplicate or unexpected files, links, devices, PAX/GNU extensions, permissive or privileged
file/directory modes, invalid checksums, payload drift, missing files, and nonzero trailing data.
Bundle, archive, host-executable, pin, and artifact paths are screened lexically and canonically;
symlink and hardlink aliases are rejected before a file is read or hashed. This includes Claude's
exact `.credentials.json` name and other credential-semantic names. The archive has a 4-GiB aggregate bound,
16,384-file bound, 1-GiB per-file bound, and depth bound.

The exact opened archive handle is rehashed immediately before it is rewound. A runner-owned pipe
then hashes and counts the bytes actually written to the one exact `colima --profile … ssh`
extraction command; a short write, changed digest, or changed count fails immediately. The guest
bundle root must first be absent, and GNU tar uses keep-old-files/no-overwrite semantics.
Every guest command runs through a cleared environment with no inherited `SSH_AUTH_SOCK` or API-key
variable. The guest then directly records:

- Linux/aarch64 and Ubuntu 24.04 identity;
- complete `findmnt --json` output, one block-device ext4 root, the bundle target resolving to that
  exact same root device/mount, and root filesystem byte size;
- absence of VirtioFS, 9p, SSHFS, host home/volume mounts, Docker socket, SSH-agent socket, and
  foreign-architecture binfmt registrations;
- two newly created, distinct, non-root users (`engram-teach` UID 12001 and `engram-eval` UID
  12002), no supplementary groups, exact owner-private homes, and a cross-UID unreadable canary
  probe;
- enabled AppArmor and exactly `bwrap-default (enforce)` from the bubblewrap process's
  `/proc/self/attr/current`; complain, kill, audit, unconfined, mixed, and extra lines fail;
- a closed-world file and directory inventory of the extracted bundle, with root ownership,
  non-writable and non-privileged directory modes, regular single-link files, exact modes/sizes,
  no setuid/setgid/sticky bits, and no special entry;
- direct canonical path, `stat`, SHA-256, independent OpenSSL SHA-256, and exact version output for
  Cargo, Claude, Codex, Engram, the evaluator, Node, rustc, and all 19 collector utilities. Every
  utility path used in the guest plan is the corresponding pinned path, not a parallel hard-coded
  executable;
- reciprocal OpenSSL/sha256sum checks so neither hash binary is accepted solely on its own report;
- a full guest rehash matched against the host-derived manifest.

Runtime binaries used by a future pilot must be executable files inside the streamed bundle and
their manifest digest must equal the runtime pin. Guest base-image utilities remain separately
preregistered pins; their package/image provenance still needs to be frozen before a real VM is
created.

The command plan is fail-fast. Host identity and guest OS, boot, root filesystem, mounts, sockets,
agent absence, user-path absence, binfmt, and exact AppArmor state are semantically checked before
the archive can be transferred. Distinct-user creation and home/cross-UID checks follow. The
bundle's filesystem, closed-world inventory, directory modes, full rehash, and special-entry scan
all pass before any bundled runtime is executed. Runtime and collector version probes are generated
from a name-specific allowlist (`--version`, except `openssl version`), never copied from caller
argv. Claude and Codex therefore cannot substitute `--print`, `exec`, login, auth, or another
provider-capable command. The closing status and boot-ID probes must match the opening live profile
and boot identity.

## Durable lifecycle and audit boundary

Preparation creates a new mode-0700 collection root, a new raw-artifact directory, contract and
manifest snapshots, and a `prepared` intent. Every file is create-new, mode 0600, single-link,
path/handle bound, fsynced, and paired with a SHA-256 sidecar; directories are fsynced. Any existing
root fails closed. A crash leaves the intent and partial output in place, so rerunning cannot replay
the VM start or archive transfer.

Execution uses absolute binaries, an exact cleared environment, bounded stdout/stderr files,
per-command deadlines, output-overflow termination, exact exit codes, and one receipt for each
derived command. Immediately after every spawn, an RAII guard owns the direct child and its exact
process-group ID. Every subsequent error explicitly attempts process-group `SIGKILL` and a
synchronous direct-child reap; cleanup errors are reported alongside the primary error. The same
cleanup runs without replacing an active panic during unwind. The guard stays armed through pipe,
thread, wait, and join handling and is disarmed only after a confirmed terminal reap. The
raw-artifact directory is fsynced before the terminal receipt. The terminal receipt structurally
binds the contract, intent, command plan, bundle, canonical run-root/raw-directory device and
inode, live Colima profile/config and guest boot identity, command chronology/transcript, raw
artifact paths/digests/sizes, and the bytes actually written to stdin. It does not contain a
caller-asserted provider-authorization bit or unverifiable counters claiming zero
provider/authentication access. Production preparation and audit obtain time from the runner's
system clock; only test-only entry points accept a synthetic clock. The complete audit rejects
stale or future-dated receipts, reordered/extra/missing commands, extra residue, symlinks,
hardlinks, permissive modes, changed live binaries/config/archive, forged raw output, and a forged
receipt that has been re-digested around semantically false observations.

The public offline audit derives structural consistency from the raw outputs, but same-UID callers
can fabricate or replay self-consistent JSON and raw files. Its result is therefore named
`NativeVmStructuralAudit`, uses `artifacts_consistent`, states
`structural artifact consistency only; no causal provenance or provider authority`, and exposes no
authorization boolean. It can never be consumed as execution authority.

Only `execute_native_vm_collection` can construct `NativeVmExecutionWitness`: a private-sealed,
non-cloneable, non-serializable in-process value that owns open run-root/raw-directory handles and
binds the exact contract, receipt, in-memory command transcript, canonical root, live profile, and
guest boot identity. Persisted files cannot recreate it. The runner returns it inside a
private-field `RunnerObservedNativeVmCollection`. This proves only that this runner executed the
exact plan against the observed endpoints. It does not make self-pinned binaries independent or
authorize a provider. A future provider gate must consume the witness by value and immediately
perform a fresh live critical-state recheck; no such gate exists in this module.

## Exact claim and nonclaims

A standalone receipt can support only structural consistency. A live runner witness, combined with
independently anchored contract and guest-image/tool provenance, may support only:

> bounded separate OS/filesystem/runtime on the same physical Mac, account, and network

It is not an independent site, account, physical host, or network. Claude managed policy remains
account-scoped and may influence both the macOS and VM runs; this replication cannot isolate or
attribute that influence. The collector also does not claim defense against a compromised macOS
kernel/hypervisor, compromised guest kernel/base image, malicious same-UID process racing trusted
files, provider-side correlation, or innocuous-looking bundle files whose reviewed source already
contains a secret. Process-group cleanup also cannot claim control over a malicious descendant that
deliberately escapes into a different session or process group before termination.

## Gates still required before a real VM

1. independent adversarial review of this module, tests, and the exact frozen contract;
2. a reviewed deterministic ustar bundle with no credentials or host-specific state;
3. independently anchored (not caller-self-asserted) canonical contract, Colima/limactl hashes,
   deterministic guest disk-image hash, and guest package/tool provenance; the exact guest
   collector utilities must also satisfy the regular-file, single-link, root-owner, executable,
   non-writable, and no-`0o7000`-mode policy (a stock image that does not satisfy it must fail);
4. exact positive disk reserve, absent repository `target/debug`, and a fresh profile/root;
5. integration into the shared evaluator/CLI without weakening the standalone invariants.

Before any provider phase, the integrated runner must additionally consume the opaque witness in
the same process, freshly recheck status/config/boot/mounts/sockets/users/AppArmor/bundle state,
and prove subscription login inside the VM without importing host authentication, distinct
teaching/evaluation homes and state, the retention boundary, a fresh re-audit of this provider-free
receipt, and all provider trace, identity, cost, no-replay, scope-leakage, and stale-influence
checks. Provider authorization must be a separate explicit gate; this module must remain
non-authorizing.

## Provider-free verification

The dedicated adversarial suite covers exact launch flags and stage ordering, default-profile and
command-plan drift, runtime-to-manifest binding, archive extras, bundle
symlinks/hardlinks/permissive modes/auth-like paths, unexpected archive directories, a contaminated
synthetic host home, create-new replay refusal, a complete raw-evidence audit, stale receipts,
extra residue, artifact symlinks/hardlinks, Claude `.credentials.json` and
canonical/symlink/hardlink credential aliases before reads, setuid/setgid/sticky
source/archive/guest modes, exact AppArmor enforce state, provider-capable Claude/Codex/sudo argv
substitution, every incremental checkpoint,
injected post-spawn pipe/thread/`try_wait`/`wait` failures and panic-unwind cleanup with a confirmed
direct-child reap and absent process group, preservation of primary errors when cleanup also fails,
semantically forged/re-digested output, supplementary-group and runtime-group forgeries, a false
bundle mount source, and disagreement between the two hash implementations.
Formatting and strict Clippy are required in addition to the focused tests. All build output is
directed outside the repository, and repository `target/debug` must remain absent.
