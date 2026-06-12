#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repo_root"

target="${RELEASE_TARGET:-ga}"
pr_number="${PR_NUMBER:-}"
hosted_run_id="${HOSTED_RUN_ID:-}"
default_expected_workflow="CI"
expected_workflow="${EXPECTED_WORKFLOW_NAME-$default_expected_workflow}"
allow_expected_workflow_override="${ALLOW_EXPECTED_WORKFLOW_NAME_OVERRIDE:-0}"
expected_event="${EXPECTED_EVENT:-}"
expected_branch="${EXPECTED_BRANCH:-}"
package_version="$(cargo pkgid --locked -p engram-cli | sed 's/.*#//')"
release_version="${RELEASE_VERSION:-}"
default_release_notes_path="$repo_root/docs/RELEASE_NOTES_V0_2_0.md"
release_notes_path="${RELEASE_NOTES_PATH-$default_release_notes_path}"
allow_release_notes_path_override="${ALLOW_RELEASE_NOTES_PATH_OVERRIDE:-0}"
default_release_repo="ymeiri/engram"
release_repo="${RELEASE_REPOSITORY:-$default_release_repo}"
allow_release_repo_override="${ALLOW_RELEASE_REPOSITORY_OVERRIDE:-0}"
default_min_free_space_kib=10485760
min_free_space_kib="${RELEASE_GATE_MIN_FREE_KIB:-$default_min_free_space_kib}"
allow_min_free_space_override="${ALLOW_RELEASE_GATE_MIN_FREE_OVERRIDE:-0}"
run_local_ci=1
run_package_smoke=1
run_homebrew_render=1
allow_tracked_changes=0
json_output=0
expected_jobs=(Check Test Format Clippy Docs)

usage() {
    cat <<'USAGE'
Usage: scripts/release-gate-report.sh [options]

Collect release-owner evidence for GA or beta release gates.

Options:
  --target <ga|beta>            Release target type (default: RELEASE_TARGET or ga)
  --pr <number>                 GitHub PR number to inspect for beta targets
  --hosted-run <id>             Hosted CI run ID to verify for the current head
  --release-version <version>   Intended release version (default: current version for beta,
                                prerelease suffix stripped for GA)
  --expected-event <event>      Expected GitHub Actions event (default: push for GA, pull_request for beta)
  --expected-branch <branch>    Expected current branch (default: main for GA, unset for beta)
  --quick                       Skip local CI and package/install smoke
  --skip-local-ci               Skip ./scripts/local-ci.sh
  --skip-package-smoke          Skip ./scripts/package-install-smoke.sh
  --skip-homebrew-render        Skip Homebrew formula render/syntax validation
  --allow-tracked-changes       Allow tracked working-tree/index changes during development
  --json                        Emit final evidence as machine-readable JSON
  -h, --help                    Show this help

Environment overrides:
  RELEASE_TARGET, PR_NUMBER, HOSTED_RUN_ID, EXPECTED_WORKFLOW_NAME,
  ALLOW_EXPECTED_WORKFLOW_NAME_OVERRIDE, EXPECTED_EVENT, EXPECTED_BRANCH, RELEASE_VERSION,
  RELEASE_NOTES_PATH, ALLOW_RELEASE_NOTES_PATH_OVERRIDE, RELEASE_REPOSITORY,
  ALLOW_RELEASE_REPOSITORY_OVERRIDE,
  RELEASE_GATE_MIN_FREE_KIB, ALLOW_RELEASE_GATE_MIN_FREE_OVERRIDE.

This script is evidence only. It does not accept a hosted-CI fallback, mark a
PR ready, merge, tag, publish, mutate harness state, or change release scope.
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
        --target)
            [[ $# -ge 2 ]] || fail "--target requires ga or beta"
            case "$2" in
                ga | beta) target="$2" ;;
                *) fail "--target must be ga or beta" ;;
            esac
            shift 2
            ;;
        --pr)
            [[ $# -ge 2 ]] || fail "--pr requires a PR number"
            pr_number="$2"
            shift 2
            ;;
        --hosted-run)
            [[ $# -ge 2 ]] || fail "--hosted-run requires a run ID"
            hosted_run_id="$2"
            shift 2
            ;;
        --release-version)
            [[ $# -ge 2 ]] || fail "--release-version requires a version"
            release_version="$2"
            shift 2
            ;;
        --expected-event)
            [[ $# -ge 2 ]] || fail "--expected-event requires an event name"
            expected_event="$2"
            shift 2
            ;;
        --expected-branch)
            [[ $# -ge 2 ]] || fail "--expected-branch requires a branch name"
            expected_branch="$2"
            shift 2
            ;;
        --quick)
            run_local_ci=0
            run_package_smoke=0
            run_homebrew_render=0
            shift
            ;;
        --skip-local-ci)
            run_local_ci=0
            shift
            ;;
        --skip-package-smoke)
            run_package_smoke=0
            shift
            ;;
        --skip-homebrew-render)
            run_homebrew_render=0
            shift
            ;;
        --allow-tracked-changes)
            allow_tracked_changes=1
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

case "$target" in
    ga | beta) ;;
    *) fail "--target must be ga or beta" ;;
esac

case "$allow_expected_workflow_override" in
    0 | 1) ;;
    *)
        workflow_override_error="ALLOW_EXPECTED_WORKFLOW_NAME_OVERRIDE must be 0 or 1"
        workflow_override_error+=", got $allow_expected_workflow_override"
        fail "$workflow_override_error"
        ;;
esac
if [[ -z "$expected_workflow" ]]; then
    fail "EXPECTED_WORKFLOW_NAME must not be empty"
fi
workflow_name_pattern='^[A-Za-z0-9_. -]+$'
if [[ ! "$expected_workflow" =~ $workflow_name_pattern ]]; then
    workflow_name_error="EXPECTED_WORKFLOW_NAME must contain only letters, numbers, spaces,"
    workflow_name_error+=" dot, underscore, and hyphen; got $expected_workflow"
    fail "$workflow_name_error"
