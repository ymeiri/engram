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

## Current Disk Preflight Blocker

The latest release-facing head is newer than the historical full-gate candidate above. On the
current `main` line, exact-head hosted CI and the quick GA gate are green, but the default full GA
gate still fails before local CI/package smoke because this host is below the default 10 GiB
release-gate free-space threshold.

The latest exact-head full-gate rehearsal still failed closed at the disk preflight after reporting
the intended release target as available:

```text
release_gate_state=disk_space_cleanup_required
failure.kind=disk_space_preflight
release_target.state=available
min_required_kib=10485760
cleanup_candidate: path=target size_kib=103776236
cleanup_candidate: path=dist size_kib=74608
```

The exact `free_space_kib` and `shortfall_kib` values are host-local and can move between
rehearsals; use the final full gate JSON as the authoritative disk evidence.

Those cleanup candidates are non-destructive evidence only. This runbook does not authorize
deleting `target/`, `dist/`, or any other local artifact. Before the post-approval sequence below
can produce final owner-review proof on this host, the release owner must either approve generated
artifact cleanup or provide another disk-space remedy, then rerun the full GA release gate and
confirm that its `disk_space.state` is `passed` with the default
`disk_space.min_required_kib=10485760` threshold.

`scripts/package-release.sh` also refuses to overwrite the expected release archive or checksum if
they already exist in `dist/`. Treat that as stale generated-artifact evidence: remove the old
files only after explicit cleanup approval, or use `ALLOW_PACKAGE_ASSET_OVERWRITE=1` only for a
local rehearsal that is not final owner-review evidence.

## Release-Owner Signoff Checklist

Before tagging or publishing `v0.2.0`, the release owner should explicitly confirm:

1. Accept the current `main` head reported by the full GA release gate as the GA release head.
2. Accept the hosted CI run named in the full GA release gate as exact-head hosted CI proof.
3. Accept that generated-artifact cleanup or another disk-space remedy was explicitly approved
   before collecting final local release evidence, if the default gate had reported
   `disk_space_cleanup_required`.
4. Accept the full GA release gate as disk-space preflight, local CI, package/install,
   Homebrew formula render, including archive checksum, manifest identity, root, and payload-hash
   checks, and
   release-scope proof.
5. Accept that the full GA release gate reported the intended `v0.2.0` local tag, remote Git tag,
   and GitHub release as unavailable before owner review.
6. Accept that the post-publish verifier must prove the signed local tag and remote Git tag both
   resolve to the accepted release head before published assets count as release evidence.
7. Accept `docs/RELEASE_NOTES_V0_2_0.md` as the public release notes for this GA scope.
8. Accept that native Claude prompt-bearing proof, live `/hooks` effective-hook visibility, and
   live Claude host-label proof are explicitly not claimed by this release.
9. Accept that broad legacy deprecation, destructive cleanup, and unrestricted automated lifecycle
   mutation are explicitly not claimed by this release.
10. Approve the post-approval command sequence below.

## Post-Approval Command Sequence

Run these commands only after explicit release-owner approval.

Set the hosted run ID to a completed push CI run for the exact head being released:

```bash
hosted_run_id=<exact-head-ci-run-id>
gate_json="$(mktemp)"

git fetch --tags --prune origin
git status --branch --short
test "$(git branch --show-current)" = "main"
test "$(git rev-parse --abbrev-ref --symbolic-full-name '@{upstream}')" = "origin/main"
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
  and .branch == "main"
  and .expected_branch == "main"
  and .upstream.name == "origin/main"
  and .upstream.ahead == 0
  and .upstream.behind == 0
  and .tracked_changes_present == false
  and .release_target.tag == "v0.2.0"
  and .release_target.repository == "ymeiri/engram"
  and .release_target.state == "available"
  and .release_target.local_tag_exists == false
  and .release_target.remote_git_tag_exists == false
  and .release_target.github_release_exists == false
  and .hosted_ci.state == "passing"
  and .disk_space.state == "passed"
  and .disk_space.min_required_kib == 10485760
  and (.disk_space.free_kib >= .disk_space.min_required_kib)
  and .disk_space.shortfall_kib == 0
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

# Retain this as a race check after the gate reports release_target.state=available.
if git rev-parse -q --verify "refs/tags/${tag}" >/dev/null; then
  echo "${tag} tag already exists" >&2
  exit 1
fi
remote_tag_refs="$(git ls-remote --tags https://github.com/ymeiri/engram.git "$tag" "${tag}^{}")"
if test -n "$remote_tag_refs"; then
  echo "${tag} remote Git tag already exists" >&2
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

verify_json="$(mktemp)"
scripts/verify-published-release-install.sh \
  --tag "$tag" \
  --expected-git-head "$release_head" \
  --json | tee "$verify_json"
jq --arg release_head "$release_head" -e '
  .tag == "v0.2.0"
  and .tag_object == .remote_tag.object
  and .tag_commit == $release_head
  and .local_tag_signature_verified == true
  and .remote_tag.commit == $release_head
  and .remote_tag.verified == true
  and .assets.source == "github_release"
  and .assets.downloaded == true
  and .assets.release_asset_list_verified == true
  and .assets.release_asset_digests_verified == true
  and .asset_install_verified == true
  and .published_install_verified == true
  and .release_actions_performed == false
' "$verify_json"
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
