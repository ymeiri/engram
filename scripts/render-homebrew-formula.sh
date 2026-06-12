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
