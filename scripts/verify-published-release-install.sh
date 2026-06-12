#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repo_root"

default_repo="ymeiri/engram"
repo="${GITHUB_REPOSITORY:-$default_repo}"
allow_release_repo_override="${ALLOW_RELEASE_REPOSITORY_OVERRIDE:-0}"
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

Verify release archive/checksum assets by running the package install smoke
against downloaded GitHub release assets or a local pre-publish asset directory.

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

Environment overrides:
  GITHUB_REPOSITORY, ALLOW_RELEASE_REPOSITORY_OVERRIDE.

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

sha256_file() {
    shasum -a 256 "$1" | awk '{ print $1 }'
}

release_asset_digest() {
    local asset_name="$1"

    jq -er --arg name "$asset_name" '
        .assets[]
        | select(.name == $name)
        | .digest
        | strings
    ' "$release_json"
}

validate_local_release_tag_signature() {
    git tag -v "$tag" >/dev/null
    local_tag_signature_verified=true
}

validate_remote_release_tag() {
    local remote_url="https://github.com/${repo}.git"
    local refs tag_ref peeled_ref

    tag_ref="refs/tags/${tag}"
    peeled_ref="${tag_ref}^{}"
    if ! refs="$(git ls-remote --tags "$remote_url" "$tag" "${tag}^{}")"; then
        fail "could not inspect remote git tag $tag in $repo"
    fi

    remote_tag_object="$(
        awk -v ref="$tag_ref" '$2 == ref { print $1 }' <<<"$refs" | tail -n 1
    )"
    remote_tag_commit="$(
        awk -v ref="$peeled_ref" '$2 == ref { print $1 }' <<<"$refs" | tail -n 1
    )"
    [[ -n "$remote_tag_object" ]] || fail "remote git tag is missing: $tag in $repo"
    [[ -n "$remote_tag_commit" ]] || remote_tag_commit="$remote_tag_object"

    [[ "$remote_tag_object" == "$tag_object" ]] ||
        fail "remote tag object mismatch for $tag: expected $tag_object, got $remote_tag_object"
    [[ "$remote_tag_commit" == "$expected_git_head" ]] ||
        fail "remote tag commit mismatch for $tag: expected $expected_git_head, got $remote_tag_commit"

    remote_tag_verified=true
}

validate_release_asset_list() {
    if jq -e \
        --arg archive "$tarball_name" \
        --arg checksum "$checksum_name" \
        '
            (.assets | type == "array")
            and ([.assets[].name] | sort == ([$archive, $checksum] | sort))
            and all(.assets[];
                .state == "uploaded"
                and ((.size | type) == "number")
                and (.size > 0)
                and ((.digest | type) == "string")
                and (.digest | test("^sha256:[0-9a-f]{64}$")))
        ' "$release_json" >/dev/null; then
        release_asset_list_verified=true
        return 0
    fi

    printf 'error: GitHub release assets must be exactly the expected archive and checksum:\n' >&2
    printf '  %s\n' "$tarball_name" >&2
    printf '  %s\n' "$checksum_name" >&2
    printf 'actual release assets:\n' >&2
    jq -r '
        (.assets // [])
        | .[]
        | "  name=\(.name // "<none>") state=\(.state // "<none>")"
            + " size=\(.size // "<none>") digest=\(.digest // "<none>")"
    ' "$release_json" >&2
    exit 1
}

validate_downloaded_asset_digests() {
    local asset_name expected_digest actual_digest file_path

    for asset_name in "$tarball_name" "$checksum_name"; do
        file_path="$asset_dir/$asset_name"
        [[ -s "$file_path" ]] || fail "release asset is missing or empty: $file_path"

        expected_digest="$(release_asset_digest "$asset_name")"
        actual_digest="sha256:$(sha256_file "$file_path")"
        if [[ "$actual_digest" != "$expected_digest" ]]; then
            printf 'error: GitHub asset digest mismatch for %s: expected %s, got %s\n' \
                "$asset_name" "$expected_digest" "$actual_digest" >&2
            exit 1
        fi
    done

    release_asset_digests_verified=true
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

case "$allow_release_repo_override" in
    0 | 1) ;;
    *) fail "ALLOW_RELEASE_REPOSITORY_OVERRIDE must be 0 or 1, got $allow_release_repo_override" ;;
esac
if [[ "$repo" != "$default_repo" && "$allow_release_repo_override" != "1" ]]; then
    printf 'error: release repository override requires explicit approval\n' >&2
    printf 'expected default: %s\n' "$default_repo" >&2
    printf 'got: %s\n' "$repo" >&2
    printf 'hint: set ALLOW_RELEASE_REPOSITORY_OVERRIDE=1 only for local rehearsals\n' >&2
    exit 1
fi
if [[ ! "$repo" =~ ^[A-Za-z0-9_.-]+/[A-Za-z0-9_.-]+$ ]]; then
    fail "release repository must be owner/name, got $repo"
fi
if [[ ! "$expected_git_head" =~ ^[0-9a-f]{40}$ ]]; then
    fail "--expected-git-head must be a 40-character Git SHA, got $expected_git_head"
