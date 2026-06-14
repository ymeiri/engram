#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repo_root"

target="${RELEASE_TARGET-ga}"
pr_number=""
pr_number_explicit=0
if [[ "${PR_NUMBER+x}" == "x" ]]; then
    pr_number="$PR_NUMBER"
    pr_number_explicit=1
fi
hosted_run_id=""
hosted_run_id_explicit=0
if [[ "${HOSTED_RUN_ID+x}" == "x" ]]; then
    hosted_run_id="$HOSTED_RUN_ID"
    hosted_run_id_explicit=1
fi
default_expected_workflow="CI"
expected_workflow="${EXPECTED_WORKFLOW_NAME-$default_expected_workflow}"
allow_expected_workflow_override="${ALLOW_EXPECTED_WORKFLOW_NAME_OVERRIDE:-0}"
expected_event=""
expected_event_explicit=0
if [[ "${EXPECTED_EVENT+x}" == "x" ]]; then
    expected_event="$EXPECTED_EVENT"
    expected_event_explicit=1
fi
allow_expected_event_override="${ALLOW_EXPECTED_EVENT_OVERRIDE:-0}"
expected_branch=""
expected_branch_explicit=0
if [[ "${EXPECTED_BRANCH+x}" == "x" ]]; then
    expected_branch="$EXPECTED_BRANCH"
    expected_branch_explicit=1
fi
allow_expected_branch_override="${ALLOW_EXPECTED_BRANCH_OVERRIDE:-0}"
package_version=""
release_version=""
release_tag=""
release_version_explicit=0
if [[ "${RELEASE_VERSION+x}" == "x" ]]; then
    release_version="$RELEASE_VERSION"
    release_version_explicit=1
fi
default_release_notes_path="$repo_root/docs/RELEASE_NOTES_V0_2_0.md"
release_notes_path="${RELEASE_NOTES_PATH-$default_release_notes_path}"
allow_release_notes_path_override="${ALLOW_RELEASE_NOTES_PATH_OVERRIDE:-0}"
default_release_repo="ymeiri/engram"
release_repo="${RELEASE_REPOSITORY-$default_release_repo}"
allow_release_repo_override="${ALLOW_RELEASE_REPOSITORY_OVERRIDE:-0}"
default_min_free_space_kib=10485760
min_free_space_kib="${RELEASE_GATE_MIN_FREE_KIB-$default_min_free_space_kib}"
allow_min_free_space_override="${ALLOW_RELEASE_GATE_MIN_FREE_OVERRIDE:-0}"
run_local_ci=1
run_package_smoke=1
run_homebrew_render=1
allow_tracked_changes=0
json_output=0
verify_generated_output_cleanup_manifest=""
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
  --verify-generated-output-cleanup <gate-json>
                                Read-only verification that current generated outputs match
                                a prior generated_outputs_cleanup_required gate JSON
  --json                        Emit final evidence as machine-readable JSON
  -h, --help                    Show this help

Environment overrides:
  RELEASE_TARGET, PR_NUMBER, HOSTED_RUN_ID, EXPECTED_WORKFLOW_NAME,
  ALLOW_EXPECTED_WORKFLOW_NAME_OVERRIDE, EXPECTED_EVENT,
  ALLOW_EXPECTED_EVENT_OVERRIDE, EXPECTED_BRANCH,
  ALLOW_EXPECTED_BRANCH_OVERRIDE, RELEASE_VERSION,
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

emit_config_failure_json() {
    local failure_message="$1"

    jq -n \
        --arg target "$target" \
        --arg package_version "$package_version" \
        --arg release_version "$release_version" \
        --arg release_tag "$release_tag" \
        --arg release_repo "$release_repo" \
        --arg expected_branch "$expected_branch" \
        --arg expected_workflow "$expected_workflow" \
        --arg expected_event "$expected_event" \
        --arg hosted_run_id "$hosted_run_id" \
        --arg min_free_space_kib "$min_free_space_kib" \
        --arg release_notes_path "$release_notes_path" \
        --arg failure_message "$failure_message" \
        '{
            target: $target,
            package_version: (if $package_version == "" then null else $package_version end),
            release_version: (if $release_version == "" then null else $release_version end),
            workspace_version_matches_release: (
                if $package_version == "" or $release_version == "" then null
                else ($package_version == $release_version)
                end
            ),
            branch: null,
            expected_branch: (if $expected_branch == "" then null else $expected_branch end),
            upstream: null,
            head: null,
            tracked_changes_present: null,
            release_target: {
                tag: (if $release_tag == "" then null else $release_tag end),
                repository: $release_repo,
                state: "not_checked",
                local_tag_exists: null,
                remote_git_tag_exists: null,
                github_release_exists: null
            },
            pr: null,
            hosted_ci: {
                state: "not_checked",
                repository: $release_repo,
                expected_workflow: $expected_workflow,
                expected_event: (if $expected_event == "" then null else $expected_event end),
                run_id: (
                    if ($hosted_run_id | test("^[0-9]+$")) then ($hosted_run_id | tonumber)
                    else null
                    end
                ),
                run: null,
                verifier: null
            },
            local_ci: "not_run",
            package_install_smoke: "not_run",
            disk_space: {
                state: "not_checked",
                free_kib: null,
                min_required_kib: (
                    if ($min_free_space_kib | test("^[0-9]+$")) then
                        ($min_free_space_kib | tonumber)
                    else
                        null
                    end
                ),
                shortfall_kib: null,
                cleanup_candidates: []
            },
            generated_outputs: {
                state: "not_checked",
                host_triple: null,
                outputs: [],
                error: null
            },
            generated_artifacts: {
                state: "not_checked",
                host_triple: null,
                artifacts: [],
                error: null
            },
            homebrew_formula_render: "not_run",
            homebrew_formula: {
                output: null
            },
            release_scope: {
                release_notes_path: $release_notes_path,
                state: "not_checked",
                native_claude_proof_limits_acknowledged: false,
                lifecycle_m6_limits_acknowledged: false
            },
            release_gate_state: "configuration_preflight_failed",
            ready_for_release_owner_review: false,
            hosted_ci_fallback_decision_required: false,
            remaining_release_actions: [
                "fix_release_gate_configuration",
                "rerun_ga_release_gate_report"
            ],
            failure: {
                kind: "configuration_preflight",
                message: $failure_message
            },
            release_owner_decision_required: true,
            actions_performed: {
                release_actions: false,
                git_tag: false,
                github_release: false,
                package_asset_upload: false,
                homebrew_tap_update: false,
                generated_output_cleanup: false
            },
            release_actions_performed: false
        }'
}

fail_config_preflight() {
    local failure_message="$1"

    if [[ "$json_output" == "1" ]] && command -v jq >/dev/null 2>&1; then
        emit_config_failure_json "$failure_message"
    fi
    fail "$failure_message"
}

