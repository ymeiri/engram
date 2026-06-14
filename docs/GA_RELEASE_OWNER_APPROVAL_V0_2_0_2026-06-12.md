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

## Current Local Cleanup Blockers

The latest recorded release-gate behavior checkpoint is newer than the historical full-gate
candidate above. At that checkpoint, exact-head hosted CI run `27497957513` is green for
`2adc10c3e6e77c107636abac59b202bdbc89b920`, and the quick GA gate is green for the same head.
That checkpoint also clarifies that branch-sync divergence is a stop-and-inspect condition, not
approval to run `git pull`, and that early operator/configuration failures in JSON mode emit
`configuration_preflight_failed` evidence instead of looking like script crashes. The hosted-CI
pre-step verifier and published release verifier both emit structured configuration-preflight JSON
for `--json` operator failures without marking release actions as performed. The generated-output
cleanup verifier now repeats the full-gate manifest evidence and fingerprints the manifest file
itself with `manifest_size_bytes` and `manifest_sha256`. The exact-head default full GA gate now
passes the default 10 GiB disk preflight on this host, but still fails before local CI/package
smoke because stale generated outputs already exist at the paths the full gate would write.

The latest exact-head full-gate rehearsal failed closed after reporting the intended release target
as available:

```text
release_gate_state=generated_outputs_cleanup_required
failure.kind=generated_outputs_preflight
release_target.state=available
disk_space.state=passed
min_required_kib=10485760
generated_artifacts.state=not_checked
generated_output: path=dist/engram-0.2.0-aarch64-apple-darwin.tar.gz exists=true will_write=true
  file_type=file size_bytes=25455708
  sha256=f48a9fb1f5d5d815b9dfcec0db71aaf0120f5e38fcffc625880c6a0972c2efc5
generated_output: path=dist/engram-0.2.0-aarch64-apple-darwin.tar.gz.sha256 exists=true will_write=true
  file_type=file size_bytes=107
  sha256=5aa7217ffe054bcd58b23c4014468c44575671d31b69a537d16313c121ab9c5d
generated_output: path=dist/homebrew/Formula/engram.rb exists=true will_write=true
  file_type=file size_bytes=1110
  sha256=596b582ab3f603e6c4c5f098f8dec46db175e28ec85d16a0b5b67bcf6386beef
```

`generated_artifacts.state=not_checked` is intentional in this failure path: local
package/Homebrew proof did not run, so the JSON must not claim publishable artifact evidence.
The forced disk-space and release-target conflict rehearsals on the same head also report
`generated_artifacts.state=not_checked`, proving those preflight failures are not artifact
publication evidence.
The generated-output entries also report `file_type`, `size_bytes`, and `sha256` for existing
regular files; use the final gate JSON as the authoritative stale-output fingerprint before
approving cleanup.

The exact `free_space_kib` value is host-local and can move between rehearsals; use the final full
gate JSON as the authoritative disk evidence. If disk space drops below the default threshold
again, `disk_space_cleanup_required` may appear before generated-output cleanup.

Those cleanup signals are non-destructive evidence only. This runbook does not authorize deleting
`target/`, `dist/`, or any other local artifact. Before the post-approval sequence below can
produce final owner-review proof on this host, the release owner must approve generated-output
cleanup for the listed `dist/` archive, checksum, and formula files. The post-approval cleanup
block below verifies the exact approved size and SHA-256 fingerprints before removing only those
three ignored generated release outputs. If the disk preflight regresses, the release owner must
also approve disk cleanup or provide another disk-space remedy. Then rerun the full GA release gate
and confirm that
`disk_space.state=passed`, `generated_outputs.state=clear`, and
`disk_space.min_required_kib=10485760`.

`scripts/package-release.sh` also refuses to overwrite the expected release archive or checksum if
they already exist in `dist/`. Treat that as stale generated-artifact evidence: remove the old
files only after explicit cleanup approval, or use `ALLOW_PACKAGE_ASSET_OVERWRITE=1` only for a
local rehearsal that is not final owner-review evidence. The package script stages new archive and
checksum outputs as temporary files and verifies the final checksum after moving them into `dist/`;
do not accept package evidence from a failed or interrupted run unless the full GA gate reruns and
reports package/install smoke success.