fi
if [[ "$expected_workflow" != "$default_expected_workflow" &&
    "$allow_expected_workflow_override" != "1" ]]; then
    printf 'error: EXPECTED_WORKFLOW_NAME override requires explicit approval\n' >&2
    printf 'expected default: %s\n' "$default_expected_workflow" >&2
    printf 'got: %s\n' "$expected_workflow" >&2
    printf 'hint: set ALLOW_EXPECTED_WORKFLOW_NAME_OVERRIDE=1 only for local rehearsals\n' >&2
    exit 1
fi

if [[ -z "$expected_event" ]]; then
    if [[ "$target" == "ga" ]]; then
        expected_event="push"
    else
        expected_event="pull_request"
    fi
fi
event_name_pattern='^[A-Za-z0-9_]+$'
if [[ ! "$expected_event" =~ $event_name_pattern ]]; then
    expected_event_error="EXPECTED_EVENT/--expected-event must be a GitHub event name token"
    expected_event_error+=", got $expected_event"
    fail "$expected_event_error"
fi
if [[ -z "$expected_branch" && "$target" == "ga" ]]; then
    expected_branch="main"
fi

if [[ "$target" == "beta" && -z "$pr_number" ]]; then
    pr_number=3
fi

if [[ -z "$release_version" ]]; then
    if [[ "$target" == "ga" ]]; then
        release_version="${package_version%%-*}"
    else
        release_version="$package_version"
    fi
fi
release_version_pattern='^[0-9]+[.][0-9]+[.][0-9]+(-[0-9A-Za-z]+([.-][0-9A-Za-z]+)*)?$'
if [[ ! "$release_version" =~ $release_version_pattern ]]; then
    release_version_error="RELEASE_VERSION/--release-version must be x.y.z"
    release_version_error+=" with an optional prerelease suffix, got $release_version"
    fail "$release_version_error"
fi
release_tag="v${release_version}"

if [[ -n "$hosted_run_id" && ! "$hosted_run_id" =~ ^[0-9]+$ ]]; then
    fail "HOSTED_RUN_ID/--hosted-run must be a numeric GitHub Actions run id, got $hosted_run_id"
fi
if [[ -n "$pr_number" && ! "$pr_number" =~ ^[0-9]+$ ]]; then
    fail "PR_NUMBER/--pr must be a numeric GitHub pull request number, got $pr_number"
fi

require_tool git
require_tool gh
require_tool jq
case "$allow_release_notes_path_override" in
    0 | 1) ;;
    *)
        release_notes_override_error="ALLOW_RELEASE_NOTES_PATH_OVERRIDE must be 0 or 1"
        release_notes_override_error+=", got $allow_release_notes_path_override"
        fail "$release_notes_override_error"
        ;;
esac
if [[ -z "$release_notes_path" ]]; then
    fail "RELEASE_NOTES_PATH must not be empty"
fi
if [[ "$release_notes_path" != "$default_release_notes_path" &&
    "$allow_release_notes_path_override" != "1" ]]; then
    printf 'error: RELEASE_NOTES_PATH override requires explicit approval\n' >&2
    printf 'expected default: %s\n' "$default_release_notes_path" >&2
    printf 'got: %s\n' "$release_notes_path" >&2
    printf 'hint: set ALLOW_RELEASE_NOTES_PATH_OVERRIDE=1 only for local rehearsals\n' >&2
    exit 1
fi
case "$allow_release_repo_override" in
    0 | 1) ;;
    *) fail "ALLOW_RELEASE_REPOSITORY_OVERRIDE must be 0 or 1, got $allow_release_repo_override" ;;
esac
if [[ "$release_repo" != "$default_release_repo" &&
    "$allow_release_repo_override" != "1" ]]; then
    printf 'error: RELEASE_REPOSITORY override requires explicit approval\n' >&2
    printf 'expected default: %s\n' "$default_release_repo" >&2
    printf 'got: %s\n' "$release_repo" >&2
    printf 'hint: set ALLOW_RELEASE_REPOSITORY_OVERRIDE=1 only for local rehearsals\n' >&2
    exit 1
fi
if [[ ! "$release_repo" =~ ^[A-Za-z0-9_.-]+/[A-Za-z0-9_.-]+$ ]]; then
    fail "release repository must be owner/name, got $release_repo"
fi
case "$allow_min_free_space_override" in
    0 | 1) ;;
    *) fail "ALLOW_RELEASE_GATE_MIN_FREE_OVERRIDE must be 0 or 1, got $allow_min_free_space_override" ;;
esac
[[ "$min_free_space_kib" =~ ^[0-9]+$ ]] ||
    fail "RELEASE_GATE_MIN_FREE_KIB must be a non-negative integer"
if [[ "$min_free_space_kib" != "$default_min_free_space_kib" &&
    "$allow_min_free_space_override" != "1" ]]; then
    printf 'error: RELEASE_GATE_MIN_FREE_KIB override requires explicit approval\n' >&2
    printf 'expected default: %s\n' "$default_min_free_space_kib" >&2
    printf 'got: %s\n' "$min_free_space_kib" >&2
    printf 'hint: set ALLOW_RELEASE_GATE_MIN_FREE_OVERRIDE=1 only for local rehearsals\n' >&2
    exit 1
fi

pr_json="$(mktemp "${TMPDIR:-/tmp}/engram-release-pr.XXXXXX")"
checks_file="$(mktemp "${TMPDIR:-/tmp}/engram-release-checks.XXXXXX")"
hosted_run_json="$(mktemp "${TMPDIR:-/tmp}/engram-release-hosted-run.XXXXXX")"
hosted_verifier_file="$(mktemp "${TMPDIR:-/tmp}/engram-release-hosted-verifier.XXXXXX")"
release_target_file="$(mktemp "${TMPDIR:-/tmp}/engram-release-target.XXXXXX")"
release_target_error_file="$(mktemp "${TMPDIR:-/tmp}/engram-release-target-error.XXXXXX")"
cleanup() {
    rm -f "$pr_json" "$checks_file" "$hosted_run_json" "$hosted_verifier_file" \
        "$release_target_file" "$release_target_error_file"
}
trap cleanup EXIT