require_tool() {
    local tool="$1"
    command -v "$tool" >/dev/null 2>&1 ||
        fail_config_preflight "required tool is missing: $tool"
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

for arg in "$@"; do
    if [[ "$arg" == "--json" ]]; then
        json_output=1
        break
    fi
done

while [[ $# -gt 0 ]]; do
    case "$1" in
        --target)
            [[ $# -ge 2 ]] || fail_config_preflight "--target requires ga or beta"
            case "$2" in
                ga | beta) target="$2" ;;
                *) fail_config_preflight "--target must be ga or beta" ;;
            esac
            shift 2
            ;;
        --pr)
            [[ $# -ge 2 ]] || fail_config_preflight "--pr requires a PR number"
            [[ -n "$2" ]] || fail_config_preflight "PR_NUMBER/--pr must not be empty"
            pr_number="$2"
            pr_number_explicit=1
            shift 2
            ;;
        --hosted-run)
            [[ $# -ge 2 ]] || fail_config_preflight "--hosted-run requires a run ID"
            [[ -n "$2" ]] || fail_config_preflight "HOSTED_RUN_ID/--hosted-run must not be empty"
            hosted_run_id="$2"
            hosted_run_id_explicit=1
            shift 2
            ;;
        --release-version)
            [[ $# -ge 2 ]] || fail_config_preflight "--release-version requires a version"
            [[ -n "$2" ]] ||
                fail_config_preflight "RELEASE_VERSION/--release-version must not be empty"
            release_version="$2"
            release_version_explicit=1
            shift 2
            ;;
        --expected-event)
            [[ $# -ge 2 ]] || fail_config_preflight "--expected-event requires an event name"
            expected_event="$2"
            expected_event_explicit=1
            shift 2
            ;;
        --expected-branch)
            [[ $# -ge 2 ]] || fail_config_preflight "--expected-branch requires a branch name"
            [[ -n "$2" ]] ||
                fail_config_preflight "EXPECTED_BRANCH/--expected-branch must not be empty"
            expected_branch="$2"
            expected_branch_explicit=1
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
        --verify-generated-output-cleanup)
            [[ $# -ge 2 ]] ||
                fail_config_preflight "--verify-generated-output-cleanup requires a gate JSON path"
            [[ -n "$2" ]] ||
                fail_config_preflight \
                    "--verify-generated-output-cleanup gate JSON path must not be empty"
            verify_generated_output_cleanup_manifest="$2"
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
            fail_config_preflight "unknown option: $1"
            ;;
    esac
done

case "$target" in
    ga | beta) ;;
    *) fail_config_preflight "--target must be ga or beta" ;;
esac

case "$allow_expected_workflow_override" in
    0 | 1) ;;
    *)
        workflow_override_error="ALLOW_EXPECTED_WORKFLOW_NAME_OVERRIDE must be 0 or 1"
        workflow_override_error+=", got $allow_expected_workflow_override"
        fail_config_preflight "$workflow_override_error"
        ;;
esac
if [[ -z "$expected_workflow" ]]; then
    fail_config_preflight "EXPECTED_WORKFLOW_NAME must not be empty"
fi
workflow_name_pattern='^[A-Za-z0-9_. -]+$'
if [[ ! "$expected_workflow" =~ $workflow_name_pattern ]]; then
    workflow_name_error="EXPECTED_WORKFLOW_NAME must contain only letters, numbers, spaces,"
    workflow_name_error+=" dot, underscore, and hyphen; got $expected_workflow"
    fail_config_preflight "$workflow_name_error"
fi
if [[ "$expected_workflow" != "$default_expected_workflow" &&
    "$allow_expected_workflow_override" != "1" ]]; then
    workflow_override_error="EXPECTED_WORKFLOW_NAME override requires explicit approval"
    workflow_override_error+="; expected default: $default_expected_workflow"
    workflow_override_error+="; got: $expected_workflow"
    workflow_override_error+="; hint: set ALLOW_EXPECTED_WORKFLOW_NAME_OVERRIDE=1"
    workflow_override_error+=" only for local rehearsals"
    fail_config_preflight "$workflow_override_error"
fi

if [[ "$target" == "ga" ]]; then
    default_expected_event="push"
else
    default_expected_event="pull_request"
fi
if [[ -z "$expected_event" && "$expected_event_explicit" == "0" ]]; then
    expected_event="$default_expected_event"
fi
case "$allow_expected_event_override" in
    0 | 1) ;;
    *)
        fail_config_preflight \
            "ALLOW_EXPECTED_EVENT_OVERRIDE must be 0 or 1, got $allow_expected_event_override"
        ;;
esac
if [[ -z "$expected_event" ]]; then
    fail_config_preflight "EXPECTED_EVENT/--expected-event must not be empty"
fi
event_name_pattern='^[A-Za-z0-9_]+$'
if [[ ! "$expected_event" =~ $event_name_pattern ]]; then
    expected_event_error="EXPECTED_EVENT/--expected-event must be a GitHub event name token"
    expected_event_error+=", got $expected_event"
    fail_config_preflight "$expected_event_error"
fi
if [[ "$expected_event" != "$default_expected_event" &&
    "$allow_expected_event_override" != "1" ]]; then
    expected_event_error="EXPECTED_EVENT override requires explicit approval"
    expected_event_error+="; expected default: $default_expected_event"
    expected_event_error+="; got: $expected_event"
    expected_event_error+="; hint: set ALLOW_EXPECTED_EVENT_OVERRIDE=1 only for local rehearsals"
    fail_config_preflight "$expected_event_error"
fi

if [[ "$pr_number_explicit" == "1" && -z "$pr_number" ]]; then
    fail_config_preflight "PR_NUMBER/--pr must not be empty"
fi
if [[ "$hosted_run_id_explicit" == "1" && -z "$hosted_run_id" ]]; then
    fail_config_preflight "HOSTED_RUN_ID/--hosted-run must not be empty"
fi
if [[ -n "$verify_generated_output_cleanup_manifest" ]]; then
    if [[ "$target" != "ga" ]]; then
        fail_config_preflight \
            "--verify-generated-output-cleanup is only supported for GA release gates"
    fi
    if [[ "$run_package_smoke" != "1" ]]; then
        cleanup_verify_error="--verify-generated-output-cleanup requires package outputs"
        cleanup_verify_error+=" in the generated-output inventory"
        cleanup_verify_error+="; do not combine it with --quick or --skip-package-smoke"
        fail_config_preflight "$cleanup_verify_error"
    fi
    if [[ "$target" == "ga" && "$run_homebrew_render" != "1" ]]; then
        cleanup_verify_error="--verify-generated-output-cleanup requires Homebrew formula output"
        cleanup_verify_error+=" in the GA generated-output inventory"
        cleanup_verify_error+="; do not combine it with --quick or --skip-homebrew-render"
        fail_config_preflight "$cleanup_verify_error"
    fi
    if [[ ! -f "$verify_generated_output_cleanup_manifest" ]]; then
        cleanup_verify_error="generated-output cleanup manifest does not exist or is not a file:"
        cleanup_verify_error+=" $verify_generated_output_cleanup_manifest"
        fail_config_preflight "$cleanup_verify_error"
    fi
fi

if [[ "$target" == "beta" && -z "$pr_number" ]]; then
    pr_number=3
fi

require_tool cargo
if ! package_id="$(cargo pkgid --locked -p engram-cli)"; then
    fail_config_preflight "could not determine workspace package version with cargo pkgid"
fi
package_version="${package_id##*#}"
package_version_pattern='^[0-9]+[.][0-9]+[.][0-9]+(-[0-9A-Za-z]+([.-][0-9A-Za-z]+)*)?$'
if [[ -z "$package_version" ]]; then
    fail_config_preflight "workspace package version could not be determined from cargo pkgid"
fi
if [[ ! "$package_version" =~ $package_version_pattern ]]; then
    package_version_error="workspace package version must be x.y.z"
    package_version_error+=" with an optional prerelease suffix, got $package_version"
    fail_config_preflight "$package_version_error"
fi

if [[ "$release_version_explicit" == "1" && -z "$release_version" ]]; then
    fail_config_preflight "RELEASE_VERSION/--release-version must not be empty"
fi
if [[ -z "$release_version" ]]; then
    if [[ "$target" == "ga" ]]; then
        release_version="${package_version%%-*}"
    else
        release_version="$package_version"
    fi
fi
release_version_pattern="$package_version_pattern"
if [[ ! "$release_version" =~ $release_version_pattern ]]; then
    release_version_error="RELEASE_VERSION/--release-version must be x.y.z"
    release_version_error+=" with an optional prerelease suffix, got $release_version"
    fail_config_preflight "$release_version_error"
fi
release_tag="v${release_version}"

if [[ -n "$hosted_run_id" && ! "$hosted_run_id" =~ ^[0-9]+$ ]]; then
    fail_config_preflight \
        "HOSTED_RUN_ID/--hosted-run must be a numeric GitHub Actions run id, got $hosted_run_id"
fi
if [[ -n "$pr_number" && ! "$pr_number" =~ ^[0-9]+$ ]]; then
    pr_number_error="PR_NUMBER/--pr must be a numeric GitHub pull request number"
    pr_number_error+=", got $pr_number"
    fail_config_preflight "$pr_number_error"
fi

require_tool jq
require_tool git
if [[ -z "$verify_generated_output_cleanup_manifest" ]]; then
    require_tool gh
fi
default_expected_branch=""
if [[ "$target" == "ga" ]]; then
    default_expected_branch="main"
fi
case "$allow_expected_branch_override" in
    0 | 1) ;;
    *)
        fail_config_preflight \
            "ALLOW_EXPECTED_BRANCH_OVERRIDE must be 0 or 1, got $allow_expected_branch_override"
        ;;
esac
if [[ "$expected_branch_explicit" == "1" && -z "$expected_branch" ]]; then
    fail_config_preflight "EXPECTED_BRANCH/--expected-branch must not be empty"
fi
if [[ "$target" == "ga" && -z "$expected_branch" ]]; then
    expected_branch="$default_expected_branch"
fi
if [[ -n "$expected_branch" ]]; then
    if ! git check-ref-format --branch "$expected_branch" >/dev/null 2>&1; then
        expected_branch_error="EXPECTED_BRANCH/--expected-branch must be a valid Git branch name"
        expected_branch_error+=", got $expected_branch"
        fail_config_preflight "$expected_branch_error"
    fi
fi
if [[ "$target" == "ga" && "$expected_branch" != "$default_expected_branch" &&
    "$allow_expected_branch_override" != "1" ]]; then
    expected_branch_error="EXPECTED_BRANCH override requires explicit approval"
    expected_branch_error+="; expected default: $default_expected_branch"
    expected_branch_error+="; got: $expected_branch"
    expected_branch_error+="; hint: set ALLOW_EXPECTED_BRANCH_OVERRIDE=1 only for local rehearsals"
    fail_config_preflight "$expected_branch_error"
fi
case "$allow_release_notes_path_override" in
    0 | 1) ;;
    *)
        release_notes_override_error="ALLOW_RELEASE_NOTES_PATH_OVERRIDE must be 0 or 1"
        release_notes_override_error+=", got $allow_release_notes_path_override"
        fail_config_preflight "$release_notes_override_error"
        ;;
esac
if [[ -z "$release_notes_path" ]]; then
    fail_config_preflight "RELEASE_NOTES_PATH must not be empty"
fi
if [[ "$release_notes_path" != "$default_release_notes_path" &&
    "$allow_release_notes_path_override" != "1" ]]; then
    release_notes_override_error="RELEASE_NOTES_PATH override requires explicit approval"
    release_notes_override_error+="; expected default: $default_release_notes_path"
    release_notes_override_error+="; got: $release_notes_path"
    release_notes_override_error+="; hint: set ALLOW_RELEASE_NOTES_PATH_OVERRIDE=1"
    release_notes_override_error+=" only for local rehearsals"
    fail_config_preflight "$release_notes_override_error"
fi
case "$allow_release_repo_override" in
    0 | 1) ;;
    *)
        fail_config_preflight \
            "ALLOW_RELEASE_REPOSITORY_OVERRIDE must be 0 or 1, got $allow_release_repo_override"
        ;;
esac
if [[ -z "$release_repo" ]]; then
    fail_config_preflight "RELEASE_REPOSITORY must not be empty"
fi
if [[ "$release_repo" != "$default_release_repo" &&
    "$allow_release_repo_override" != "1" ]]; then
    release_repo_error="RELEASE_REPOSITORY override requires explicit approval"
    release_repo_error+="; expected default: $default_release_repo"
    release_repo_error+="; got: $release_repo"
    release_repo_error+="; hint: set ALLOW_RELEASE_REPOSITORY_OVERRIDE=1 only for local rehearsals"
    fail_config_preflight "$release_repo_error"
