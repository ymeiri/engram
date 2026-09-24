# Native strict successor Stage C2B2a — frozen calibration and containment contract

Date: 2026-09-07 (Asia/Jerusalem)

## Status and exact authority boundary

This document freezes a provider-free C2B2a implementation and calibration contract. It does not
freeze C2B2b, accept C2B2, authorize a provider or adapter run, or prove RocksDB persistence.

The research direction remains intact, but production resource values cannot honestly be selected
from an inert probe, the current macOS build or a workload that omits C2B2b's final wire. C2B2a
therefore has two purposes:

1. prove the exact collector containment mechanism with positive and negative kernel evidence; and
2. characterize the exact Linux-arm64 stock-Core backend under conservative safety ceilings.

If accepted, C2B2a may prove only:

> For the exact pinned host tools, guest image/kernel, supervisor and calibration collector, the
> supervisor caused the fixed unprivileged workload to execute inside the frozen cgroup, mount,
> credential, descriptor, filesystem and collector-private network boundaries; every required
> negative limit probe produced the corresponding kernel evidence; the bounded calibration
> journeys produced the required kernel-recorded and explicitly sampled measurements; and every
> workload, VM and disposable-disk resource was reaped.

It proves neither a networkless management guest nor C2B2's persistent journey. It does not prove
hidden close-error propagation, crash recovery, power-loss durability, physical-media sync,
filesystem immutability, secure erasure, hostile-root resistance, complete buffer erasure, a live
Engram store, a correction action, deletion propagation, a coding-agent journey or flagship
completion.

Implementation may begin inside the exact source boundary below. A real VM calibration run remains
blocked until the implementation, deterministic payload build, runtime pins and run plan each pass
their own exact-source review and every preflight gate in this document.

## Authoritative inputs and superseded observations

```text
C2B2 research
  NATIVE_STRICT_SUCCESSOR_STAGE_C2B2_PERSISTENT_CONTAINMENT_RESEARCH_2026-09-06.md
  00735349dd10c8c18414a4e611fc5030a82ae014c9ac95bfd9872b3f9427ece0

C2B2 research review
  NATIVE_STRICT_SUCCESSOR_STAGE_C2B2_PERSISTENT_CONTAINMENT_REVIEW_2026-09-06.md
  f68400713ceef924d2e0f24d642ffa77b61bfc3dd15c7a1ad010a60970ba07cd

Accepted C2B1 report
  NATIVE_STRICT_SUCCESSOR_STAGE_C2B1_ACCEPTED_2026-09-06.md
  75b88affe568d1ab70686f0d58050e3f05ba213c7658ab981b67aeb81d1e82ef

C2B1 frozen design
  NATIVE_STRICT_SUCCESSOR_STAGE_C2B1_MEMORY_ACQUISITION_FROZEN_2026-09-06.md
  04f1be24c758f4a7f52baafab2d9ff27b9fd59241eecc48545deb3e5365600cb

Accepted C2B1 acquisition source
  engram-eval/src/native_successor_semantic/acquisition.rs
  895fa4188bcaeebc907ef0f014f7ed5053ad6b413017ac6402c52f217bcb6d9a

Existing VM foundation source, evidence primitives only
  engram-eval/src/native_vm.rs
  7236583a702a85998e1be5a4c32950df239d32a384d4bf82e2b9dc961c90b69a

Existing VM foundation record
  NATIVE_VM_COLLECTOR_FOUNDATION_2026-09-05.md
  5ad40650a92cde03a35db6017aaa2cf72c1d7deb6dea14b4189e531dc6649981

Workspace manifest
  Cargo.toml
  ec61531f8ffd401211f649c7b91e28a4ee5f0fa1a0fe2f199c63a841c9724f5a

Evaluator manifest
  engram-eval/Cargo.toml
  4176c6a4886492f7d5adaa0bdc3134d7e04bfc8d1536de4d5eb8d4b8985ce2bb

Current workspace lockfile
  Cargo.lock
  0eb924bb9008bb417cb9990081df8429520131aa6b3f7367866c5b805ec474de
```

The research record's historical lockfile digest `874169…`, low-disk gate and deferred-C2A next
step are superseded observations, not inputs to this freeze. Current authoritative state has
accepted C2A/C2B1 evidence, an absent confirmed cache path, positive disk reserve and absent
repository `target/debug`.

## Exact sequencing

```text
C2B2a source implementation
  -> provider-free source/unit/integration review
  -> deterministic Linux-arm64 payload build and independent provenance review
  -> exact guest-image/kernel/tool/run-plan freeze
  -> one non-acceptance final-ELF characterization guest, three fresh filesystems
  -> no source, binary, image or classifier change
  -> three fresh calibration guests, twenty complete journeys per guest
  -> boundary-failure probes and cleanup evidence
  -> C2B2a measurement/acceptance record
  -> C2B2b full-wire/full-semantic implementation freeze
  -> fresh C2B2b holdout guests under every proposed final limit
  -> C2B2b implementation and acceptance
```

C2B2a acceptance is infrastructure and measurement evidence only. C2B2b may be frozen only against
the exact accepted C2B2a identities and full-workload holdout gates. C2B2 is accepted only after
C2B2b, inherited C2A/C2B1 suites, cleanup, provenance, source/binary firewalls and three fresh
exact-source audits are green.

## Management and threat boundary

A fresh direct Lima 2.0.3/VZ instance inside an isolated, create-new `LIMA_HOME` is trusted
management infrastructure. Colima is not in the execution path. The instance is created from one
reviewed local YAML with `--plain`, `vmType=vz`, `arch=aarch64`, exactly two CPUs, exactly 4 GiB of
memory, a 40-GiB root-disk maximum, `containerd=none`, no Rosetta, no video, no host mounts, no
port forwards, no additional networks, no DNS override, no SSH-agent/X11 forwarding and no host
environment propagation. It retains only Lima's management transport and VZ NAT needed for
private management SSH. The host owns `limactl start --foreground` and its process group through
terminal cleanup. The complete input YAML, Lima expanded YAML, generated cloud-init, instance
directory and live guest state are byte/structurally bound; selected scalars are insufficient.
No host directory, user home, authentication path, agent socket, provider state or live Engram data
may be mounted or copied.

The host launches the absolute pinned `limactl` with a cleared environment containing only the
runtime-frozen locale, `PATH`, `SSH=/usr/bin/ssh`, isolated `LIMA_HOME` and an empty owner-only
task-specific home/tmp root. It transfers the hash-bound two-ELF bundle only through
`limactl copy --backend=scp --tty=false`; the guest verifies every byte, installs it under the fixed
root-owned `/opt/engram-c2b2a` directory and removes the transfer copy before execution. The sole
binary control session is the no-TTY/no-preserve-env/no-reconnect/no-autostart equivalent of:

```text
/opt/homebrew/bin/limactl shell --tty=false --workdir=/ INSTANCE \
  sudo -n -- /opt/engram-c2b2a/supervisor
```

Its stdin and stdout are the host protocol's byte streams; stderr is separate and bounded. The
first stdout byte belongs to `HostReady`, any management banner/content is a protocol failure, and
EOF/process status are both required. SCP/SSH use only the create-new key under isolated
`LIMA_HOME`. The runtime freeze must bind the absolute `ssh`, `scp`, `sudo`, install and hash-tool
identities and every exact command token; no shell-expanded or runtime-selected token is permitted.

Every normal collector and every collector allowed to reach `Start` or Core is network-incapable
before `exec`:

- it enters a new network namespace with no usable interface or route;
- every inherited descriptor is closed except the exact standard and supervisor-pipe descriptors;
- no inherited descriptor is a socket; and
- a supervisor-installed seccomp filter denies `socket`, `socketpair` and the unneeded network
  syscall surface for every address family.

Fault-injection probes 17 and 18 are the sole pre-`Start` exceptions to the second and third bullets.
They carry only the exact FD-5 sentinel frozen below through `exec`, are rejected at the first
post-exec descriptor gate, and are killed/reaped before `Start`, fixture construction or Core. This
negative path does not weaken or satisfy the normal containment claim.

This is a collector-private network claim, not a networkless-management-guest claim. The run is
offline after preflight: any image, package or tool download is terminal failure.

Trusted-capable principals are the macOS/VZ host, guest kernel/root and exact guest supervisor.
C2B2a does not defend against any of them, a compromised guest image, or a malicious same-UID host
process racing trusted host files.

## Disk and host-space boundary

The collector database mount is a distinct, freshly created block-device filesystem. It is never
tmpfs, overlay, VirtioFS, 9p, SSHFS, bind-mounted host storage or the guest root filesystem. The
child first makes mount propagation recursively private, detaches every inherited submount and
constructs a new closed-world root. The root is a 4-MiB, 64-inode tmpfs used only during setup and
then remounted read-only. The statically linked collector executable is entered through a
preopened file descriptor and is not visible by path. The only other visible mounts are a private
read-only `proc` mount and the three entries below; there is no visible sysfs, cgroupfs, management
disk, guest root, VirtioFS, device tree or dynamic-loader path. The workload may write only:

- the fixed database filesystem at `/data`;
- `/tmp`, a tmpfs with exact options
  `size=67108864,nr_inodes=1024,mode=0700,uid=65532,gid=65532,nosuid,nodev,noexec,noatime`;
  and
- `/diag`, a dedicated 16,777,216-byte ext4 partition with 512 inodes, mounted
  `nosuid,nodev,noexec,noatime`.

The calibration private data device is one named raw Lima disk of exactly 4,294,967,296 bytes. Under
the isolated `LIMA_HOME`, the host invokes the exact absolute `limactl` identity with
`disk create NAME --size 4GiB --format raw --tty=false`, requires the create-new path
`_disks/NAME/datadisk`, immediately changes that regular file to mode `0600` and rejects any ACL or
extra link. Before attachment it calls `F_PREALLOCATE` with `F_ALLOCATEALL`, `F_PEOFPOSMODE`,
offset zero and the exact length, requires the returned allocated length, calls `ftruncate` to the
exact size and `fsync`, and requires `st_blocks * 512 >= 4,294,967,296`. It binds the open handle,
canonical path, device and inode throughout. The reviewed YAML names this disk once with
`format: false`; any automatic format/mount or different backing path is terminal. It has a
GPT with 512-byte sectors: partition 1 spans sectors 2,048 through 8,353,791 inclusive
(4,276,092,928 bytes) and partition 2 spans sectors 8,353,792 through 8,386,559 inclusive
(16,777,216 bytes). Before every journey, after all prior roles and handles are proven terminal,
both partitions receive new random UUIDs and are formatted by the pinned guest tools: partition 1
as ext4 with 4-KiB blocks, 256-byte inodes and exactly 65,536 inodes, and partition 2 as ext4 with
1-KiB blocks, 128-byte inodes and exactly 512 inodes. These are the exact `mke2fs` argument vectors;
`UUID` and `DEVICE` are bound values, not shell-expanded inputs:

