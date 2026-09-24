# Connect-existing-only and daemon persistence repair — 2026-09-06

## Scope and claim

This provider-free repair covers the CLI connect-existing transport boundary and ordinary daemon
control persistence. It does **not** authorize a native pilot or establish process identity.

`engram serve --connect-existing-only` promises only that the CLI will not start, repair, or clean
up daemon controls. It is not a read-only operation:

- `/health` commits creation and deletion of a unique datastore canary in one transaction.
- Forwarded MCP calls may mutate data within the selected tool profile.
- Proxy shutdown deletes the proxy's MCP session.

Any future authoritative evaluator use must own an isolated daemon `Child` and store for the whole
phase, retain the exact launch record, verify live process and authenticated MCP identity, and
constrain MCP traffic to that phase.

## Closed findings

- Unix persistence retains one `O_DIRECTORY|O_NOFOLLOW|O_CLOEXEC` parent descriptor plus every
  `O_EXCL|O_NOFOLLOW|O_CLOEXEC` control descriptor, full identity, and expected bytes.
- A `startup-pending` PID control is published and parent-synced before child spawn. Every later
  control name is parent-synced immediately after exclusive publication, before fallible metadata
  or write work; a fault injected in the pre-sync seam triggers an emergency held-parent sync
  before return. Completed contents are then file-synced and identity/byte validated. Unproven
  termination also unconditionally syncs every held leaf and the held parent, aggregating failures.
- Unix and non-Unix automatic rollback preserve all controls and return an explicit recovery error.
  Neither platform contract supplies an atomic compare-identity-and-unlink operation, so a
  pathname deletion cannot truthfully promise to preserve a same-owner replacement. A
  deterministic hook after validation proves replacements of PID, port, and metadata survive
  byte- and inode-identically.
- Rollback is not attempted unless termination and reap of the owned child are positively
  established. Injected persistence failures at PID, port, and metadata combined with injected
  `try_wait`, `kill`, and `wait` failures preserve the complete observed control snapshot.
- The first existing-control check is read-only and precedes directory permission repair, lock-file
  creation, or token creation. Existing controls take a reuse-or-block path with no permission
  repair. The no-control path locks and rechecks before preparation. Complete controls may be
  inspected for valid healthy reuse; every unresolved complete or partial set blocks automatic
  cleanup and a second spawn. Recovery is deliberately outside this automatic path: an operator
  must independently establish process identity and termination before managing preserved
  controls.
- Ordinary readiness and recorded-daemon reuse require `health.pid == Some(expected_pid)`.
- Loopback clients ignore ambient proxies and redirects. Bearer header values are marked sensitive;
  authenticated HTTP failures expose only status and declared response length, never response
  bodies or bearer material.

## Provider-free verification

- `CARGO_TARGET_DIR=/private/tmp/engram-connect-existing-cli-20260905 cargo test -p
  engram-cli --locked`: 89 passed, 0 failed.
- The focused daemon-identity matrix covers PID, port, and metadata persistence failures;
  failures before and after each publication's first parent sync; pre-spawn fencing; read-only
  symlinked-parent preflight; post-validation replacement; parent ABA; entry replacement;
  same-inode mutation; and every persistence phase crossed with injected `try_wait`, `kill`, and
  `wait` failures.
- `CARGO_TARGET_DIR=/private/tmp/engram-connect-existing-cli-20260905 cargo test -p
  engram-store writable_probe_commits_without_leaving_a_record -- --nocapture`: 1 passed.
- `CARGO_TARGET_DIR=/private/tmp/engram-connect-existing-cli-20260905 cargo test -p
  engram-mcp health_response_attests_runtime_contract -- --nocapture`: 1 passed.
- `CARGO_TARGET_DIR=/private/tmp/engram-connect-existing-cli-20260905 cargo clippy -p
  engram-cli --locked --all-targets -- -D warnings`: passed.
- `rustfmt --edition 2021 --check` for the three scoped CLI sources and targeted
  `git diff --check`: passed.

## Frozen source hashes

- `engram-cli/src/main.rs`:
  `42bc8b34cbdc133fa120860c7fe7b17f4e83b753ec051fef036e275566bf943f`
- `engram-cli/src/daemon.rs`:
  `2875af440d490e4ddd5a98524f930ecf5b54b1d6fdbb6812dc7fcff1a70ede92`
- `engram-cli/src/proxy.rs`:
  `a8bbfa06d4df0d0ef1143c495da31e37dea005d42dcf8999440af78b292fb10f`

All completed verification commands listed above use
`CARGO_TARGET_DIR=/private/tmp/engram-connect-existing-cli-20260905`. One accidentally misspelled
external target invocation was interrupted during compilation and its exact cache directory was
removed; it did not use the repository target. Repository `target/debug` remains absent. No live
daemon, provider, authentication, VM, or settings operation is part of this evidence.
