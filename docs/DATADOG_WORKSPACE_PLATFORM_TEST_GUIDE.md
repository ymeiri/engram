# Datadog Workspace Platform Test Guide

This guide validates the release-packaging work before the GA release. A Linux Datadog workspace
can directly prove the Linux host it runs on. The remaining macOS and Windows package paths are
proved by hosted CI or by matching native hosts.

Use this branch:

```bash
yuval.meiri/ga-linux-intel-mac
```

After Codex pushes the latest work, pin to the current branch head shown by:

```bash
git rev-parse HEAD
```

## What Was Already Checked Locally

Codex already ran these checks on the local Apple Silicon Mac before this workspace test:

```bash
bash -n scripts/render-homebrew-formula.sh scripts/release-gate-report.sh \
  scripts/package-release.sh scripts/package-install-smoke.sh \
  scripts/verify-published-release-install.sh

cargo fmt --all --check
git diff --check
cargo test --all-targets
cargo clippy --all-targets -- -D warnings
cargo test -p engram-tests --test release_gate_script_tests
```

Codex also ran:

- A temporary package/install smoke for the local Apple Silicon artifact.
- A Homebrew formula render regression test with temporary synthetic archives for:
  - `aarch64-apple-darwin`
  - `x86_64-apple-darwin`
  - `x86_64-unknown-linux-gnu`
  - `aarch64-unknown-linux-gnu`

One local execution gap remains:

```bash
pwsh ./scripts/package-install-smoke-windows.ps1
```

`pwsh` is not installed on the local Apple Silicon Mac. Windows package scripts must be executed on
Windows x64 hosts or by the GitHub Actions package-smoke matrix.

## 1. Check Out The Branch

Do not use bare `git pull` if Git reports divergent branches. Use `fetch` plus an explicit branch
checkout.

```bash
git fetch origin
git switch --track origin/yuval.meiri/ga-linux-intel-mac
```

If the branch already exists locally:

```bash
git fetch origin
git switch yuval.meiri/ga-linux-intel-mac
git merge --ff-only origin/yuval.meiri/ga-linux-intel-mac
```

Confirm the commit:

```bash
git rev-parse HEAD
```

Use that SHA in any Datadog workspace evidence you send back.

## 2. Confirm The Workspace Host Triple

```bash
rustc -vV | awk '/^host:/ { print $2 }'
```

Expected for a Linux x64 Datadog workspace:

```text
x86_64-unknown-linux-gnu
```

Expected for a Linux ARM64 Datadog workspace:

```text
aarch64-unknown-linux-gnu
```

If the value is different, the workspace is testing a different release artifact than expected.

## 3. Check Required Tools

```bash
command -v cargo rustc jq ruby tar curl nc shasum
```

On Ubuntu-like systems, install missing tools with:

```bash
sudo apt-get update
sudo apt-get install -y jq ruby netcat-openbsd perl curl
```

`shasum` normally comes from Perl.

## 4. Run Fast Script And Test Validation

```bash
bash -n scripts/render-homebrew-formula.sh scripts/release-gate-report.sh \
  scripts/package-release.sh scripts/package-install-smoke.sh

cargo fmt --all --check
git diff --check
cargo test -p engram-tests --test release_gate_script_tests
```

Expected result: all commands pass.

## 5. Run Linux Package Install Smoke

This builds the Linux release tarball, validates checksum and manifest, installs the binary into a
temporary prefix, starts `engram serve --http --memory`, and checks `/health`.

```bash
tmp_dist="$(mktemp -d)"
ALLOW_PACKAGE_DIST_DIR_OVERRIDE=1 \
DIST_DIR="$tmp_dist" \
scripts/package-install-smoke.sh
```

Expected output includes:

```text
Package install smoke passed
engram 0.2.0
{"status":"ok","service":"engram","version":"0.2.0"}
```

Confirm the Linux artifact names:

```bash
host_triple="$(rustc -vV | awk '/^host:/ { print $2 }')"
ls -lh "$tmp_dist"/engram-0.2.0-"$host_triple".tar.gz*
```

Expected files:

```text
engram-0.2.0-<host-triple>.tar.gz
engram-0.2.0-<host-triple>.tar.gz.sha256
```

