#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repo_root"

repo="${GITHUB_REPOSITORY:-ymeiri/engram}"
package_version="$(cargo pkgid --locked -p engram-cli | sed 's/.*#//')"
host_triple="$(rustc -vV | awk '/^host:/ { print $2 }')"
tag="v${package_version}"
asset_dir=""
expected_git_head="$(git rev-parse HEAD)"
expected_tracked_changes_present=false
expected_prerelease="auto"
json_output=0

usage() {
    cat <<'USAGE'
Usage: scripts/verify-published-release-install.sh [options]

Verify published release assets by downloading the archive/checksum and running
the package install smoke against those assets.

Options:
  --repo <owner/name>                    GitHub repository (default: GITHUB_REPOSITORY or ymeiri/engram)
  --tag <tag>                            GitHub release tag (default: v<workspace package version>)
  --host-triple <triple>                 Release host triple (default: current rustc host)
  --expected-git-head <sha>              Expected MANIFEST.json git head (default: current HEAD)
  --expected-tracked-changes-present <bool>
                                        Expected MANIFEST.json tracked-change flag (default: false)
  --expected-prerelease <auto|true|false>
                                        Expected GitHub release prerelease state (default: auto from tag suffix)
  --asset-dir <path>                     Validate existing archive/checksum directory instead of downloading
  --json                                 Emit final evidence as machine-readable JSON
  -h, --help                             Show this help

This script is evidence only. It does not create a GitHub release, upload assets,
accept a hosted-CI fallback, mark a PR ready, merge, tag, publish, or mutate release state.
USAGE
}

fail() {
    printf 'error: %s\n' "$*" >&2
    exit 1
}

require_tool() {
    local tool="$1"
    command -v "$tool" >/dev/null 2>&1 || fail "required tool is missing: $tool"
}

run_step() {
    local name="$1"
    shift
    if [[ "$json_output" == "1" ]]; then
        printf '\n==> %s\n' "$name" >&2
        "$@" >&2
    else
        printf '\n==> %s\n' "$name"
        "$@"
    fi
}