```text
partition 1:
-F -q -t ext4 -b 4096 -I 256 -N 65536 -m 0
-O none,has_journal,ext_attr,dir_index,filetype,extent,64bit,flex_bg,sparse_super,
   large_file,huge_file,dir_nlink,extra_isize,metadata_csum
-J size=64 -E lazy_itable_init=0,lazy_journal_init=0,nodiscard
-U UUID -L ENGRAM_DATA DEVICE

partition 2:
-F -q -t ext4 -b 1024 -I 128 -N 512 -m 0
-O none,has_journal,ext_attr,dir_index,filetype,extent,flex_bg,sparse_super,
   large_file,dir_nlink,extra_isize,metadata_csum
-J size=1 -E lazy_itable_init=0,lazy_journal_init=0,nodiscard
-U UUID -L ENGRAM_DIAG DEVICE
```
The exact guest `mke2fs`, `mke2fs.conf`, `tune2fs`, `mount` and partitioning binaries/configuration
are pinned. `/data` and `/diag` use
`rw,nosuid,nodev,noexec,noatime,nodiratime,data=ordered,errors=remount-ro`; the supervisor binds the
complete effective superblock, feature set, block/inode counts and mount options. Formatting or an
effective-value mismatch is terminal. Immediately after each mount, through already bound root
dirfds, the supervisor calls, in order, `fchown(data_root_fd, 65532, 65532)`,
`fchmod(data_root_fd, 0700)`, `fchown(diag_root_fd, 65532, 65532)` and
`fchmod(diag_root_fd, 0700)` and requires four zero returns. It then verifies both root inodes have
UID 65532, GID 65532 and mode `0700`; `/tmp` must report those same root values from its mount
options. `/data/store` must return `ENOENT` through the bound data dirfd before writer creation.
These values, calls and the absence check are source-bound. A fresh store leaf is never a freshness
substitute. The supervisor binds partition/filesystem UUIDs, device, mount ID, filesystem type,
UID, GID, mode and link count before and after every role.

The direct Lima root disk remains a sparse management disk with an exact 40-GiB logical maximum.
C2B2a does not call it a hard aggregate host-byte quota. Before instance creation the
host must have at least 134,217,728 KiB available; after cleanup it must retain at least
19,427,004 KiB. The input bundle is capped at 512 MiB, retained host evidence at 64 MiB, and the
run is offline against already pinned local images and artifacts, so no image or package download
may consume unbound space during execution.

The security root is created below the pre-attested owner-only Engram evaluation root, never below
`/private/tmp`. Its parent chain is checked before any tool write. At every later binding gate,
regular files and directories are opened without following symlinks and checked with
`acl_get_fd_np(..., ACL_TYPE_EXTENDED)`. The runtime freeze must enumerate every Lima-created socket
or symlink by exact relative path and mode; only those entries use dirfd-relative
`fstatat(..., AT_SYMLINK_NOFOLLOW)` plus `acl_get_link_np`, with type/device/inode/owner/mode bound
before and after the call. Every unexpected entry or ACL, wrong owner/mode, extra regular-file link,
changed identity or preexisting instance/disk/path is terminal. POSIX mode is additional evidence,
not an ACL substitute; same-UID host races remain an explicit nonclaim.

Logical disk size, `SURREAL_ROCKSDB_SST_MAX_ALLOWED_SPACE_USAGE` and a successful write do not prove
aggregate Rocks or host usage. C2B2a records checkpointed filesystem logical/allocated bytes and
file counts, not a complete transient high-water mark. C2B2b retains the 4-GiB safety ceiling until
its full-workload fresh-filesystem holdouts independently justify and validate any other value.

## Calibration-only operational envelope

These are safety ceilings for the calibration runner, not C2B2b production limits:

| Boundary | Calibration ceiling |
|---|---:|
| Guest CPUs | exactly 2 |
| Guest physical memory | exactly 4 GiB |
| Supervisor `memory.max` | 512 MiB |
| Supervisor `memory.swap.max` | 0 |
| Supervisor `pids.max` | 32 |
| Supervisor `memory.oom.group` | 1 |
| Supervisor `cpu.max` | `100000 100000` |
| Supervisor `RLIMIT_NOFILE` | 512 |
| Supervisor `RLIMIT_FSIZE` | 64 MiB |
| Supervisor `RLIMIT_CORE` | 0 |
| Workload-parent `memory.max` | 1 GiB |
| Workload-parent `memory.swap.max` | 0 |
| Guest swap | absent/disabled |
| Workload-parent `pids.max` | 128 |
| Workload-parent `memory.oom.group` | 1 |
| Workload-parent `cpu.max` | `100000 100000` |
| Normal-role `RLIMIT_CPU` | soft=30 seconds, hard=30 seconds; default `SIGXCPU` action |
| CPU-probe `RLIMIT_CPU` | soft=1 second, hard=2 seconds; fixed continuing handler |
| Per-role `RLIMIT_NOFILE` | 256 |
| Per-role `RLIMIT_FSIZE` | 256 MiB |
| Per-role `RLIMIT_CORE` | 0 |
| Per-role wall deadline | 60 seconds |
| Complete journey deadline | 240 seconds |
| Characterization guest deadline | 10 minutes |
| Positive guest deadline, all 20 journeys | 45 minutes |
| Shared boundary guest deadline | 45 minutes |
| Each dedicated boundary guest deadline | 10 minutes |
| Complete ten-instance host-run deadline | 300 minutes |
| Complete characterization-plus-acceptance deadline | 310 minutes |
| Per-role stdout plus stderr | 64 KiB |
| Whole-run retained host evidence | 64 MiB |
| Terminal structural receipt | 256 KiB |

The nested deadlines are independent fail-fast ceilings, not expected durations or retry windows.
The runner contains no sleep, pacing or retry loop and proceeds immediately whenever a gate passes.
The supervisor and workload have separate sibling cgroups. The root supervisor, trusted PID-1
role-init and probe-39 launcher remain in the supervisor cgroup. Workload aggregate limits use one
parent cgroup with per-role children containing only untrusted collector tasks. Every positive
journey uses exactly the workload ceilings in the table; there is no resource grid and no runtime
override. A workload limit event kills and reaps the whole workload cgroup while leaving its
trusted reaper alive, and can never authorize retry on the same disk or intent. Any supervisor
limit event is a host-observed terminal calibration failure and triggers whole-guest cleanup.

Every collector is non-dumpable before and after exec. The supervisor binds
`/proc/sys/kernel/core_pattern`, `/proc/sys/kernel/core_uses_pid`,
`/proc/sys/fs/suid_dumpable` and the role's `/proc/<pid>/coredump_filter`; the latter three are
exactly `0`, `0` and `00000000`. Probe 12 uses two
dedicated fresh guests: one with the exact ordinary pattern `/diag/core.%p.%s` and one with the
exact pipe pattern `|/opt/engram-c2b2a/supervisor --core-counter`. The latter is a fixed,
kernel-only entrypoint in the same accepted supervisor ELF, writes only the fixed
`/diag/core-helper-count` counter and therefore does not add a third payload executable. In both
guests, `PR_SET_DUMPABLE=0`, zero helper invocations and absence of any core artifact are required;
`RLIMIT_CORE=0` alone is never proof.

## Minimal source and package boundary

C2B2a adds a new foundation rather than widening or rewriting `native_vm.rs`:

```text
engram-eval/src/native_c2b2a.rs
engram-eval/src/bin/native-c2b2a.rs
engram-eval/tests/native_c2b2a_foundation.rs
engram-eval/native-c2b2a-payload/Cargo.toml
engram-eval/native-c2b2a-payload/Cargo.lock
engram-eval/native-c2b2a-payload/rust-toolchain.toml
engram-eval/native-c2b2a-payload/.cargo/config.toml
engram-eval/native-c2b2a-payload/src/lib.rs
engram-eval/native-c2b2a-payload/src/contract.rs
engram-eval/native-c2b2a-payload/src/fixtures.rs
engram-eval/native-c2b2a-payload/src/protocol.rs
engram-eval/native-c2b2a-payload/src/rocks.rs
engram-eval/native-c2b2a-payload/src/seccomp.rs
engram-eval/native-c2b2a-payload/src/bin/supervisor.rs
engram-eval/native-c2b2a-payload/src/bin/collector.rs
engram-eval/native-c2b2a-payload/tests/contract.rs
engram-eval/native-c2b2a-payload/tests/protocol.rs
engram-eval/native-c2b2a-payload/tests/rocks.rs
engram-eval/native-c2b2a-payload/tests/seccomp.rs
```

The nested payload manifest declares an empty `[workspace]` table, is unpublished, has its own
lockfile and exact `rust-toolchain.toml` pin to Rust 1.93.0. It is not a member of the broad Engram
workspace. `engram-eval/src/bin/native-c2b2a.rs` is the sole real-run host entrypoint and includes
the host module directly, so no shared CLI or manifest edit is needed through Cargo binary
auto-discovery. The existing
workspace manifest, workspace lockfile, evaluator manifest, `native_vm.rs`, C2A and C2B1 sources
remain byte-for-byte unchanged during the C2B2a foundation implementation.

The payload contains exactly two Linux-arm64 ELF executables:

- a small privileged `supervisor` that does not link SurrealDB or Engram; and
- one unprivileged multicall `collector` with the fixed modes `boundary-probe`, `writer`,
  `contender`, `release-probe` and `reader`.

Mode selection is one exact argv token supplied by the supervisor, never input data. Unknown,
missing or extra argv fails before initialization. The supervisor owns every cgroup, namespace,
mount, pidfd, dirfd, disk, pipe, timer and guest-local witness. The collector owns no management
handle and has no source path that can spawn another process or select another role. Required
pinned-libc/Core/Rocks threads are permitted only by the exact thread rule below.

The standalone collector package directly requests exact
`surrealdb-core = "=2.6.0"` with `default-features = false` and only `kv-rocksdb`. It may use the
minimal Tokio features, hashing, `getrandom` and `libc`. It must not depend on `engram-core`,
`engram-store`, `engram-index`, `engram-mcp`,
`engram-embed`, the high-level `surrealdb` SDK, Clap, Anyhow, HTTP, TLS, WebSocket, JWKS, scripting,
ML, Claude, Codex, Node or provider code.

C2B2a deliberately does not copy, extract, reimplement or claim equivalence to C2A/C2B1's private
production projection. Its fixtures are deterministic structural backend surrogates whose complete
native values and fixed query ASTs are embedded and hash-bound. They contain no live or user data,
no semantic secret and no MAC key. C2B2a therefore cannot prove C2A/C2B1 behavior or set final
C2B2b limits. C2B2b must compile the accepted production projection through one separately frozen
mechanical procedure, bind the complete `engram-core` source dependency, rerun every inherited
differential fixture and validate the complete final wire/workload in fresh holdout guests.

## Guest process containment order

The root supervisor creates and configures one fresh workload-parent cgroup once per positive
journey, characterization subattempt or boundary subattempt. It retains that parent until every
role in the item is reaped.
For each role it creates only a fresh child cgroup beneath that parent. Every `clone3` input is a
zero-filled `sizeof(struct clone_args)` object from the pinned guest headers; only fields named
below are nonzero, and every call uses that exact size. The sequence is:

1. preopen and bind the collector ELF, role-cgroup FD, mount-source handles, role pipes and one
   trusted root-to-role-init `SOCK_SEQPACKET` channel; close every other inheritable descriptor;
2. create a single-threaded, allocation-free role-init with `flags=CLONE_PIDFD | CLONE_NEWNS |
   CLONE_NEWPID | CLONE_NEWNET`, `pidfd` naming the designated zeroed output word and
   `exit_signal=SIGCHLD`; `cgroup=0`, so it atomically inherits the trusted supervisor cgroup, and
   no post-clone cgroup migration is permitted; root immediately closes the role-init endpoint of
   their socketpair and role-init closes the root endpoint before creating any child;
3. the role-init is PID 1 inside the new PID namespace and remains trusted supervisor code; it makes
   mounts private, constructs the closed root and mounts its private proc, but never execs the
   collector and never receives fixture content;
4. from that namespace, role-init creates the collector with `flags=CLONE_INTO_CGROUP |
   CLONE_PIDFD`, `cgroup=role_cgroup_fd`, `pidfd` naming a distinct zeroed output word and
   `exit_signal=SIGCHLD`; the collector therefore enters the role cgroup without migration,
   inherits the namespaces and is PID 2, except for probe 39's exact launcher topology below;