`scripts/render-homebrew-formula.sh` similarly refuses to overwrite
`dist/homebrew/Formula/engram.rb` unless `ALLOW_HOMEBREW_FORMULA_OVERWRITE=1` is set. Treat an
existing formula file as generated evidence that needs the same cleanup approval before final
owner-review proof. The formula renderer stages new formula text as a temporary file and runs
`ruby -c` before moving it into `dist/homebrew/Formula/engram.rb`; do not accept formula evidence
from a failed or interrupted render unless the full GA gate reruns and reports Homebrew formula
render success.

The full GA release gate reports these expected local outputs under `generated_outputs`. Final
owner-review proof should require `generated_outputs.state=clear`; if it reports
`cleanup_required`, the listed files need the same explicit cleanup approval or a local-only
rehearsal overwrite decision before package/Homebrew evidence is final.

When disk space is sufficient, the full GA release gate now fails before local CI/package/Homebrew
validation with `release_gate_state=generated_outputs_cleanup_required` if any listed generated
output both exists and would be written by the gate. Treat that state as a hard cleanup-approval
gate, not as owner-review-ready evidence.

After a successful full gate, `generated_artifacts` reports the artifacts produced for publication.
Each existing regular artifact includes `file_type`, `size_bytes`, and `sha256`, so owner-review
proof can verify the exact archive, checksum, and formula files that will be published instead of
accepting path existence alone. The gate reports `generated_artifacts_missing` if any required
post-run artifact is missing, non-regular, empty, or lacks a SHA-256 fingerprint.

## Release-Owner Signoff Checklist

Before tagging or publishing `v0.2.0`, the release owner should explicitly confirm:

1. Accept the current `main` head reported by the full GA release gate as the GA release head.
2. Accept the hosted CI run named in the full GA release gate as exact-head hosted CI proof.
3. Accept that generated-output cleanup was explicitly approved and completed before collecting
   final local release evidence, if the default gate had reported
   `generated_outputs_cleanup_required`.
4. Accept that disk cleanup or another disk-space remedy was explicitly approved before collecting
   final local release evidence, if the default gate had reported `disk_space_cleanup_required`.
5. Accept the full GA release gate as disk-space preflight, generated-output cleanup, local CI,
   package/install, Homebrew formula render, including archive checksum, manifest identity, root,
   and payload-hash checks, release-scope proof, and the source of the archive, checksum, and
   formula files to publish.
6. Accept that the full GA release gate reported the intended `v0.2.0` local tag, remote Git tag,
   and GitHub release as unavailable before owner review.
7. Accept that the post-publish verifier must prove the signed local tag and remote Git tag both
   resolve to the accepted release head before published assets count as release evidence.
8. Accept `docs/RELEASE_NOTES_V0_2_0.md` as the public release notes for this GA scope.
9. Accept that native Claude prompt-bearing proof, live `/hooks` effective-hook visibility, and
   live Claude host-label proof are explicitly not claimed by this release.
10. Accept that broad legacy deprecation, destructive cleanup, and unrestricted automated lifecycle
   mutation are explicitly not claimed by this release.
11. Approve the post-approval command sequence below.

## Post-Approval Command Sequence

Run these commands only after explicit release-owner approval.

Set the hosted run ID to a completed push CI run for the exact head being released:

```bash
hosted_run_id=<exact-head-ci-run-id>
gate_json="$(mktemp)"
```

If the pre-approval gate reported `generated_outputs_cleanup_required`, first verify that the
local stale outputs still match the approved fingerprint manifest, then remove exactly those
ignored generated release outputs. The verification command is read-only and must report
`actions_performed.generated_output_cleanup=false`; if the manifest or current fingerprints do not
match, it reports `release_gate_state=generated_output_cleanup_fingerprints_mismatch` with
`failure.kind=generated_output_cleanup_verification`. The verifier also rejects manifests that are
not full-gate `generated_outputs_cleanup_required` evidence with `release_target.state=available`,
`disk_space.state=passed`, and no release actions. Its verification JSON repeats the validated
manifest's hosted-CI, release-target, disk, remaining-action, and no-action evidence under
`generated_output_cleanup_verification.manifest_evidence` and fingerprints the manifest itself with
`manifest_size_bytes` and `manifest_sha256`, so the cleanup result is self-contained enough for
review. Stop and rerun the full GA gate to collect fresh cleanup evidence before approving deletion.

