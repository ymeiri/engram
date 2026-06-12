#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repo_root"

dist_dir="${DIST_DIR:-$repo_root/dist}"
package_version="$(cargo pkgid --locked -p engram-cli | sed 's/.*#//')"
release_notes_slug="$(printf '%s' "$package_version" | tr '[:lower:]' '[:upper:]' | sed 's/[^A-Z0-9]/_/g')"
release_notes_source="$repo_root/docs/RELEASE_NOTES_V${release_notes_slug}.md"
host_triple="$(rustc -vV | awk '/^host:/ { print $2 }')"
archive_name="engram-${package_version}-${host_triple}"
allow_tracked_changes="${ALLOW_TRACKED_CHANGES:-0}"
work_dir="$(mktemp -d "${TMPDIR:-/tmp}/engram-package.XXXXXX")"

cleanup() {
    rm -rf "$work_dir"
}
trap cleanup EXIT

run_step() {
    local name="$1"
    shift
    printf '\n==> %s\n' "$name"
    "$@"
}

sha256_file() {
    shasum -a 256 "$1" | awk '{ print $1 }'
}

command -v jq >/dev/null 2>&1 || {
    printf 'error: required tool is missing: jq\n' >&2
    exit 1
}

git_head="$(git rev-parse HEAD)"
if git diff --quiet --ignore-submodules -- && git diff --cached --quiet --ignore-submodules --; then
    tracked_changes_present=false
else
    tracked_changes_present=true
fi

if [[ "$tracked_changes_present" == "true" && "$allow_tracked_changes" != "1" ]]; then
    printf 'error: tracked working-tree or index changes are present; commit changes first\n' >&2
    printf 'hint: set ALLOW_TRACKED_CHANGES=1 only for local development rehearsals\n' >&2
    exit 1
fi

run_step "build release binary" cargo build --locked --release -p engram-cli

if [[ ! -f "$release_notes_source" ]]; then
    printf 'error: release notes not found for version %s: %s\n' \
        "$package_version" "$release_notes_source" >&2
    exit 1
fi

binary="$repo_root/target/release/engram"
if [[ ! -x "$binary" ]]; then
    printf 'error: release binary was not built at %s\n' "$binary" >&2
    exit 1
fi

expected_version="engram ${package_version}"
actual_version="$("$binary" --version)"
if [[ "$actual_version" != "$expected_version" ]]; then
    printf 'error: binary version mismatch: expected "%s", got "%s"\n' \
        "$expected_version" "$actual_version" >&2
    exit 1
fi

staging_dir="$work_dir/$archive_name"
mkdir -p "$staging_dir" "$dist_dir"
cp "$binary" "$staging_dir/engram"
cp README.md LICENSE CHANGELOG.md "$staging_dir/"
cp "$release_notes_source" "$staging_dir/RELEASE_NOTES.md"
chmod 755 "$staging_dir/engram"

cargo_lock_sha256="$(sha256_file Cargo.lock)"

manifest="$staging_dir/MANIFEST.json"
cat >"$manifest" <<EOF
{
  "package":"engram",
  "version":"${package_version}",
  "host_triple":"${host_triple}",
  "archive_name":"${archive_name}",
  "git_head":"${git_head}",
  "tracked_changes_present":${tracked_changes_present},
  "cargo_lock_sha256":"${cargo_lock_sha256}",
  "files":[
    {"path":"engram","sha256":"$(sha256_file "$staging_dir/engram")"},
    {"path":"README.md","sha256":"$(sha256_file "$staging_dir/README.md")"},
    {"path":"LICENSE","sha256":"$(sha256_file "$staging_dir/LICENSE")"},
    {"path":"CHANGELOG.md","sha256":"$(sha256_file "$staging_dir/CHANGELOG.md")"},
    {"path":"RELEASE_NOTES.md","sha256":"$(sha256_file "$staging_dir/RELEASE_NOTES.md")"}
  ]
}
EOF

jq -e \
    --arg version "$package_version" \
    --arg host_triple "$host_triple" \
    --arg archive_name "$archive_name" \
    --arg git_head "$git_head" \
    --arg tracked_changes_present "$tracked_changes_present" \
    --arg cargo_lock_sha256 "$cargo_lock_sha256" \
    '
        .package == "engram"
        and .version == $version
        and .host_triple == $host_triple
        and .archive_name == $archive_name
        and .git_head == $git_head
        and (.git_head | test("^[0-9a-f]{40}$"))
        and .tracked_changes_present == ($tracked_changes_present == "true")
        and .cargo_lock_sha256 == $cargo_lock_sha256
        and (.cargo_lock_sha256 | test("^[0-9a-f]{64}$"))
        and (.files | type == "array")
        and ([.files[].path] | sort == [
            "CHANGELOG.md",
            "LICENSE",
            "README.md",
            "RELEASE_NOTES.md",
            "engram"
        ])
        and all(.files[]; (.path | type == "string")
            and (.sha256 | type == "string")
            and (.sha256 | test("^[0-9a-f]{64}$")))
    ' "$manifest" >/dev/null || {
        printf 'error: generated package manifest is invalid: %s\n' "$manifest" >&2
        exit 1
    }

for package_file in engram README.md LICENSE CHANGELOG.md RELEASE_NOTES.md; do
    actual_sha256="$(sha256_file "$staging_dir/$package_file")"
    if ! manifest_sha256="$(
        jq -er --arg path "$package_file" '
            .files[]
            | select(.path == $path)
            | .sha256
            | select(test("^[0-9a-f]{64}$"))
        ' "$manifest"
    )"; then
        printf 'error: manifest is missing a valid SHA-256 entry for %s\n' \
            "$package_file" >&2
        exit 1
    fi

    if [[ "$manifest_sha256" != "$actual_sha256" ]]; then
        printf 'error: manifest hash mismatch for %s: expected %s, got %s\n' \
            "$package_file" "$actual_sha256" "$manifest_sha256" >&2
        exit 1
    fi
done

tarball="$dist_dir/$archive_name.tar.gz"
checksum="$tarball.sha256"
rm -f "$tarball" "$checksum"

run_step "create archive" tar -C "$work_dir" -czf "$tarball" "$archive_name"
(
    cd "$dist_dir"
    shasum -a 256 "$(basename "$tarball")" > "$(basename "$checksum")"
)

printf '\nRelease package created:\n'
printf '  %s\n' "$tarball"
printf '  %s\n' "$checksum"