branch="$(git branch --show-current)"
[[ -n "$branch" ]] || fail "could not determine current branch"
if [[ -n "$expected_branch" && "$branch" != "$expected_branch" ]]; then
    fail "branch mismatch: expected $expected_branch, got $branch"
fi
head_sha="$(git rev-parse HEAD)"

if git diff --quiet --ignore-submodules -- &&
    git diff --cached --quiet --ignore-submodules --; then
    tracked_changes_present=false
else
    tracked_changes_present=true
fi

if [[ "$tracked_changes_present" == "true" && "$allow_tracked_changes" != "1" ]]; then
    fail "tracked working-tree or index changes are present; commit or pass --allow-tracked-changes"
fi

upstream="$(git rev-parse --abbrev-ref --symbolic-full-name '@{upstream}' 2>/dev/null || true)"
[[ -n "$upstream" ]] || fail "current branch has no upstream"
read -r ahead_count behind_count < <(git rev-list --left-right --count HEAD..."$upstream")
if [[ "$ahead_count" != "0" || "$behind_count" != "0" ]]; then
    fail "branch is not synced with $upstream: ahead=$ahead_count behind=$behind_count"
fi

hosted_ci_state="unknown"
hosted_ci_verifier_json="null"
hosted_run_report_json="null"
pr_report_json="null"
release_scope_state="not_applicable"
release_scope_native_claude_ack=false
release_scope_lifecycle_m6_ack=false
homebrew_formula_state="not_applicable"
homebrew_formula_output="$repo_root/dist/homebrew/Formula/engram.rb"
disk_space_state="not_checked"
disk_space_error=""
disk_space_shortfall_kib=""
disk_cleanup_candidates_json="[]"
free_space_kib=""
release_target_state="not_applicable"
release_target_local_tag_exists=false
release_target_remote_tag_exists=false
release_target_github_release_exists=false
release_target_error=""

preflight_log() {
    if [[ "$json_output" == "1" ]]; then
        printf '%s\n' "$*" >&2
    else
        printf '%s\n' "$*"
    fi
}

check_release_target_available() {
    [[ "$target" == "ga" ]] || return 0

    release_target_state="available"
    preflight_log "tag=$release_tag"
    preflight_log "repository=$release_repo"

    if git rev-parse -q --verify "refs/tags/${release_tag}" >/dev/null 2>&1; then
        release_target_local_tag_exists=true
        release_target_state="unavailable"
    fi

    if gh release view "$release_tag" --repo "$release_repo" \
        --json tagName >"$release_target_file" 2>"$release_target_error_file"; then
        release_target_github_release_exists=true
        release_target_state="unavailable"
    elif ! grep -Fq "release not found" "$release_target_error_file"; then
        release_target_state="unknown"
        release_target_error="could not check GitHub release target $release_tag in $release_repo"
        return 1
    fi

    if git ls-remote --tags "https://github.com/${release_repo}.git" \
        "$release_tag" "${release_tag}^{}" >"$release_target_file" 2>"$release_target_error_file"; then
        if [[ -s "$release_target_file" ]]; then
            release_target_remote_tag_exists=true
            release_target_state="unavailable"
        fi
    else
        release_target_state="unknown"
        release_target_error="could not check remote Git tag $release_tag in $release_repo"
        return 1
    fi

    preflight_log "local_tag_exists=$release_target_local_tag_exists"
    preflight_log "remote_git_tag_exists=$release_target_remote_tag_exists"
    preflight_log "github_release_exists=$release_target_github_release_exists"

    if [[ "$release_target_state" == "unavailable" ]]; then
        release_target_error="release target $release_tag is unavailable:"
        if [[ "$release_target_local_tag_exists" == "true" ]]; then
            release_target_error+=" local tag exists;"
        fi
        if [[ "$release_target_remote_tag_exists" == "true" ]]; then
            release_target_error+=" remote Git tag exists in $release_repo;"
        fi
        if [[ "$release_target_github_release_exists" == "true" ]]; then
            release_target_error+=" GitHub release exists in $release_repo;"
        fi
        release_target_error+=" resolve the release-target conflict before owner review."
        return 1
    fi

    return 0
}

collect_disk_cleanup_candidates() {
    local candidates_json="[]"
    local rel_path abs_path size_kib

    for rel_path in target dist; do
        abs_path="$repo_root/$rel_path"
        if [[ -e "$abs_path" ]]; then
            size_kib="$(du -sk "$abs_path" | awk '{ print $1 }')"
            if [[ "$size_kib" =~ ^[0-9]+$ ]]; then
                candidates_json="$(
                    jq -c \
                        --arg path "$rel_path" \
                        --arg absolute_path "$abs_path" \
                        --arg size_kib "$size_kib" \
                        '. + [{
                            path: $path,
                            absolute_path: $absolute_path,
                            size_kib: ($size_kib | tonumber)
                        }]' <<<"$candidates_json"
                )"
            fi
        fi
    done

    disk_cleanup_candidates_json="$candidates_json"
}

check_free_space_for_local_steps() {
    if [[ "$run_local_ci" != "1" && "$run_package_smoke" != "1" ]]; then
        disk_space_state="skipped"
        preflight_log "skipped: local CI and package/install smoke are disabled"
        return 0
    fi

    free_space_kib="$(df -Pk "$repo_root" | awk 'NR == 2 { print $4 }')"
    [[ "$free_space_kib" =~ ^[0-9]+$ ]] ||
        fail "could not determine free disk space for $repo_root"

    preflight_log "free_space_kib=$free_space_kib"
    preflight_log "min_required_kib=$min_free_space_kib"

    if (( free_space_kib < min_free_space_kib )); then
        disk_space_state="insufficient"
        disk_space_shortfall_kib=$((min_free_space_kib - free_space_kib))
        collect_disk_cleanup_candidates
        preflight_log "shortfall_kib=$disk_space_shortfall_kib"
        jq -r '
            .[]
            | "cleanup_candidate: path=\(.path) size_kib=\(.size_kib)"
        ' <<<"$disk_cleanup_candidates_json" | while IFS= read -r candidate; do
            preflight_log "$candidate"
        done
        disk_space_error="insufficient free disk space for local release validation: "
        disk_space_error+="have ${free_space_kib} KiB, require ${min_free_space_kib} KiB "
        disk_space_error+="under $repo_root. Free generated build/cache space, then rerun; "
        disk_space_error+="use --skip-local-ci/--skip-package-smoke only for partial evidence."
        return 1
    fi

    disk_space_state="passed"
    disk_space_shortfall_kib=0
    return 0
}

