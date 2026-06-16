#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repo_root"

default_dist_dir="$repo_root/dist"
dist_dir="${DIST_DIR-$default_dist_dir}"
allow_dist_dir_override="${ALLOW_HOMEBREW_DIST_DIR_OVERRIDE:-0}"
allow_formula_overwrite="${ALLOW_HOMEBREW_FORMULA_OVERWRITE:-0}"

command -v cargo >/dev/null 2>&1 || {
    printf 'error: required tool is missing: cargo\n' >&2
    exit 1
}
command -v rustc >/dev/null 2>&1 || {
    printf 'error: required tool is missing: rustc\n' >&2
    exit 1
}
command -v jq >/dev/null 2>&1 || {
    printf 'error: required tool is missing: jq\n' >&2
    exit 1
}
command -v ruby >/dev/null 2>&1 || {
    printf 'error: required tool is missing: ruby\n' >&2
    exit 1
}

if ! package_id="$(cargo pkgid --locked -p engram-cli)"; then
    printf 'error: could not determine workspace package version for engram-cli\n' >&2
    exit 1
fi
package_version="${package_id##*#}"
package_version_pattern='^[0-9]+\.[0-9]+\.[0-9]+(-[A-Za-z0-9][A-Za-z0-9.-]*)?$'
if [[ ! "$package_version" =~ $package_version_pattern ]]; then
    printf 'error: workspace package version must be x.y.z with an optional prerelease suffix, got %s\n' \
        "$package_version" >&2
    exit 1
fi
default_host_triple="$(rustc -vV | awk '/^host:/ { print $2 }')"
host_triple_pattern='^[A-Za-z0-9_.+-]+(-[A-Za-z0-9_.+-]+)+$'
if [[ -z "$default_host_triple" ]]; then
    printf 'error: host triple could not be determined from rustc -vV\n' >&2
    exit 1
fi
if [[ ! "$default_host_triple" =~ $host_triple_pattern ]]; then
    printf 'error: host triple must be a Rust target triple, got %s\n' \
        "$default_host_triple" >&2
    exit 1
fi

supported_homebrew_triples=(
    "aarch64-apple-darwin"
    "x86_64-apple-darwin"
    "x86_64-unknown-linux-gnu"
)
homebrew_package_triples=("${supported_homebrew_triples[@]}")
homebrew_host_triple_explicit=0
if [[ "${HOMEBREW_HOST_TRIPLE+x}" == "x" ]]; then
    homebrew_package_triples=("$HOMEBREW_HOST_TRIPLE")
    homebrew_host_triple_explicit=1
fi
allow_host_triple_override="${ALLOW_HOMEBREW_HOST_TRIPLE_OVERRIDE:-0}"
allow_formula_output_override="${ALLOW_HOMEBREW_FORMULA_OUTPUT_OVERRIDE:-0}"
default_release_base_url="https://github.com/ymeiri/engram/releases/download/v${package_version}"
release_base_url="${HOMEBREW_RELEASE_BASE_URL-$default_release_base_url}"
allow_release_base_url_override="${ALLOW_HOMEBREW_RELEASE_BASE_URL_OVERRIDE:-0}"
allow_package_identity_override="${ALLOW_PACKAGE_IDENTITY_OVERRIDE:-0}"
expected_tracked_changes_present_explicit=0
if [[ "${EXPECTED_TRACKED_CHANGES_PRESENT+x}" == "x" ]]; then
    expected_tracked_changes_present_explicit=1
fi
work_dir=""
tmp_formula=""

cleanup() {
    if [[ -n "$tmp_formula" ]]; then
        rm -f "$tmp_formula"
    fi
    if [[ -n "$work_dir" ]]; then
        rm -rf "$work_dir"
    fi
}
trap cleanup EXIT

if [[ "$allow_dist_dir_override" != "0" &&
    "$allow_dist_dir_override" != "1" ]]; then
    printf 'error: ALLOW_HOMEBREW_DIST_DIR_OVERRIDE must be 0 or 1, got %s\n' \
        "$allow_dist_dir_override" >&2
    exit 1
fi
if [[ -z "$dist_dir" ]]; then
    printf 'error: DIST_DIR must not be empty\n' >&2
    exit 1
fi
if [[ "$dist_dir" != "$default_dist_dir" && "$allow_dist_dir_override" != "1" ]]; then
    printf 'error: DIST_DIR override requires explicit Homebrew approval\n' >&2
    printf 'expected default dist dir: %s\n' "$default_dist_dir" >&2
    printf 'got: %s\n' "$dist_dir" >&2
    printf 'hint: set ALLOW_HOMEBREW_DIST_DIR_OVERRIDE=1 only for local rehearsals\n' >&2
    exit 1