while [[ $# -gt 0 ]]; do
    case "$1" in
        --repo)
            [[ $# -ge 2 ]] || fail "--repo requires owner/name"
            repo="$2"
            shift 2
            ;;
        --tag)
            [[ $# -ge 2 ]] || fail "--tag requires a tag"
            tag="$2"
            shift 2
            ;;
        --host-triple)
            [[ $# -ge 2 ]] || fail "--host-triple requires a host triple"
            host_triple="$2"
            shift 2
            ;;
        --expected-git-head)
            [[ $# -ge 2 ]] || fail "--expected-git-head requires a commit SHA"
            expected_git_head="$2"
            shift 2
            ;;
        --expected-tracked-changes-present)
            [[ $# -ge 2 ]] || fail "--expected-tracked-changes-present requires true or false"
            case "$2" in
                true | false) expected_tracked_changes_present="$2" ;;
                *) fail "--expected-tracked-changes-present must be true or false" ;;
            esac
            shift 2
            ;;
        --expected-prerelease)
            [[ $# -ge 2 ]] || fail "--expected-prerelease requires auto, true, or false"
            case "$2" in
                auto | true | false) expected_prerelease="$2" ;;
                *) fail "--expected-prerelease must be auto, true, or false" ;;
            esac
            shift 2
            ;;
        --asset-dir)
            [[ $# -ge 2 ]] || fail "--asset-dir requires a path"
            asset_dir="$2"
            shift 2
            ;;
        --json)
            json_output=1
            shift
            ;;
        -h | --help)
            usage
            exit 0
            ;;
        *)
            fail "unknown option: $1"
            ;;
    esac
done

require_tool cargo
require_tool rustc
require_tool jq

expected_tag="v${package_version}"
[[ "$tag" == "$expected_tag" ]] ||
    fail "release tag version mismatch: expected $expected_tag for workspace package version $package_version, got $tag"

archive_name="engram-${package_version}-${host_triple}"
tarball_name="${archive_name}.tar.gz"
checksum_name="${tarball_name}.sha256"
resolved_expected_prerelease="$expected_prerelease"
if [[ "$resolved_expected_prerelease" == "auto" ]]; then
    if [[ "$tag" == *"-"* ]]; then
        resolved_expected_prerelease=true
    else
        resolved_expected_prerelease=false
    fi
fi
downloaded_assets=false
tag_commit=""
release_json="$(mktemp "${TMPDIR:-/tmp}/engram-release-view.XXXXXX")"
work_dir="$(mktemp -d "${TMPDIR:-/tmp}/engram-release-install.XXXXXX")"

cleanup() {
    rm -f "$release_json"
    rm -rf "$work_dir"
}
trap cleanup EXIT

if [[ -z "$asset_dir" ]]; then
    require_tool gh
    asset_dir="$work_dir/assets"
    mkdir -p "$asset_dir"

    if [[ "$json_output" == "1" ]]; then
        printf '\n==> inspect GitHub release\n' >&2
    else
        printf '\n==> inspect GitHub release\n'
    fi
    gh release view "$tag" \
        --repo "$repo" \
        --json tagName,name,isDraft,isPrerelease,url,targetCommitish >"$release_json"

    release_tag="$(jq -r '.tagName // empty' "$release_json")"
    release_draft="$(jq -r '.isDraft' "$release_json")"
    release_prerelease="$(jq -r '.isPrerelease' "$release_json")"
    [[ "$release_tag" == "$tag" ]] ||
        fail "release tag mismatch: expected $tag, got ${release_tag:-<none>}"
    [[ "$release_draft" == "false" ]] ||
        fail "release is still a draft: $tag"
    [[ "$release_prerelease" == "$resolved_expected_prerelease" ]] ||
        fail "release prerelease state mismatch for $tag: expected $resolved_expected_prerelease, got $release_prerelease"

    if ! tag_commit="$(git rev-parse "${tag}^{commit}" 2>/dev/null)"; then
        fail "local git tag is missing or not peelable: $tag"
    fi
    [[ "$tag_commit" == "$expected_git_head" ]] ||
        fail "release tag commit mismatch for $tag: expected $expected_git_head, got $tag_commit"

    run_step "download release assets" gh release download "$tag" \
        --repo "$repo" \
        --pattern "$tarball_name" \
        --pattern "$checksum_name" \
        --dir "$asset_dir"
    downloaded_assets=true
else
    [[ -d "$asset_dir" ]] || fail "asset directory does not exist: $asset_dir"
    printf '{}' >"$release_json"
fi

if [[ ! -s "$asset_dir/$tarball_name" ]]; then
    fail "release archive asset is missing or empty: $asset_dir/$tarball_name"
fi
if [[ ! -s "$asset_dir/$checksum_name" ]]; then
    fail "release checksum asset is missing or empty: $asset_dir/$checksum_name"
fi

run_step "verify release install" env \
    DIST_DIR="$asset_dir" \
    SKIP_PACKAGE_BUILD=1 \
    EXPECTED_PACKAGE_GIT_HEAD="$expected_git_head" \
    EXPECTED_TRACKED_CHANGES_PRESENT="$expected_tracked_changes_present" \
    "$repo_root/scripts/package-install-smoke.sh"

if [[ "$json_output" == "1" ]]; then
    jq -n \
        --arg repo "$repo" \
        --arg tag "$tag" \
        --arg version "$package_version" \
        --arg host_triple "$host_triple" \
        --arg archive "$tarball_name" \
        --arg checksum "$checksum_name" \
        --arg asset_dir "$asset_dir" \
        --arg expected_git_head "$expected_git_head" \
        --arg expected_tracked_changes_present "$expected_tracked_changes_present" \
        --arg expected_prerelease "$resolved_expected_prerelease" \
        --arg downloaded_assets "$downloaded_assets" \
        --arg tag_commit "$tag_commit" \
        --slurpfile release "$release_json" \
        '{
            repo: $repo,
            tag: $tag,
            tag_commit: (if $tag_commit == "" then null else $tag_commit end),
            version: $version,
            host_triple: $host_triple,
            assets: {
                source: (if $downloaded_assets == "true" then "github_release" else "asset_dir" end),
                directory: $asset_dir,
                archive: $archive,
                checksum: $checksum,
                downloaded: ($downloaded_assets == "true")
            },
            release: ($release[0] // {}),
            expected_git_head: $expected_git_head,
            expected_tracked_changes_present: ($expected_tracked_changes_present == "true"),
            expected_prerelease: ($expected_prerelease == "true"),
            install_smoke: "passed",
            published_install_verified: true,
            release_actions_performed: false
        }'
else
    printf '\nPublished release install evidence collected:\n'
    printf '  repo: %s\n' "$repo"
    printf '  tag: %s\n' "$tag"
    if [[ -n "$tag_commit" ]]; then
        printf '  tag_commit: %s\n' "$tag_commit"
    fi
    printf '  version: %s\n' "$package_version"
    printf '  host_triple: %s\n' "$host_triple"
    printf '  expected_prerelease: %s\n' "$resolved_expected_prerelease"
    printf '  assets_source: %s\n' \
        "$([[ "$downloaded_assets" == "true" ]] && printf 'github_release' || printf 'asset_dir')"
    printf '  archive: %s\n' "$tarball_name"
    printf '  checksum: %s\n' "$checksum_name"
    printf '  install_smoke: passed\n'
    printf '  published_install_verified: true\n'
    printf '  release_actions_performed: false\n'
fi