emit_release_target_failure_json() {
    local failure_gate_state="release_target_unavailable"
    if [[ "$release_target_state" == "unknown" ]]; then
        failure_gate_state="release_target_check_failed"
    fi

    jq -n \
        --arg target "$target" \
        --arg package_version "$package_version" \
        --arg release_version "$release_version" \
        --arg release_tag "$release_tag" \
        --arg release_repo "$release_repo" \
        --arg release_target_state "$release_target_state" \
        --arg release_target_local_tag "$release_target_local_tag_exists" \
        --arg release_target_remote_tag "$release_target_remote_tag_exists" \
        --arg release_target_github_release "$release_target_github_release_exists" \
        --arg branch "$branch" \
        --arg expected_branch "$expected_branch" \
        --arg upstream "$upstream" \
        --arg ahead "$ahead_count" \
        --arg behind "$behind_count" \
        --arg head "$head_sha" \
        --arg tracked "$tracked_changes_present" \
        --arg expected_event "$expected_event" \
        --arg hosted_ci_state "$hosted_ci_state" \
        --arg hosted_run_id "$hosted_run_id" \
        --arg min_free_space_kib "$min_free_space_kib" \
        --arg release_notes_path "$release_notes_path" \
        --arg release_scope_state "$release_scope_state" \
        --arg release_scope_native_claude "$release_scope_native_claude_ack" \
        --arg release_scope_lifecycle_m6 "$release_scope_lifecycle_m6_ack" \
        --arg release_gate_state "$failure_gate_state" \
        --arg release_target_error "$release_target_error" \
        --argjson pr "$pr_report_json" \
        --argjson hosted_run "$hosted_run_report_json" \
        --argjson hosted_ci_verifier "$hosted_ci_verifier_json" \
        '{
            target: $target,
            package_version: $package_version,
            release_version: $release_version,
            workspace_version_matches_release: ($package_version == $release_version),
            branch: $branch,
            expected_branch: (if $expected_branch == "" then null else $expected_branch end),
            upstream: {
                name: $upstream,
                ahead: ($ahead | tonumber),
                behind: ($behind | tonumber)
            },
            head: $head,
            tracked_changes_present: ($tracked == "true"),
            release_target: {
                tag: $release_tag,
                repository: $release_repo,
                state: $release_target_state,
                local_tag_exists: ($release_target_local_tag == "true"),
                remote_git_tag_exists: ($release_target_remote_tag == "true"),
                github_release_exists: ($release_target_github_release == "true")
            },
            pr: $pr,
            hosted_ci: {
                state: $hosted_ci_state,
                expected_event: $expected_event,
                run_id: (if $hosted_run_id == "" then null else ($hosted_run_id | tonumber) end),
                run: $hosted_run,
                verifier: $hosted_ci_verifier
            },
            local_ci: "not_run",
            package_install_smoke: "not_run",
            disk_space: {
                state: "not_checked",
                free_kib: null,
                min_required_kib: ($min_free_space_kib | tonumber),
                shortfall_kib: null,
                cleanup_candidates: []
            },
            homebrew_formula_render: "not_run",
            homebrew_formula: {
                output: null
            },
            release_scope: {
                release_notes_path: $release_notes_path,
                state: $release_scope_state,
                native_claude_proof_limits_acknowledged: ($release_scope_native_claude == "true"),
                lifecycle_m6_limits_acknowledged: ($release_scope_lifecycle_m6 == "true")
            },
            release_gate_state: $release_gate_state,
            ready_for_release_owner_review: false,
            hosted_ci_fallback_decision_required: false,
            remaining_release_actions: (
                if $release_target_state == "unknown" then
                    [
                        "restore_release_target_lookup_access",
                        "rerun_ga_release_gate_report"
                    ]
                else
                    [
                        "inspect_existing_release_target",
                        "resolve_release_target_conflict_before_owner_review",
                        "rerun_ga_release_gate_report"
                    ]
                end
            ),
            failure: {
                kind: "release_target_preflight",
                message: $release_target_error
            },
            release_owner_decision_required: true,
            release_actions_performed: false
        }'
}