5. role-init retains one child pidfd for `waitid`, transfers a duplicate plus the fixed inner PID
   identity to the root supervisor over the trusted socket, closes every collector copy of every
   management descriptor in the trusted setup child and blocks as the namespace reaper; root alone
   derives the outer PID by requiring one PID in the role cgroup and matching that PID, the pidfd's
   `/proc/self/fdinfo` `Pid`/`NSpid` fields and the inner identity, then binds the mapping;
6. in the collector setup child, set rlimits and construct the exact future `envp`; while privileged,
   drop every capability from the bounding set, clear the ambient set and call `setgroups(0)`; set
   all real/effective/saved GIDs and UIDs to 65532, zero the remaining capability sets and verify
   every `CapInh/CapPrm/CapEff/CapBnd/CapAmb` field is zero;
7. set `PR_SET_PDEATHSIG=SIGKILL` after the credential transition, read it back, and read the expected
   inner parent PID twice: PID 1 normally and the fixed launcher PID for probe 39; then set
   `PR_SET_DUMPABLE=0`, read it back, and set/read `PR_SET_NO_NEW_PRIVS`;
8. install the bootstrap seccomp filter, report fixed structural readiness and block;
9. while blocked, the root supervisor binds both pidfds/start identities, its derived inner/outer
   parent mapping, credentials, cgroup, namespace IDs, complete mountinfo, descriptor types/targets,
   ordinary socket absence or the sole manifest-selected probe-17/18 FD-5 sentinel, seccomp mode,
   non-dumpable state and `no_new_privs` through `/proc/<outer-pid>`;
10. release exactly one `execveat` of the preopened sealed ELF using `AT_EMPTY_PATH`;
11. at the first collector application instruction, before fixture construction, thread creation or
    Core reference, reassert `PR_SET_DUMPABLE=0`, stack the steady-state filter and block;
12. while post-exec blocked, verify `/proc/<outer-pid>/environ` byte-for-byte, repeat every identity,
    parent, mount, descriptor, seccomp and dumpability check; probes 17/18 terminate at the expected
    FD-5 rejection, while every other role receives the fixed `Start` frame; and
13. after role completion, role-init zeroes `siginfo_t` and calls exactly
    `waitid(P_PIDFD, collector_pidfd, &siginfo, WEXITED)`, with no `WNOWAIT`; it validates the
    manifest-selected `si_code`/`si_status`, thereby reaps the child exactly once, proves no other
    namespace child exists and exits. The root performs the same no-`WNOWAIT` call on the
    role-init pidfd, validates its normal status and proves the role child cgroup empty.

The PID-1 role-init is the namespace reaper and prevents collector PID-1 signal semantics from
entering any test. It has no collector control FD and holds no Rocks/store handle. Root-supervisor
and role-init channels are trusted management state. The trusted collector setup child closes every
inherited management endpoint before credential transition; none is available to collector
application code. "Before first dependency reference" excludes only the audited collector setup and
entry stubs. Collector control and data use preopened pipes, never application sockets.

Both seccomp programs are constant, reviewed classic BPF for `AUDIT_ARCH_AARCH64`; an architecture
mismatch kills the process. The bootstrap filter permits only the exact FD/flag form of the single
`execveat`: FD 7, zero pathname bytes in the pinned call site and `AT_EMPTY_PATH` as the sole flag.
It returns `EPERM` for all socket calls and every other exec/spawn form, permits exact
`PR_SET_DUMPABLE=0`, `PR_GET_DUMPABLE`, `PR_GET_PDEATHSIG` and
`seccomp(SECCOMP_SET_MODE_FILTER, 0, ...)` calls needed before the single steady-state stack, kills
on architecture mismatch and otherwise allows the pinned static startup path. A source/ELF firewall
proves that the only code reachable before the steady filter is the static runtime entry plus the fixed filter
installer; any pre-filter syscall outside the reviewed trace fails binary acceptance. The
steady-state filter has the closed allowlist below. `execve`, `execveat`, `fork` and `vfork` return
`EPERM`; `clone3` returns `ENOSYS`; and legacy `clone` is allowed only when the low and high flag
words equal `0x007d0f00` and zero respectively. That is the exact observed pinned-musl pthread mask
(`CLONE_VM|CLONE_FS|CLONE_FILES|CLONE_SIGHAND|CLONE_THREAD|CLONE_SYSVSEM|CLONE_SETTLS|`
`CLONE_PARENT_SETTID|CLONE_CHILD_CLEARTID|CLONE_DETACHED`) with zero exit signal and no namespace
or `CLONE_VFORK` bit. Every other clone returns `EPERM`.

The steady allowlist is exactly: `read`, `write`, `readv`, `writev`, `pread64`, `pwrite64`, `close`,
`close_range`, `dup`, `dup3`, `fcntl`, `ioctl`, `lseek`, `openat`, `newfstatat`, `fstat`, `statx`,
`fstatfs`, `getdents64`, `readlinkat`, `access`, `faccessat`, `faccessat2`, `mkdirat`, `unlinkat`,
`renameat`, `renameat2`, `linkat`, `ftruncate`, `fallocate`, `fsync`, `fdatasync`, `sync_file_range`,
`readahead`, `fadvise64`, `mmap`, `mprotect`, `munmap`, `mremap`, `madvise`, `mincore`, `brk`, `getrandom`,
`rt_sigaction`, `rt_sigprocmask`, `rt_sigreturn`, `sigaltstack`, `futex`, `futex_waitv`, `set_tid_address`,
`set_robust_list`, `rseq`, `membarrier`, `sched_yield`, `sched_getaffinity`, `sched_setaffinity`,
`clock_gettime`, `clock_nanosleep`, `nanosleep`, `gettimeofday`, `getrusage`, `times`, `uname`,
`sysinfo`, `getpid`, `getppid`, `gettid`, `getuid`, `geteuid`, `getgid`, `getegid`, `prlimit64`,
`umask`, `getxattr`, `lgetxattr`, `fgetxattr`, `setxattr`, `lsetxattr`, `fsetxattr`, `removexattr`,
`lremovexattr`, `fremovexattr`, `epoll_create1`, `epoll_ctl`, `epoll_pwait`, `epoll_pwait2`,
`eventfd2`, `timerfd_create`, `timerfd_settime`, `timerfd_gettime`, `ppoll`, `pselect6`, `restart_syscall`,
`tgkill`, `exit` and `exit_group`, plus the exact conditional `clone` rule above. `prctl` permits
only `PR_SET_NAME` and `PR_GET_NAME`. All other syscalls trap as `SIGSYS`; an unexpected trap is a
terminal failed calibration and requires a new reviewed contract identity, never runtime widening.

Network probes receive `EPERM` for `socket`, `socketpair`, `connect`, `bind`, `listen`, `accept`,
`accept4`, `sendto`, `sendmsg`, `sendmmsg`, `recvfrom`, `recvmsg`, `recvmmsg`, `getsockname`,
`getpeername`, `setsockopt`, `getsockopt` and `shutdown`. The boundary suite attempts `AF_INET`,
`AF_INET6`, `AF_PACKET`, `AF_NETLINK`, `AF_VSOCK`, `AF_UNIX` and `socketpair`. It also proves
positive creation and join of one pinned-musl pthread under the exact allowed clone mask, zero
`pids.events:max` in every positive journey, and negative exec, child-process, namespace, mount,
privilege, tracing, module, BPF, perf, keyring, io_uring and userfaultfd probes. Native
affinity/Core/Rocks thread existence and per-class counts are not claimed or inferred. Only
aggregate cgroup `pids.current`, `pids.peak` and `pids.events:max` are retained and bounded by
`pids.max`. The pinned reader must also prove a successful `POSIX_FADV_RANDOM` call under the
`fadvise64` rule.

## Stock Core and configuration boundary

Every collector role is a fresh `execveat`. The supervisor validates the complete exact environment
before the first Core reference and binds its source-order digest. The only 35 `SURREAL_*` entries
are the exact newline-terminated table below, in this order. Its SHA-256 is
`2e53756bdfbc02d2bd7d3dc8fe7ab13541badf8e114ecb48cd446d49d067b092`.

```text
SURREAL_SYNC_DATA=true
SURREAL_ROCKSDB_BACKGROUND_FLUSH=false
SURREAL_ROCKSDB_BACKGROUND_FLUSH_INTERVAL=200
SURREAL_ROCKSDB_THREAD_COUNT=2
SURREAL_ROCKSDB_JOBS_COUNT=2
SURREAL_ROCKSDB_MAX_OPEN_FILES=128
SURREAL_ROCKSDB_BLOCK_SIZE=65536
SURREAL_ROCKSDB_WAL_SIZE_LIMIT=64
SURREAL_ROCKSDB_MAX_WRITE_BUFFER_NUMBER=2
SURREAL_ROCKSDB_WRITE_BUFFER_SIZE=33554432
SURREAL_ROCKSDB_TARGET_FILE_SIZE_BASE=67108864
SURREAL_ROCKSDB_TARGET_FILE_SIZE_MULTIPLIER=2
SURREAL_ROCKSDB_MIN_WRITE_BUFFER_NUMBER_TO_MERGE=2
SURREAL_ROCKSDB_FILE_COMPACTION_TRIGGER=4
SURREAL_ROCKSDB_COMPACTION_READAHEAD_SIZE=4194304
SURREAL_ROCKSDB_MAX_CONCURRENT_SUBCOMPACTIONS=2
SURREAL_ROCKSDB_ENABLE_PIPELINED_WRITES=false
SURREAL_ROCKSDB_ENABLE_BLOB_FILES=false
SURREAL_ROCKSDB_MIN_BLOB_SIZE=4096
SURREAL_ROCKSDB_BLOB_FILE_SIZE=268435456
SURREAL_ROCKSDB_BLOB_COMPRESSION_TYPE=none
SURREAL_ROCKSDB_ENABLE_BLOB_GC=false
SURREAL_ROCKSDB_BLOB_GC_AGE_CUTOFF=0.25
SURREAL_ROCKSDB_BLOB_GC_FORCE_THRESHOLD=1.0
SURREAL_ROCKSDB_BLOB_COMPACTION_READAHEAD_SIZE=0
SURREAL_ROCKSDB_BLOCK_CACHE_SIZE=33554432
SURREAL_ROCKSDB_ENABLE_MEMORY_MAPPED_READS=false
SURREAL_ROCKSDB_ENABLE_MEMORY_MAPPED_WRITES=false
SURREAL_ROCKSDB_KEEP_LOG_FILE_NUM=3
SURREAL_ROCKSDB_STORAGE_LOG_LEVEL=warn
SURREAL_ROCKSDB_COMPACTION_STYLE=level
SURREAL_ROCKSDB_DELETION_FACTORY_WINDOW_SIZE=1000
SURREAL_ROCKSDB_DELETION_FACTORY_DELETE_COUNT=50
SURREAL_ROCKSDB_DELETION_FACTORY_RATIO=0.5
SURREAL_ROCKSDB_SST_MAX_ALLOWED_SPACE_USAGE=0
```

The remaining environment is exactly `LANG=C`, `LC_ALL=C`, `TZ=UTC`, `RUST_BACKTRACE=0` and
`TMPDIR=/tmp`, followed by the 35 lines above; `PATH`, `HOME`, every proxy/certificate variable and
every other name beginning `SURREAL_` are absent. The binary embeds the same table and rejects a
byte mismatch before Core reference. Configuration values are inputs, never calibration outputs;
changing one requires a new reviewed contract identity. A runtime plan carries only this table's
digest and cannot override an entry. Malformed-value fallback and CPU/memory-derived defaults are
never relied upon.

