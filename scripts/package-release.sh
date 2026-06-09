#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repo_root"

dist_dir="${DIST_DIR:-$repo_root/dist}"
package_version="$(cargo pkgid --locked -p engram-cli | sed 's/.*#//')"
host_triple="$(rustc -vV | awk '/^host:/ { print $2 }')"
archive_name="engram-${package_version}-${host_triple}"
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

run_step "build release binary" cargo build --locked --release -p engram-cli

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
cp docs/RELEASE_NOTES_V0_2_0_BETA_1.md "$staging_dir/RELEASE_NOTES.md"
chmod 755 "$staging_dir/engram"

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