emit_disk_space_failure_json() {
    jq -n \
        --arg target "$target" \
        --arg package_version "$package_version" \
        --arg release_version "$release_version" \
        --arg release_tag "$release_tag" \
        --arg release_repo "$release_repo" \
        --arg release_target_state "$release_target_state" \
        --arg release_target_local_tag "$release_target_local_tag_exists" \
        --arg release_target_remote_tag "$release_target_remote_tag_exists" \
        --arg release_target_github_release "$release_target_github_release_exists" \
        --arg branch "$branch" \
        --arg expected_branch "$expected_branch" \
        --arg upstream "$upstream" \
        --arg ahead "$ahead_count" \
        --arg behind "$behind_count" \
        --arg head "$head_sha" \
        --arg tracked "$tracked_changes_present" \
        --arg expected_event "$expected_event" \
        --arg hosted_ci_state "$hosted_ci_state" \
        --arg hosted_run_id "$hosted_run_id" \
        --arg disk_space_state "$disk_space_state" \
        --arg free_space_kib "$free_space_kib" \
        --arg min_free_space_kib "$min_free_space_kib" \
        --arg disk_space_shortfall_kib "$disk_space_shortfall_kib" \
        --arg disk_space_error "$disk_space_error" \
        --arg release_notes_path "$release_notes_path" \
        --arg release_scope_state "$release_scope_state" \
        --arg release_scope_native_claude "$release_scope_native_claude_ack" \
        --arg release_scope_lifecycle_m6 "$release_scope_lifecycle_m6_ack" \
        --argjson pr "$pr_report_json" \
        --argjson hosted_run "$hosted_run_report_json" \
        --argjson hosted_ci_verifier "$hosted_ci_verifier_json" \
        --argjson disk_cleanup_candidates "$disk_cleanup_candidates_json" \
        '{
            target: $target,
            package_version: $package_version,
            release_version: $release_version,
            workspace_version_matches_release: ($package_version == $release_version),
            branch: $branch,
            expected_branch: (if $expected_branch == "" then null else $expected_branch end),
            upstream: {
                name: $upstream,
                ahead: ($ahead | tonumber),
                behind: ($behind | tonumber)
            },
            head: $head,
            tracked_changes_present: ($tracked == "true"),
            release_target: {
                tag: $release_tag,
                repository: $release_repo,
                state: $release_target_state,
                local_tag_exists: ($release_target_local_tag == "true"),
                remote_git_tag_exists: ($release_target_remote_tag == "true"),
                github_release_exists: ($release_target_github_release == "true")
            },
            pr: $pr,
            hosted_ci: {
                state: $hosted_ci_state,
                expected_event: $expected_event,
                run_id: (if $hosted_run_id == "" then null else ($hosted_run_id | tonumber) end),
                run: $hosted_run,
                verifier: $hosted_ci_verifier
            },
            local_ci: "not_run",
            package_install_smoke: "not_run",
            disk_space: {
                state: $disk_space_state,
                free_kib: ($free_space_kib | tonumber),
                min_required_kib: ($min_free_space_kib | tonumber),
                shortfall_kib: ($disk_space_shortfall_kib | tonumber),
                cleanup_candidates: $disk_cleanup_candidates
            },
            homebrew_formula_render: "not_run",
            homebrew_formula: {
                output: null
            },
            release_scope: {
                release_notes_path: $release_notes_path,
                state: $release_scope_state,
                native_claude_proof_limits_acknowledged: ($release_scope_native_claude == "true"),
                lifecycle_m6_limits_acknowledged: ($release_scope_lifecycle_m6 == "true")
            },
            release_gate_state: "disk_space_cleanup_required",
            ready_for_release_owner_review: false,
            hosted_ci_fallback_decision_required: false,
            remaining_release_actions: [
                "free_local_disk_space_or_get_cleanup_approval",
                "rerun_full_release_gate_report_with_local_ci_and_package_smoke"
            ],
            failure: {
                kind: "disk_space_preflight",
                message: $disk_space_error
            },
            release_owner_decision_required: true,
            release_actions_performed: false
        }'
}

run_release_target_preflight() {
    [[ "$target" == "ga" ]] || return 0

    if [[ "$json_output" == "1" ]]; then
        printf '\n==> release target availability\n' >&2
    else
        printf '\n==> release target availability\n'
    fi

    if ! check_release_target_available; then
        if [[ "$json_output" == "1" ]]; then
            emit_release_target_failure_json
        fi
        fail "$release_target_error"
    fi
}

run_disk_space_preflight() {
    if [[ "$json_output" == "1" ]]; then
        printf '\n==> disk space preflight\n' >&2
    else
        printf '\n==> disk space preflight\n'
    fi

    if ! check_free_space_for_local_steps; then
        if [[ "$json_output" == "1" ]]; then
            emit_disk_space_failure_json
        fi
        fail "$disk_space_error"
    fi
}

collect_hosted_run_checks() {
    if [[ -z "$hosted_run_id" ]]; then
        hosted_run_id="$(
            gh run list \
                --workflow "$expected_workflow" \
                --branch "$branch" \
                --event "$expected_event" \
                --limit 1 \
                --json databaseId \
                --jq '.[0].databaseId // empty'
        )"
    fi

    [[ -n "$hosted_run_id" ]] || fail "no hosted CI run id was provided or discovered"

    gh run view "$hosted_run_id" \
        --json databaseId,headSha,status,conclusion,workflowName,event,jobs,url >"$hosted_run_json"

    actual_run_id="$(jq -r '.databaseId // empty' "$hosted_run_json")"
    actual_head="$(jq -r '.headSha // empty' "$hosted_run_json")"
    actual_status="$(jq -r '.status // empty' "$hosted_run_json")"
    actual_conclusion="$(jq -r '.conclusion // empty' "$hosted_run_json")"
    actual_workflow="$(jq -r '.workflowName // empty' "$hosted_run_json")"
    actual_event="$(jq -r '.event // empty' "$hosted_run_json")"

    [[ "$actual_run_id" == "$hosted_run_id" ]] ||
        fail "hosted run id mismatch: expected $hosted_run_id, got $actual_run_id"
    [[ "$actual_head" == "$head_sha" ]] ||
        fail "hosted run head mismatch: expected $head_sha, got $actual_head"
    [[ "$actual_status" == "completed" ]] ||
        fail "hosted run is not completed: $actual_status"
    [[ "$actual_conclusion" == "success" ]] ||
        fail "hosted run conclusion is not success: $actual_conclusion"
    [[ "$actual_workflow" == "$expected_workflow" ]] ||
        fail "workflow mismatch: expected $expected_workflow, got $actual_workflow"
    [[ "$actual_event" == "$expected_event" ]] ||
        fail "hosted run event mismatch: expected $expected_event, got $actual_event"

    jq -r '.jobs[] | [.name, .status, (.conclusion // "")] | @tsv' \
        "$hosted_run_json" >"$checks_file"

    expected_jobs_sorted="$(printf '%s\n' "${expected_jobs[@]}" | LC_ALL=C sort)"
    actual_jobs_sorted="$(awk -F '\t' '{ print $1 }' "$checks_file" | LC_ALL=C sort)"
    if [[ "$actual_jobs_sorted" != "$expected_jobs_sorted" ]]; then
        {
            printf 'error: hosted CI jobs did not match expected release gate jobs\n'
            printf 'expected:\n%s\n' "$expected_jobs_sorted"
            printf 'actual:\n%s\n' "$actual_jobs_sorted"
        } >&2
        exit 1
    fi

    awk -F '\t' 'BEGIN { ok = 1 } $2 != "completed" || $3 != "success" { ok = 0 } END { exit ok ? 0 : 1 }' \
        "$checks_file" || fail "hosted CI jobs are not all completed successfully"

    hosted_ci_state="passing"
}

