# Brain Harness T394 Release Archive Path Hardening

Status: completed release-package smoke hardening
Date: 2026-06-09

## Research Question

Can the local package/install smoke fail closed on malformed release archives before extraction,
while preserving the current beta package happy path?

## Hypotheses

| Hypothesis | Expected result |
| --- | --- |
| Preferred | `package-install-smoke.sh` validates archive member paths before extraction, requiring a single expected package root and required files. |
| Null | The existing checksum and post-extract required-file checks are sufficient; no additional release hardening is needed. |
| Simpler alternative | Document the expected archive layout without enforcing it in the smoke. |
| Failure | The smoke accepts an archive with the wrong root or unsafe path, or the new check breaks the valid release package. |

## Measurement

- `bash -n scripts/package-install-smoke.sh`
- `git diff --check`
- `./scripts/package-install-smoke.sh`
- Synthetic malformed archive smoke with `SKIP_PACKAGE_BUILD=1` and a wrong archive root, expecting
  `release archive member is outside expected root`.

## Evidence

`scripts/package-install-smoke.sh` now lists the release tarball before extraction and rejects:

- empty archive listings;
- absolute member paths;
- `.` / `..` / parent-directory member paths;
- members outside the exact `engram-<version>-<host-triple>/` root;
- archives missing `engram`, `README.md`, `LICENSE`, `CHANGELOG.md`, or `RELEASE_NOTES.md` under
  that root.

The smoke also uses fixed-string checks for the `/health` JSON fields, avoiding regex matching for
the version string.

## Validation

The happy path passed:

```text
./scripts/package-install-smoke.sh
```

The run rebuilt the release archive, verified the checksum, inspected archive paths, extracted the
package, installed the packaged binary into a temporary prefix, confirmed `engram 0.2.0-beta.1`,
started packaged `engram serve --http --memory`, and verified:

```json
{"status":"ok","service":"engram","version":"0.2.0-beta.1"}
```

The negative-path smoke passed by failing closed before extraction:

```text
Malformed archive smoke failed closed as expected.
```

## Decision

Accept the preferred hypothesis. This is a small production-readiness improvement for the
release-artifact verification path, especially when `SKIP_PACKAGE_BUILD=1` is used to validate an
already-built or downloaded archive.

## Boundary

T394 does not change Engram runtime behavior, accept the hosted-CI fallback, mark PR #3 ready,
merge, tag, publish, launch native Claude, prove hooks or host labels, mutate M6, run lifecycle
cleanup, or make Engram production/GA ready.