```bash
cleanup_manifest="$(mktemp)"
cleanup_verify_json="$(mktemp)"

set +e
scripts/release-gate-report.sh \
  --target ga \
  --hosted-run "$hosted_run_id" \
  --json >"$cleanup_manifest"
cleanup_gate_status=$?
set -e

test "$cleanup_gate_status" != "0"
jq -e '
  .release_gate_state == "generated_outputs_cleanup_required"
  and .failure.kind == "generated_outputs_preflight"
  and .hosted_ci.state == "passing"
  and .hosted_ci.repository == .release_target.repository
  and .hosted_ci.expected_event == "push"
  and (.hosted_ci.run_id | type == "number")
  and .hosted_ci.run.status == "completed"
  and .hosted_ci.run.conclusion == "success"
  and .hosted_ci.run.headSha == .head
  and .hosted_ci.run.event == .hosted_ci.expected_event
  and .hosted_ci.run.workflowName == "CI"
  and .release_target.state == "available"
  and .disk_space.state == "passed"
  and .generated_outputs.state == "cleanup_required"
  and .generated_artifacts.state == "not_checked"
  and .ready_for_release_owner_review == false
  and .release_owner_decision_required == true
  and .hosted_ci_fallback_decision_required == false
  and (.remaining_release_actions | sort) == ([
    "remove_stale_generated_release_outputs_or_get_cleanup_approval",
    "rerun_full_release_gate_report_with_local_ci_and_package_smoke"
  ] | sort)
  and (. as $manifest
    | ($manifest.actions_performed | type == "object")
      and (
        $manifest.actions_performed as $actions
        | all([
            "release_actions",
            "git_tag",
            "github_release",
            "package_asset_upload",
            "homebrew_tap_update",
            "generated_output_cleanup"
          ][]; $actions[.] == false)
        and all($actions[]; . == false)))
  and all(.generated_outputs.outputs[];
    .exists == true
    and .will_write == true
    and .file_type == "file"
    and (.size_bytes | type == "number")
    and .size_bytes > 0
    and (.sha256 | test("^[0-9a-f]{64}$")))
  and .release_actions_performed == false
' "$cleanup_manifest"
cleanup_manifest_size_bytes="$(wc -c <"$cleanup_manifest" | tr -d '[:space:]')"
cleanup_manifest_sha256="$(shasum -a 256 "$cleanup_manifest" | awk '{ print $1 }')"
test "$cleanup_manifest_size_bytes" -gt 0
test -n "$cleanup_manifest_sha256"

scripts/release-gate-report.sh \
  --target ga \
  --verify-generated-output-cleanup "$cleanup_manifest" \
  --json | tee "$cleanup_verify_json"
jq -e \
  --argjson cleanup_manifest_size_bytes "$cleanup_manifest_size_bytes" \
  --arg cleanup_manifest_sha256 "$cleanup_manifest_sha256" \
  '
  .release_gate_state == "generated_output_cleanup_fingerprints_verified"
  and .generated_output_cleanup_verification.state == "verified"
  and .generated_output_cleanup_verification.manifest_size_bytes == $cleanup_manifest_size_bytes
  and .generated_output_cleanup_verification.manifest_sha256 == $cleanup_manifest_sha256
  and .generated_output_cleanup_verification.manifest_evidence.target == "ga"
  and .generated_output_cleanup_verification.manifest_evidence.head == .head
  and .generated_output_cleanup_verification.manifest_evidence.hosted_ci.state == "passing"
  and .generated_output_cleanup_verification.manifest_evidence.hosted_ci.repository == "ymeiri/engram"
  and .generated_output_cleanup_verification.manifest_evidence.hosted_ci.expected_event == "push"
  and (.generated_output_cleanup_verification.manifest_evidence.hosted_ci.run_id | type == "number")
  and .generated_output_cleanup_verification.manifest_evidence.hosted_ci.run.status == "completed"
  and .generated_output_cleanup_verification.manifest_evidence.hosted_ci.run.conclusion == "success"
  and .generated_output_cleanup_verification.manifest_evidence.hosted_ci.run.headSha == .head
  and .generated_output_cleanup_verification.manifest_evidence.hosted_ci.run.event == "push"
  and .generated_output_cleanup_verification.manifest_evidence.hosted_ci.run.workflowName == "CI"
  and .generated_output_cleanup_verification.manifest_evidence.release_target.state == "available"
  and .generated_output_cleanup_verification.manifest_evidence.disk_space.state == "passed"
  and .generated_output_cleanup_verification.manifest_evidence.generated_artifacts_state == "not_checked"
  and .failure == null
  and (.generated_outputs.outputs | length) > 0
  and all(.generated_outputs.outputs[];
    .exists == true
    and .will_write == true
    and .file_type == "file")
  and (.generated_output_cleanup_verification.manifest_evidence.remaining_release_actions | sort) == ([
    "remove_stale_generated_release_outputs_or_get_cleanup_approval",
    "rerun_full_release_gate_report_with_local_ci_and_package_smoke"
  ] | sort)
  and all(.generated_output_cleanup_verification.manifest_evidence.actions_performed[]; . == false)
  and .actions_performed.generated_output_cleanup == false
  and .release_actions_performed == false
' "$cleanup_verify_json"

while IFS= read -r stale_output; do
  rm -- "$stale_output"
done < <(jq -r '.generated_outputs.outputs[] | select(.will_write == true) | .path' \
  "$cleanup_verify_json")

while IFS= read -r stale_output; do
  test ! -e "$stale_output"
done < <(jq -r '.generated_outputs.outputs[] | select(.will_write == true) | .path' \
  "$cleanup_verify_json")
```