Stock Core offers no complete runtime configuration getter. C2B2a may claim only configuration
inference from a fresh process, exact environment, exact source and exact binary. If C2B2b requires
runtime option attestation or the stock path cannot remain within the selected envelope, the
separately reviewed narrow patch fallback becomes mandatory.

`SURREAL_ROCKSDB_BACKGROUND_FLUSH=false` disables only Core's explicit background-flush loop.
Direct Core attempts to initialize a process-global affinity pool configured for
`max(8, visible CPUs)`, while individual affinity-worker spawn failures are ignored; native Rocks
may create workers. C2B2a claims only the aggregate cgroup PID measurements above, not worker
existence or per-class counts.

The target is exactly `aarch64-unknown-linux-musl`; both ELFs are static PIE with no interpreter.
The payload pins Rust/Cargo 1.93.0 and the already installed musl-cross GCC/G++ 14.2.0 and binutils
2.44 identities. `.cargo/config.toml` fixes the linker and archiver and the sole entropy cfg
`--cfg getrandom_backend="linux_getrandom"`. The accepted binary must import/use the Linux
`getrandom` syscall and contain no custom, `/dev/urandom`, RDRAND, RNDR or fallback backend. The
accepted macOS `getentropy` evidence does not transfer.

## Calibration workload and measurements

Three fresh guests execute strictly serially. Each guest executes the same embedded ordered
`CalibrationCaseManifestV1[20]` below, one newly formatted filesystem and fresh role set per entry.
Every positive entry must succeed on all three guests. Every attempt, failure, timeout and complete
measurement is retained; analysis cannot select only survivors. A post-authorization failure fails the
complete run and disposes the disk without reuse. The runtime plan cannot add, remove, reorder or
override a case, fixture, resource, command, path or expected category.

| ID | Frozen structural fixture | Rows/shape | Expected |
|---:|---|---|---|
| 1 | `sparse_a_open` | nine tables; A-shaped sparse surrogate | `Success` |
| 2 | `sparse_b_open` | nine tables; B-shaped sparse surrogate | `Success` |
| 3 | `empty_generation` | nine empty table arrays | `Success` |
| 4 | `one_each` | one bounded row in each table | `Success` |
| 5 | `memory_item_64` | 64 rows only in `memory_item` | `Success` |
| 6 | `correction_proposal_32` | 32 rows only in `correction_proposal` | `Success` |
| 7 | `forget_receipt_32` | 32 structural rows only in `memory_forget_receipt` | `Success` |
| 8 | `work_project_8` | 8 rows only in `work_project` | `Success` |
| 9 | `work_task_64` | 64 rows only in `work_task` | `Success` |
| 10 | `git_repository_16` | 16 rows only in `git_repository` | `Success` |
| 11 | `local_checkout_32` | 32 rows only in `local_checkout` | `Success` |
| 12 | `monorepo_component_64` | 64 rows only in `monorepo_component` | `Success` |
| 13 | `project_repository_link_64` | 64 rows only in link table | `Success` |
| 14 | `scalar_heavy` | fixed 1,048,576-byte ASCII scalar split across 16 rows | `Success` |
| 15 | `vector_heavy` | 4,096 fixed eight-byte scalar elements across 16 rows | `Success` |
| 16 | `object_key_heavy` | 4,096 fixed key/value pairs across 16 rows | `Success` |
| 17 | `depth_heavy` | 64 rows, each exactly 32 object levels deep | `Success` |
| 18 | `raw_2mib` | exact 2,097,152-byte canonical fixture encoding | `Success` |
| 19 | `sparse_a_close` | byte-identical to case 1 | `Success` |
| 20 | `sparse_b_close` | byte-identical to case 2 | `Success` |

For every row, `case_id=fixture_id=ID`, `profile_id=1`, the Rocks-table digest is the one frozen
above, and the resource tuple is exactly the complete calibration ceiling table. These are the only
positive cases.

The fixture module contains no generic generator input. It embeds the exact UUIDs, strings, native
value constructors, row order and fixed parsed ASTs for all cases. Cases 19 and 20 must have the
same canonical fixture bytes as cases 1 and 2. Before payload acceptance, the emitted canonical
fixture bytes, generator source, parsed AST debug-independent canonical form and complete ordered
case manifest receive exact hashes in a reviewed build record. C2A/C2B1 cap-plus-one and semantic
fixtures remain outside C2B2a; they return only in C2B2b's full inherited suite.

Every role uses only the literal origin `rocksdb:/data/store`, namespace `engram` and database
`main`. The writer parses and source/binary-binds the accepted 572-byte nine-table insert query
whose SHA-256 is `9f6c41b6c7d0006db0a03f29fd951bb3c81df19946748faaac768af013c98a05`.
The reader parses and binds the accepted 570-byte nine-table read query whose SHA-256 is
`cf1034a335af240c0726e5385207acc4e4e49273dc016891138c863e8ba096f5`. The writer requires the
exact nine empty-array outcomes after commit. The fresh reader requires one exact outer object with
the nine fixed arrays, normalizes only the structural surrogate subset, and compares the complete
canonical bytes to the independently reconstructed embedded fixture. A digest-only comparison is
insufficient and no canonical bytes leave the guest.

## Closed role and host control protocols

The supervisor and collector embed the same case/probe manifests and Rocks-table digest. Collector
descriptors after setup are exactly: 0 read-only `/dev/null`; 1 bounded stdout pipe; 2 bounded
stderr pipe; 3 supervisor-to-collector role control; 4 collector-to-supervisor role result; and, in
the trampoline only, 7 the `O_PATH|O_CLOEXEC` collector ELF used by `execveat`. Ordinarily,
descriptors 5, 6 and every descriptor above 7 are closed. Probes 17 and 18 alone retain exactly one
non-`CLOEXEC` sentinel as FD 5 and still close FD 6 and every FD above 7. Probe 17's FD 5 is the
`O_RDONLY|O_NOFOLLOW` regular file `/diag/inherited-fd-sentinel`; probe 18's FD 5 is one unconnected
`socket(AF_UNIX, SOCK_SEQPACKET, 0)`. Before probe 17 role setup, the supervisor creates that exact
empty file with mode `0600` through the bound diag dirfd, `fsync`s and closes it. With FDs 0 through
4 and 7 already occupied and 5/6 free, the selected `openat` or `socket` call must return FD 5
directly; any other result fails before `exec`. The supervisor verifies the exact type, flags,
mount/path and sole-open-file-description identity before `exec`.

For those two manifest tuples only, the pre-exec gate requires that exact FD 5 and no other
deviation, but grants authority only to execute into the negative post-exec gate. The ordinary
post-exec inventory then observes and rejects FD 5 before `Start`; it can never authorize a started
role or success witness. Every other tuple rejects FD 5 before `exec`. Roles receive no semantic
data, path or string over IPC.

`C2B2aRoleControlV1` and the distinct host/supervisor `C2B2aHostControlV1` use the same 160-byte
big-endian header followed by a message-specific fixed payload of at most 256 bytes:

```text
0..8      ASCII magic "ENGC2A01"
8..10     u16 version = 1
10..12    u16 kind
12..16    u32 sequence
16..18    u16 case_or_probe_id
18..20    u16 fixture_or_subattempt_id
20..22    u16 profile_id = 1
22..24    u16 role_id: 0 supervisor, 1 boundary, 2 writer, 3 contender,
                         4 release, 5 reader, 6 journey aggregate
24..26    u16 status/category
26..28    u16 flags = 0
28..32    u32 payload_len
32..64    32-byte run nonce
64..96    32-byte contract SHA-256
96..128   32-byte applicable manifest SHA-256
128..160  SHA-256 of the exact payload, or SHA-256(empty)
```

Every payload integer is unsigned big-endian; fixed byte arrays have the order shown. Each
unidirectional stream starts at sequence 1 and increments by exactly one for every accepted frame;
sequences never reset inside a role or instance channel. Status is zero on every progress frame. A
successful `RoleTerminal` has status zero. A failed role `Abort` has the exact nonzero category in
both its header and sole four-byte payload. Every fixed binding is identical across a stream. On
role streams, every case/probe, fixture/subattempt, profile and role ID is nonzero and must equal the
manifest-selected tuple. On host streams, `HostStart`, `HostReady`, `TerminalReady`, `Challenge` and
`Terminal` require zero item IDs and role zero; `Measurement` requires the exact nonzero tuple and
role below. A wrong sequence, duplicate, missing, out-of-state, wrong
direction/length/digest/binding, nonzero flag, trailing byte, early EOF, late byte or additional
frame is terminal `ProtocolFailure`.

On role control, kinds are exactly `Start=1`, `Ready=2`, `Continue=3`, `WriterCommitted=4`,
`LockObserved=5`, `ExpectedLockRejected=6`, `HandlesDropped=7`, `RoleTerminal=8` and `Abort=12`.
All progress and successful-terminal payloads are empty. For positive journeys,
`case_or_probe_id=fixture_or_subattempt_id=1..20`; for a boundary role,
`case_or_probe_id=probe_id=1..41` and `fixture_or_subattempt_id` is the one-based ordered call within
that probe. It is 1 for every single-call probe. For characterization,
`case_or_probe_id=1` and `fixture_or_subattempt_id=1..3`. The exact legal frame sequences are:

```text
writer     S>C Start; C>S Ready; S>C Continue; C>S WriterCommitted;
           C>S LockObserved; S>C Continue; C>S HandlesDropped;
           S>C Continue; C>S RoleTerminal; C>S EOF; S>C EOF
contender  S>C Start; C>S Ready; S>C Continue; C>S ExpectedLockRejected;
           C>S RoleTerminal; C>S EOF; S>C EOF
release    S>C Start; C>S Ready; S>C Continue; C>S LockObserved;
           S>C Continue; C>S HandlesDropped; S>C Continue;
           C>S RoleTerminal; C>S EOF; S>C EOF
reader     S>C Start; C>S Ready; S>C Continue; C>S RoleTerminal;
           C>S EOF; S>C EOF
boundary-R S>C Start; C>S Ready; S>C Continue; C>S RoleTerminal;
           C>S EOF; S>C EOF
boundary-K S>C Start; C>S Ready; S>C Continue; C>S EOF; S>C EOF;
           no RoleTerminal and the exact manifest-selected pidfd/cgroup evidence
```

Within each row, the textual order is the total inter-direction order. Role `Abort` may replace only the
next sender frame after a locally observed terminal failure; it is followed immediately by that
sender's EOF and the receiver's EOF. A manifest-selected supervisor-terminal boundary attempt uses
only one of these four exact prefixes: probe 09 ends after `Start/Ready/Continue` and the overflow;
probes 10, 11 and 14 end after `Start/Ready/Continue/RoleTerminal`, followed respectively by the
invalid extra output or exact post-terminal link-count rejection; probes 15 through 18 end before
`Start`: probes 15 and 16 fail before collector creation at the device/mount gate, while probes 17
and 18 fail at the post-exec descriptor gate. After every supervisor-terminal observation, the
supervisor closes its role-input writer, kills any live role, accepts EOF on all three role-output
pipes and reaps the role. All other non-host attempts use `boundary-R` or `boundary-K` exactly as
the manifest states. Host-terminal probes 40 and 41 create no role channel. No other transition,
prefix or half-close is legal.

Stdout and stderr are distinct unframed bounded pipes and are empty for every positive role and
every boundary subattempt except probes 09 and 11. After its last role-control write, a normally
terminating collector closes FD 4, then FD 1, then FD 2 and exits; the supervisor permits any kernel
readiness order but requires the terminal frame before all three EOFs, zero unexpected bytes and
reap only after all EOFs. A kernel-terminal role emits no terminal frame and all three EOFs may
arrive in any order after the exact signal evidence. Probe 09 retains the first 65,536 stdout bytes,
observes but does not retain the 65,537th byte, closes the input writer, kills and reaps the role and
rejects any further byte. Probe 10 places its sole extra byte on FD 4 after `RoleTerminal`; probe 11
closes FD 4 after `RoleTerminal` and then places its sole byte on FD 1. Any other content, missing
EOF, write after the named byte, or stdout-plus-stderr count above 65,536 is terminal.

