# T398 Release Package Manifest

Date: 2026-06-09

## Research Question

Can the beta release package carry machine-readable provenance so release-owner and post-publish
checks can verify what artifact was built without relying only on terminal logs?

## Hypotheses

| Type | Hypothesis |
| --- | --- |
| Preferred | Adding a small `MANIFEST.json` to the release tarball can tie the package version, host triple, git head, tracked-change state, `Cargo.lock` hash, and payload file hashes to the artifact. |
| Null | The manifest adds little value because the external tarball checksum and install smoke already cover the artifact. |
| Simpler alternative | Keep the current tarball contents and document the git head/checksum in release notes only. |
| Failure | The manifest check could accept stale or inconsistent provenance, or the smoke could validate from the wrong working directory. |

## Measurement

The package smoke must fail unless:

- the archive contains `MANIFEST.json`,
- the manifest package is `engram`,
- the manifest version and host triple match the build inputs,
- the manifest archive name matches the expected tarball root,
- the manifest git head and tracked-change flag match the current repository state or explicit
  expected values,
- the manifest `Cargo.lock` hash matches the current lockfile or explicit expected value,
- every payload hash in the manifest matches the extracted file.

The manifest is release evidence only. It does not approve the beta, mark PR #3 ready, merge, tag,
publish, or close hosted CI.

## Implementation

`./scripts/package-release.sh` now writes:

```text
MANIFEST.json
```

inside the package root before creating the archive. The manifest includes:

- `package`,
- `version`,
- `host_triple`,
- `archive_name`,
- `git_head`,
- `tracked_changes_present`,
- `cargo_lock_sha256`,
- hashes for `engram`, `README.md`, `LICENSE`, `CHANGELOG.md`, and `RELEASE_NOTES.md`.

`./scripts/package-install-smoke.sh` now treats `MANIFEST.json` as a required member and validates
the manifest before installing the binary.

## Validation

- `bash -n scripts/package-release.sh scripts/package-install-smoke.sh`
- `git diff --check`
- `./scripts/package-install-smoke.sh`
- A temporary tarball with a corrupted `engram` hash in `MANIFEST.json` failed closed with:

```text
error: manifest hash mismatch for engram
```
- `./scripts/local-ci.sh`

## Decision

The beta release artifact now carries enough machine-readable provenance for local and published
install checks to verify the package contents against the source checkout and lockfile.

## Boundary

T398 does not accept the hosted-CI fallback, mark PR #3 ready, merge, tag, publish, launch native
Claude, run `/hooks`, prove effective-hook visibility, prove live host labels, mutate M6, run broad
lifecycle cleanup, or make Engram production/GA ready.