## 6. Render The Linux Formula Path

This validates that the Homebrew formula renderer can consume the current Linux artifact in
isolation.

```bash
ALLOW_HOMEBREW_DIST_DIR_OVERRIDE=1 \
ALLOW_HOMEBREW_HOST_TRIPLE_OVERRIDE=1 \
HOMEBREW_HOST_TRIPLE="$host_triple" \
DIST_DIR="$tmp_dist" \
scripts/render-homebrew-formula.sh

ruby -c "$tmp_dist/homebrew/Formula/engram.rb"
rg -n "on_linux|$host_triple" "$tmp_dist/homebrew/Formula/engram.rb"
```

Expected:

```text
Syntax OK
```

The `rg` output should show `on_linux` and the Linux tarball URL.

## 7. Optional Linux Binary Sanity Check

```bash
work_dir="$(mktemp -d)"
tar -xzf "$tmp_dist"/engram-0.2.0-"$host_triple".tar.gz -C "$work_dir"
"$work_dir"/engram-0.2.0-"$host_triple"/engram --version
```

Expected:

```text
engram 0.2.0
```

## 8. What This Workspace Cannot Prove

A Linux Datadog workspace proves only the matching Linux package path for that workspace host
triple. It does not prove macOS or Windows install behavior.

macOS and Windows proof must come from one of these:

- The GitHub Actions jobs `Package Smoke (macOS Apple Silicon)` and `Package Smoke (macOS Intel)`.
- The GitHub Actions job `Package Smoke (Windows x64)`.
- Real native hosts running the matching smoke script.

The branch currently only triggers GitHub CI on a pull request to `main`, because the workflow is
configured for `push` to `main` and `pull_request` to `main`.

## 9. Hosted CI Check After PR Creation

After opening a PR from `yuval.meiri/ga-linux-intel-mac` to `main`, verify these jobs pass:

```text
Check
Test
Format
Clippy
Docs
Package Smoke (Linux x64)
Package Smoke (Linux ARM64)
Package Smoke (macOS Apple Silicon)
Package Smoke (macOS Intel)
Package Smoke (Windows x64)
```

The package-smoke jobs are the release artifact proof jobs. Each job verifies its Rust host triple
before running the Unix or Windows install-smoke script.

## 10. Full Formula Rehearsal After Hosted Artifacts Exist

After hosted CI passes, download the uploaded Homebrew artifacts from the CI run and place all eight
files in `dist/`:

```text
dist/engram-0.2.0-aarch64-apple-darwin.tar.gz
dist/engram-0.2.0-aarch64-apple-darwin.tar.gz.sha256
dist/engram-0.2.0-x86_64-apple-darwin.tar.gz
dist/engram-0.2.0-x86_64-apple-darwin.tar.gz.sha256
dist/engram-0.2.0-x86_64-unknown-linux-gnu.tar.gz
dist/engram-0.2.0-x86_64-unknown-linux-gnu.tar.gz.sha256
dist/engram-0.2.0-aarch64-unknown-linux-gnu.tar.gz
dist/engram-0.2.0-aarch64-unknown-linux-gnu.tar.gz.sha256
```

Then render and inspect the full formula:

```bash
scripts/render-homebrew-formula.sh
ruby -c dist/homebrew/Formula/engram.rb
rg -n "aarch64-apple-darwin|x86_64-apple-darwin|x86_64-unknown-linux-gnu|aarch64-unknown-linux-gnu|on_macos|on_linux" \
  dist/homebrew/Formula/engram.rb
```

Expected:

- Ruby syntax passes.
- The formula includes macOS ARM, macOS Intel, Linux x64, and Linux ARM64 URLs.

## 11. Release Eligibility Boundary

This workspace test is pre-release validation. It does not make the release GA-eligible by itself.

GA eligibility still requires:

1. PR opened and hosted CI passing on the platform package-smoke jobs.
2. Branch merged to `main`.
3. Final `main` synced with `origin/main`.
4. All three platform release artifacts available.
5. Full GA release gate rerun on the exact final head.
6. Release-owner approval to tag, publish GitHub assets, update the Homebrew tap, and verify
   published installs.