Host control uses only `Abort=12`, `HostStart=19`, `HostReady=20`, `Measurement=21`,
`Challenge=22`, `Terminal=23` and `TerminalReady=24`. It is a separate bounded pipe carried by the
already trusted management transport, not a collector descriptor. Each instance has exactly one
bidirectional channel. The host first sends `HostStart` as host-to-supervisor sequence 1 with zero
IDs/role/status and this exact eight-byte big-endian payload:

```text
0..2  u16 phase_code
2..4  u16 instance_class_code
4..6  u16 instance_ordinal
6..8  u16 reserved = 0
```

Its header conveys the fresh run nonce and exact contract and phase-schedule manifest bindings; its
payload digest binds the three selector fields. The supervisor accepts no role creation before that
frame. It rejects every tuple not matching exactly one compiled table row or whose class does not
match its phase-specific ordinal, makes the accepted tuple immutable channel state, and returns
`HostReady` as supervisor-to-host sequence 1 with identical bindings and the byte-identical
eight-byte payload. Every later frame is accepted only by the selected automaton and under the
selected phase-schedule manifest digest. The closed tuples are:

```text
phase  class  ordinal  name   selected automaton
1      1      1        C01    three-subattempt characterization
2      101    1        P01    positive cases 01..20
2      102    2        P02    positive cases 01..20
2      103    3        P03    positive cases 01..20
2      201    4        B01    shared boundary schedule
2      212    5        B12O   probe 12 ordinary-pattern subcase
2      213    6        B12P   probe 12 pipe-pattern subcase
2      215    7        B15    dedicated probe 15
2      216    8        B16    dedicated probe 16
2      240    9        B40    host-terminal probe 40
2      241    10       B41    host-terminal probe 41
```

Phase 1 is the single-entry `CharacterizationScheduleV1`; phase 2 is the complete ten-entry
`AcceptanceScheduleV1`. Both tables and their digests are compiled into the host and supervisor.
The exact host runner, not a plan or command-line value, emits only these tuples in this order.

For C01, the supervisor sends exactly 21 `Measurement` frames: seven for each of three serial
characterization subattempts, ordered first by subattempt ID and then by scope. Each header carries
case ID 1 and the matching subattempt ID. These frames and C01's terminal are retained in a distinct
non-acceptance characterization transcript.

For a positive acceptance instance, the supervisor sends exactly 140 `Measurement` frames: seven
for each of twenty cases, ordered first by case ID and then by scope. Their `(scope, role)` pairs are
exactly
`(1,2)` writer-lock, `(2,3)` contender-terminal, `(3,2)` writer-dropped, `(4,4)` release-lock,
`(5,4)` release-dropped, `(6,5)` reader-terminal and `(7,6)` journey-aggregate. Each header carries
the matching case and fixture IDs.

For a boundary instance, the supervisor sends exactly one scope-8, role-1 `Measurement` per
completed subattempt, including supervisor- and kernel-terminal subattempts. Its header carries the
probe ID and one-based subattempt ID. Probe 12 has two subattempts in two instances; probes 32, 33,
35 and 36 have respectively 2, 14, 8 and 4 ordered subattempts; every other non-host-terminal probe
has one. Measurements retain the uninterrupted supervisor-to-host sequence after `HostReady`.

On success, the exact instance-specific measurement cardinality is followed by one `TerminalReady`
at the next supervisor sequence. On structured failure, all complete preceding measurements are
followed immediately by `TerminalReady`; no later scheduled measurement is permitted. Its payload
is the exact 32-byte eight-field state frozen below and its header status equals
`primary_category`. After validating and retaining that state, the host obtains a fresh challenge
and sends it as host-to-supervisor sequence 2 with zero IDs/role/status and a 32-byte payload, then
closes its write side. The supervisor returns one `Terminal` at its next sequence number with a
64-byte payload comprising the challenge followed by the byte-identical committed state. Its header
status again equals `primary_category`; it then closes its write side. The host requires the exact
echo, committed state and EOF with no late byte.

A role `Abort` carries one four-byte category. A host-stream `Abort` carries the complete 32-byte
state, uses role zero and either the active item tuple or all-zero item IDs before any item, and has
header status equal to `primary_category`. It may replace only the sender's next legal frame when
that sender cannot reach `TerminalReady`; the sender then closes its write side and no Challenge or
Terminal is legal. Such an unchallenged path is retained failure evidence and can never authorize
acceptance. The manifest digest in role frames is the complete positive-case or boundary-probe
manifest as applicable, or `CharacterizationManifestV1` for C01. In host frames it is the selected
complete characterization or acceptance schedule manifest. The contract digest and run nonce are
identical in every stream for that instance.

The complete host-channel cardinality is therefore fixed:

```text
instance class          measurements  TerminalReady seq  Terminal seq  authority
C01                            21               23              24      non-acceptance
P01/P02/P03 each             140              142             143      acceptance
B01                           60               62              63      acceptance
B12O/B12P/B15/B16 each        1                3               4      acceptance
B40/B41                       0                -               -      acceptance
acceptance campaign total    484                -               -      acceptance
```

Every instance has host-to-supervisor `HostStart` sequence 1. C01 and each of the eight normal
acceptance instances also have `Challenge` sequence 2; no other host-to-supervisor frame is legal
on success.

`Measurement` has exactly 256 bytes: thirty-two `u64` fields in the order frozen below.
`TerminalReady` has exactly 32 bytes: eight `u32` fields. `Terminal` has exactly 64 bytes: the echoed
32-byte challenge followed by the same eight fields. `HostStart` and `HostReady` each have the exact
eight-byte selector payload above. Host-terminal probes 40 and 41 use the exact host-runner receipt
frozen below because their purpose destroys or faults the guest control channel. They receive
`HostStart` and emit `HostReady`, but have no `TerminalReady`, `Challenge` or `Terminal`; any such
frame is invalid. No generic deserializer, map, UTF-8 string, path, row, key or variable-length
collection exists in either protocol.

For each instance, the macOS host obtains the run nonce through one direct `getentropy` call of
exactly 32 bytes before `HostStart`. It rejects equality with any earlier nonce or challenge in this
campaign. Only after a valid `TerminalReady` does it obtain the challenge through a separate direct
`getentropy` call and reject any campaign reuse or equality with the instance nonce. A nonzero,
interrupted or failing return is terminal and is not retried. The Linux supervisor obtains each
fresh filesystem UUID's sixteen random bytes through one direct `getrandom` call; a short,
interrupted or failing return is also terminal and is not retried.

A normal unit/integration test cannot start Lima. The only real-run entrypoint requires exact
contract, payload, image and plan digests and an explicit provider-free calibration phase token.
The plan contains identities and expected hashes only; it contains no executable, argv, cwd,
environment, path, limit or selector field. The protocol selector above is generated solely by the
compiled two-phase schedule and is never a shell token, plan value or caller-selected override.

For every role, journey and boundary subattempt, the supervisor records only the exact structural
measurements encoded below: cgroup current/peak and selected event counters, CPU counters,
checkpointed filesystem and descriptor observations, tmpfs/diagnostic use, pipe byte counts and
scope elapsed time. No unencoded event counter is claimed as retained evidence.

The thirty-two `Measurement` payload fields are, in order: `scope_code`, `memory_current_bytes`,
`memory_peak_bytes`, `memory_events_oom`, `memory_events_oom_kill`, `swap_current_bytes`,
`swap_peak_bytes`, `swap_events_max`, `pids_current`, `pids_peak`, `pids_events_max`,
`cpu_usage_usec`, `cpu_user_usec`, `cpu_system_usec`, `cpu_nr_periods`, `cpu_nr_throttled`,
`cpu_throttled_usec`, `filesystem_logical_bytes`, `filesystem_allocated_bytes`,
`filesystem_inode_count`, `filesystem_file_count`, `tmpfs_bytes`, `tmpfs_inode_count`,
`diagnostic_bytes`, `diagnostic_inode_count`, `stdout_bytes`, `stderr_bytes`,
`role_control_input_bytes`, `role_control_output_bytes`, `descriptor_count`,
`descriptor_type_bitmap` and `monotonic_elapsed_ns`. Scope codes 1 through 7 are exactly the seven
positive-case measurement positions above; scope code 8 is the final boundary-probe measurement.
Checked saturation is forbidden; an unreadable or overflowing counter fails the case.

Scopes 1 through 6 read the named fresh role-child cgroup; scope 7 reads the one fresh workload
parent for the complete journey; scope 8 reads the one fresh workload parent for that subattempt.
The cgroup is created before its first process, so every peak/event baseline is zero. Current fields
are the exact endpoint values; peak fields are kernel-maintained peaks since cgroup creation; event
fields are the exact `oom`, `oom_kill`, swap `max` and pids `max` counters. CPU fields are the exact
named `cpu.stat` values. No sampling is substituted for a kernel peak.

Every elapsed field is checked `end-start` nanoseconds from Linux `CLOCK_MONOTONIC_RAW`. Scope
boundaries are exact:

```text
1  full writer dispatch-Continue write -> validated LockObserved while writer is blocked
2  full contender Continue write -> expected rejection, all EOFs and contender reap
3  full writer drop-Continue write -> validated HandlesDropped while writer is blocked
4  full release Continue write -> validated LockObserved while release role is blocked
5  full release drop-Continue write -> validated HandlesDropped while release role is blocked
6  full reader Continue write -> validated RoleTerminal, all EOFs and reader reap
7  post-format filesystem binding -> all journey roles reaped and workload parent empty
8  full boundary Continue write, or pre-Start fault injection -> expected evidence and group reap
```

Probes 15 and 16 fail before collector creation. Their workload parent exists but has never
contained a task. Their scope-8 `Measurement` sets `scope_code=8`, retains the checked elapsed time
from the start of the binding gate through completed guest cleanup, and sets each of the other
thirty fields to zero. Zero in those fields means inapplicable only for these two probe IDs; any
nonzero value fails the probe. This prevents an invalid data device or mount namespace from being
treated as a valid filesystem measurement.

Probes 17 and 18 reach the post-exec, pre-`Start` descriptor checkpoint. Probe 17 intentionally
retains the exact read-only FD-5 regular descriptor for `/diag/inherited-fd-sentinel`; probe 18
retains the exact FD-5 `AF_UNIX` socket. The supervisor records the complete rejecting
inventory as that scope's sole descriptor checkpoint, then kills/reaps the collector and records
the remaining cgroup, filesystem and pipe fields by the ordinary scope-8 rules. The injected
descriptor contributes type bit 7 for probe 17 and type bit 14 for probe 18. Beyond the audited
collector entry stub in step 11, no manifest-selected role action, fixture construction or Core
reference occurs in either probe.

The supervisor traverses `/data` from its bound dirfd without following symlinks at each endpoint.
`filesystem_logical_bytes` is the checked sum of `st_size` over distinct regular-file inodes;
`filesystem_allocated_bytes` is the checked sum of `st_blocks * 512` over each distinct reachable
inode including the root and directories; `filesystem_inode_count` includes that same distinct set;
and `filesystem_file_count` counts non-directory entries. A symlink, special file, device change or
traversal race fails the gate. Tmpfs and diagnostic byte/inode fields are the exact checked used
values from `statfs`: `(f_blocks-f_bfree)*f_frsize` and `f_files-f_ffree`.