fi

default_output="$dist_dir/homebrew/Formula/engram.rb"
output="${FORMULA_OUTPUT-$default_output}"

if [[ "$allow_host_triple_override" != "0" &&
    "$allow_host_triple_override" != "1" ]]; then
    printf 'error: ALLOW_HOMEBREW_HOST_TRIPLE_OVERRIDE must be 0 or 1, got %s\n' \
        "$allow_host_triple_override" >&2
    exit 1
fi
if [[ "$homebrew_host_triple_explicit" == "1" &&
    "$allow_host_triple_override" != "1" ]]; then
    printf 'error: HOMEBREW_HOST_TRIPLE override requires explicit approval\n' >&2
    printf 'default formula package triples:\n' >&2
    printf '  %s\n' "${supported_homebrew_triples[@]}" >&2
    printf 'got: %s\n' "${homebrew_package_triples[0]}" >&2
    printf 'hint: set ALLOW_HOMEBREW_HOST_TRIPLE_OVERRIDE=1 only for local rehearsals\n' >&2
    exit 1
fi
for package_triple in "${homebrew_package_triples[@]}"; do
    if [[ -z "$package_triple" ]]; then
        printf 'error: HOMEBREW_HOST_TRIPLE must not be empty\n' >&2
        exit 1
    fi
    if [[ ! "$package_triple" =~ $host_triple_pattern ]]; then
        printf 'error: Homebrew package triple must be a Rust target triple, got %s\n' \
            "$package_triple" >&2
        exit 1
    fi

    package_triple_supported=0
    for supported_triple in "${supported_homebrew_triples[@]}"; do
        if [[ "$package_triple" == "$supported_triple" ]]; then
            package_triple_supported=1
            break
        fi
    done
    if [[ "$package_triple_supported" != "1" ]]; then
        printf 'error: Homebrew formula supports these package triples only:\n' >&2
        printf '  %s\n' "${supported_homebrew_triples[@]}" >&2
        printf 'got: %s\n' "$package_triple" >&2
        exit 1
    fi
done

if [[ "$allow_formula_output_override" != "0" &&
    "$allow_formula_output_override" != "1" ]]; then
    printf 'error: ALLOW_HOMEBREW_FORMULA_OUTPUT_OVERRIDE must be 0 or 1, got %s\n' \
        "$allow_formula_output_override" >&2
    exit 1
fi
if [[ -z "$output" ]]; then
    printf 'error: FORMULA_OUTPUT must not be empty\n' >&2
    exit 1
fi
if [[ "$(basename "$output")" != "engram.rb" ]]; then
    printf 'error: FORMULA_OUTPUT must end with engram.rb, got %s\n' "$output" >&2
    exit 1
fi
if [[ -d "$output" && ! -L "$output" ]]; then
    printf 'error: FORMULA_OUTPUT must be a file path, got directory %s\n' "$output" >&2
    exit 1
fi
if [[ "$output" != "$default_output" && "$allow_formula_output_override" != "1" ]]; then
    printf 'error: FORMULA_OUTPUT override requires explicit approval\n' >&2
    printf 'expected default formula output: %s\n' "$default_output" >&2
    printf 'got: %s\n' "$output" >&2
    printf 'hint: set ALLOW_HOMEBREW_FORMULA_OUTPUT_OVERRIDE=1 only for local rehearsals\n' >&2
    exit 1
fi

if [[ "$allow_formula_overwrite" != "0" &&
    "$allow_formula_overwrite" != "1" ]]; then
    printf 'error: ALLOW_HOMEBREW_FORMULA_OVERWRITE must be 0 or 1, got %s\n' \
        "$allow_formula_overwrite" >&2
    exit 1
fi
if [[ -e "$output" || -L "$output" ]]; then
    if [[ "$allow_formula_overwrite" != "1" ]]; then
        printf 'error: Homebrew formula output already exists; refusing to overwrite\n' >&2
        printf 'existing output: %s\n' "$output" >&2
        printf 'hint: remove stale generated formula evidence after approval, or set ' >&2
        printf 'ALLOW_HOMEBREW_FORMULA_OVERWRITE=1 only for local rehearsals\n' >&2
        exit 1
    fi
fi

if [[ "$allow_release_base_url_override" != "0" &&
    "$allow_release_base_url_override" != "1" ]]; then
    printf 'error: ALLOW_HOMEBREW_RELEASE_BASE_URL_OVERRIDE must be 0 or 1, got %s\n' \
        "$allow_release_base_url_override" >&2
    exit 1
