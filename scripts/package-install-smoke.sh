#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repo_root"

dist_dir="${DIST_DIR:-$repo_root/dist}"
package_version="$(cargo pkgid -p engram-cli | sed 's/.*#//')"
host_triple="$(rustc -vV | awk '/^host:/ { print $2 }')"
archive_name="engram-${package_version}-${host_triple}"
tarball="$dist_dir/$archive_name.tar.gz"
checksum="$tarball.sha256"
embed_cache_dir="${ENGRAM_EMBED_CACHE_DIR:-$repo_root/.fastembed_cache}"
work_dir="$(mktemp -d "${TMPDIR:-/tmp}/engram-install-smoke.XXXXXX")"
server_pid=""

cleanup() {
    if [[ -n "$server_pid" ]] && kill -0 "$server_pid" 2>/dev/null; then
        kill "$server_pid" 2>/dev/null || true
        wait "$server_pid" 2>/dev/null || true
    fi
    rm -rf "$work_dir"
}
trap cleanup EXIT

run_step() {
    local name="$1"
    shift
    printf '\n==> %s\n' "$name"
    "$@"
}

choose_port() {
    if [[ -n "${SMOKE_PORT:-}" ]]; then
        printf '%s\n' "$SMOKE_PORT"
        return 0
    fi

    local port
    for port in 8765 8766 8767 8768 8769 8770 8771 8772 8773 8774; do
        if ! nc -z 127.0.0.1 "$port" >/dev/null 2>&1; then
            printf '%s\n' "$port"
            return 0
        fi
    done

    printf 'error: no free local smoke-test port found in 8765-8774\n' >&2
    return 1
}

validate_archive_paths() {
    local archive="$1"
    local listing="$2"
    local member
    local required_member

    tar -tzf "$archive" >"$listing"
    if [[ ! -s "$listing" ]]; then
        printf 'error: release archive is empty: %s\n' "$archive" >&2
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
    done <"$listing"

    for required_member in \
        "$archive_name/engram" \
        "$archive_name/README.md" \
        "$archive_name/LICENSE" \
        "$archive_name/CHANGELOG.md" \
        "$archive_name/RELEASE_NOTES.md"
    do
        if ! grep -Fxq "$required_member" "$listing"; then
            printf 'error: release archive is missing required member: %s\n' \
                "$required_member" >&2
            exit 1
        fi
    done
}

if [[ "${SKIP_PACKAGE_BUILD:-0}" != "1" ]]; then
    run_step "build release package" "$repo_root/scripts/package-release.sh"
fi

if [[ ! -f "$tarball" ]]; then
    printf 'error: release tarball not found at %s\n' "$tarball" >&2
    exit 1
fi
if [[ ! -f "$checksum" ]]; then
    printf 'error: release checksum not found at %s\n' "$checksum" >&2
    exit 1
fi

cp "$tarball" "$checksum" "$work_dir/"
cd "$work_dir"

run_step "verify checksum" shasum -a 256 -c "$(basename "$checksum")"
run_step "inspect archive paths" validate_archive_paths "$(basename "$tarball")" \
    "$work_dir/archive-contents.txt"
run_step "extract archive" tar -xzf "$(basename "$tarball")"

package_dir="$work_dir/$archive_name"
for required_path in \
    "$package_dir/engram" \
    "$package_dir/README.md" \
    "$package_dir/LICENSE" \
    "$package_dir/CHANGELOG.md" \
    "$package_dir/RELEASE_NOTES.md"
do
    if [[ ! -s "$required_path" ]]; then
        printf 'error: expected packaged file is missing or empty: %s\n' "$required_path" >&2
        exit 1
    fi
done
if [[ ! -x "$package_dir/engram" ]]; then
    printf 'error: packaged engram binary is not executable: %s\n' "$package_dir/engram" >&2
    exit 1
fi

mkdir -p "$work_dir/prefix/bin" "$work_dir/home" "$work_dir/data" "$embed_cache_dir"
run_step "install binary in temp prefix" install -m 755 "$package_dir/engram" "$work_dir/prefix/bin/engram"

export PATH="$work_dir/prefix/bin:$PATH"
resolved_engram="$(command -v engram)"
if [[ "$resolved_engram" != "$work_dir/prefix/bin/engram" ]]; then
    printf 'error: expected PATH to resolve temp engram, got %s\n' "$resolved_engram" >&2
    exit 1
fi

expected_version="engram ${package_version}"
actual_version="$(
    HOME="$work_dir/home" \
    ENGRAM_DATA_DIR="$work_dir/data" \
    engram --version
)"
if [[ "$actual_version" != "$expected_version" ]]; then
    printf 'error: installed binary version mismatch: expected "%s", got "%s"\n' \
        "$expected_version" "$actual_version" >&2
    exit 1
fi

port="$(choose_port)"
server_log="$work_dir/server.log"
health_json="$work_dir/health.json"

printf '\n==> start packaged HTTP server\n'
# Reuse the warmed fastembed cache while keeping Engram data and cwd isolated.
env \
    -u HF_HOME \
    HOME="$work_dir/home" \
    ENGRAM_DATA_DIR="$work_dir/data" \
    ENGRAM_EMBED_CACHE_DIR="$embed_cache_dir" \
    engram serve --http --memory --port "$port" >"$server_log" 2>&1 &
server_pid=$!

for _ in $(seq 1 300); do
    if ! kill -0 "$server_pid" 2>/dev/null; then
        cat "$server_log" >&2
        printf 'error: packaged HTTP server exited before health check passed\n' >&2
        exit 1
    fi

    if curl -fsS "http://127.0.0.1:${port}/health" >"$health_json" 2>/dev/null; then
        break
    fi
    sleep 0.1
done

if [[ ! -s "$health_json" ]]; then
    cat "$server_log" >&2
    printf 'error: packaged HTTP server did not pass health check on port %s\n' "$port" >&2
    exit 1
fi

grep -Fq '"status":"ok"' "$health_json"
grep -Fq '"service":"engram"' "$health_json"
grep -Fq "\"version\":\"${package_version}\"" "$health_json"

printf '\nPackage install smoke passed:\n'
printf '  %s\n' "$tarball"
printf '  %s\n' "$checksum"
printf '  %s\n' "$actual_version"
printf '  %s\n' "$(cat "$health_json")"