Stdout, stderr and role-control counts include every observed byte from role creation through the
scope endpoint; EOF contributes zero. Scope 7 uses checked sums across all journey roles. Scope 8
uses the current subattempt. `descriptor_count` is the maximum collector descriptor count across
post-exec, every progress-frame and pre-Continue checkpoint in the scope; the bitmap is their union.
Its bits are exactly:

```text
0 /dev/null character FD       1 pipe read end          2 pipe write end
3 regular file below /data     4 directory below /data  5 regular file below /tmp
6 directory below /tmp         7 regular file /diag     8 directory /diag
9 anon_inode eventfd          10 anon_inode eventpoll   11 anon_inode timerfd
12 read-only procfs FD         13 other anon_inode       14 socket
15 block device               16 other character FD     17 other regular file
18 other directory            19 unknown                20..63 reserved
```

Bits 13 through 63 are forbidden and must remain zero, except that probe 18 requires exactly the
expected injected socket contribution at bit 14 and forbids every other such bit. Classification
uses `fstat`, `fdinfo`, bound mount IDs and link targets; ambiguity fails. Scope 7 counts the maximum
simultaneous descriptors across all live collectors and unions their bits; it reads parent cgroup
counters directly rather than reducing role counters. Filesystem/tmpfs/diagnostic fields are the
scope-7 endpoint values and pipe counts are checked journey sums.

Kernel-maintained memory and PID peaks are identified as complete kernel peaks. Descriptor and
filesystem observations are checkpointed maxima and are never called complete, never used to lower
a hard cap and never treated as a transient-allocation proof. Raw rows, canaries, dependency error
strings, store paths, source text, keys and content-derived digests never cross guest-to-host IPC or
enter persisted receipts. The guest may retain private content only until its local witness is
consumed and cleanup begins.

## Lock and close calibration

Stock Core does not expose a stable typed Rocks lock error. Generic open failure is never lock
proof. An exploratory, non-acceptance predecessor may suggest candidate UTF-8 error bytes. The exact
length, bytes and SHA-256 must then be embedded in reviewed source before the deterministic payload
build. Matching is byte-for-byte UTF-8 equality with no trimming, normalization, wildcard, case
fold, path substitution or alternate spelling; the store path is fixed as `/data/store`. The
resulting final collector ELF is characterized unchanged under the pinned image and `C` locale.

`CharacterizationManifestV1` has exactly three ordered entries, IDs 1 through 3. Each reformats both
partitions, uses calibration case 1 `sparse_a_open`, and performs the complete writer/contender/
writer-drop/release-probe/reader lifecycle below. The role tuple is `(case_id=1, subattempt_id=ID)`.
The contender must produce the embedded exact bytes and the writer-side kernel witness must be
valid; raw error bytes never enter host IPC. Each entry emits the seven ordinary role/scope
measurements. Thus the sole C01 guest emits exactly 21 measurements, commits the successful terminal
state below with three completed subattempts, completes the ordinary challenge/Terminal handshake,
and is then shut down and proven stably absent before P01. Its transcript, terminal, host cleanup
state and exact manifest digest are retained as characterization evidence. A missing, additional or
different frame, observation or byte is failure. C01 and its 21 measurements are excluded from all
acceptance journey, probe, measurement and subattempt totals.

The exact same ELF is then used unchanged for every acceptance guest. Characterization never derives
or edits a classifier at runtime and never counts as acceptance evidence. Any byte mismatch is
`ContenderFailure` and requires a source edit, new source and ELF identities, complete
recharacterization and a fresh acceptance campaign; no existing attempt survives. Acceptance
requires all of:

- exact `LOCK` device and inode;
- a matching `F_GETLK` or `/proc/locks` entry owned by the live writer PID;
- unchanged writer PID/start identity and path binding;
- contender Core-open failure matching the one exact target/binary-specific byte string entirely
  inside the guest;
- disappearance of the lock after every datastore/transaction handle is dropped while the writer
  remains alive;
- absence of writer descriptors into the data mount;
- release-probe open and independently observed lock acquisition;
- disappearance of the release-probe lock and mount descriptors after it drops every handle while
  it remains alive; and
- reap of the release probe, then reap of the writer, followed only then by fresh-reader open.

Only the boolean categorical observation leaves the guest. If exact kernel attribution and the
contained template cannot be made stable, the stock path fails calibration and the narrow typed
patch is required. The raw error is never formatted into outward evidence.

The exact lifecycle is writer write/ack/lock; contender reject and reap; writer drops every handle
but stays alive; prove writer lock and mount descriptors absent; release probe opens and its lock is
observed; release probe drops every handle but stays alive; prove its lock and mount descriptors
absent; reap release probe; reap writer; then start the fresh reader. Every role source and final
ELF is firewalled against `.restart(` and `.shutdown(`.

Neither `Datastore::restart()` nor `Datastore::shutdown()` supplies close authority. Stock Rocks
logs and discards some WAL/memtable flush errors. Calibration can support only observable handle
release and ordinary close/reopen behavior; it cannot upgrade hidden close faults into successes.

## Required boundary-failure probes

Separate disposable runs must prove:

- cgroup OOM classification from `memory.events`, with whole-group kill/reap;
- swap denial;
- PID exhaustion from `pids.events`;
- CPU throttling and cumulative CPU-limit classification;
- descriptor exhaustion;
- `RLIMIT_FSIZE` and aggregate multi-file ENOSPC behavior;
- output overflow, trailing output and late-output rejection;
- core-dump absence;
- mount/path/symlink/hardlink/device/overlay/share escape rejection;
- unexpected inherited descriptor and socket rejection;
- every forbidden socket-family attempt denied by seccomp;
- role signal, timeout and parent-death handling;
- guest-supervisor loss causing host-side terminal cleanup; and
- cleanup failure preserved separately from the primary categorical failure.

Each classification uses kernel evidence such as cgroup event files, pidfd siginfo, filesystem
exhaustion, seccomp state and timer state. Exit code or signal alone is insufficient. No negative
probe can mint a success witness or authorize retry.

The supervisor embeds `BoundaryProbeManifestV1` in the exact order below. The authenticated
`(probe_id, subattempt_id)` tuple in the `Start` frame is the sole selector accepted by
`boundary-probe`; the manifest maps each tuple to exactly one action, and no argv, environment or
plan field can select one. Single-call probes require subattempt 1. Isolation code `F` means the
shared boundary guest plus a newly formatted data and diagnostic filesystem and a fresh role
cgroup. `G` means a dedicated fresh guest, disk, filesystems and roles. Termination code `R` is
`boundary-R`, `K` is `boundary-K`, `S` is the exact supervisor-terminal prefix above, and `H` is the
host-terminal receipt below. An asterisk means the named calls are ordered fresh-role subattempts;
every one is required and produces its own measurement before the row is complete.

```text
ID  name                    iso  term  exact expected observation
01  memory_oom              F    K     oom and oom_kill deltas; whole-cgroup reap
02  swap_absent             F    R     guest swap absent; memory.swap current/max/events bound
03  pids_exhaustion         F    R     pthread EAGAIN at pids.max; max delta; group reap
04  cpu_throttle            F    R     nr_throttled and throttled_usec deltas
05  cpu_rlimit              F    K     handled soft SIGXCPU then hard SIGKILL; CPU evidence
06  nofile                  F    R     EMFILE at exact role limit; descriptor inventory bound
07  fsize                   F    K     default SIGXFSZ at exact limit; bounded file size
08  filesystem_enospc       F    R     ENOSPC on fresh bounded data filesystem
09  stdout_overflow         F    S     exactly 65,536 bytes plus one attempted byte
10  trailing_output         F    S     one byte immediately after RoleTerminal
11  late_output             F    S     result EOF followed by one stdout byte
12  core_absent             G    K     ordinary and pipe-pattern subcases; no core/helper call
13  symlink_escape          F    R     openat(O_NOFOLLOW|O_RDONLY|O_CLOEXEC) returns ELOOP
14  hardlink_escape         F    S     post-RoleTerminal link-count binding rejection
15  wrong_device            G    S     pre-Start device and UUID binding rejection
16  writable_mount_escape   G    S     pre-Start complete mountinfo-allowlist rejection
17  inherited_fd            F    S     pre-Start descriptor-set rejection
18  inherited_socket        F    S     pre-Start socket-descriptor rejection
19  socket_inet             F    R     AF_INET socket returns EPERM
20  socket_inet6            F    R     AF_INET6 socket returns EPERM
21  socket_packet           F    R     AF_PACKET socket returns EPERM
22  socket_netlink          F    R     AF_NETLINK socket returns EPERM
23  socket_vsock            F    R     AF_VSOCK socket returns EPERM
24  socket_unix             F    R     AF_UNIX socket returns EPERM
25  socketpair              F    R     socketpair returns EPERM
26  execve                  F    R     execve returns EPERM
27  execveat                F    R     second execveat returns EPERM
28  fork                    F    R     fork returns EPERM
29  vfork                   F    R     vfork returns EPERM
30  clone_process           F    R     one exact pthread creates/joins; non-pthread clone is EPERM
31  clone3                  F    R     clone3 returns ENOSYS and creates no task
32  setns_unshare           F    R*    each call raises exact SYS_SECCOMP SIGSYS
33  mount_privilege         F    R*    mount, umount2, pivot_root, chroot, setuid, setgid,
                                          setreuid, setregid, setresuid, setresgid, setfsuid,
                                          setfsgid, setgroups and capset each raise SIGSYS
34  ptrace                  F    R     ptrace raises exact SYS_SECCOMP SIGSYS
35  bpf_perf_keyring_module F    R*    bpf, perf_event_open, add_key, request_key, keyctl,
                                          init_module, finit_module and delete_module each SIGSYS
36  io_uring_userfaultfd    F    R*    io_uring_setup, io_uring_enter, io_uring_register and
                                          userfaultfd each raise exact SYS_SECCOMP SIGSYS
37  role_signal             F    K     pidfd SIGTERM identity and whole-cgroup reap
38  role_timeout            F    K     timer identity, SIGKILL and whole-cgroup reap
39  parent_death            F    K     PDEATHSIG identity and whole-cgroup reap
40  supervisor_loss         G    H     hostagent guest teardown and stable absence
41  cleanup_fault           G    H     primary retained with host_cleanup_proven=false
```

For trap rows, a preinstalled fixed `SA_SIGINFO` handler accepts only `SIGSYS`,
`si_code=SYS_SECCOMP`, the pinned AArch64 audit architecture and the manifest-selected syscall
number. It writes one precomputed successful `RoleTerminal` frame to FD 4 and calls `_exit(0)`; any
other signal metadata or handler path is terminal failure.

For each probe-12 subattempt, the single-threaded collector resets and verifies `SIGABRT` to
`SIG_DFL`, unblocks it and reports `Ready`. After `Continue`, the root supervisor delivers exactly
one `SIGABRT` through the bound collector pidfd. Role-init must observe
`waitid(P_PIDFD) = CLD_KILLED/SIGABRT`, never `CLD_DUMPED`; the core-dump bit, ordinary-pattern
directory entries and pipe-helper counter must all remain zero. `SIGKILL`, self-exit or any other
stimulus cannot satisfy the probe.

For probe 37, the single-threaded collector verifies `SIGTERM` is `SIG_DFL` and unblocked, then
reports `Ready`. After `Continue`, the root supervisor calls exactly
`pidfd_send_signal(collector_pidfd, SIGTERM, NULL, 0)` once. Role-init's exact no-`WNOWAIT`
`waitid(P_PIDFD, collector_pidfd, &siginfo, WEXITED)` must return `si_code=CLD_KILLED` and
`si_status=SIGTERM`; the collector emits no `RoleTerminal`, and the workload cgroup must become
empty. Any other signal, sender, call tuple, status or surviving task fails the probe.

