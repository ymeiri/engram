#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repo_root"

default_expected_branch="main"
expected_branch="${EXPECTED_BRANCH-$default_expected_branch}"
allow_expected_branch_override="${ALLOW_NATIVE_CLAUDE_BRANCH_OVERRIDE:-0}"
default_claude_bin="/Users/yuval.meiri/.local/bin/claude"
expected_claude_bin="${CLAUDE_BIN-$default_claude_bin}"
allow_claude_bin_override="${ALLOW_NATIVE_CLAUDE_BIN_OVERRIDE:-0}"
default_claude_target="/Users/yuval.meiri/.local/share/claude/versions/2.1.174"
default_claude_version="2.1.174 (Claude Code)"
default_claude_sha256="20c5380b4423be9963c510f5464cc1f443235a9b4423179f9c01f28021b81bad"
expected_claude_target="${EXPECTED_CLAUDE_TARGET-$default_claude_target}"
expected_claude_version="${EXPECTED_CLAUDE_VERSION-$default_claude_version}"
expected_claude_sha256="${EXPECTED_CLAUDE_SHA256-$default_claude_sha256}"
allow_claude_identity_override="${ALLOW_NATIVE_CLAUDE_IDENTITY_OVERRIDE:-0}"
default_engram_bin="/Users/yuval.meiri/.local/bin/engram"
engram_bin="${ENGRAM_BIN-$default_engram_bin}"
allow_engram_bin_override="${ALLOW_NATIVE_CLAUDE_ENGRAM_BIN_OVERRIDE:-0}"
default_vault_path="/Users/yuval.meiri/.engram/vault"
vault_path="${ENGRAM_VAULT_PATH-$default_vault_path}"
allow_vault_path_override="${ALLOW_NATIVE_CLAUDE_VAULT_PATH_OVERRIDE:-0}"
require_ready=0
allow_worktree_changes=0
json_output=0

for arg in "$@"; do
    if [[ "$arg" == "--json" ]]; then
        json_output=1
        break
    fi
done

usage() {
    cat <<'USAGE'
Usage: scripts/native-claude-gate-preflight.sh [options]

Collect read-only evidence for the native Claude prompt-bearing, /hooks, and
live host-label production gates.

Options:
  --expected-branch <branch>   Expected current branch (default: EXPECTED_BRANCH or main)
  --require-ready             Exit non-zero unless the gate is ready to execute
  --allow-worktree-changes    Allow tracked or extra untracked source changes
  --json                      Emit machine-readable JSON instead of text
  -h, --help                  Show this help

Environment overrides:
  EXPECTED_BRANCH, CLAUDE_BIN, EXPECTED_CLAUDE_TARGET,
  EXPECTED_CLAUDE_VERSION, EXPECTED_CLAUDE_SHA256, ENGRAM_BIN,
  ENGRAM_VAULT_PATH, ALLOW_NATIVE_CLAUDE_BRANCH_OVERRIDE,
  ALLOW_NATIVE_CLAUDE_BIN_OVERRIDE, ALLOW_NATIVE_CLAUDE_IDENTITY_OVERRIDE,
  ALLOW_NATIVE_CLAUDE_ENGRAM_BIN_OVERRIDE,
  ALLOW_NATIVE_CLAUDE_VAULT_PATH_OVERRIDE.

This script is evidence only. It never launches Claude, sends /hooks or prompts,
signals processes, mutates settings/adapters, accepts release fallback, marks a
PR ready, merges, tags, or publishes.
USAGE
}

fail() {
    local message="$*"
    printf 'error: %s\n' "$message" >&2
    exit_config_failure "$message"
}

print_config_failure_json() {
    local message="$1"
    jq -n \
        --arg message "$message" \
        '{
            gate_state: "configuration_preflight_failed",
            failure: {
                kind: "configuration_preflight",
                message: $message
            },
            actions_performed: {
                native_claude_launch: false,
                hooks_command: false,
                process_signals: false,
                release_actions: false
            },
            release_actions_performed: false
        }'
}

exit_config_failure() {
    local message="$1"
    if [[ "${json_output:-0}" == "1" ]] && command -v jq >/dev/null 2>&1; then
        print_config_failure_json "$message"
    fi
    exit 1
}

require_tool() {
    local tool="$1"
    command -v "$tool" >/dev/null 2>&1 || fail "required tool is missing: $tool"
}

sha256_file() {
    shasum -a 256 "$1" | awk '{ print $1 }'
}

add_blocker() {
    printf '%s\n' "$1" >>"$blockers_file"
}