validate_homebrew_formula_render() {
    FORMULA_OUTPUT="$homebrew_formula_output" "$repo_root/scripts/render-homebrew-formula.sh"
    ruby -c "$homebrew_formula_output"
    if grep -E "Homebrew beta|beta Homebrew|Homebrew beta currently" "$homebrew_formula_output"; then
        fail "rendered Homebrew formula contains beta-specific wording: $homebrew_formula_output"
    fi
}

if [[ "$target" == "ga" ]]; then
    release_scope_state="incomplete"
    if [[ -f "$release_notes_path" ]]; then
        if grep -Fq "Native Claude prompt-bearing proof" "$release_notes_path" &&
            grep -Fq "live \`/hooks\` effective-hook visibility" "$release_notes_path"; then
            release_scope_native_claude_ack=true
        fi
        if grep -Fq "v0.2.0 does not claim" "$release_notes_path" &&
            grep -Fq "broad legacy deprecation" "$release_notes_path" &&
            grep -Fq "unrestricted automated" "$release_notes_path"; then
            release_scope_lifecycle_m6_ack=true
        fi
        if [[ "$release_scope_native_claude_ack" == "true" &&
            "$release_scope_lifecycle_m6_ack" == "true" ]]; then
            release_scope_state="complete"
        fi
    else
        release_scope_state="missing_release_notes"
    fi

    collect_hosted_run_checks
    hosted_run_report_json="$(jq -c '.' "$hosted_run_json")"
else
    gh pr view "$pr_number" \
        --json number,url,isDraft,headRefOid,mergeStateStatus,statusCheckRollup >"$pr_json"

    pr_head="$(jq -r '.headRefOid // empty' "$pr_json")"
    pr_url="$(jq -r '.url // empty' "$pr_json")"
    pr_draft="$(jq -r '.isDraft' "$pr_json")"
    pr_merge_state="$(jq -r '.mergeStateStatus // empty' "$pr_json")"

    [[ "$pr_head" == "$head_sha" ]] ||
        fail "PR #$pr_number head mismatch: expected $head_sha, got $pr_head"

    jq -r '
        .statusCheckRollup[]
        | select(.__typename == "CheckRun" and .workflowName == "CI")
        | [.name, .status, (.conclusion // "")] | @tsv
    ' "$pr_json" >"$checks_file"

    expected_jobs_sorted="$(printf '%s\n' "${expected_jobs[@]}" | LC_ALL=C sort)"
    actual_jobs_sorted="$(awk -F '\t' '{ print $1 }' "$checks_file" | LC_ALL=C sort)"

    if [[ "$actual_jobs_sorted" == "$expected_jobs_sorted" ]] &&
        awk -F '\t' 'BEGIN { ok = 1 } $2 != "COMPLETED" || $3 != "SUCCESS" { ok = 0 } END { exit ok ? 0 : 1 }' \
            "$checks_file"; then
        hosted_ci_state="passing"
    else
        verify_args=()
        if [[ -n "$hosted_run_id" ]]; then
            verify_args+=("$hosted_run_id")
        fi

        if [[ "$json_output" == "1" ]]; then
            printf '\n==> verify hosted CI pre-step fallback\n' >&2
            env EXPECTED_HEAD_SHA="$head_sha" \
                "$repo_root/scripts/verify-hosted-ci-prestep-blocker.sh" \
                --event "$expected_event" --json "${verify_args[@]}" >"$hosted_verifier_file"
            jq -e '.condition_verified == true' "$hosted_verifier_file" >/dev/null ||
                fail "hosted CI pre-step verifier did not emit verified JSON"
            hosted_ci_verifier_json="$(jq -c '.' "$hosted_verifier_file")"
            if [[ -z "$hosted_run_id" ]]; then
                hosted_run_id="$(jq -r '.run.id // empty' "$hosted_verifier_file")"
            fi
        else
            run_step "verify hosted CI pre-step fallback" env EXPECTED_HEAD_SHA="$head_sha" \
                "$repo_root/scripts/verify-hosted-ci-prestep-blocker.sh" \
                --event "$expected_event" "${verify_args[@]}"
        fi
        hosted_ci_state="pre_step_blocker_verified"
    fi
fi

run_release_target_preflight
run_disk_space_preflight

if [[ "$run_local_ci" == "1" ]]; then
    run_step "local CI-equivalent validation" "$repo_root/scripts/local-ci.sh"
else
    if [[ "$json_output" == "1" ]]; then
        printf '\n==> local CI-equivalent validation\nskipped\n' >&2
    else
        printf '\n==> local CI-equivalent validation\nskipped\n'
    fi
fi

if [[ "$run_package_smoke" == "1" ]]; then
    if [[ "$allow_tracked_changes" == "1" ]]; then
        run_step "package/install smoke validation" env ALLOW_TRACKED_CHANGES=1 \
            "$repo_root/scripts/package-install-smoke.sh"
    else
        run_step "package/install smoke validation" "$repo_root/scripts/package-install-smoke.sh"
    fi
else
    if [[ "$json_output" == "1" ]]; then
        printf '\n==> package/install smoke validation\nskipped\n' >&2
    else
        printf '\n==> package/install smoke validation\nskipped\n'
    fi
fi

if [[ "$target" == "ga" ]]; then
    if [[ "$run_homebrew_render" == "1" && "$run_package_smoke" == "1" ]]; then
        require_tool ruby
        run_step "Homebrew formula render validation" validate_homebrew_formula_render
        homebrew_formula_state="passed"
    else
        homebrew_formula_state="skipped"
        if [[ "$json_output" == "1" ]]; then
            printf '\n==> Homebrew formula render validation\nskipped\n' >&2
        else
            printf '\n==> Homebrew formula render validation\nskipped\n'
        fi
    fi
fi

local_ci_state="$([[ "$run_local_ci" == "1" ]] && printf 'passed' || printf 'skipped')"
package_smoke_state="$([[ "$run_package_smoke" == "1" ]] && printf 'passed' || printf 'skipped')"

release_gate_state="evidence_incomplete"
ready_for_release_owner_review=false
hosted_ci_fallback_decision_required=false

if [[ "$target" == "ga" && "$package_version" != "$release_version" ]]; then
    release_gate_state="version_bump_required"
elif [[ "$target" == "ga" && "$release_scope_state" != "complete" ]]; then
    release_gate_state="release_scope_acknowledgement_required"
elif [[ "$target" == "ga" && "$local_ci_state" == "passed" &&
    "$package_smoke_state" == "passed" && "$homebrew_formula_state" != "passed" ]]; then
    release_gate_state="homebrew_formula_render_required"
elif [[ "$local_ci_state" == "passed" && "$package_smoke_state" == "passed" ]]; then
    if [[ "$hosted_ci_state" == "passing" ]]; then
        if [[ "$target" != "ga" || "$homebrew_formula_state" == "passed" ]]; then
            release_gate_state="hosted_ci_passing_release_owner_review_required"
            ready_for_release_owner_review=true
        fi
    elif [[ "$target" == "beta" && "$hosted_ci_state" == "pre_step_blocker_verified" ]]; then
        release_gate_state="fallback_release_owner_decision_required"
        ready_for_release_owner_review=true
        hosted_ci_fallback_decision_required=true
    fi
fi

checks_json="$(
    jq -R -s '
        split("\n")
        | map(select(length > 0) | split("\t") | {
            name: .[0],
            status: .[1],
            conclusion: .[2]
        })
    ' "$checks_file"
)"

if [[ "$target" == "beta" ]]; then
    pr_report_json="$(
        jq -n \
            --arg pr_number "$pr_number" \
            --arg pr_url "$pr_url" \
            --arg pr_draft "$pr_draft" \
            --arg pr_merge_state "$pr_merge_state" \
            --argjson checks "$checks_json" \
            '{
                number: ($pr_number | tonumber),
                url: $pr_url,
                draft: ($pr_draft == "true"),
                merge_state: $pr_merge_state,
                checks: $checks
            }'
    )"