Probe 39 alone has role-init create a dedicated `SOCK_SEQPACKET | SOCK_CLOEXEC` role-init-to-launcher
channel before creating a single-threaded supervisor launcher as PID 2. The launcher is created with
the zero-filled clone ABI above, `flags=CLONE_PIDFD`, its own pidfd output,
`exit_signal=SIGCHLD` and `cgroup=0`, so it remains in the trusted supervisor cgroup. It creates
collector PID 3 with `flags=CLONE_INTO_CGROUP | CLONE_PIDFD`, the role-cgroup FD, a distinct pidfd
output and `exit_signal=SIGCHLD`. Immediately after launcher creation, role-init closes the launcher
endpoint and launcher closes the role-init endpoint. The trusted collector setup child closes the
remaining launcher endpoint and every management descriptor before credential transition or
collector application code.

The launcher sends exactly one 24-byte `LauncherChildV1` packet plus one and only one SCM_RIGHTS
collector-pidfd duplicate to role-init, then closes that duplicate. Its big-endian bytes are:

```text
0..8    ASCII magic "ENGC2LC1"
8..10   u16 version = 1
10..12  u16 probe_id = 39
12..16  u32 launcher_inner_pid = 2
16..20  u32 collector_inner_pid = 3
20..24  u32 reserved = 0
```

The launcher sends the packet, calls `shutdown(launcher_channel, SHUT_WR)` and retains only its read
half and original collector pidfd. Role-init validates the inner PID and pidfd against its private
`/proc`, requires peer EOF, then sends the byte-identical packet plus exactly one new pidfd duplicate
to root over the distinct root channel, closes the duplicate and calls
`shutdown(root_channel, SHUT_WR)`. Root requires that packet, one descriptor and EOF; it derives and
binds `collector_outer_pid` itself from the one role-cgroup PID and the pidfd's outer
`Pid`/`NSpid` evidence. The launcher and role-init never claim an outer PID.

The only root-to-role-init packet is exactly 16 big-endian bytes with no ancillary data:

```text
0..8    ASCII magic "ENGC2RT1"
8..10   u16 version = 1
10..12  u16 probe_id = 39
12..14  u16 event = 1 (collector Continue fully delivered)
14..16  u16 reserved = 0
```

After root binds the pidfd, both namespace PIDs and collector parent, the ordinary `Start/Ready`
sequence occurs. Root writes the role's complete 160-byte `Continue` frame first. Only after that
write succeeds in full does it send the one trigger packet and call `shutdown(root_channel,
SHUT_WR)`. Role-init requires that packet and EOF, sends exactly one private byte `0xa5` on the
launcher channel and calls `shutdown(launcher_channel, SHUT_WR)`. The launcher accepts only that
byte plus EOF and exits status zero.

Role-init first calls exactly `waitid(P_PIDFD, launcher_pidfd, &siginfo, WEXITED | WNOWAIT)` and
requires `CLD_EXITED/status 0`; this proves launcher exit and collector reparenting without reaping
the launcher. It then requires live `/proc` and pidfd evidence that collector PPid is inner PID 1,
calls the exact no-`WNOWAIT` collector wait and requires `CLD_KILLED/SIGKILL`, then reaps the
launcher through a no-`WNOWAIT` `waitid(P_PIDFD, launcher_pidfd, &siginfo, WEXITED)` returning the
same `CLD_EXITED/status 0`. Root proves the role cgroup empty after role-init exits. Each trusted
channel direction has exactly the cardinality and EOF above. Unknown, additional, reordered or
cross-channel data, credentials or descriptors are `ProtocolFailure`; no untrusted collector code
ever owns a trusted socket or launcher-control descriptor.

The host first executes C01 alone under phase 1 and proves its guest, disk and paths stably absent.
Only then does it enter phase 2 and execute exactly ten serial fresh acceptance instances in this
order:

```text
P01 P02 P03                 each runs cases 01..20 in order
B01                         probes 01..39 except 12, 15 and 16, in numeric order
B12O B12P                   probe 12 ordinary-pattern and pipe-pattern subcases
B15 B16                     dedicated probes 15 and 16
B40 B41                     host-terminal probes 40 and 41
```

Before every positive journey and every `F` row, both partitions are reformatted and every earlier
role/cgroup/handle is proven gone. Every `G` row receives a create-new instance and disk. Probe 12's
two measurements are jointly one logical probe. B01 executes and retains 60 subattempts; the four
other non-host boundary instances retain one each. The final host report requires 41 completed
logical probes, 64 guest measurements and 66 total subattempts including B40/B41. Each row's named
observation is its predeclared expected result. C01's three subattempts and 21 measurements remain
in their separate characterization record and cannot satisfy any of those acceptance totals.
A missing, additional or different observation fails the probe. Instances, journeys, probes and
subattempts are serial; no characterization or failed attempt may be omitted from the final record.

For B40 and B41, the supervisor accepts exact `HostStart`, emits `HostReady` and then self-terminates
with `SIGKILL`; the host accepts no subsequent guest byte. B40 performs the ordinary authorized
cleanup. B41 causes the create-new ownership wrapper's first instance-removal operation to return
the frozen injected `EIO` without deleting anything. The host first seals the failure receipt below,
then performs emergency cleanup through the same already-owned handles; this is cleanup recovery,
never workload retry. The first receipt remains byte-for-byte unchanged. Failure to reach stable
absence after recovery fails the complete campaign.

`C2B2aHostTerminalReceiptV1` is exactly 256 big-endian bytes:

```text
0..8      ASCII magic "ENGC2HR1"
8..10     u16 version = 1
10..12    u16 probe_id = 40 or 41
12..16    u32 primary_category = 2 GuestConfiguration
16..20    u32 host_cleanup_proven = 1 for 40, 0 for 41
20..24    u32 injected_cleanup_fault = 0 for 40, 1 for 41
24..32    u64 monotonic_elapsed_ns
32..64    32-byte per-instance run nonce
64..96    32-byte contract SHA-256
96..128   32-byte ten-instance schedule SHA-256
128..160  32-byte accepted host executable SHA-256
160..192  32-byte instance-identity-record SHA-256
192..224  32-byte disk-identity-record SHA-256
224..256  SHA-256 of bytes 0..224
```

The persisted host-run aggregate requires B40's receipt, B41's original false-cleanup receipt and a
separate B41 emergency-cleanup record proving stable absence. That record has the identical 256-byte
layout with magic `ENGC2RC1`, probe 41, the original primary category 2,
`host_cleanup_proven=1`, `injected_cleanup_fault=1`, the same nonce/bindings/identities, recovery-only
elapsed time and a recomputed final digest. A digest is corruption evidence only;
the live `C2B2aHostWitness`, not this forgeable record, authorizes deletion or acceptance. Receipt
elapsed time is a checked nanosecond difference from macOS `CLOCK_MONOTONIC_RAW`: B40 spans
HostStart through stable absence, while B41's original receipt ends at the injected cleanup failure.
Its separate recovery record spans the subsequent emergency cleanup through stable absence.

## Error and uncertainty precedence

Fixed numeric categories contain no source or dependency string:

1. host contract, executable, image, disk and ACL gate;
2. guest boot/effective configuration gate;
3. cgroup, mount, namespace, credential, fd, environment and seccomp gate;
4. static query/source/configuration/fixture/protocol firewall;
5. structural fixture and checked numeric bounds;
6. store open;
7. write dispatch and exact outcome;
8. lock witness and contender;
9. handle release and release probe;
10. fresh reopen, read and native-value gate;
11. complete canonical structural equality;
12. calibration measurement bounds; and
13. challenge, receipt and cleanup.

The terminal `u32` primary-category codes are exactly: `0 Success`, `1 HostContract`,
`2 GuestConfiguration`, `3 Containment`, `4 StaticFirewall`, `5 LimitExceeded`,
`6 InvalidFixture`, `7 StoreOpenFailure`, `8 OutcomeUncertain`, `9 ExclusivityBroken`,
`10 ContenderFailure`, `11 ReleaseUnproven`, `12 ReopenFailure`, `13 EngineReadFailure`,
`14 CanonicalMismatch`, `15 MeasurementFailure`, `16 ProtocolFailure` and
`17 ChallengeFailure`. Cleanup is never a replacing primary category. The terminal payload's eight
`u32` fields are, in order, `primary_category`, `active_item_code`, `guest_cleanup_proven`,
`write_may_have_been_dispatched`, `writer_acknowledged`, `roles_reaped`, `completed_case_count` and
`completed_subattempt_count`; boolean fields accept only 0 or 1. `active_item_code` is zero when no
item is active; otherwise it is `(case_or_probe_id << 16) | fixture_or_subattempt_id`.

The complete successful-state table is:

```text
instance       primary active guest-clean may-dispatch ack reaped cases subattempts
C01               0       0        1             1       1    1       0       3
P01/P02/P03       0       0        1             1       1    1      20       0
B01               0       0        1             0       0    1       0      60
B12O/B12P/B15/B16 0       0        1             0       0    1       0       1
```

`write_may_have_been_dispatched` and `writer_acknowledged` are active-item-local, never campaign
ORs. When each characterization subattempt, positive case or boundary subattempt becomes active,
the supervisor sets `active_item_code` and both flags to zero before that item's filesystem format,
binding or other action. Only that item's named milestones may set them. A structured failure
reports the current item's values. After complete instance success and
`active_item_code=0`, the terminal reports the final completed item's values: 1/1 for C01 and each
positive instance, and 0/0 for boundary instances, exactly as shown above. Starting a later item can
never inherit an earlier item's authorization phase.

For structured failure, `primary` is the first-match code below; `active_item_code` names the
current item; the two completed counts include only wholly completed prior items; dispatch and
acknowledgement are the witnessed milestones below; and `roles_reaped` and `guest_cleanup_proven`
are the observed booleans. Guest cleanup means all guest-owned roles, cgroups, mounts, device and
descriptor capabilities are released; it cannot be 1 when `roles_reaped` is 0. The byte-identical
state is committed in `TerminalReady` and echoed in `Terminal`. B40/B41 use their separate receipt.

Classification is a deterministic first-match decision over a frozen event snapshot, not callback
arrival order. The phase is the latest irreversible milestone proven before the earliest failing
observation. Within that phase, the first true predicate in the corresponding row below is primary:

```text
before full delivery of the dispatch-authorizing writer Continue
  ProtocolFailure > HostContract > GuestConfiguration > Containment > StaticFirewall >
  LimitExceeded > InvalidFixture > StoreOpenFailure > MeasurementFailure > ChallengeFailure

full dispatch-authorizing Continue through exact outcome plus supervisor acknowledgement
  OutcomeUncertain, irrespective of simultaneous protocol, containment, limit or I/O evidence

after writer acknowledgement
  ProtocolFailure > HostContract > GuestConfiguration > Containment > LimitExceeded >
  current-stage failure > MeasurementFailure > ChallengeFailure
```

The post-acknowledgement current-stage mapping is itself exact. Missing or mismatched writer-side
`LOCK` device/inode, live-writer owner or path evidence before contender dispatch is
`ExclusivityBroken`. Contender-open success is `ExclusivityBroken`; a contender failure whose bytes
are not exactly the embedded byte string, or any other unexpected contender result, is
`ContenderFailure`. Missing or invalid writer handle/lock release is `ReleaseUnproven`.
Release-probe open failure, lock-acquisition failure, wrong lock owner/path, or missing/invalid
release-probe handle/lock release is `ReleaseUnproven`. Reader-open failure is `ReopenFailure`;
read dispatch or native-value decode failure is `EngineReadFailure`; and complete canonical
disagreement is `CanonicalMismatch`. Expected contender lock rejection is a progress observation,
never acceptance authority. A category that is impossible in the selected phase is never
considered.