print_json_summary() {
    local status_json="$1"
    jq -c "$2" "$status_json"
}

while [[ $# -gt 0 ]]; do
    case "$1" in
        --expected-branch)
            [[ $# -ge 2 ]] || fail "--expected-branch requires a branch name"
            expected_branch="$2"
            shift 2
            ;;
        --require-ready)
            require_ready=1
            shift
            ;;
        --allow-worktree-changes)
            allow_worktree_changes=1
            shift
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

require_tool git
require_tool jq
require_tool ps
require_tool realpath
require_tool shasum

case "$allow_expected_branch_override" in
    0 | 1) ;;
    *)
        fail "ALLOW_NATIVE_CLAUDE_BRANCH_OVERRIDE must be 0 or 1, got" \
            "$allow_expected_branch_override"
        ;;
esac
case "$allow_claude_bin_override" in
    0 | 1) ;;
    *) fail "ALLOW_NATIVE_CLAUDE_BIN_OVERRIDE must be 0 or 1, got $allow_claude_bin_override" ;;
esac
case "$allow_claude_identity_override" in
    0 | 1) ;;
    *)
        fail "ALLOW_NATIVE_CLAUDE_IDENTITY_OVERRIDE must be 0 or 1, got" \
            "$allow_claude_identity_override"
        ;;
esac
case "$allow_engram_bin_override" in
    0 | 1) ;;
    *)
        fail "ALLOW_NATIVE_CLAUDE_ENGRAM_BIN_OVERRIDE must be 0 or 1, got" \
            "$allow_engram_bin_override"
        ;;
esac
case "$allow_vault_path_override" in
    0 | 1) ;;
    *)
        fail "ALLOW_NATIVE_CLAUDE_VAULT_PATH_OVERRIDE must be 0 or 1, got" \
            "$allow_vault_path_override"
        ;;
esac

[[ -n "$expected_branch" ]] || fail "EXPECTED_BRANCH/--expected-branch must not be empty"
git check-ref-format --branch "$expected_branch" >/dev/null 2>&1 ||
    fail "EXPECTED_BRANCH/--expected-branch must be a valid Git branch name, got $expected_branch"
if [[ "$expected_branch" != "$default_expected_branch" &&
    "$allow_expected_branch_override" != "1" ]]; then
    message="EXPECTED_BRANCH override requires explicit native Claude approval"
    printf 'error: %s\n' "$message" >&2
    printf 'expected default branch: %s\n' "$default_expected_branch" >&2
    printf 'got: %s\n' "$expected_branch" >&2
    printf 'hint: set ALLOW_NATIVE_CLAUDE_BRANCH_OVERRIDE=1 only for local rehearsals\n' >&2
    exit_config_failure "$message"
fi

