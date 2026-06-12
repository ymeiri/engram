#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repo_root"

dist_dir="${DIST_DIR:-$repo_root/dist}"
package_version="$(cargo pkgid --locked -p engram-cli | sed 's/.*#//')"
host_triple="${HOMEBREW_HOST_TRIPLE:-$(rustc -vV | awk '/^host:/ { print $2 }')}"
archive_name="engram-${package_version}-${host_triple}"
tarball="$dist_dir/$archive_name.tar.gz"
checksum="$tarball.sha256"
output="${FORMULA_OUTPUT:-$dist_dir/homebrew/Formula/engram.rb}"
release_base_url="${HOMEBREW_RELEASE_BASE_URL:-https://github.com/ymeiri/engram/releases/download/v${package_version}}"

command -v jq >/dev/null 2>&1 || {
    printf 'error: required tool is missing: jq\n' >&2
    exit 1
}

if [[ "$host_triple" != "aarch64-apple-darwin" ]]; then
    printf 'error: Homebrew formula currently supports aarch64-apple-darwin only, got %s\n' \
        "$host_triple" >&2
    exit 1
fi

if [[ ! -f "$tarball" ]]; then
    printf 'error: release tarball not found at %s\n' "$tarball" >&2
    printf 'hint: run scripts/package-release.sh first\n' >&2
    exit 1
fi
if [[ ! -f "$checksum" ]]; then
    printf 'error: release checksum not found at %s\n' "$checksum" >&2
    printf 'hint: run scripts/package-release.sh first\n' >&2
    exit 1
fi

sha256="$(shasum -a 256 "$tarball" | awk '{ print $1 }')"
checksum_line_count="$(wc -l <"$checksum" | tr -d '[:space:]')"
if [[ "$checksum_line_count" != "1" ]]; then
    printf 'error: checksum file must contain exactly one line: %s\n' "$checksum" >&2
    exit 1
fi

read -r checksum_sha256 checksum_name checksum_extra <"$checksum" || {
    printf 'error: checksum file is unreadable: %s\n' "$checksum" >&2
    exit 1
}
if [[ -n "${checksum_extra:-}" ]]; then
    printf 'error: checksum file has unexpected extra fields: %s\n' "$checksum" >&2
    exit 1
fi
if [[ ! "$checksum_sha256" =~ ^[0-9a-f]{64}$ ]]; then
    printf 'error: checksum digest is not a SHA-256 hex value: %s\n' "$checksum" >&2
    exit 1
fi
if [[ "$checksum_name" != "$(basename "$tarball")" ]]; then
    printf 'error: checksum filename mismatch: expected %s, got %s\n' \
        "$(basename "$tarball")" "$checksum_name" >&2
    exit 1
fi
if [[ "$checksum_sha256" != "$sha256" ]]; then
    printf 'error: checksum digest mismatch for %s: expected %s, got %s\n' \
        "$tarball" "$sha256" "$checksum_sha256" >&2
    exit 1
fi

expected_git_head="${EXPECTED_PACKAGE_GIT_HEAD:-$(git rev-parse HEAD)}"
expected_cargo_lock_sha256="$(
    shasum -a 256 Cargo.lock | awk '{ print $1 }'
)"
if [[ -n "${EXPECTED_CARGO_LOCK_SHA256:-}" ]]; then
    expected_cargo_lock_sha256="$EXPECTED_CARGO_LOCK_SHA256"
fi
if [[ -n "${EXPECTED_TRACKED_CHANGES_PRESENT:-}" ]]; then
    expected_tracked_changes_present="$EXPECTED_TRACKED_CHANGES_PRESENT"
elif git diff --quiet --ignore-submodules -- &&
    git diff --cached --quiet --ignore-submodules --; then
    expected_tracked_changes_present=false
else
    expected_tracked_changes_present=true
fi
if [[ "$expected_tracked_changes_present" != "true" &&
    "$expected_tracked_changes_present" != "false" ]]; then
    printf 'error: EXPECTED_TRACKED_CHANGES_PRESENT must be true or false, got %s\n' \
        "$expected_tracked_changes_present" >&2
    exit 1
fi