fi

if [[ "$target" != "ga" ]]; then
    hosted_run_report_json="null"
fi

remaining_release_actions_json="$(
    jq -n \
        --arg target "$target" \
        --arg state "$release_gate_state" \
        --arg package_version "$package_version" \
        --arg release_version "$release_version" \
        'if $target == "ga" and $package_version != $release_version then
            [
                "bump_workspace_version_to_\($release_version)",
                "rerun_exact_head_hosted_ci",
                "run_full_ga_release_gate_report_with_local_ci_and_package_smoke"
            ]
        elif $state == "fallback_release_owner_decision_required" then
            [
                "release_owner_accept_hosted_ci_fallback_or_restore_hosted_ci",
                "mark_pr_ready",
                "merge_pr",
                "tag_v\($release_version)",
                "publish_release_artifacts",
                "publish_homebrew_tap",
                "verify_published_release_install"
            ]
        elif $state == "hosted_ci_passing_release_owner_review_required" and $target == "ga" then
            [
                "release_owner_approve_release",
                "tag_v\($release_version)",
                "publish_release_artifacts",
                "publish_homebrew_tap",
                "verify_published_release_install"
            ]
        elif $state == "release_scope_acknowledgement_required" and $target == "ga" then
            [
                "restore_release_notes_ga_scope_acknowledgements",
                "rerun_ga_release_gate_report"
            ]
        elif $state == "homebrew_formula_render_required" and $target == "ga" then
            [
                "rerun_ga_release_gate_report_with_homebrew_formula_render"
            ]
        elif $state == "hosted_ci_passing_release_owner_review_required" then
            [
                "release_owner_approve_release",
                "mark_pr_ready",
                "merge_pr",
                "tag_v\($release_version)",
                "publish_release_artifacts",
                "publish_homebrew_tap",
                "verify_published_release_install"
            ]
        else
            [
                "run_full_\($target)_release_gate_report_with_local_ci_and_package_smoke"
            ]
        end'
)"