[[ -n "$expected_claude_bin" ]] || fail "CLAUDE_BIN must not be empty"
[[ "$expected_claude_bin" == /* ]] ||
    fail "CLAUDE_BIN must be an absolute path, got $expected_claude_bin"
if [[ "$expected_claude_bin" != "$default_claude_bin" &&
    "$allow_claude_bin_override" != "1" ]]; then
    message="CLAUDE_BIN override requires explicit native Claude approval"
    printf 'error: %s\n' "$message" >&2
    printf 'expected default Claude binary: %s\n' "$default_claude_bin" >&2
    printf 'got: %s\n' "$expected_claude_bin" >&2
    printf 'hint: set ALLOW_NATIVE_CLAUDE_BIN_OVERRIDE=1 only for local rehearsals\n' >&2
    exit_config_failure "$message"
fi

[[ -n "$expected_claude_target" ]] || fail "EXPECTED_CLAUDE_TARGET must not be empty"
[[ "$expected_claude_target" == /* ]] ||
    fail "EXPECTED_CLAUDE_TARGET must be an absolute path, got $expected_claude_target"
[[ -n "$expected_claude_version" ]] || fail "EXPECTED_CLAUDE_VERSION must not be empty"
if [[ ! "$expected_claude_sha256" =~ ^[0-9a-f]{64}$ ]]; then
    fail "EXPECTED_CLAUDE_SHA256 must be a SHA-256 hex value, got $expected_claude_sha256"
fi
if { [[ "$expected_claude_target" != "$default_claude_target" ]] ||
    [[ "$expected_claude_version" != "$default_claude_version" ]] ||
    [[ "$expected_claude_sha256" != "$default_claude_sha256" ]]; } &&
    [[ "$allow_claude_identity_override" != "1" ]]; then
    message="Claude identity override requires explicit native Claude approval"
    printf 'error: %s\n' "$message" >&2
    printf 'expected target: %s\n' "$default_claude_target" >&2
    printf 'expected version: %s\n' "$default_claude_version" >&2
    printf 'expected sha256: %s\n' "$default_claude_sha256" >&2
    printf 'got target: %s\n' "$expected_claude_target" >&2
    printf 'got version: %s\n' "$expected_claude_version" >&2
    printf 'got sha256: %s\n' "$expected_claude_sha256" >&2
    printf 'hint: set ALLOW_NATIVE_CLAUDE_IDENTITY_OVERRIDE=1 only for local rehearsals\n' >&2
    exit_config_failure "$message"
fi

[[ -n "$engram_bin" ]] || fail "ENGRAM_BIN must not be empty"
[[ "$engram_bin" == /* ]] || fail "ENGRAM_BIN must be an absolute path, got $engram_bin"
if [[ "$engram_bin" != "$default_engram_bin" &&
    "$allow_engram_bin_override" != "1" ]]; then
    message="ENGRAM_BIN override requires explicit native Claude approval"
    printf 'error: %s\n' "$message" >&2
    printf 'expected default Engram binary: %s\n' "$default_engram_bin" >&2
    printf 'got: %s\n' "$engram_bin" >&2
    printf 'hint: set ALLOW_NATIVE_CLAUDE_ENGRAM_BIN_OVERRIDE=1 only for local rehearsals\n' >&2
    exit_config_failure "$message"
fi

[[ -n "$vault_path" ]] || fail "ENGRAM_VAULT_PATH must not be empty"
[[ "$vault_path" == /* ]] || fail "ENGRAM_VAULT_PATH must be an absolute path, got $vault_path"
if [[ "$vault_path" != "$default_vault_path" &&
    "$allow_vault_path_override" != "1" ]]; then
    message="ENGRAM_VAULT_PATH override requires explicit native Claude approval"
    printf 'error: %s\n' "$message" >&2
    printf 'expected default vault path: %s\n' "$default_vault_path" >&2
    printf 'got: %s\n' "$vault_path" >&2
    printf 'hint: set ALLOW_NATIVE_CLAUDE_VAULT_PATH_OVERRIDE=1 only for local rehearsals\n' >&2
    exit_config_failure "$message"
fi

[[ -x "$expected_claude_bin" ]] || fail "Claude binary is not executable: $expected_claude_bin"
[[ -x "$engram_bin" ]] || fail "Engram binary is not executable: $engram_bin"

blockers_file="$(mktemp "${TMPDIR:-/tmp}/engram-native-claude-blockers.XXXXXX")"
process_file="$(mktemp "${TMPDIR:-/tmp}/engram-native-claude-processes.XXXXXX")"
harness_status_json="$(mktemp "${TMPDIR:-/tmp}/engram-native-claude-status.XXXXXX")"
harness_doctor_json="$(mktemp "${TMPDIR:-/tmp}/engram-native-claude-doctor.XXXXXX")"
harness_install_json="$(mktemp "${TMPDIR:-/tmp}/engram-native-claude-install.XXXXXX")"
obligations_json="$(mktemp "${TMPDIR:-/tmp}/engram-native-claude-obligations.XXXXXX")"
vault_json="$(mktemp "${TMPDIR:-/tmp}/engram-native-claude-vault.XXXXXX")"
daemon_status="$(mktemp "${TMPDIR:-/tmp}/engram-native-claude-daemon.XXXXXX")"

cleanup() {
    rm -f "$blockers_file" "$process_file" "$harness_status_json" "$harness_doctor_json" \
        "$harness_install_json" "$obligations_json" "$vault_json" "$daemon_status"
}
trap cleanup EXIT

branch="$(git branch --show-current)"
head_sha="$(git rev-parse HEAD)"
[[ "$branch" == "$expected_branch" ]] ||
    add_blocker "branch mismatch: expected $expected_branch, got ${branch:-<none>}"

upstream="$(git rev-parse --abbrev-ref --symbolic-full-name '@{upstream}' 2>/dev/null || true)"
if [[ -z "$upstream" ]]; then
    add_blocker "current branch has no upstream"
    ahead_count="unknown"
    behind_count="unknown"
else
    read -r ahead_count behind_count < <(git rev-list --left-right --count HEAD..."$upstream")
    if [[ "$ahead_count" != "0" || "$behind_count" != "0" ]]; then
        add_blocker "branch is not synced with $upstream: ahead=$ahead_count behind=$behind_count"
    fi
fi

if git diff --quiet --ignore-submodules -- &&
    git diff --cached --quiet --ignore-submodules --; then
    tracked_changes_present=false
else
    tracked_changes_present=true
fi

untracked_files="$(git ls-files --others --exclude-standard)"
extra_untracked_files="$(
    printf '%s\n' "$untracked_files" |
        awk 'NF && $0 != "AGENTS.md" { print }'
)"

if [[ "$allow_worktree_changes" != "1" ]]; then
    [[ "$tracked_changes_present" == "false" ]] ||
        add_blocker "tracked working-tree or index changes are present"
    [[ -z "$extra_untracked_files" ]] ||
        add_blocker "unexpected untracked files are present"
fi

claude_target="$(realpath "$expected_claude_bin")"
claude_version="$("$expected_claude_bin" --version)"
claude_sha256="$(sha256_file "$claude_target")"

[[ "$claude_target" == "$expected_claude_target" ]] ||
    add_blocker "Claude target mismatch: expected $expected_claude_target, got $claude_target"
[[ "$claude_version" == "$expected_claude_version" ]] ||
    add_blocker "Claude version mismatch: expected $expected_claude_version, got $claude_version"
[[ "$claude_sha256" == "$expected_claude_sha256" ]] ||
    add_blocker "Claude target SHA-256 mismatch: expected $expected_claude_sha256, got $claude_sha256"

"$engram_bin" harness status --harness claude-code --json >"$harness_status_json"
"$engram_bin" harness doctor --harness claude-code --json >"$harness_doctor_json"
"$engram_bin" harness install --harness claude-code --settings-target snippet-only --json \
    >"$harness_install_json"
"$engram_bin" obligations doctor --scope-project engram --cwd "$repo_root" --limit 20 --json \
    >"$obligations_json"
"$engram_bin" vault status "$vault_path" --json >"$vault_json"
"$engram_bin" daemon status >"$daemon_status"

jq -e '.ready == true' "$harness_status_json" >/dev/null ||
    add_blocker "Claude Code harness status is not ready"
jq -e '.ready == true' "$harness_doctor_json" >/dev/null ||
    add_blocker "Claude Code harness doctor is not ready"
jq -e '(.planned // []) | length == 0' "$harness_install_json" >/dev/null ||
    add_blocker "snippet-only harness install dry-run has planned changes"
jq -e '((.open // []) | length == 0) and ((.warnings // []) | length == 0)' \
    "$obligations_json" >/dev/null || add_blocker "obligations doctor has open items or warnings"
jq -e '
    .initialized == true
    and .generated_file_count == .expected_generated_file_count
    and .user_file_count == 0
' "$vault_json" >/dev/null || add_blocker "canonical vault is not generated-count aligned"
grep -q 'Daemon status: .*running' "$daemon_status" ||
    add_blocker "Engram daemon is not running"

ps -axo pid,ppid,tty,stat,etime,command >"$process_file"
native_processes="$(
    awk '
        NR == 1 { next }
        $6 == "claude" || $6 ~ /\/claude$/ { print }
    ' "$process_file"
)"
claude_family_process_count="$(
    awk '
        BEGIN { count = 0 }
        NR > 1 && tolower($0) ~ /claude|anthropic/ { count++ }
        END { print count }
    ' "$process_file"
)"

if [[ -n "$native_processes" ]]; then
    add_blocker "native Claude CLI processes are already running"
fi

if [[ -s "$blockers_file" ]]; then
    gate_state="blocked"
else
    gate_state="ready"
fi

if [[ "$json_output" == "1" ]]; then
    blockers_json="$(jq -R -s 'split("\n") | map(select(length > 0))' "$blockers_file")"
    extra_untracked_json="$(
        printf '%s\n' "$extra_untracked_files" |
            jq -R -s 'split("\n") | map(select(length > 0))'
    )"
    native_processes_json="$(
        printf '%s\n' "$native_processes" |
            jq -R -s 'split("\n") | map(select(length > 0))'
    )"
    daemon_text="$(cat "$daemon_status")"
    harness_status_summary="$(print_json_summary "$harness_status_json" '{ready,warnings}')"
    harness_doctor_summary="$(print_json_summary "$harness_doctor_json" '{ready,warnings}')"
    snippet_only_summary="$(print_json_summary "$harness_install_json" '{planned,warnings}')"
    obligations_summary="$(print_json_summary "$obligations_json" '{open,warnings}')"
    vault_summary="$(
        print_json_summary "$vault_json" \
            '{initialized,total_file_count,generated_file_count,user_file_count,expected_generated_file_count}'
    )"

    jq -n \
        --arg gate_state "$gate_state" \
        --arg branch "$branch" \
        --arg upstream "${upstream:-}" \
        --arg ahead "$ahead_count" \
        --arg behind "$behind_count" \
        --arg head "$head_sha" \
        --arg tracked "$tracked_changes_present" \
        --arg claude_bin "$expected_claude_bin" \
        --arg claude_target "$claude_target" \
        --arg claude_version "$claude_version" \
        --arg claude_sha256 "$claude_sha256" \
        --arg engram_bin "$engram_bin" \
        --arg daemon "$daemon_text" \
        --arg family_count "$claude_family_process_count" \
        --argjson extra_untracked "$extra_untracked_json" \
        --argjson harness_status "$harness_status_summary" \
        --argjson harness_doctor "$harness_doctor_summary" \
        --argjson snippet_only "$snippet_only_summary" \
        --argjson obligations "$obligations_summary" \
        --argjson vault "$vault_summary" \
        --argjson native_processes "$native_processes_json" \
        --argjson blockers "$blockers_json" \
        '{
            gate_state: $gate_state,
            branch: $branch,
            upstream: {
                name: (if $upstream == "" then null else $upstream end),
                ahead: $ahead,
                behind: $behind
            },
            head: $head,
            tracked_changes_present: ($tracked == "true"),
            extra_untracked_files: $extra_untracked,
            claude: {
                bin: $claude_bin,
                target: $claude_target,
                version: $claude_version,
                sha256: $claude_sha256
            },
            engram: {
                bin: $engram_bin,
                daemon_status: $daemon
            },
            harness_status: $harness_status,
            harness_doctor: $harness_doctor,
            snippet_only_dry_run: $snippet_only,
            obligations: $obligations,
            vault: $vault,
            native_claude_processes_present: ($native_processes | length > 0),
            native_claude_processes: $native_processes,
            claude_family_process_count: ($family_count | tonumber),
            blockers: $blockers,
            actions_performed: {
                native_claude_launch: false,
                hooks_command: false,
                process_signals: false,
                release_actions: false
            },
            release_actions_performed: false
        }'
else
    printf 'Native Claude production gate preflight:\n'
    printf '  gate_state: %s\n' "$gate_state"
    printf '  branch: %s\n' "$branch"
    printf '  upstream: %s (ahead=%s behind=%s)\n' "${upstream:-<none>}" "$ahead_count" "$behind_count"
    printf '  head: %s\n' "$head_sha"
    printf '  tracked_changes_present: %s\n' "$tracked_changes_present"
    printf '  extra_untracked_files_present: %s\n' "$([[ -n "$extra_untracked_files" ]] && printf true || printf false)"
    printf '  claude_bin: %s\n' "$expected_claude_bin"
    printf '  claude_target: %s\n' "$claude_target"
    printf '  claude_version: %s\n' "$claude_version"
    printf '  claude_sha256: %s\n' "$claude_sha256"
    printf '  engram_bin: %s\n' "$engram_bin"
    printf '  daemon: %s\n' "$(tr '\n' ';' <"$daemon_status" | sed 's/; */; /g')"
    printf '  harness_status: %s\n' "$(print_json_summary "$harness_status_json" '{ready,warnings}')"
    printf '  harness_doctor: %s\n' "$(print_json_summary "$harness_doctor_json" '{ready,warnings}')"
    printf '  snippet_only_dry_run: %s\n' "$(print_json_summary "$harness_install_json" '{planned,warnings}')"
    printf '  obligations: %s\n' "$(print_json_summary "$obligations_json" '{open,warnings}')"
    printf '  vault: %s\n' \
        "$(print_json_summary "$vault_json" '{initialized,total_file_count,generated_file_count,user_file_count,expected_generated_file_count}')"
    printf '  native_claude_processes_present: %s\n' "$([[ -n "$native_processes" ]] && printf true || printf false)"
    printf '  claude_family_process_count: %s\n' "$claude_family_process_count"

    if [[ -n "$native_processes" ]]; then
        printf '  native_claude_processes:\n'
        printf '%s\n' "$native_processes" | sed 's/^/    /'
    fi

    if [[ -s "$blockers_file" ]]; then
        printf '  blockers:\n'
        sed 's/^/    - /' "$blockers_file"
    fi

    printf '  native_claude_launch_performed: false\n'
    printf '  hooks_command_performed: false\n'
    printf '  process_signals_performed: false\n'
    printf '  release_actions_performed: false\n'
fi

if [[ "$gate_state" != "ready" && "$require_ready" == "1" ]]; then
    exit 2
fi