The writer cannot call Core before `Continue`. The 160-byte empty-payload frame is below `PIPE_BUF`;
only a full successful supervisor write sets `write_may_have_been_dispatched=1`. From that instant
until the supervisor has accepted the exact `WriterCommitted` outcome and set
`writer_acknowledged=1`, any error, signal, OOM, timeout, disk exhaustion, malformed outcome, lost
acknowledgement or supervisor uncertainty is terminal non-authorizing `OutcomeUncertain`, even when
the collector may not have reached Core. Every result after authorization burns the intent and
filesystem and forbids workload retry or replay. For a non-host-terminal probe, an expected negative
observation produces a successful probe terminal and status zero; only a missing, additional or
different observation enters this failure classifier. B40 and B41 instead retain their injected
underlying category in the fixed host receipt; the outer aggregate accepts the probe only when the
entire predeclared receipt tuple matches.

The primary result, dispatch/acknowledgement milestones and guest cleanup are retained in the guest
state. A missing or wrong post-ready challenge is host-classified `ChallengeFailure` while retaining
the committed guest state as secondary evidence. The later host aggregate separately records
`host_cleanup_proven`. Neither guest nor host cleanup failure replaces the primary result, and a
primary success with either cleanup boundary unproven is not accepted.

## Measurement interpretation and C2B2b holdout gate

C2B2a measurements are empirical backend characterization only. They neither authorize nor derive
C2B2b production limits, and no sampled descriptor/filesystem maximum can lower a hard cap. A
required positive case that reaches any ceiling, fails, times out or varies categorically across
the three guests fails C2B2a; failed or incomplete observations remain in the report.

C2B2b begins from these same conservative safety ceilings. It must add the exact bounded final wire
decoder, expected-state construction, accepted C2A/C2B1 production projection, writer/contender/
release/reader lifecycle, canonical comparison, witness allocations and a statically checked codec
allocation budget. Every proposed final resource limit must then pass the maximum frame and full
canonical journey in at least two fresh holdout guests that were not used for C2B2a. No production
limit becomes frozen until that end-to-end validation succeeds. An exceedance or required widening
returns to a new reviewed identity; it cannot be justified from C2B2a survivor measurements.

Inherited C2A row/node/depth/key/vector/scalar/table limits and exact AST/query hashes are semantic
constants restored and independently bound in C2B2b, not C2B2a calibration outputs.

## Host, payload and runtime provenance gates

Before any real calibration run, acceptance must bind:

- canonical host path, version, raw version-output hash, SHA-256, code-signing identity and metadata
  for limactl 2.0.3; Colima must be absent from every command and process tree;
- exact macOS/Darwin/VZ host identity;
- an independently anchored Ubuntu 24.04 arm64 image manifest, image, kernel and initrd hash;
- every effective generated Lima field plus live CPU, memory, disk, mount, network, agent,
  resolver and forwarding state;
- exact Linux kernel, cgroup v2, seccomp, AppArmor and filesystem identities;
- freshly extracted read-only crate sources and exact archive checksums;
- exact Rust/Cargo 1.93.0, `aarch64-unknown-linux-musl` target std/sysroot, GCC/G++ 14.2.0,
  binutils 2.44, musl libc, linker, archiver, headers and build flags;
- exact standalone payload manifest/lock/source/build-script/bindgen hashes;
- exact supervisor and collector ELF hashes, architecture, build ID, absent interpreter, empty
  dynamic-dependency set and dependency-file hashes; and
- a deterministic bundle manifest containing only those two executables and fixed metadata.

The mutable Cargo registry source tree and every existing Lima/Colima instance are research
evidence, not acceptance authority. Existing default/devc profiles are negative controls only.

The standalone lock/source firewall must contain these exact package/archive identities:

```text
surrealdb-core 2.6.0
  c48e42c81713be2f9b3dae64328999eafe8b8060dd584059445a908748b39787
surrealdb-rocksdb 0.24.0-surreal.1
  057727f56d48825ddbe45e4e7401cda6e99d864fbc004e7474b4689a5e72c86d
surrealdb-librocksdb-sys 0.17.3+10.6.2
  db194f1cf601bb6f2d0f4cbf0931bc3e5a602bac41ef2e9a87eccdfb28b7fed2
getrandom 0.3.4
  899def5c37c4fd7b2664648c28120ecec138e4d395b459e5ca34f9cce2dd77fd
```

The collector build sets `ROCKSDB_COMPILE=true`. External library/include/static/pkg-config paths,
arbitrary C/C++ flags and package-discovery influence are absent unless a later freeze enumerates
them. The final feature tree must prove direct Core `kv-rocksdb` only and reject `kv-mem`,
`kv-surrealkv`, high-level `surrealdb`, HTTP, TLS, WebSocket, JWKS, scripting, ML and provider code.
Stock Core's transitive compression set is accepted only if enumerated and attested; it is never
called compression-minimal. For this pin the required wrapper feature closure is exactly
`snappy,lz4,zstd,zlib,bzip2,bindgen-runtime`. Every transitive build-script environment input,
libclang identity, sysroot and header is enumerated; an unlisted input is terminal.

## Witnesses, transcript and cleanup

Two guest values have distinct names and authority:

- `CalibrationExpectedState` is bounded private data used only for an in-guest comparison. It is
  neither persisted nor sent guest-to-host.
- `GuestExecutionWitness` owns guest pidfds, dirfds, cgroup, namespace, device, mount and transcript
  state. It owns no host backing-file or instance capability. It is move-only, noncloneable and
  nonserializable and never crosses the VM boundary.

The distinct host `C2B2aHostWitness` solely owns the live foreground hostagent/process-group
identity, host backing-file handle, instance/run-root handles and sole supervisor channel. Neither
witness can recreate or duplicate the other's capability, and a persisted receipt can recreate
neither witness.

On success, the guest may emit `TerminalReady` only after every role is terminal/reaped, its workload
cgroup is empty, guest mounts/handles are released, streams are closed, measurements are complete
and canary scans pass. On failure, it first performs the same bounded cleanup attempt, freezes the
honest false/true cleanup fields and emits `TerminalReady` only if the supervisor channel remains
valid. The host then creates and sends the fresh challenge. The guest consumes its witness into the
one bounded `Terminal` response. The host separately consumes its witness through guest shutdown,
instance deletion, data-disk disposal and stable-absence checks at 0, 1, 2 and 5 seconds, then seals
the distinct `host_cleanup_proven` field in the host aggregate.

`max_concurrent_guests=1`. A guest's instance, processes, sockets, raw disk and paths must be
disposed and stable-absent before the next guest is created. The host records available bytes on
the exact backing APFS volume before each instance/disk allocation and after cleanup. Post-cleanup
availability must be at least the pre-run baseline minus the retained-evidence bytes and a frozen
1,073,741,824-byte filesystem/noise tolerance, and must also exceed the absolute reserve above.
Deletion is authorized only by the create-new ownership witness; a preexisting path is never
deleted.

Disk disposal is cleanup, not secure erasure or an APFS snapshot claim. Structural receipts and
reports are forgeable/replayable evidence; only the live nonserializable witnesses can produce an
accepted in-process result.

## Required provider-free source and test evidence

Before a VM run, the implementation must pass:

1. exact C2B2a source, manifest, lock, protocol and forbidden-dependency firewalls;
2. closed-world contract decoding with duplicate/unknown/missing/trailing rejection;
3. exact numeric-bound and checked-arithmetic tests for every host/payload/output cap;
4. negative-trait tests for every owner, state and witness;
5. exact command/argv/environment/cgroup/mount/seccomp/rlimit and HostStart-selector derivation
   tests, including every legal and adjacent illegal phase/class/ordinal tuple;
6. proof that every collector mode is fixed and cannot spawn a process or select another mode;
7. exact structural fixture/case-manifest identities and A/B opening/closing equality;
8. exact stock-Core configuration-key enumeration and no dynamic numeric fallback;
9. simulated kernel-evidence precedence for every boundary failure, every post-ack classifier row
   and the exact pre-Start measurement rules;
10. process/whole-cgroup/VM cleanup fault injection at every post-spawn boundary, including trusted
    reaper survival, clone exit/reap semantics and probe-39 channel ordering;
11. aggregate output and whole-run deadline enforcement;
12. host ACL, create-new ownership, direct-Lima and generated/effective-config adversarial tests;
13. canary scans over every persistable/outward surface;
14. deterministic payload build reproduction from two fresh build roots;
15. final ELF dependency/import/string/source firewalls;
16. repeated focused tests in serial and parallel, then the full provider-free evaluator matrix;
17. strict Clippy, format and whitespace checks with all build output external to the repository;
18. absent repository `target/debug` and positive disk reserve;
19. positive exact pthread creation/join plus negative clone/exec/socket/mount/namespace probes; and
20. three fresh independent exact-source/binary audits with P0=0 and P1=0.

The real calibration run first adds phase-1 C01: one fresh guest, three serial fresh-filesystem
subattempts, 21 non-acceptance measurements, one challenged terminal and stable absence. Only after
C01 succeeds does phase 2 execute the exact ten-instance acceptance schedule: three positive guests
with twenty journeys each and seven boundary guests covering all 41 logical probes, 64 guest
measurements and 66 total subattempts. It requires both distinct complete transcripts, all retained
measurements, cleanup/stable absence and a separately reviewed C2B2a acceptance record. C01 evidence
cannot satisfy an acceptance count.

## Deferred C2B2b wire and persistence gate

C2B2b must separately freeze a handwritten bounded canonical wire protocol. The preferred shape is
a fixed-width big-endian header and strictly ordered typed TLV payload: exact magic/version/kind,
zero flags, sequence, run nonce, contract/plan bindings, payload length/digest, fixed member counts,
strictly increasing `u16` tags, fixed-width integers, exact optional discriminants, bounded UTF-8
strings and vectors, checked counters, exact EOF and one-shot pipes. It may not decode through
`serde_json::Value`, an arbitrary map or a generic decoder.

The final wire freeze must enumerate every field/tag/discriminant/cap and golden vector, preserve
non-finite and noncanonical timestamp values for inherited C2B1 categorization, and require exact
encode-after-decode equality. Structurally invalid frames do not gain typed-secret precedence.

C2B2b must also define the full non-MAC canonical expected state, writer/contender/release-probe/
reader sequence, fresh Linux key, C2A-first semantic precedence, canonical equality, live host
challenge and cleanup. The reader compares complete canonical bytes, never MACs or only a digest.

Before C2B2b coding, its freeze must choose exactly one reproducible production-projection method:
a syntax-aware generated module with ordered source symbols/spans and exact permitted rewrite
hunks, or a recursively hash-bound immutable source dependency. It must bind the accepted C2A and
C2B1 seals, all required `engram-core` sources, generated output and an independently sealed
Memory-reference oracle. A hand-copied second implementation or known-fixture-only comparison is
not equivalence evidence.

For each of two fresh executions, the eventual C2B2b claim is limited to ordinary observable
disk-backed close/reopen persistence for the exact software, configuration and fixture. Stock Core
cannot prove complete close-fault detection, typed lock failure without corroborating kernel
evidence, hard native heap/thread/FD limits from Rocks options, hard aggregate host bytes, zero
background workers, read-only open, path-race protection, crash/power durability, secure erasure or
protection from trusted principals.

## Explicit nonclaims

This freeze is not C2B2a implementation or acceptance and authorizes no provider, authentication,
adapter, daemon, live-store, user-data, correction, deletion or harness execution. It does not
prove a Linux payload, guest-image provenance, VM containment, RocksDB behavior, persistence,
cleanup, runtime configuration, lock attribution, resource envelope or flagship completion.