fi
if [[ -z "$release_base_url" ]]; then
    printf 'error: HOMEBREW_RELEASE_BASE_URL must not be empty\n' >&2
    exit 1
fi
if [[ "$release_base_url" != "$default_release_base_url" &&
    "$allow_release_base_url_override" != "1" ]]; then
    printf 'error: HOMEBREW_RELEASE_BASE_URL override requires explicit approval\n' >&2
    printf 'expected default release URL base: %s\n' "$default_release_base_url" >&2
    printf 'got: %s\n' "$release_base_url" >&2
    printf 'hint: set ALLOW_HOMEBREW_RELEASE_BASE_URL_OVERRIDE=1 only for local rehearsals\n' >&2
    exit 1
fi
if [[ "$release_base_url" != https://* ]]; then
    printf 'error: Homebrew release URL base must use https: %s\n' "$release_base_url" >&2
    exit 1
fi
if [[ "$release_base_url" == */ ]]; then
    printf 'error: Homebrew release URL base must not end with a slash: %s\n' \
        "$release_base_url" >&2
    exit 1
fi

default_expected_git_head="$(git rev-parse HEAD)"
default_expected_cargo_lock_sha256="$(
    shasum -a 256 Cargo.lock | awk '{ print $1 }'
)"
expected_git_head="${EXPECTED_PACKAGE_GIT_HEAD-$default_expected_git_head}"
expected_cargo_lock_sha256="${EXPECTED_CARGO_LOCK_SHA256-$default_expected_cargo_lock_sha256}"
if [[ "$allow_package_identity_override" != "0" &&
    "$allow_package_identity_override" != "1" ]]; then
    printf 'error: ALLOW_PACKAGE_IDENTITY_OVERRIDE must be 0 or 1, got %s\n' \
        "$allow_package_identity_override" >&2
    exit 1
fi
if [[ -z "$expected_git_head" ]]; then
    printf 'error: EXPECTED_PACKAGE_GIT_HEAD must not be empty\n' >&2
    exit 1
fi
if [[ ! "$expected_git_head" =~ ^[0-9a-f]{40}$ ]]; then
    printf 'error: EXPECTED_PACKAGE_GIT_HEAD must be a 40-character Git SHA, got %s\n' \
        "$expected_git_head" >&2
    exit 1
fi
if [[ -z "$expected_cargo_lock_sha256" ]]; then
    printf 'error: EXPECTED_CARGO_LOCK_SHA256 must not be empty\n' >&2
    exit 1
fi
if [[ ! "$expected_cargo_lock_sha256" =~ ^[0-9a-f]{64}$ ]]; then
    printf 'error: EXPECTED_CARGO_LOCK_SHA256 must be a SHA-256 hex value, got %s\n' \
        "$expected_cargo_lock_sha256" >&2
    exit 1
fi
if [[ "$expected_git_head" != "$default_expected_git_head" &&
    "$allow_package_identity_override" != "1" ]]; then
    printf '%s\n' \
        'error: EXPECTED_PACKAGE_GIT_HEAD override requires explicit package identity approval' >&2
    printf 'expected default: %s\n' "$default_expected_git_head" >&2
    printf 'got: %s\n' "$expected_git_head" >&2
    printf 'hint: set ALLOW_PACKAGE_IDENTITY_OVERRIDE=1 only for local rehearsals\n' >&2
    exit 1
fi
if [[ "$expected_cargo_lock_sha256" != "$default_expected_cargo_lock_sha256" &&
    "$allow_package_identity_override" != "1" ]]; then
    printf '%s\n' \
        'error: EXPECTED_CARGO_LOCK_SHA256 override requires explicit package identity approval' >&2
    printf 'expected default: %s\n' "$default_expected_cargo_lock_sha256" >&2
    printf 'got: %s\n' "$expected_cargo_lock_sha256" >&2
    printf 'hint: set ALLOW_PACKAGE_IDENTITY_OVERRIDE=1 only for local rehearsals\n' >&2
    exit 1
fi
if [[ "$expected_tracked_changes_present_explicit" == "1" ]]; then
    if [[ -z "$EXPECTED_TRACKED_CHANGES_PRESENT" ]]; then
        printf 'error: EXPECTED_TRACKED_CHANGES_PRESENT must not be empty\n' >&2
        exit 1
    fi
    if [[ "$EXPECTED_TRACKED_CHANGES_PRESENT" != "true" &&
        "$EXPECTED_TRACKED_CHANGES_PRESENT" != "false" ]]; then
        printf 'error: EXPECTED_TRACKED_CHANGES_PRESENT must be true or false, got %s\n' \
            "$EXPECTED_TRACKED_CHANGES_PRESENT" >&2
        exit 1
    fi
fi

if [[ "$expected_tracked_changes_present_explicit" == "1" ]]; then
    expected_tracked_changes_present="$EXPECTED_TRACKED_CHANGES_PRESENT"
elif git diff --quiet --ignore-submodules -- &&
    git diff --cached --quiet --ignore-submodules --; then
    expected_tracked_changes_present=false
else
    expected_tracked_changes_present=true
fi
work_dir="$(mktemp -d "${TMPDIR:-/tmp}/engram-homebrew-archive.XXXXXX")"

validate_release_archive() {
    local package_triple="$1"
    local archive_out_var="$2"
    local sha_out_var="$3"
    local archive_name="engram-${package_version}-${package_triple}"
    local tarball="$dist_dir/$archive_name.tar.gz"
    local checksum="$tarball.sha256"
    local sha256
    local checksum_line_count
    local checksum_sha256
    local checksum_name
    local checksum_extra
    local archive_listing="$work_dir/archive-${package_triple}.txt"
    local manifest_member="$archive_name/MANIFEST.json"
    local manifest="$work_dir/MANIFEST-${package_triple}.json"
    local payload_dir="$work_dir/payload-${package_triple}"
    local member
    local required_member
    local package_file
    local payload
    local actual_sha256
    local manifest_sha256
    local manifest_package
    local manifest_version
    local manifest_host_triple
    local manifest_archive_name
    local manifest_git_head
    local manifest_tracked_changes_present
    local manifest_cargo_lock_sha256

    if [[ ! -f "$tarball" ]]; then
        printf 'error: release tarball not found at %s\n' "$tarball" >&2
        printf 'hint: run scripts/package-release.sh on %s first\n' "$package_triple" >&2
        exit 1
    fi
    if [[ ! -f "$checksum" ]]; then
        printf 'error: release checksum not found at %s\n' "$checksum" >&2
        printf 'hint: run scripts/package-release.sh on %s first\n' "$package_triple" >&2
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

    if ! tar -tzf "$tarball" >"$archive_listing"; then
        printf 'error: release archive is unreadable: %s\n' "$tarball" >&2
        exit 1
    fi
    if [[ ! -s "$archive_listing" ]]; then
        printf 'error: release archive is empty: %s\n' "$tarball" >&2
        exit 1
    fi

    while IFS= read -r member; do
        if [[ -z "$member" ]]; then
            printf 'error: release archive contains an empty member path\n' >&2
            exit 1
        fi
        if [[ "$member" = /* || "$member" == "../"* || "$member" == *"/../"* ||
            "$member" == "." || "$member" == ".." || "$member" == *"/.." ]]; then
            printf 'error: release archive contains unsafe member path: %s\n' "$member" >&2
            exit 1
        fi

        case "$member" in
            "$archive_name" | "$archive_name/" | "$archive_name/"*) ;;
            *)
                printf 'error: release archive member is outside expected root %s: %s\n' \
                    "$archive_name" "$member" >&2
                exit 1
                ;;
        esac
    done <"$archive_listing"

    for required_member in \
        "$archive_name/engram" \
        "$archive_name/README.md" \
        "$archive_name/LICENSE" \
        "$archive_name/CHANGELOG.md" \
        "$archive_name/RELEASE_NOTES.md" \
        "$archive_name/MANIFEST.json"
    do
        if ! grep -Fxq "$required_member" "$archive_listing"; then
            printf 'error: release archive is missing required member: %s\n' \
                "$required_member" >&2
            exit 1
        fi
    done

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
    if [[ "$manifest_host_triple" != "$package_triple" ]]; then
        printf 'error: manifest host triple mismatch: expected %s, got %s\n' \
            "$package_triple" "$manifest_host_triple" >&2
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

    mkdir -p "$payload_dir"
    for package_file in engram README.md LICENSE CHANGELOG.md RELEASE_NOTES.md; do
        payload="$payload_dir/$package_file"
        if ! tar -xzOf "$tarball" "$archive_name/$package_file" >"$payload"; then
            printf 'error: release archive is missing required member: %s\n' \
                "$archive_name/$package_file" >&2
            exit 1
        fi
        if [[ ! -s "$payload" ]]; then
            printf 'error: expected packaged file is missing or empty: %s\n' \
                "$archive_name/$package_file" >&2
            exit 1
        fi

        actual_sha256="$(shasum -a 256 "$payload" | awk '{ print $1 }')"
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

    printf -v "$archive_out_var" '%s' "$archive_name"
    printf -v "$sha_out_var" '%s' "$sha256"
}

mac_arm_archive_name=""
mac_arm_sha256=""
mac_intel_archive_name=""
mac_intel_sha256=""
linux_intel_archive_name=""
linux_intel_sha256=""
validated_archive_names=()
validated_sha256s=()

for package_triple in "${homebrew_package_triples[@]}"; do
    archive_name_var=""
    sha256_var=""
    case "$package_triple" in
        aarch64-apple-darwin)
            archive_name_var="mac_arm_archive_name"
            sha256_var="mac_arm_sha256"
            ;;
        x86_64-apple-darwin)
            archive_name_var="mac_intel_archive_name"
            sha256_var="mac_intel_sha256"
            ;;
        x86_64-unknown-linux-gnu)
            archive_name_var="linux_intel_archive_name"
            sha256_var="linux_intel_sha256"
            ;;
        *)
            printf 'error: unsupported Homebrew package triple reached renderer: %s\n' \
                "$package_triple" >&2
            exit 1
            ;;
    esac
    validate_release_archive "$package_triple" "$archive_name_var" "$sha256_var"
    validated_archive_names+=("${!archive_name_var}")
    validated_sha256s+=("${!sha256_var}")
done

output_dir="$(dirname "$output")"
mkdir -p "$output_dir"
tmp_formula="$(mktemp "$output_dir/.engram.rb.XXXXXX")"

cat >"$tmp_formula" <<EOF
class Engram < Formula
  desc "Personal Knowledge Augmentation System for AI coding agents"
  homepage "https://github.com/ymeiri/engram"
  license "Apache-2.0"
EOF

if [[ -n "$mac_arm_archive_name" || -n "$mac_intel_archive_name" ]]; then
    cat >>"$tmp_formula" <<EOF

  on_macos do
EOF
    if [[ -n "$mac_arm_archive_name" ]]; then
        cat >>"$tmp_formula" <<EOF
    on_arm do
      url "${release_base_url}/${mac_arm_archive_name}.tar.gz"
      sha256 "${mac_arm_sha256}"
    end
EOF
    fi
    if [[ -n "$mac_intel_archive_name" ]]; then
        cat >>"$tmp_formula" <<EOF
    on_intel do
      url "${release_base_url}/${mac_intel_archive_name}.tar.gz"
      sha256 "${mac_intel_sha256}"
    end
EOF
    fi
    cat >>"$tmp_formula" <<EOF
  end
EOF
fi

if [[ -n "$linux_intel_archive_name" ]]; then
    cat >>"$tmp_formula" <<EOF

  on_linux do
    on_intel do
      url "${release_base_url}/${linux_intel_archive_name}.tar.gz"
      sha256 "${linux_intel_sha256}"
    end
  end
EOF
fi

cat >>"$tmp_formula" <<'EOF'

  def install
    if OS.mac?
      unless Hardware::CPU.arm? || Hardware::CPU.intel?
        odie "engram #{version} Homebrew package supports Apple Silicon and Intel macOS only"
      end
    elsif OS.linux?
      odie "engram #{version} Homebrew package supports Linux x86_64 only" unless Hardware::CPU.intel?
    else
      odie "engram #{version} Homebrew package supports macOS and Linux only"
    end

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

ruby -c "$tmp_formula" >/dev/null

if [[ -e "$output" || -L "$output" ]]; then
    if [[ -d "$output" && ! -L "$output" ]]; then
        printf 'error: FORMULA_OUTPUT must be a file path, got directory %s\n' "$output" >&2
        exit 1
    fi
    if [[ "$allow_formula_overwrite" != "1" ]]; then
        printf 'error: Homebrew formula output appeared during rendering; refusing to overwrite\n' >&2
        printf 'existing output: %s\n' "$output" >&2
        printf 'hint: remove stale generated formula evidence after approval, or set ' >&2
        printf 'ALLOW_HOMEBREW_FORMULA_OVERWRITE=1 only for local rehearsals\n' >&2
        exit 1
    fi
fi

mv "$tmp_formula" "$output"
tmp_formula=""

printf 'Homebrew formula rendered:\n'
printf '  %s\n' "$output"
printf 'Tarball SHA-256 values:\n'
for index in "${!validated_archive_names[@]}"; do
    printf '  %s.tar.gz  %s\n' \
        "${validated_archive_names[$index]}" "${validated_sha256s[$index]}"
done
