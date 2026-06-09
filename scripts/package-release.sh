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

sha256_file() {
    shasum -a 256 "$1" | awk '{ print $1 }'
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

git_head="$(git rev-parse HEAD)"
if git diff --quiet --ignore-submodules -- && git diff --cached --quiet --ignore-submodules --; then
    tracked_changes_present=false
else
    tracked_changes_present=true
fi
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