fi
if [[ ! "$release_repo" =~ ^[A-Za-z0-9_.-]+/[A-Za-z0-9_.-]+$ ]]; then
    fail_config_preflight "release repository must be owner/name, got $release_repo"
fi
case "$allow_min_free_space_override" in
    0 | 1) ;;
    *)
        min_free_override_error="ALLOW_RELEASE_GATE_MIN_FREE_OVERRIDE must be 0 or 1"
        min_free_override_error+=", got $allow_min_free_space_override"
        fail_config_preflight "$min_free_override_error"
        ;;
esac
if [[ -z "$min_free_space_kib" ]]; then
    fail_config_preflight "RELEASE_GATE_MIN_FREE_KIB must not be empty"
fi
[[ "$min_free_space_kib" =~ ^[0-9]+$ ]] ||
    fail_config_preflight "RELEASE_GATE_MIN_FREE_KIB must be a non-negative integer"
if [[ "$min_free_space_kib" != "$default_min_free_space_kib" &&
    "$allow_min_free_space_override" != "1" ]]; then
    min_free_error="RELEASE_GATE_MIN_FREE_KIB override requires explicit approval"
    min_free_error+="; expected default: $default_min_free_space_kib"
    min_free_error+="; got: $min_free_space_kib"
    min_free_error+="; hint: set ALLOW_RELEASE_GATE_MIN_FREE_OVERRIDE=1 only for local rehearsals"
    fail_config_preflight "$min_free_error"
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

branch=""
head_sha=""
tracked_changes_present=""
upstream=""
ahead_count=""
behind_count=""
upstream_remote=""
upstream_remote_ref=""
upstream_remote_head=""

emit_repo_state_failure_json() {
    local release_gate_state="$1"
    local failure_kind="$2"
    local failure_message="$3"

    jq -n \
        --arg target "$target" \
        --arg package_version "$package_version" \
        --arg release_version "$release_version" \
        --arg release_tag "$release_tag" \
        --arg release_repo "$release_repo" \
        --arg branch "$branch" \
        --arg expected_branch "$expected_branch" \
        --arg upstream "$upstream" \
        --arg upstream_remote "$upstream_remote" \
        --arg upstream_remote_ref "$upstream_remote_ref" \
        --arg upstream_remote_head "$upstream_remote_head" \
        --arg ahead "$ahead_count" \
        --arg behind "$behind_count" \
        --arg head "$head_sha" \
        --arg tracked "$tracked_changes_present" \
        --arg expected_event "$expected_event" \
        --arg hosted_run_id "$hosted_run_id" \
        --arg min_free_space_kib "$min_free_space_kib" \
        --arg release_notes_path "$release_notes_path" \
        --arg release_gate_state "$release_gate_state" \
        --arg failure_kind "$failure_kind" \
        --arg failure_message "$failure_message" \
        '{
            target: $target,
            package_version: $package_version,
            release_version: $release_version,
            workspace_version_matches_release: ($package_version == $release_version),
            branch: (if $branch == "" then null else $branch end),
            expected_branch: (if $expected_branch == "" then null else $expected_branch end),
            upstream: (
                if $upstream == "" then null
                else {
                    name: $upstream,
                    ahead: (if $ahead == "" then null else ($ahead | tonumber) end),
                    behind: (if $behind == "" then null else ($behind | tonumber) end),
                    remote: (if $upstream_remote == "" then null else $upstream_remote end),
                    remote_ref: (
                        if $upstream_remote_ref == "" then null else $upstream_remote_ref end
                    ),
                    remote_head: (
                        if $upstream_remote_head == "" then null
                        else $upstream_remote_head
                        end
                    ),
                    matches_remote_head: (
                        if $head == "" or $upstream_remote_head == "" then null
                        else ($head == $upstream_remote_head)
                        end
                    )
                }
                end
            ),
            head: (if $head == "" then null else $head end),
            tracked_changes_present: (
                if $tracked == "" then null else ($tracked == "true") end
            ),
            release_target: {
                tag: $release_tag,
                repository: $release_repo,
                state: "not_checked",
                local_tag_exists: null,
                remote_git_tag_exists: null,
                github_release_exists: null
            },
            pr: null,
            hosted_ci: {
                state: "not_checked",
                repository: $release_repo,
                expected_event: $expected_event,
                run_id: (if $hosted_run_id == "" then null else ($hosted_run_id | tonumber) end),
                run: null,
                verifier: null
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
            generated_outputs: {
                state: "not_checked",
                host_triple: null,
                outputs: [],
                error: null
            },
            generated_artifacts: {
                state: "not_checked",
                host_triple: null,
                artifacts: [],
                error: null
            },
            homebrew_formula_render: "not_run",
            homebrew_formula: {
                output: null
            },
            release_scope: {
                release_notes_path: $release_notes_path,
                state: "not_checked",
                native_claude_proof_limits_acknowledged: false,
                lifecycle_m6_limits_acknowledged: false
            },
            release_gate_state: $release_gate_state,
            ready_for_release_owner_review: false,
            hosted_ci_fallback_decision_required: false,
            remaining_release_actions: (
                if $release_gate_state == "tracked_changes_present" then
                    ["commit_or_revert_tracked_changes", "rerun_ga_release_gate_report"]
                elif $release_gate_state == "branch_mismatch" then
                    [
                        "checkout_expected_release_branch",
                        "rerun_exact_head_hosted_ci",
                        "rerun_ga_release_gate_report"
                    ]
                else
                    [
                        "sync_release_branch_with_remote",
                        "rerun_exact_head_hosted_ci",
                        "rerun_ga_release_gate_report"
                    ]
                end
            ),
            failure: {
                kind: $failure_kind,
                message: $failure_message
            },
            release_owner_decision_required: true,
            actions_performed: {
                release_actions: false,
                git_tag: false,
                github_release: false,
                package_asset_upload: false,
                homebrew_tap_update: false,
                generated_output_cleanup: false
            },
            release_actions_performed: false
        }'
}

fail_repo_state_preflight() {
    local release_gate_state="$1"
    local failure_kind="$2"
    local failure_message="$3"

    if [[ "$json_output" == "1" ]]; then
        emit_repo_state_failure_json "$release_gate_state" "$failure_kind" "$failure_message"
    fi
    fail "$failure_message"
}

branch="$(git branch --show-current)"
if [[ -z "$branch" ]]; then
    fail_repo_state_preflight \
        "branch_sync_required" \
        "branch_preflight" \
        "could not determine current branch"
fi
head_sha="$(git rev-parse HEAD)"
if [[ -n "$expected_branch" && "$branch" != "$expected_branch" ]]; then
    branch_error="branch mismatch: expected $expected_branch, got $branch"
    fail_repo_state_preflight "branch_mismatch" "branch_preflight" "$branch_error"
fi

if git diff --quiet --ignore-submodules -- &&
    git diff --cached --quiet --ignore-submodules --; then
    tracked_changes_present=false
else
    tracked_changes_present=true
fi

if [[ "$tracked_changes_present" == "true" && "$allow_tracked_changes" != "1" ]]; then
    tracked_error="tracked working-tree or index changes are present"
    tracked_error+="; commit or pass --allow-tracked-changes"
    fail_repo_state_preflight \
        "tracked_changes_present" \
        "tracked_changes_preflight" \
        "$tracked_error"
fi

upstream="$(git rev-parse --abbrev-ref --symbolic-full-name '@{upstream}' 2>/dev/null || true)"
branch_sync_hint="run git fetch and inspect local/remote commits; do not use git pull "
branch_sync_hint+="as release approval; any reconciliation needs fresh exact-head CI plus gate"
if [[ -z "$upstream" ]]; then
    upstream_error="current branch has no upstream; $branch_sync_hint"
    fail_repo_state_preflight \
        "branch_sync_required" \
        "branch_sync_preflight" \
        "$upstream_error"
fi
read -r ahead_count behind_count < <(git rev-list --left-right --count HEAD..."$upstream")
if [[ "$ahead_count" != "0" || "$behind_count" != "0" ]]; then
    sync_error="branch is not synced with $upstream:"
    sync_error+=" ahead=$ahead_count behind=$behind_count; $branch_sync_hint"
    fail_repo_state_preflight "branch_sync_required" "branch_sync_preflight" "$sync_error"
fi

branch_ref="refs/heads/$branch"
upstream_remote="$(git for-each-ref --format='%(upstream:remotename)' "$branch_ref")"
upstream_remote_ref="$(git for-each-ref --format='%(upstream:remoteref)' "$branch_ref")"
if [[ -z "$upstream_remote" ]]; then
    remote_error="could not determine upstream remote for $branch"
    fail_repo_state_preflight "branch_sync_required" "branch_sync_preflight" "$remote_error"
fi
if [[ -z "$upstream_remote_ref" ]]; then
    remote_ref_error="could not determine upstream remote ref for $branch"
    fail_repo_state_preflight \
        "branch_sync_required" \
        "branch_sync_preflight" \
        "$remote_ref_error"
