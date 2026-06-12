# Engram v0.2.0 GA Release-Owner Approval Runbook

Date: 2026-06-12
Status: approval runbook only

## Research Question

Can the remaining `v0.2.0` GA release-management gap be reduced to an explicit,
auditable release-owner decision without changing source behavior, widening release scope, or
publishing artifacts before approval?

## Evidence Snapshot

Before this runbook was added, the validated GA owner-review candidate was:

```text
1eefa11aff32e4d3802cc327ddc8d8957fd2f56f
```

That head had exact-head hosted CI run `27379891728` green for Check, Test, Clippy, Docs, and
Format. The full GA release gate also passed on that head:

```bash
scripts/release-gate-report.sh --target ga --hosted-run 27379891728 --json
```

The report emitted `workspace_version_matches_release=true`, `tracked_changes_present=false`,
`hosted_ci.state=passing`, `local_ci=passed`, `package_install_smoke=passed`,
`release_scope.state=complete`, `release_gate_state=hosted_ci_passing_release_owner_review_required`,
and `ready_for_release_owner_review=true`.

The package/install smoke built and verified:

```text
dist/engram-0.2.0-aarch64-apple-darwin.tar.gz
dist/engram-0.2.0-aarch64-apple-darwin.tar.gz.sha256
```

The archive checksum was:

```text
57e404714d3ebb2df3dd4622075748742526c9ead63725e4844cd775ce4b9642  engram-0.2.0-aarch64-apple-darwin.tar.gz
```

The packaged binary reported `engram 0.2.0`, and packaged HTTP `/health` returned:

```json
{"status":"ok","service":"engram","version":"0.2.0"}
```

This snapshot is evidence for the prior candidate head only. If this runbook or any later docs
change is included in the release head, rerun exact-head hosted CI and the full GA release gate on
the new head before tagging.

## Release-Owner Signoff Checklist

Before tagging or publishing `v0.2.0`, the release owner should explicitly confirm:

1. Accept the current `main` head reported by the full GA release gate as the GA release head.
2. Accept the hosted CI run named in the full GA release gate as exact-head hosted CI proof.
3. Accept the full GA release gate as local CI, package/install, Homebrew formula render, and
   release-scope proof.
4. Accept `docs/RELEASE_NOTES_V0_2_0.md` as the public release notes for this GA scope.
5. Accept that native Claude prompt-bearing proof, live `/hooks` effective-hook visibility, and
   live Claude host-label proof are explicitly not claimed by this release.
6. Accept that broad legacy deprecation, destructive cleanup, and unrestricted automated lifecycle
   mutation are explicitly not claimed by this release.
7. Approve the post-approval command sequence below.

## Post-Approval Command Sequence

Run these commands only after explicit release-owner approval.

Set the hosted run ID to a completed push CI run for the exact head being released:

```bash
hosted_run_id=<exact-head-ci-run-id>
gate_json="$(mktemp)"

git fetch --tags --prune origin
git status --branch --short
read ahead behind < <(git rev-list --left-right --count main...origin/main)
test "$ahead" = "0" && test "$behind" = "0"
git diff --quiet --ignore-submodules --
git diff --cached --quiet --ignore-submodules --

scripts/release-gate-report.sh --target ga --hosted-run "$hosted_run_id" --json | tee "$gate_json"
jq -e '
  .target == "ga"
  and .package_version == "0.2.0"
  and .release_version == "0.2.0"
  and .workspace_version_matches_release == true
  and .tracked_changes_present == false
  and .hosted_ci.state == "passing"
  and .local_ci == "passed"
  and .package_install_smoke == "passed"
  and .homebrew_formula_render == "passed"
  and .release_scope.state == "complete"
  and .release_gate_state == "hosted_ci_passing_release_owner_review_required"
  and .ready_for_release_owner_review == true
' "$gate_json"

release_version="$(jq -r '.release_version' "$gate_json")"
release_head="$(jq -r '.head' "$gate_json")"
tag="v${release_version}"
test "$(git rev-parse HEAD)" = "$release_head"

if git rev-parse -q --verify "refs/tags/${tag}" >/dev/null; then
  echo "${tag} tag already exists" >&2
  exit 1
fi
if gh release view "$tag" --repo ymeiri/engram >/dev/null 2>&1; then
  echo "${tag} GitHub release already exists" >&2
  exit 1
fi
```

Create the signed tag and publish the GitHub release assets:

```bash
scripts/package-release.sh
scripts/render-homebrew-formula.sh
ruby -c dist/homebrew/Formula/engram.rb

git tag -s "$tag" -m "engram ${tag}" "$release_head"
git push origin "$tag"

gh release create "$tag" \
  "dist/engram-${release_version}-aarch64-apple-darwin.tar.gz" \
  "dist/engram-${release_version}-aarch64-apple-darwin.tar.gz.sha256" \
  --repo ymeiri/engram \
  --title "engram ${tag}" \
  --notes-file docs/RELEASE_NOTES_V0_2_0.md \
  --latest

scripts/verify-published-release-install.sh --tag "$tag" --json
```

After the release assets verify, update the Homebrew tap:

```bash
tap_dir="$(mktemp -d)"
git clone git@github.com:ymeiri/homebrew-engram.git "$tap_dir"
mkdir -p "$tap_dir/Formula"
cp dist/homebrew/Formula/engram.rb "$tap_dir/Formula/engram.rb"
git -C "$tap_dir" diff -- Formula/engram.rb
git -C "$tap_dir" status --short
git -C "$tap_dir" add Formula/engram.rb
git -C "$tap_dir" commit -m "Update engram formula to ${tag}"
git -C "$tap_dir" push origin main
```

Finally, verify the user-facing install path:

```bash
brew update
brew upgrade engram || brew install ymeiri/engram/engram
engram --version
```

## Boundary

This runbook is documentation and release-gate clarification only. It does not approve the release,
create a tag, publish GitHub assets, update the Homebrew tap, mutate harness settings, launch
native Claude, run `/hooks`, signal processes, change Memory OS lifecycle state, broaden M6, or
expand the `v0.2.0` support claims.
