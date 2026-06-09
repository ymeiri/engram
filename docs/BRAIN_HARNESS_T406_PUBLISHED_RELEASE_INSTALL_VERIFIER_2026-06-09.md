# Brain Harness T406 Published Release Install Verifier

Date: 2026-06-09
Status: completed post-publish verification hardening.

## Scope

T406 adds `scripts/verify-published-release-install.sh`, an evidence-only post-publish verifier for
the beta release artifact. The script does not create a GitHub release, upload assets, accept the
hosted-CI fallback, mark PR #3 ready, merge, tag, publish, mutate release state, launch native
Claude, run `/hooks`, signal processes, mutate M6, or perform lifecycle cleanup.

## Research Question

Can Engram rehearse and later verify the post-publish install path with the same package checks used
before publication, without performing any release action?

## Hypotheses

| Type | Hypothesis | Result |
| --- | --- | --- |
| Preferred | A post-publish verifier can download the expected archive/checksum or validate a local asset mirror, then delegate to `package-install-smoke.sh` with `SKIP_PACKAGE_BUILD=1`. | Supported. |
| Null | The README install commands are enough. | Rejected because release-owner automation needs a single fail-closed verifier. |
| Simpler alternative | Duplicate package verification logic in a new script. | Rejected because `package-install-smoke.sh` already owns manifest, checksum, install, and `/health` checks. |
| Failure | The verifier creates, uploads, tags, or publishes release state. | Avoided. It only reads/downloads assets or validates an explicit local directory. |

## Script Contract

Default mode:

```bash
./scripts/verify-published-release-install.sh --tag v0.2.0-beta.1 --json
```

The script:

1. inspects the GitHub release,
2. downloads `engram-<version>-<host-triple>.tar.gz` and `.sha256`,
3. runs `./scripts/package-install-smoke.sh` with `SKIP_PACKAGE_BUILD=1`,
4. verifies manifest head and tracked-change expectations,
5. starts the packaged HTTP server and verifies `/health`,
6. emits JSON evidence on success when `--json` is supplied.

Pre-publish rehearsal mode:

```bash
./scripts/verify-published-release-install.sh --asset-dir <dir> --json
```

This skips GitHub release inspection/download and validates the provided asset directory.

## Validation

Validation performed for this slice:

- `bash -n scripts/verify-published-release-install.sh`
- `git diff --check`
- `DIST_DIR=/tmp/engram-t406-dist ./scripts/package-release.sh`
- `scripts/verify-published-release-install.sh --asset-dir /tmp/engram-t406-dist
  --expected-tracked-changes-present true --json`
- JSON assertion that local asset rehearsal passed and performed no release actions
- synthetic missing-release check with tag `v0.2.0-beta.1-t406-missing-probe`, which failed
  closed before any install verification
- focused quick beta report assertion that incomplete evidence still asks for
  `run_full_beta_release_gate_report_with_local_ci_and_package_smoke`
- full beta report assertion that release-ready fallback evidence includes
  `verify_published_release_install`
- `./scripts/local-ci.sh`
- `./scripts/package-install-smoke.sh`

## Gate Impact

T406 makes the final post-publish install step executable and repeatable once the release owner
approves release mechanics. It does not accept the waiver, close hosted CI, mark PR #3 ready,
merge, tag, publish, close native Claude prompt-bearing proof, close effective-hook visibility,
close live host labels, complete multi-host parity, mutate M6, run lifecycle cleanup, or make
Engram production/GA ready.