fi

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
release_asset_list_verified=false
release_asset_digests_verified=false
local_tag_signature_verified=false
remote_tag_verified=false
tag_object=""
tag_commit=""
remote_tag_object=""
remote_tag_commit=""
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
        --json tagName,name,isDraft,isPrerelease,url,targetCommitish,assets >"$release_json"

    release_tag="$(jq -r '.tagName // empty' "$release_json")"
    release_draft="$(jq -r '.isDraft' "$release_json")"
    release_prerelease="$(jq -r '.isPrerelease' "$release_json")"
    [[ "$release_tag" == "$tag" ]] ||
        fail "release tag mismatch: expected $tag, got ${release_tag:-<none>}"
    [[ "$release_draft" == "false" ]] ||
        fail "release is still a draft: $tag"
    [[ "$release_prerelease" == "$resolved_expected_prerelease" ]] ||
        fail "release prerelease state mismatch for $tag: expected $resolved_expected_prerelease, got $release_prerelease"

    if ! tag_object="$(git rev-parse "$tag" 2>/dev/null)"; then
        fail "local git tag is missing: $tag"
    fi
    if ! tag_commit="$(git rev-parse "${tag}^{commit}" 2>/dev/null)"; then
        fail "local git tag is missing or not peelable: $tag"
    fi
    [[ "$tag_commit" == "$expected_git_head" ]] ||
        fail "release tag commit mismatch for $tag: expected $expected_git_head, got $tag_commit"

    run_step "verify local release tag signature" validate_local_release_tag_signature
    run_step "verify remote release tag" validate_remote_release_tag
    run_step "verify GitHub release asset list" validate_release_asset_list
    run_step "download release assets" gh release download "$tag" \
        --repo "$repo" \
        --pattern "$tarball_name" \
        --pattern "$checksum_name" \
        --dir "$asset_dir"
    downloaded_assets=true
    run_step "verify GitHub release asset digests" validate_downloaded_asset_digests
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
        --arg release_asset_list_verified "$release_asset_list_verified" \
        --arg release_asset_digests_verified "$release_asset_digests_verified" \
        --arg local_tag_signature_verified "$local_tag_signature_verified" \
        --arg remote_tag_verified "$remote_tag_verified" \
        --arg tag_object "$tag_object" \
        --arg tag_commit "$tag_commit" \
        --arg remote_tag_object "$remote_tag_object" \
        --arg remote_tag_commit "$remote_tag_commit" \
        --slurpfile release "$release_json" \
        '{
            repo: $repo,
            tag: $tag,
            tag_object: (if $tag_object == "" then null else $tag_object end),
            tag_commit: (if $tag_commit == "" then null else $tag_commit end),
            local_tag_signature_verified: ($local_tag_signature_verified == "true"),
            remote_tag: {
                object: (if $remote_tag_object == "" then null else $remote_tag_object end),
                commit: (if $remote_tag_commit == "" then null else $remote_tag_commit end),
                verified: ($remote_tag_verified == "true")
            },
            version: $version,
            host_triple: $host_triple,
            assets: {
                source: (
                    if $downloaded_assets == "true" then "github_release"
                    else "asset_dir"
                    end
                ),
                directory: $asset_dir,
                archive: $archive,
                checksum: $checksum,
                downloaded: ($downloaded_assets == "true"),
                release_asset_list_verified: ($release_asset_list_verified == "true"),
                release_asset_digests_verified: ($release_asset_digests_verified == "true")
            },
            release: ($release[0] // {}),
            expected_git_head: $expected_git_head,
            expected_tracked_changes_present: ($expected_tracked_changes_present == "true"),
            expected_prerelease: ($expected_prerelease == "true"),
            install_smoke: "passed",
            asset_install_verified: true,
            published_install_verified: ($downloaded_assets == "true"),
            release_actions_performed: false
        }'
else
    if [[ "$downloaded_assets" == "true" ]]; then
        printf '\nPublished release install evidence collected:\n'
    else
        printf '\nLocal release asset install evidence collected:\n'
    fi
    printf '  repo: %s\n' "$repo"
    printf '  tag: %s\n' "$tag"
    if [[ -n "$tag_object" ]]; then
        printf '  tag_object: %s\n' "$tag_object"
    fi
    if [[ -n "$tag_commit" ]]; then
        printf '  tag_commit: %s\n' "$tag_commit"
    fi
    printf '  local_tag_signature_verified: %s\n' "$local_tag_signature_verified"
    if [[ -n "$remote_tag_object" ]]; then
        printf '  remote_tag_object: %s\n' "$remote_tag_object"
    fi
    if [[ -n "$remote_tag_commit" ]]; then
        printf '  remote_tag_commit: %s\n' "$remote_tag_commit"
    fi
    printf '  remote_tag_verified: %s\n' "$remote_tag_verified"
    printf '  version: %s\n' "$package_version"
    printf '  host_triple: %s\n' "$host_triple"
    printf '  expected_prerelease: %s\n' "$resolved_expected_prerelease"
    printf '  assets_source: %s\n' \
        "$([[ "$downloaded_assets" == "true" ]] && printf 'github_release' || printf 'asset_dir')"
    printf '  archive: %s\n' "$tarball_name"
    printf '  checksum: %s\n' "$checksum_name"
    printf '  release_asset_list_verified: %s\n' "$release_asset_list_verified"
    printf '  release_asset_digests_verified: %s\n' "$release_asset_digests_verified"
    printf '  install_smoke: passed\n'
    printf '  asset_install_verified: true\n'
    printf '  published_install_verified: %s\n' \
        "$([[ "$downloaded_assets" == "true" ]] && printf true || printf false)"
    printf '  release_actions_performed: false\n'
fi