fi
case "$upstream_remote_ref" in
    refs/heads/*) ;;
    *)
        remote_ref_error="upstream for $branch is not a remote branch ref: $upstream_remote_ref"
        fail_repo_state_preflight \
            "branch_sync_required" \
            "branch_sync_preflight" \
            "$remote_ref_error"
        ;;
esac
if ! upstream_remote_refs="$(git ls-remote "$upstream_remote" "$upstream_remote_ref")"; then
    inspect_error="could not inspect upstream remote branch $upstream; $branch_sync_hint"
    fail_repo_state_preflight "branch_sync_required" "branch_sync_preflight" "$inspect_error"
fi
upstream_remote_head="$(
    awk -v ref="$upstream_remote_ref" '$2 == ref { print $1 }' <<<"$upstream_remote_refs" | tail -n 1
)"
if [[ -z "$upstream_remote_head" ]]; then
    missing_remote_error="upstream remote branch is missing: $upstream; $branch_sync_hint"
    fail_repo_state_preflight \
        "branch_sync_required" \
        "branch_sync_preflight" \
        "$missing_remote_error"
fi
if [[ ! "$upstream_remote_head" =~ ^[0-9a-f]{40}$ ]]; then
    malformed_head_error="upstream remote branch head must be a 40-character Git SHA"
    malformed_head_error+=", got $upstream_remote_head"
    fail_repo_state_preflight \
        "branch_sync_required" \
        "branch_sync_preflight" \
        "$malformed_head_error"
fi
if [[ "$upstream_remote_head" != "$head_sha" ]]; then
    remote_branch_error="branch is not synced with remote $upstream:"
    remote_branch_error+=" local HEAD=$head_sha remote HEAD=$upstream_remote_head"
    remote_branch_error+="; $branch_sync_hint"
    fail_repo_state_preflight \
        "branch_sync_required" \
        "branch_sync_preflight" \
        "$remote_branch_error"
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
generated_outputs_state="not_checked"
generated_outputs_json="[]"
generated_outputs_host_triple=""
generated_outputs_error=""
generated_artifacts_state="not_checked"
generated_artifacts_json="[]"
generated_artifacts_host_triple=""
generated_artifacts_error=""
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

append_generated_output() {
    local kind="$1"
    local abs_path="$2"
    local will_write="$3"
    local overwrite_env="$4"
    local exists=false
    local file_type="missing"
    local size_bytes=""
    local sha256=""
    local rel_path="$abs_path"

    if [[ "$rel_path" == "$repo_root/"* ]]; then
        rel_path="${rel_path#$repo_root/}"
    fi
    if [[ -e "$abs_path" || -L "$abs_path" ]]; then
        exists=true
        generated_outputs_state="cleanup_required"
    fi
    if [[ -L "$abs_path" ]]; then
        file_type="symlink"
    elif [[ -f "$abs_path" ]]; then
        file_type="file"
        if ! size_bytes="$(wc -c <"$abs_path" | tr -d '[:space:]')"; then
            size_bytes=""
        fi
        if [[ ! "$size_bytes" =~ ^[0-9]+$ ]]; then
            size_bytes=""
        fi
        if command -v shasum >/dev/null 2>&1; then
            if ! sha256="$(shasum -a 256 "$abs_path" | awk '{ print $1 }')"; then
                sha256=""
            fi
            if [[ ! "$sha256" =~ ^[0-9a-f]{64}$ ]]; then
                sha256=""
            fi
        fi
    elif [[ -d "$abs_path" ]]; then
        file_type="directory"
    elif [[ -e "$abs_path" ]]; then
        file_type="other"
    fi

    generated_outputs_json="$(
        jq -c \
            --arg kind "$kind" \
            --arg path "$rel_path" \
            --arg absolute_path "$abs_path" \
            --arg will_write "$will_write" \
            --arg overwrite_env "$overwrite_env" \
            --arg exists "$exists" \
            --arg file_type "$file_type" \
            --arg size_bytes "$size_bytes" \
            --arg sha256 "$sha256" \
            '. + [{
                kind: $kind,
                path: $path,
                absolute_path: $absolute_path,
                exists: ($exists == "true"),
                will_write: ($will_write == "true"),
                overwrite_env: $overwrite_env,
                file_type: $file_type,
                size_bytes: (
                    if $size_bytes == "" then null
                    else ($size_bytes | tonumber)
                    end
                ),
                sha256: (
                    if $sha256 == "" then null
                    else $sha256
                    end
                )
            }]' <<<"$generated_outputs_json"
    )"
}

collect_generated_outputs() {
    local host_triple
    local host_triple_pattern='^[A-Za-z0-9_.+-]+(-[A-Za-z0-9_.+-]+)+$'
    local archive_name
    local package_will_write=false
    local homebrew_will_write=false

    generated_outputs_state="clear"
    generated_outputs_json="[]"
    generated_outputs_host_triple=""
    generated_outputs_error=""

    if ! command -v rustc >/dev/null 2>&1; then
        generated_outputs_state="unknown"
        generated_outputs_error="required tool is missing for generated-output inventory: rustc"
        return 0
    fi

    host_triple="$(rustc -vV | awk '/^host:/ { print $2 }')"
    if [[ -z "$host_triple" || ! "$host_triple" =~ $host_triple_pattern ]]; then
        generated_outputs_state="unknown"
        generated_outputs_error="could not determine a valid Rust host triple for generated-output inventory"
        return 0
    fi
    generated_outputs_host_triple="$host_triple"
    archive_name="engram-${package_version}-${host_triple}"

    if [[ "$run_package_smoke" == "1" ]]; then
        package_will_write=true
    fi
    append_generated_output "package_archive" \
        "$repo_root/dist/${archive_name}.tar.gz" "$package_will_write" \
        "ALLOW_PACKAGE_ASSET_OVERWRITE"
    append_generated_output "package_checksum" \
        "$repo_root/dist/${archive_name}.tar.gz.sha256" "$package_will_write" \
        "ALLOW_PACKAGE_ASSET_OVERWRITE"

    if [[ "$target" == "ga" ]]; then
        if [[ "$run_homebrew_render" == "1" && "$run_package_smoke" == "1" ]]; then
            homebrew_will_write=true
        fi
        append_generated_output "homebrew_formula" "$homebrew_formula_output" \
            "$homebrew_will_write" "ALLOW_HOMEBREW_FORMULA_OVERWRITE"
    fi
}

append_generated_artifact() {
    local kind="$1"
    local abs_path="$2"
    local required="$3"
    local exists=false
    local file_type="missing"
    local size_bytes=""
    local sha256=""
    local rel_path="$abs_path"
    local artifact_issue=""

    if [[ "$rel_path" == "$repo_root/"* ]]; then
        rel_path="${rel_path#$repo_root/}"
    fi
    if [[ -e "$abs_path" || -L "$abs_path" ]]; then
        exists=true
    fi
    if [[ -L "$abs_path" ]]; then
        file_type="symlink"
    elif [[ -f "$abs_path" ]]; then
        file_type="file"
        if ! size_bytes="$(wc -c <"$abs_path" | tr -d '[:space:]')"; then
            size_bytes=""
        fi
        if [[ ! "$size_bytes" =~ ^[0-9]+$ ]]; then
            size_bytes=""
        fi
        if command -v shasum >/dev/null 2>&1; then
            if ! sha256="$(shasum -a 256 "$abs_path" | awk '{ print $1 }')"; then
                sha256=""
            fi
            if [[ ! "$sha256" =~ ^[0-9a-f]{64}$ ]]; then
                sha256=""
            fi
        fi
    elif [[ -d "$abs_path" ]]; then
        file_type="directory"
    elif [[ -e "$abs_path" ]]; then
        file_type="other"
    fi
    if [[ "$required" == "true" ]]; then
        if [[ "$exists" != "true" ]]; then
            artifact_issue="missing"
        elif [[ "$file_type" != "file" ]]; then
            artifact_issue="file_type=$file_type"
        elif [[ -z "$size_bytes" || "$size_bytes" == "0" ]]; then
            artifact_issue="size_bytes=${size_bytes:-null}"
        elif [[ -z "$sha256" ]]; then
            artifact_issue="sha256=null"
        fi

        if [[ -n "$artifact_issue" ]]; then
            generated_artifacts_state="missing"
            if [[ -n "$generated_artifacts_error" ]]; then
                generated_artifacts_error+=", "
            else
                generated_artifacts_error="required generated release artifacts are missing or invalid after local proof: "
            fi
            generated_artifacts_error+="$rel_path ($artifact_issue)"
        fi
    fi

    generated_artifacts_json="$(
        jq -c \
            --arg kind "$kind" \
            --arg path "$rel_path" \
            --arg absolute_path "$abs_path" \
            --arg required "$required" \
            --arg exists "$exists" \
            --arg file_type "$file_type" \
            --arg size_bytes "$size_bytes" \
            --arg sha256 "$sha256" \
            '. + [{
                kind: $kind,
                path: $path,
                absolute_path: $absolute_path,
                required: ($required == "true"),
                exists: ($exists == "true"),
                file_type: $file_type,
                size_bytes: (
                    if $size_bytes == "" then null
                    else ($size_bytes | tonumber)
                    end
                ),
                sha256: (
                    if $sha256 == "" then null
                    else $sha256
                    end
                )
            }]' <<<"$generated_artifacts_json"
    )"
}

collect_generated_artifacts() {
    local host_triple="$generated_outputs_host_triple"
    local host_triple_pattern='^[A-Za-z0-9_.+-]+(-[A-Za-z0-9_.+-]+)+$'
    local archive_name
    local required_count=0
    local package_required=false
    local homebrew_required=false

    generated_artifacts_state="not_required"
    generated_artifacts_json="[]"
    generated_artifacts_host_triple=""
    generated_artifacts_error=""

    if [[ -z "$host_triple" ]]; then
        if ! command -v rustc >/dev/null 2>&1; then
            generated_artifacts_state="unknown"
            generated_artifacts_error="required tool is missing for artifact proof: rustc"
            return 0
        fi
        host_triple="$(rustc -vV | awk '/^host:/ { print $2 }')"
    fi
    if [[ -z "$host_triple" || ! "$host_triple" =~ $host_triple_pattern ]]; then
        generated_artifacts_state="unknown"
        generated_artifacts_error="could not determine a valid Rust host triple for artifact proof"
        return 0
    fi
    generated_artifacts_host_triple="$host_triple"

    archive_name="engram-${package_version}-${host_triple}"

    if [[ "$run_package_smoke" == "1" ]]; then
        package_required=true
        required_count=$((required_count + 2))
    fi
    if [[ "$target" == "ga" && "$run_homebrew_render" == "1" &&
        "$run_package_smoke" == "1" ]]; then
        homebrew_required=true
        required_count=$((required_count + 1))
    fi

    if (( required_count > 0 )); then
        generated_artifacts_state="present"
    fi

    append_generated_artifact "package_archive" \
        "$repo_root/dist/${archive_name}.tar.gz" "$package_required"
    append_generated_artifact "package_checksum" \
        "$repo_root/dist/${archive_name}.tar.gz.sha256" "$package_required"

    if [[ "$target" == "ga" ]]; then
        append_generated_artifact "homebrew_formula" "$homebrew_formula_output" \
            "$homebrew_required"
    fi

    if [[ "$generated_artifacts_state" == "missing" ]]; then
        generated_artifacts_error+=". Rerun the full release gate before owner review."
    fi
}

check_generated_outputs_for_local_steps() {
    local blocking_outputs

    blocking_outputs="$(
        jq -r '
            [.[] | select(.exists == true and .will_write == true) | .path]
            | join(", ")
        ' <<<"$generated_outputs_json"
    )"
    if [[ -z "$blocking_outputs" ]]; then
        return 0
    fi

    generated_outputs_state="cleanup_required"
    generated_outputs_error="generated release outputs already exist and this gate would write them: "
    generated_outputs_error+="$blocking_outputs"
    generated_outputs_error+=". Remove stale outputs or get cleanup approval before final local proof."
    return 1
}

emit_generated_output_cleanup_verification_json() {
    local verification_state="$1"
    local verification_error="$2"

    jq -n \
        --arg target "$target" \
        --arg package_version "$package_version" \
        --arg release_version "$release_version" \
        --arg release_tag "$release_tag" \
        --arg release_repo "$release_repo" \
        --arg branch "$branch" \
        --arg expected_branch "$expected_branch" \
        --arg upstream "$upstream" \
        --arg upstream_remote "$upstream_remote" \
        --arg upstream_remote_ref "$upstream_remote_ref" \
        --arg upstream_remote_head "$upstream_remote_head" \
        --arg ahead "$ahead_count" \
        --arg behind "$behind_count" \
        --arg head "$head_sha" \
        --arg tracked "$tracked_changes_present" \
        --arg manifest_path "$verify_generated_output_cleanup_manifest" \
        --arg verification_state "$verification_state" \
        --arg verification_error "$verification_error" \
        --arg generated_outputs_state "$generated_outputs_state" \
        --arg generated_outputs_host_triple "$generated_outputs_host_triple" \
        --arg generated_outputs_error "$generated_outputs_error" \
        --argjson generated_outputs "$generated_outputs_json" \
        --argjson expected_outputs "$generated_output_cleanup_expected_json" \
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
                behind: ($behind | tonumber),
                remote: $upstream_remote,
                remote_ref: $upstream_remote_ref,
                remote_head: $upstream_remote_head,
                matches_remote_head: ($head == $upstream_remote_head)
            },
            head: $head,
            tracked_changes_present: ($tracked == "true"),
            release_target: {
                tag: $release_tag,
                repository: $release_repo,
                state: "not_checked",
                local_tag_exists: null,
                remote_git_tag_exists: null,
                github_release_exists: null
            },
            pr: null,
            hosted_ci: {
                state: "not_checked",
                repository: $release_repo,
                run_id: null,
                run: null,
                verifier: null
            },
            local_ci: "not_run",
            package_install_smoke: "not_run",
            disk_space: {
                state: "not_checked",
                free_kib: null,
                min_required_kib: null,
                shortfall_kib: null,
                cleanup_candidates: []
            },
            generated_outputs: {
                state: $generated_outputs_state,
                host_triple: (
                    if $generated_outputs_host_triple == "" then null
                    else $generated_outputs_host_triple
                    end
                ),
                outputs: $generated_outputs,
                error: (
                    if $generated_outputs_error == "" then null
                    else $generated_outputs_error
                    end
                )
            },
            generated_output_cleanup_verification: {
                state: $verification_state,
                manifest_path: $manifest_path,
                expected_outputs: $expected_outputs,
                current_outputs: (
                    $generated_outputs
                    | map({
                        kind,
                        path,
                        absolute_path,
                        exists,
                        will_write,
                        overwrite_env,
                        file_type,
                        size_bytes,
                        sha256
                    })
                    | sort_by(.kind, .path)
                ),
                error: (
                    if $verification_error == "" then null
                    else $verification_error
                    end
                )
            },
            generated_artifacts: {
                state: "not_checked",
                host_triple: null,
                artifacts: [],
                error: null
            },
            homebrew_formula_render: "not_run",
            homebrew_formula: {
                output: null
            },
            release_scope: {
                release_notes_path: null,
                state: "not_checked",
                native_claude_proof_limits_acknowledged: false,
                lifecycle_m6_limits_acknowledged: false
            },
            release_gate_state: (
                if $verification_state == "verified" then
                    "generated_output_cleanup_fingerprints_verified"
                else
                    "generated_output_cleanup_fingerprints_mismatch"
                end
            ),
            ready_for_release_owner_review: false,
            hosted_ci_fallback_decision_required: false,
            remaining_release_actions: (
                if $verification_state == "verified" then
                    [
                        "remove_verified_stale_generated_release_outputs_after_explicit_owner_approval",
                        "rerun_full_release_gate_report_with_local_ci_and_package_smoke"
                    ]
                else
                    [
                        "refresh_generated_output_cleanup_evidence",
                        "get_release_owner_cleanup_approval",
                        "rerun_generated_output_cleanup_fingerprint_verification"
                    ]
                end
            ),
            failure: (
                if $verification_state == "verified" then
                    null
                else
                    {
                        kind: "generated_output_cleanup_verification",
                        message: $verification_error
                    }
                end
            ),
            release_owner_decision_required: true,
            actions_performed: {
                release_actions: false,
                git_tag: false,
                github_release: false,
                package_asset_upload: false,
                homebrew_tap_update: false,
                generated_output_cleanup: false
            },
            release_actions_performed: false
        }'
}

verify_generated_output_cleanup() {
    local manifest_error
    local verification_error
    local current_outputs_normalized
    local expected_outputs_normalized

    if ! jq -e 'type == "object"' "$verify_generated_output_cleanup_manifest" \
        >/dev/null 2>&1; then
        generated_output_cleanup_expected_json="[]"
        verification_error="generated-output cleanup manifest is not valid JSON"
        emit_generated_output_cleanup_verification_json "mismatch" "$verification_error"
        fail "$verification_error"
    fi

    manifest_error="$(
        jq -r \
            --arg target "$target" \
            --arg package_version "$package_version" \
            --arg release_version "$release_version" \
            --arg branch "$branch" \
            --arg head "$head_sha" \
            '
            def sha256_string:
                type == "string" and test("^[0-9a-f]{64}$");
            if (.target // "") != $target then
                "manifest target mismatch: expected \($target), got \(.target // null)"
            elif (.package_version // "") != $package_version then
                "manifest package_version mismatch: expected \($package_version), got \(.package_version // null)"
            elif (.release_version // "") != $release_version then
                "manifest release_version mismatch: expected \($release_version), got \(.release_version // null)"
            elif (.branch // "") != $branch then
                "manifest branch mismatch: expected \($branch), got \(.branch // null)"
            elif (.head // "") != $head then
                "manifest head mismatch: expected \($head), got \(.head // null)"
            elif (.release_gate_state // "") != "generated_outputs_cleanup_required" then
                "manifest release_gate_state must be generated_outputs_cleanup_required"
            elif (.failure.kind // "") != "generated_outputs_preflight" then
                "manifest failure.kind must be generated_outputs_preflight"
            elif (.release_target.state // "") != "available" then
                "manifest release_target.state must be available"
            elif (
                (.release_target.local_tag_exists | type) != "boolean"
                or .release_target.local_tag_exists != false
            ) then
                "manifest release_target.local_tag_exists must be false"
            elif (
                (.release_target.remote_git_tag_exists | type) != "boolean"
                or .release_target.remote_git_tag_exists != false
            ) then
                "manifest release_target.remote_git_tag_exists must be false"
            elif (
                (.release_target.github_release_exists | type) != "boolean"
                or .release_target.github_release_exists != false
            ) then
                "manifest release_target.github_release_exists must be false"
            elif (.disk_space.state // "") != "passed" then
                "manifest disk_space.state must be passed"
            elif ((.disk_space.shortfall_kib | type) != "number" or .disk_space.shortfall_kib != 0) then
                "manifest disk_space.shortfall_kib must be 0"
            elif (.generated_outputs.state // "") != "cleanup_required" then
                "manifest generated_outputs.state must be cleanup_required"
            elif ((.generated_outputs.outputs // null) | type) != "array" then
                "manifest generated_outputs.outputs must be an array"
            elif (.generated_outputs.outputs | length) == 0 then
                "manifest generated_outputs.outputs must not be empty"
            elif any(.generated_outputs.outputs[]; .exists != true) then
                "manifest generated_outputs.outputs must all have exists=true"
            elif any(.generated_outputs.outputs[]; .will_write != true) then
                "manifest generated_outputs.outputs must all have will_write=true"
            elif any(.generated_outputs.outputs[]; .file_type != "file") then
                "manifest generated_outputs.outputs must all be regular files"
            elif any(.generated_outputs.outputs[];
                ((.size_bytes | type) != "number") or .size_bytes <= 0
            ) then
                "manifest generated_outputs.outputs must all have positive size_bytes"
            elif any(.generated_outputs.outputs[]; (.sha256 | sha256_string | not)) then
                "manifest generated_outputs.outputs must all have sha256 fingerprints"
            elif (.generated_artifacts.state // "") != "not_checked" then
                "manifest generated_artifacts.state must be not_checked"
            elif (
                (.ready_for_release_owner_review | type) != "boolean"
                or .ready_for_release_owner_review != false
            ) then
                "manifest ready_for_release_owner_review must be false"
            elif (
                (.release_owner_decision_required | type) != "boolean"
                or .release_owner_decision_required != true
            ) then
                "manifest release_owner_decision_required must be true"
            elif (
                (.hosted_ci_fallback_decision_required | type) != "boolean"
                or .hosted_ci_fallback_decision_required != false
            ) then
                "manifest hosted_ci_fallback_decision_required must be false"
            elif ((.remaining_release_actions // null) | type) != "array" then
                "manifest remaining_release_actions must be an array"
            elif (
                (.remaining_release_actions | sort) != ([
                    "remove_stale_generated_release_outputs_or_get_cleanup_approval",
                    "rerun_full_release_gate_report_with_local_ci_and_package_smoke"
                ] | sort)
            ) then
                "manifest remaining_release_actions must require cleanup and full gate rerun"
            elif (
                (.release_actions_performed | type) != "boolean"
                or .release_actions_performed != false
            ) then
                "manifest release_actions_performed must be false"
            elif ((.actions_performed // null) | type) != "object" then
                "manifest actions_performed must be an object"
            elif (. as $manifest | any([
                "release_actions",
                "git_tag",
                "github_release",
                "package_asset_upload",
                "homebrew_tap_update",
                "generated_output_cleanup"
            ][]; (
                ($manifest.actions_performed[.] | type) != "boolean"
                or $manifest.actions_performed[.] != false
            ))) then
                "manifest actions_performed release-action values must all be false"
            elif any(.actions_performed[]; . != false) then
                "manifest actions_performed values must all be false"
            else
                empty
            end
            ' "$verify_generated_output_cleanup_manifest"
    )"
    if [[ -n "$manifest_error" ]]; then
        generated_output_cleanup_expected_json="[]"
        verification_error="$manifest_error"
        emit_generated_output_cleanup_verification_json "mismatch" "$verification_error"
        fail "$verification_error"
    fi

    expected_outputs_normalized="$(
        jq -c '
            .generated_outputs.outputs
            | map({
                kind,
                path,
                absolute_path,
                exists,
                will_write,
                overwrite_env,
                file_type,
                size_bytes,
                sha256
            })
            | sort_by(.kind, .path)
        ' "$verify_generated_output_cleanup_manifest"
    )"
    current_outputs_normalized="$(
        jq -c '
            map({
                kind,
                path,
                absolute_path,
                exists,
                will_write,
                overwrite_env,
                file_type,
                size_bytes,
                sha256
            })
            | sort_by(.kind, .path)
        ' <<<"$generated_outputs_json"
    )"
    generated_output_cleanup_expected_json="$expected_outputs_normalized"

    if [[ "$current_outputs_normalized" != "$expected_outputs_normalized" ]]; then
        verification_error="current generated outputs do not match the cleanup manifest"
        emit_generated_output_cleanup_verification_json "mismatch" "$verification_error"
        fail "$verification_error"
    fi

    if [[ "$json_output" == "1" ]]; then
        emit_generated_output_cleanup_verification_json "verified" ""
    else
        printf '\nGenerated-output cleanup fingerprints verified:\n'
        printf '  manifest: %s\n' "$verify_generated_output_cleanup_manifest"
        printf '  branch: %s\n' "$branch"
        printf '  head: %s\n' "$head_sha"
        jq -r '
            .[]
            | "  generated_output: \(.path) kind=\(.kind) size_bytes=\(.size_bytes)"
                + " sha256=\(.sha256)"
        ' <<<"$generated_outputs_json"
        printf '  release_gate_state: generated_output_cleanup_fingerprints_verified\n'
        printf '  generated_output_cleanup_performed: false\n'
    fi
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
        --arg upstream_remote "$upstream_remote" \
        --arg upstream_remote_ref "$upstream_remote_ref" \
        --arg upstream_remote_head "$upstream_remote_head" \
        --arg ahead "$ahead_count" \
        --arg behind "$behind_count" \
        --arg head "$head_sha" \
        --arg tracked "$tracked_changes_present" \
        --arg expected_event "$expected_event" \
        --arg hosted_ci_state "$hosted_ci_state" \
        --arg hosted_run_id "$hosted_run_id" \
        --arg min_free_space_kib "$min_free_space_kib" \
        --arg generated_outputs_state "$generated_outputs_state" \
        --arg generated_outputs_host_triple "$generated_outputs_host_triple" \
        --arg generated_outputs_error "$generated_outputs_error" \
        --arg generated_artifacts_state "$generated_artifacts_state" \
        --arg generated_artifacts_host_triple "$generated_artifacts_host_triple" \
        --arg generated_artifacts_error "$generated_artifacts_error" \
        --arg release_notes_path "$release_notes_path" \
        --arg release_scope_state "$release_scope_state" \
        --arg release_scope_native_claude "$release_scope_native_claude_ack" \
        --arg release_scope_lifecycle_m6 "$release_scope_lifecycle_m6_ack" \
        --arg release_gate_state "$failure_gate_state" \
        --arg release_target_error "$release_target_error" \
        --argjson pr "$pr_report_json" \
        --argjson hosted_run "$hosted_run_report_json" \
        --argjson hosted_ci_verifier "$hosted_ci_verifier_json" \
        --argjson generated_outputs "$generated_outputs_json" \
        --argjson generated_artifacts "$generated_artifacts_json" \
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
                behind: ($behind | tonumber),
                remote: $upstream_remote,
                remote_ref: $upstream_remote_ref,
                remote_head: $upstream_remote_head,
                matches_remote_head: ($head == $upstream_remote_head)
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
                repository: $release_repo,
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
            generated_outputs: {
                state: $generated_outputs_state,
                host_triple: (
                    if $generated_outputs_host_triple == "" then null
                    else $generated_outputs_host_triple
                    end
                ),
                outputs: $generated_outputs,
                error: (
                    if $generated_outputs_error == "" then null
                    else $generated_outputs_error
                    end
                )
            },
            generated_artifacts: {
                state: $generated_artifacts_state,
                host_triple: (
                    if $generated_artifacts_host_triple == "" then null
                    else $generated_artifacts_host_triple
                    end
                ),
                artifacts: $generated_artifacts,
                error: (
                    if $generated_artifacts_error == "" then null
                    else $generated_artifacts_error
                    end
                )
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
            actions_performed: {
                release_actions: false,
                git_tag: false,
                github_release: false,
                package_asset_upload: false,
                homebrew_tap_update: false,
                generated_output_cleanup: false
            },
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
        --arg upstream_remote "$upstream_remote" \
        --arg upstream_remote_ref "$upstream_remote_ref" \
        --arg upstream_remote_head "$upstream_remote_head" \
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
        --arg generated_outputs_state "$generated_outputs_state" \
        --arg generated_outputs_host_triple "$generated_outputs_host_triple" \
        --arg generated_outputs_error "$generated_outputs_error" \
        --arg generated_artifacts_state "$generated_artifacts_state" \
        --arg generated_artifacts_host_triple "$generated_artifacts_host_triple" \
        --arg generated_artifacts_error "$generated_artifacts_error" \
        --arg release_notes_path "$release_notes_path" \
        --arg release_scope_state "$release_scope_state" \
        --arg release_scope_native_claude "$release_scope_native_claude_ack" \
        --arg release_scope_lifecycle_m6 "$release_scope_lifecycle_m6_ack" \
        --argjson pr "$pr_report_json" \
        --argjson hosted_run "$hosted_run_report_json" \
        --argjson hosted_ci_verifier "$hosted_ci_verifier_json" \
        --argjson disk_cleanup_candidates "$disk_cleanup_candidates_json" \
        --argjson generated_outputs "$generated_outputs_json" \
        --argjson generated_artifacts "$generated_artifacts_json" \
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
                behind: ($behind | tonumber),
                remote: $upstream_remote,
                remote_ref: $upstream_remote_ref,
                remote_head: $upstream_remote_head,
                matches_remote_head: ($head == $upstream_remote_head)
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
                repository: $release_repo,
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
            generated_outputs: {
                state: $generated_outputs_state,
                host_triple: (
                    if $generated_outputs_host_triple == "" then null
                    else $generated_outputs_host_triple
                    end
                ),
                outputs: $generated_outputs,
                error: (
                    if $generated_outputs_error == "" then null
                    else $generated_outputs_error
                    end
                )
            },
            generated_artifacts: {
                state: $generated_artifacts_state,
                host_triple: (
                    if $generated_artifacts_host_triple == "" then null
                    else $generated_artifacts_host_triple
                    end
                ),
                artifacts: $generated_artifacts,
                error: (
                    if $generated_artifacts_error == "" then null
                    else $generated_artifacts_error
                    end
                )
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
            remaining_release_actions: (
                ["free_local_disk_space_or_get_cleanup_approval"]
                + (
                    if $generated_outputs_state == "cleanup_required" then
                        ["remove_stale_generated_release_outputs_or_get_cleanup_approval"]
                    else
                        []
                    end
                )
                + ["rerun_full_release_gate_report_with_local_ci_and_package_smoke"]
            ),
            failure: {
                kind: "disk_space_preflight",
                message: $disk_space_error
            },
            release_owner_decision_required: true,
            actions_performed: {
                release_actions: false,
                git_tag: false,
                github_release: false,
                package_asset_upload: false,
                homebrew_tap_update: false,
                generated_output_cleanup: false
            },
            release_actions_performed: false
        }'
}

emit_generated_outputs_failure_json() {
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
        --arg upstream_remote "$upstream_remote" \
        --arg upstream_remote_ref "$upstream_remote_ref" \
        --arg upstream_remote_head "$upstream_remote_head" \
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
        --arg generated_outputs_state "$generated_outputs_state" \
        --arg generated_outputs_host_triple "$generated_outputs_host_triple" \
        --arg generated_outputs_error "$generated_outputs_error" \
        --arg generated_artifacts_state "$generated_artifacts_state" \
        --arg generated_artifacts_host_triple "$generated_artifacts_host_triple" \
        --arg generated_artifacts_error "$generated_artifacts_error" \
        --arg release_notes_path "$release_notes_path" \
        --arg release_scope_state "$release_scope_state" \
        --arg release_scope_native_claude "$release_scope_native_claude_ack" \
        --arg release_scope_lifecycle_m6 "$release_scope_lifecycle_m6_ack" \
        --argjson pr "$pr_report_json" \
        --argjson hosted_run "$hosted_run_report_json" \
        --argjson hosted_ci_verifier "$hosted_ci_verifier_json" \
        --argjson disk_cleanup_candidates "$disk_cleanup_candidates_json" \
        --argjson generated_outputs "$generated_outputs_json" \
        --argjson generated_artifacts "$generated_artifacts_json" \
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
                behind: ($behind | tonumber),
                remote: $upstream_remote,
                remote_ref: $upstream_remote_ref,
                remote_head: $upstream_remote_head,
                matches_remote_head: ($head == $upstream_remote_head)
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
                repository: $release_repo,
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
                shortfall_kib: (
                    if $disk_space_shortfall_kib == "" then null
                    else ($disk_space_shortfall_kib | tonumber)
                    end
                ),
                cleanup_candidates: $disk_cleanup_candidates
            },
            generated_outputs: {
                state: $generated_outputs_state,
                host_triple: (
                    if $generated_outputs_host_triple == "" then null
                    else $generated_outputs_host_triple
                    end
                ),
                outputs: $generated_outputs,
                error: $generated_outputs_error
            },
            generated_artifacts: {
                state: $generated_artifacts_state,
                host_triple: (
                    if $generated_artifacts_host_triple == "" then null
                    else $generated_artifacts_host_triple
                    end
                ),
                artifacts: $generated_artifacts,
                error: (
                    if $generated_artifacts_error == "" then null
                    else $generated_artifacts_error
                    end
                )
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
            release_gate_state: "generated_outputs_cleanup_required",
            ready_for_release_owner_review: false,
            hosted_ci_fallback_decision_required: false,
            remaining_release_actions: [
                "remove_stale_generated_release_outputs_or_get_cleanup_approval",
                "rerun_full_release_gate_report_with_local_ci_and_package_smoke"
            ],
            failure: {
                kind: "generated_outputs_preflight",
                message: $generated_outputs_error
            },
            release_owner_decision_required: true,
            actions_performed: {
                release_actions: false,
                git_tag: false,
                github_release: false,
                package_asset_upload: false,
                homebrew_tap_update: false,
                generated_output_cleanup: false
            },
            release_actions_performed: false
        }'
}

emit_hosted_ci_failure_json() {
    local release_gate_state="$1"
    local failure_message="$2"
    local expected_jobs_json
    local checks_json

    expected_jobs_json="$(
        printf '%s\n' "${expected_jobs[@]}" |
            jq -R -s 'split("\n") | map(select(length > 0))'
    )"
    if [[ -s "$checks_file" ]]; then
        checks_json="$(
            jq -R -s '
                split("\n")
                | map(select(length > 0) | split("\t") | {
                    name: .[0],
                    status: .[1],
                    conclusion: (.[2] // "")
                })
            ' "$checks_file"
        )"
    else
        checks_json="[]"
    fi

    jq -n \
        --arg target "$target" \
        --arg package_version "$package_version" \
        --arg release_version "$release_version" \
        --arg release_tag "$release_tag" \
        --arg release_repo "$release_repo" \
        --arg branch "$branch" \
        --arg expected_branch "$expected_branch" \
        --arg upstream "$upstream" \
        --arg upstream_remote "$upstream_remote" \
        --arg upstream_remote_ref "$upstream_remote_ref" \
        --arg upstream_remote_head "$upstream_remote_head" \
        --arg ahead "$ahead_count" \
        --arg behind "$behind_count" \
        --arg head "$head_sha" \
        --arg tracked "$tracked_changes_present" \
        --arg expected_workflow "$expected_workflow" \
        --arg expected_event "$expected_event" \
        --arg hosted_run_id "$hosted_run_id" \
        --arg min_free_space_kib "$min_free_space_kib" \
        --arg release_notes_path "$release_notes_path" \
        --arg release_scope_state "$release_scope_state" \
        --arg release_scope_native_claude "$release_scope_native_claude_ack" \
        --arg release_scope_lifecycle_m6 "$release_scope_lifecycle_m6_ack" \
        --arg release_gate_state "$release_gate_state" \
        --arg failure_message "$failure_message" \
        --argjson hosted_run "$hosted_run_report_json" \
        --argjson hosted_ci_verifier "$hosted_ci_verifier_json" \
        --argjson expected_jobs "$expected_jobs_json" \
        --argjson checks "$checks_json" \
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
                behind: ($behind | tonumber),
                remote: $upstream_remote,
                remote_ref: $upstream_remote_ref,
                remote_head: $upstream_remote_head,
                matches_remote_head: ($head == $upstream_remote_head)
            },
            head: $head,
            tracked_changes_present: ($tracked == "true"),
            release_target: {
                tag: $release_tag,
                repository: $release_repo,
                state: "not_checked",
                local_tag_exists: null,
                remote_git_tag_exists: null,
                github_release_exists: null
            },
            pr: null,
            hosted_ci: {
                state: $release_gate_state,
                repository: $release_repo,
                expected_workflow: $expected_workflow,
                expected_event: $expected_event,
                expected_jobs: $expected_jobs,
                run_id: (if $hosted_run_id == "" then null else ($hosted_run_id | tonumber) end),
                run: $hosted_run,
                checks: $checks,
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
            generated_outputs: {
                state: "not_checked",
                host_triple: null,
                outputs: [],
                error: null
            },
            generated_artifacts: {
                state: "not_checked",
                host_triple: null,
                artifacts: [],
                error: null
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
                if $release_gate_state == "hosted_ci_head_mismatch" then
                    [
                        "select_hosted_ci_run_for_current_head",
                        "rerun_exact_head_hosted_ci",
                        "rerun_ga_release_gate_report"
                    ]
                elif $release_gate_state == "hosted_ci_jobs_mismatch" then
                    [
                        "restore_expected_release_ci_jobs_or_update_gate_with_approval",
                        "rerun_exact_head_hosted_ci",
                        "rerun_ga_release_gate_report"
                    ]
                else
                    [
                        "rerun_exact_head_hosted_ci",
                        "rerun_ga_release_gate_report"
                    ]
                end
            ),
            failure: {
                kind: "hosted_ci_preflight",
                message: $failure_message
            },
            release_owner_decision_required: true,
            actions_performed: {
                release_actions: false,
                git_tag: false,
                github_release: false,
                package_asset_upload: false,
                homebrew_tap_update: false,
                generated_output_cleanup: false
            },
            release_actions_performed: false
        }'
}

fail_hosted_ci_preflight() {
    local release_gate_state="$1"
    local failure_message="$2"

    if [[ "$json_output" == "1" ]]; then
        emit_hosted_ci_failure_json "$release_gate_state" "$failure_message"
    fi
    fail "$failure_message"
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

run_generated_outputs_preflight() {
    if [[ "$json_output" == "1" ]]; then
        printf '\n==> generated outputs preflight\n' >&2
    else
        printf '\n==> generated outputs preflight\n'
    fi

    if ! check_generated_outputs_for_local_steps; then
        if [[ "$json_output" == "1" ]]; then
            emit_generated_outputs_failure_json
        fi
        fail "$generated_outputs_error"
    fi
}

collect_hosted_run_checks() {
    if [[ -z "$hosted_run_id" ]]; then
        if ! hosted_run_id="$(
            gh run list \
                --repo "$release_repo" \
                --workflow "$expected_workflow" \
                --branch "$branch" \
                --event "$expected_event" \
                --limit 1 \
                --json databaseId \
                --jq '.[0].databaseId // empty'
        )"; then
            hosted_discovery_error="could not discover hosted CI run for $branch"
            hosted_discovery_error+=" in $release_repo"
            fail_hosted_ci_preflight "hosted_ci_discovery_failed" "$hosted_discovery_error"
        fi
    fi

    [[ -n "$hosted_run_id" ]] ||
        fail_hosted_ci_preflight \
            "hosted_ci_run_missing" \
            "no hosted CI run id was provided or discovered"

    if ! gh run view "$hosted_run_id" \
        --repo "$release_repo" \
        --json databaseId,headSha,status,conclusion,workflowName,event,jobs,url \
        >"$hosted_run_json"; then
        hosted_inspection_error="could not inspect hosted CI run $hosted_run_id"
        hosted_inspection_error+=" in $release_repo"
        fail_hosted_ci_preflight "hosted_ci_inspection_failed" "$hosted_inspection_error"
    fi
    hosted_run_report_json="$(jq -c '.' "$hosted_run_json")"

    actual_run_id="$(jq -r '.databaseId // empty' "$hosted_run_json")"
    actual_head="$(jq -r '.headSha // empty' "$hosted_run_json")"
    actual_status="$(jq -r '.status // empty' "$hosted_run_json")"
    actual_conclusion="$(jq -r '.conclusion // empty' "$hosted_run_json")"
    actual_workflow="$(jq -r '.workflowName // empty' "$hosted_run_json")"
    actual_event="$(jq -r '.event // empty' "$hosted_run_json")"

    [[ "$actual_run_id" == "$hosted_run_id" ]] ||
        fail_hosted_ci_preflight \
            "hosted_ci_run_mismatch" \
            "hosted run id mismatch: expected $hosted_run_id, got $actual_run_id"
    [[ "$actual_head" == "$head_sha" ]] ||
        fail_hosted_ci_preflight \
            "hosted_ci_head_mismatch" \
            "hosted run head mismatch: expected $head_sha, got $actual_head"
    [[ "$actual_status" == "completed" ]] ||
        fail_hosted_ci_preflight \
            "hosted_ci_not_completed" \
            "hosted run is not completed: $actual_status"
    [[ "$actual_conclusion" == "success" ]] ||
        fail_hosted_ci_preflight \
            "hosted_ci_not_successful" \
            "hosted run conclusion is not success: $actual_conclusion"
    [[ "$actual_workflow" == "$expected_workflow" ]] ||
        fail_hosted_ci_preflight \
            "hosted_ci_workflow_mismatch" \
            "workflow mismatch: expected $expected_workflow, got $actual_workflow"
    [[ "$actual_event" == "$expected_event" ]] ||
        fail_hosted_ci_preflight \
            "hosted_ci_event_mismatch" \
            "hosted run event mismatch: expected $expected_event, got $actual_event"

    jq -r '.jobs[] | [.name, .status, (.conclusion // "")] | @tsv' \
        "$hosted_run_json" >"$checks_file"

    expected_jobs_sorted="$(printf '%s\n' "${expected_jobs[@]}" | LC_ALL=C sort)"
    actual_jobs_sorted="$(awk -F '\t' '{ print $1 }' "$checks_file" | LC_ALL=C sort)"
    if [[ "$actual_jobs_sorted" != "$expected_jobs_sorted" ]]; then
        hosted_jobs_error="hosted CI jobs did not match expected release gate jobs"
        hosted_jobs_error+="; expected: ${expected_jobs_sorted//$'\n'/, }"
        hosted_jobs_error+="; actual: ${actual_jobs_sorted//$'\n'/, }"
        fail_hosted_ci_preflight "hosted_ci_jobs_mismatch" "$hosted_jobs_error"
    fi

    awk -F '\t' 'BEGIN { ok = 1 } $2 != "completed" || $3 != "success" { ok = 0 } END { exit ok ? 0 : 1 }' \
        "$checks_file" ||
        fail_hosted_ci_preflight \
            "hosted_ci_jobs_not_successful" \
            "hosted CI jobs are not all completed successfully"

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

    if [[ -n "$verify_generated_output_cleanup_manifest" ]]; then
        hosted_ci_state="not_checked"
    else
        collect_hosted_run_checks
    fi
else
    gh pr view "$pr_number" \
        --repo "$release_repo" \
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
                ALLOW_RELEASE_REPOSITORY_OVERRIDE="$allow_release_repo_override" \
                "$repo_root/scripts/verify-hosted-ci-prestep-blocker.sh" \
                --repo "$release_repo" --event "$expected_event" --json \
                "${verify_args[@]}" >"$hosted_verifier_file"
            jq -e '.condition_verified == true' "$hosted_verifier_file" >/dev/null ||
                fail "hosted CI pre-step verifier did not emit verified JSON"
            hosted_ci_verifier_json="$(jq -c '.' "$hosted_verifier_file")"
            if [[ -z "$hosted_run_id" ]]; then
                hosted_run_id="$(jq -r '.run.id // empty' "$hosted_verifier_file")"
            fi
        else
            run_step "verify hosted CI pre-step fallback" env EXPECTED_HEAD_SHA="$head_sha" \
                ALLOW_RELEASE_REPOSITORY_OVERRIDE="$allow_release_repo_override" \
                "$repo_root/scripts/verify-hosted-ci-prestep-blocker.sh" \
                --repo "$release_repo" --event "$expected_event" "${verify_args[@]}"
        fi
        hosted_ci_state="pre_step_blocker_verified"
    fi
fi

collect_generated_outputs
if [[ -n "$verify_generated_output_cleanup_manifest" ]]; then
    verify_generated_output_cleanup
    exit 0
fi
run_release_target_preflight
run_disk_space_preflight
run_generated_outputs_preflight

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

collect_generated_artifacts

local_ci_state="$([[ "$run_local_ci" == "1" ]] && printf 'passed' || printf 'skipped')"
package_smoke_state="$([[ "$run_package_smoke" == "1" ]] && printf 'passed' || printf 'skipped')"

release_gate_state="evidence_incomplete"
ready_for_release_owner_review=false
hosted_ci_fallback_decision_required=false

if [[ "$target" == "ga" && "$package_version" != "$release_version" ]]; then
    release_gate_state="version_bump_required"
elif [[ "$target" == "ga" && "$release_scope_state" != "complete" ]]; then
    release_gate_state="release_scope_acknowledgement_required"
elif [[ "$generated_artifacts_state" == "missing" ]]; then
    release_gate_state="generated_artifacts_missing"
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
        --arg generated_outputs_state "$generated_outputs_state" \
        'if $target == "ga" and $package_version != $release_version then
            [
                "bump_workspace_version_to_\($release_version)",
                "rerun_exact_head_hosted_ci",
                "run_full_ga_release_gate_report_with_local_ci_and_package_smoke"
            ]
        elif $target == "ga" and $state == "evidence_incomplete"
            and $generated_outputs_state == "cleanup_required" then
            [
                "remove_stale_generated_release_outputs_or_get_cleanup_approval",
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
        elif $state == "generated_artifacts_missing" then
            [
                "rerun_full_\($target)_release_gate_report_with_local_ci_and_package_smoke"
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
        --arg upstream_remote "$upstream_remote" \
        --arg upstream_remote_ref "$upstream_remote_ref" \
        --arg upstream_remote_head "$upstream_remote_head" \
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
        --arg generated_outputs_state "$generated_outputs_state" \
        --arg generated_outputs_host_triple "$generated_outputs_host_triple" \
        --arg generated_outputs_error "$generated_outputs_error" \
        --arg generated_artifacts_state "$generated_artifacts_state" \
        --arg generated_artifacts_host_triple "$generated_artifacts_host_triple" \
        --arg generated_artifacts_error "$generated_artifacts_error" \
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
        --argjson generated_outputs "$generated_outputs_json" \
        --argjson generated_artifacts "$generated_artifacts_json" \
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
                behind: ($behind | tonumber),
                remote: $upstream_remote,
                remote_ref: $upstream_remote_ref,
                remote_head: $upstream_remote_head,
                matches_remote_head: ($head == $upstream_remote_head)
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
                repository: $release_repo,
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
            generated_outputs: {
                state: $generated_outputs_state,
                host_triple: (
                    if $generated_outputs_host_triple == "" then null
                    else $generated_outputs_host_triple
                    end
                ),
                outputs: $generated_outputs,
                error: (
                    if $generated_outputs_error == "" then null
                    else $generated_outputs_error
                    end
                )
            },
            generated_artifacts: {
                state: $generated_artifacts_state,
                host_triple: (
                    if $generated_artifacts_host_triple == "" then null
                    else $generated_artifacts_host_triple
                    end
                ),
                artifacts: $generated_artifacts,
                error: (
                    if $generated_artifacts_error == "" then null
                    else $generated_artifacts_error
                    end
                )
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
            actions_performed: {
                release_actions: false,
                git_tag: false,
                github_release: false,
                package_asset_upload: false,
                homebrew_tap_update: false,
                generated_output_cleanup: false
            },
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
    printf '  upstream_remote: %s %s head=%s matches_head=true\n' \
        "$upstream_remote" "$upstream_remote_ref" "$upstream_remote_head"
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
    printf '  generated_outputs_state: %s\n' "$generated_outputs_state"
    if [[ -n "$generated_outputs_host_triple" ]]; then
        printf '  generated_outputs_host_triple: %s\n' "$generated_outputs_host_triple"
    fi
    if [[ -n "$generated_outputs_error" ]]; then
        printf '  generated_outputs_error: %s\n' "$generated_outputs_error"
    fi
    jq -r '
        .[]
        | "  generated_output: \(.path) kind=\(.kind) exists=\(.exists)"
            + " will_write=\(.will_write) overwrite_env=\(.overwrite_env)"
            + " file_type=\(.file_type)"
            + " size_bytes=\((.size_bytes // "null") | tostring)"
            + " sha256=\(.sha256 // "null")"
    ' <<<"$generated_outputs_json"
    printf '  generated_artifacts_state: %s\n' "$generated_artifacts_state"
    if [[ -n "$generated_artifacts_error" ]]; then
        printf '  generated_artifacts_error: %s\n' "$generated_artifacts_error"
    fi
    if [[ -n "$generated_artifacts_host_triple" ]]; then
        printf '  generated_artifacts_host_triple: %s\n' "$generated_artifacts_host_triple"
    fi
    jq -r '
        .[]
        | "  generated_artifact: \(.path) kind=\(.kind) required=\(.required)"
            + " exists=\(.exists)"
            + " file_type=\(.file_type)"
            + " size_bytes=\((.size_bytes // "null") | tostring)"
            + " sha256=\(.sha256 // "null")"
    ' <<<"$generated_artifacts_json"
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