manifest_member="$archive_name/MANIFEST.json"
manifest_dir="$(mktemp -d "${TMPDIR:-/tmp}/engram-homebrew-manifest.XXXXXX")"
trap 'rm -rf "$manifest_dir"' EXIT
manifest="$manifest_dir/MANIFEST.json"
if ! tar -xzOf "$tarball" "$manifest_member" >"$manifest"; then
    printf 'error: release archive is missing required member: %s\n' \
        "$manifest_member" >&2
    exit 1
fi
if [[ ! -s "$manifest" ]]; then
    printf 'error: packaged manifest is empty: %s\n' "$manifest_member" >&2
    exit 1
fi

manifest_package="$(jq -er '.package | strings' "$manifest")"
manifest_version="$(jq -er '.version | strings' "$manifest")"
manifest_host_triple="$(jq -er '.host_triple | strings' "$manifest")"
manifest_archive_name="$(jq -er '.archive_name | strings' "$manifest")"
manifest_git_head="$(jq -er '.git_head | strings' "$manifest")"
manifest_tracked_changes_present="$(
    jq -er '.tracked_changes_present | booleans | tostring' "$manifest"
)"
manifest_cargo_lock_sha256="$(jq -er '.cargo_lock_sha256 | strings' "$manifest")"

if [[ "$manifest_package" != "engram" ]]; then
    printf 'error: manifest package mismatch: expected engram, got %s\n' \
        "$manifest_package" >&2
    exit 1
fi
if [[ "$manifest_version" != "$package_version" ]]; then
    printf 'error: manifest version mismatch: expected %s, got %s\n' \
        "$package_version" "$manifest_version" >&2
    exit 1
fi
if [[ "$manifest_host_triple" != "$host_triple" ]]; then
    printf 'error: manifest host triple mismatch: expected %s, got %s\n' \
        "$host_triple" "$manifest_host_triple" >&2
    exit 1
fi
if [[ "$manifest_archive_name" != "$archive_name" ]]; then
    printf 'error: manifest archive name mismatch: expected %s, got %s\n' \
        "$archive_name" "$manifest_archive_name" >&2
    exit 1
fi
if [[ "$manifest_git_head" != "$expected_git_head" ]]; then
    printf 'error: manifest git head mismatch: expected %s, got %s\n' \
        "$expected_git_head" "$manifest_git_head" >&2
    exit 1
fi
if [[ "$manifest_tracked_changes_present" != "$expected_tracked_changes_present" ]]; then
    printf 'error: manifest tracked-changes flag mismatch: expected %s, got %s\n' \
        "$expected_tracked_changes_present" "$manifest_tracked_changes_present" >&2
    exit 1
fi
if [[ "$manifest_cargo_lock_sha256" != "$expected_cargo_lock_sha256" ]]; then
    printf 'error: manifest Cargo.lock hash mismatch: expected %s, got %s\n' \
        "$expected_cargo_lock_sha256" "$manifest_cargo_lock_sha256" >&2
    exit 1
fi

mkdir -p "$(dirname "$output")"

cat >"$output" <<EOF
class Engram < Formula
  desc "Personal Knowledge Augmentation System for AI coding agents"
  homepage "https://github.com/ymeiri/engram"
  url "${release_base_url}/${archive_name}.tar.gz"
  sha256 "${sha256}"
  license "Apache-2.0"
  depends_on arch: :arm64

  def install
    odie "engram #{version} Homebrew package currently supports macOS only" if OS.linux?
    odie "engram #{version} Homebrew package currently supports Apple Silicon only" unless Hardware::CPU.arm?

    bin.install "engram"
    prefix.install "README.md", "CHANGELOG.md", "LICENSE", "RELEASE_NOTES.md", "MANIFEST.json"
  end

  def caveats
    <<~EOS
      First run:
        engram warmup embeddings
        engram init
        engram setup

      Restart your agent after writing setup files, then ask it to run orient.
    EOS
  end

  test do
    assert_match "engram #{version}", shell_output("#{bin/"engram"} --version")
    system bin/"engram", "harness", "status", "--harness", "codex", "--root", testpath
  end
end
EOF

printf 'Homebrew formula rendered:\n'
printf '  %s\n' "$output"
printf 'Tarball SHA-256:\n'
printf '  %s\n' "$sha256"