if [[ "$json_output" == "1" ]]; then
    jq -n \
        --arg target "$target" \
        --arg package_version "$package_version" \
        --arg release_version "$release_version" \
        --arg release_tag "$release_tag" \
        --arg release_repo "$release_repo" \
        --arg release_target_state "$release_target_state" \
        --arg release_target_local_tag "$release_target_local_tag_exists" \
        --arg release_target_remote_tag "$release_target_remote_tag_exists" \
        --arg release_target_github_release "$release_target_github_release_exists" \
        --arg branch "$branch" \
        --arg expected_branch "$expected_branch" \
        --arg upstream "$upstream" \
        --arg ahead "$ahead_count" \
        --arg behind "$behind_count" \
        --arg head "$head_sha" \
        --arg tracked "$tracked_changes_present" \
        --arg expected_event "$expected_event" \
        --arg hosted_ci_state "$hosted_ci_state" \
        --arg hosted_run_id "$hosted_run_id" \
        --arg local_ci "$local_ci_state" \
        --arg package_install_smoke "$package_smoke_state" \
        --arg homebrew_formula_render "$homebrew_formula_state" \
        --arg homebrew_formula_output "$homebrew_formula_output" \
        --arg disk_space_state "$disk_space_state" \
        --arg free_space_kib "$free_space_kib" \
        --arg min_free_space_kib "$min_free_space_kib" \
        --arg disk_space_shortfall_kib "$disk_space_shortfall_kib" \
        --arg release_notes_path "$release_notes_path" \
        --arg release_scope_state "$release_scope_state" \
        --arg release_scope_native_claude "$release_scope_native_claude_ack" \
        --arg release_scope_lifecycle_m6 "$release_scope_lifecycle_m6_ack" \
        --arg release_gate_state "$release_gate_state" \
        --arg ready_for_review "$ready_for_release_owner_review" \
        --arg fallback_decision "$hosted_ci_fallback_decision_required" \
        --argjson pr "$pr_report_json" \
        --argjson hosted_run "$hosted_run_report_json" \
        --argjson hosted_ci_verifier "$hosted_ci_verifier_json" \
        --argjson disk_cleanup_candidates "$disk_cleanup_candidates_json" \
        --argjson remaining_release_actions "$remaining_release_actions_json" \
        '{
            target: $target,
            package_version: $package_version,
            release_version: $release_version,
            workspace_version_matches_release: ($package_version == $release_version),
            branch: $branch,
            expected_branch: (if $expected_branch == "" then null else $expected_branch end),
            upstream: {
                name: $upstream,
                ahead: ($ahead | tonumber),
                behind: ($behind | tonumber)
            },
            head: $head,
            tracked_changes_present: ($tracked == "true"),
            release_target: {
                tag: $release_tag,
                repository: $release_repo,
                state: $release_target_state,
                local_tag_exists: ($release_target_local_tag == "true"),
                remote_git_tag_exists: ($release_target_remote_tag == "true"),
                github_release_exists: ($release_target_github_release == "true")
            },
            pr: $pr,
            hosted_ci: {
                state: $hosted_ci_state,
                expected_event: $expected_event,
                run_id: (if $hosted_run_id == "" then null else ($hosted_run_id | tonumber) end),
                run: $hosted_run,
                verifier: $hosted_ci_verifier
            },
            local_ci: $local_ci,
            package_install_smoke: $package_install_smoke,
            disk_space: {
                state: $disk_space_state,
                free_kib: (
                    if $free_space_kib == "" then null
                    else ($free_space_kib | tonumber)
                    end
                ),
                min_required_kib: ($min_free_space_kib | tonumber),
                shortfall_kib: (
                    if $disk_space_shortfall_kib == "" then null
                    else ($disk_space_shortfall_kib | tonumber)
                    end
                ),
                cleanup_candidates: $disk_cleanup_candidates
            },
            homebrew_formula_render: $homebrew_formula_render,
            homebrew_formula: {
                output: (if $homebrew_formula_render == "not_applicable" then null else $homebrew_formula_output end)
            },
            release_scope: {
                release_notes_path: $release_notes_path,
                state: $release_scope_state,
                native_claude_proof_limits_acknowledged: ($release_scope_native_claude == "true"),
                lifecycle_m6_limits_acknowledged: ($release_scope_lifecycle_m6 == "true")
            },
            release_gate_state: $release_gate_state,
            ready_for_release_owner_review: ($ready_for_review == "true"),
            hosted_ci_fallback_decision_required: ($fallback_decision == "true"),
            remaining_release_actions: $remaining_release_actions,
            release_owner_decision_required: true,
            release_actions_performed: false
        }'
else
    release_target_label="$([[ "$target" == "ga" ]] && printf 'GA' || printf 'Beta')"
    printf '\n%s release gate evidence collected:\n' "$release_target_label"
    printf '  package_version: %s\n' "$package_version"
    printf '  release_version: %s\n' "$release_version"
    printf '  workspace_version_matches_release: %s\n' \
        "$([[ "$package_version" == "$release_version" ]] && printf true || printf false)"
    printf '  branch: %s\n' "$branch"
    printf '  expected_branch: %s\n' "${expected_branch:-<none>}"
    printf '  upstream: %s (ahead=%s behind=%s)\n' "$upstream" "$ahead_count" "$behind_count"
    printf '  head: %s\n' "$head_sha"
    printf '  tracked_changes_present: %s\n' "$tracked_changes_present"
    if [[ "$target" == "beta" ]]; then
        printf '  pr: #%s %s\n' "$pr_number" "$pr_url"
        printf '  pr_draft: %s\n' "$pr_draft"
        printf '  pr_merge_state: %s\n' "$pr_merge_state"
    else
        printf '  hosted_run: %s\n' "$hosted_run_id"
        printf '  hosted_event: %s\n' "$expected_event"
        printf '  release_target_tag: %s\n' "$release_tag"
        printf '  release_target_repository: %s\n' "$release_repo"
        printf '  release_target_state: %s\n' "$release_target_state"
        printf '  release_target_local_tag_exists: %s\n' "$release_target_local_tag_exists"
        printf '  release_target_remote_git_tag_exists: %s\n' \
            "$release_target_remote_tag_exists"
        printf '  release_target_github_release_exists: %s\n' \
            "$release_target_github_release_exists"
    fi
    printf '  hosted_ci_state: %s\n' "$hosted_ci_state"
    printf '  local_ci: %s\n' "$local_ci_state"
    printf '  package_install_smoke: %s\n' "$package_smoke_state"
    printf '  disk_space_preflight: %s' "$disk_space_state"
    if [[ -n "$free_space_kib" ]]; then
        printf ' (free_kib=%s min_required_kib=%s)' "$free_space_kib" "$min_free_space_kib"
        if [[ -n "$disk_space_shortfall_kib" ]]; then
            printf ' shortfall_kib=%s' "$disk_space_shortfall_kib"
        fi
    fi
    printf '\n'
    jq -r '
        .[]
        | "  disk_cleanup_candidate: \(.path) size_kib=\(.size_kib)"
    ' <<<"$disk_cleanup_candidates_json"
    if [[ "$target" == "ga" ]]; then
        printf '  homebrew_formula_render: %s\n' "$homebrew_formula_state"
        if [[ "$homebrew_formula_state" == "passed" ]]; then
            printf '  homebrew_formula_output: %s\n' "$homebrew_formula_output"
        fi
    fi
    if [[ "$target" == "ga" ]]; then
        printf '  release_scope_state: %s\n' "$release_scope_state"
        printf '  release_scope_native_claude_proof_limits_acknowledged: %s\n' \
            "$release_scope_native_claude_ack"
        printf '  release_scope_lifecycle_m6_limits_acknowledged: %s\n' \
            "$release_scope_lifecycle_m6_ack"
    fi
    printf '  release_gate_state: %s\n' "$release_gate_state"
    printf '  ready_for_release_owner_review: %s\n' "$ready_for_release_owner_review"
    printf '  hosted_ci_fallback_decision_required: %s\n' \
        "$hosted_ci_fallback_decision_required"
    printf '  release_owner_decision_required: true\n'
    printf '  release_actions_performed: false\n'
fi