Do not use `git pull` to satisfy the branch-sync checks below. If the ahead/behind check is
nonzero, stop and inspect the local and remote commits first; any merge, rebase, or fast-forward
changes the release head and requires fresh exact-head hosted CI plus a fresh full GA gate.

```bash
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
  and .generated_outputs.state == "clear"
  and all(.generated_outputs.outputs[]; .exists == false)
  and .generated_artifacts.state == "present"
  and all(.generated_artifacts.artifacts[] | select(.required == true);
    .exists == true
    and .file_type == "file"
    and (.size_bytes | type == "number")
    and .size_bytes > 0
    and (.sha256 | test("^[0-9a-f]{64}$")))
  and any(.generated_artifacts.artifacts[];
    .path == "dist/engram-0.2.0-aarch64-apple-darwin.tar.gz"
    and .required == true
    and .exists == true
    and .file_type == "file"
    and (.size_bytes | type == "number")
    and .size_bytes > 0
    and (.sha256 | test("^[0-9a-f]{64}$")))
  and any(.generated_artifacts.artifacts[];
    .path == "dist/engram-0.2.0-aarch64-apple-darwin.tar.gz.sha256"
    and .required == true
    and .exists == true
    and .file_type == "file"
    and (.size_bytes | type == "number")
    and .size_bytes > 0
    and (.sha256 | test("^[0-9a-f]{64}$")))
  and any(.generated_artifacts.artifacts[];
    .path == "dist/homebrew/Formula/engram.rb"
    and .required == true
    and .exists == true
    and .file_type == "file"
    and (.size_bytes | type == "number")
    and .size_bytes > 0
    and (.sha256 | test("^[0-9a-f]{64}$")))
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

archive="dist/engram-${release_version}-aarch64-apple-darwin.tar.gz"
checksum="${archive}.sha256"
formula="dist/homebrew/Formula/engram.rb"
test -f "$archive"
test -f "$checksum"
test -f "$formula"
(cd dist && shasum -a 256 -c "$(basename "$checksum")")
ruby -c "$formula"
```

Create the signed tag and publish the GitHub release assets that the full GA gate just produced.
Do not rerun `scripts/package-release.sh` or `scripts/render-homebrew-formula.sh` here: the
successful full gate already ran package/install smoke and Homebrew formula render, and the release
scripts intentionally refuse to overwrite those generated outputs by default.

```bash
git tag -s "$tag" -m "engram ${tag}" "$release_head"
git push origin "$tag"

gh release create "$tag" \
  "$archive" \
  "$checksum" \
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
