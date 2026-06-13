#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repo_root"

default_expected_workflow="CI"
expected_workflow="${EXPECTED_WORKFLOW_NAME-$default_expected_workflow}"
allow_expected_workflow_override="${ALLOW_EXPECTED_WORKFLOW_NAME_OVERRIDE:-0}"
default_expected_event="pull_request"
expected_event="$default_expected_event"
if [[ "${EXPECTED_EVENT+x}" == "x" ]]; then
    expected_event="$EXPECTED_EVENT"
fi
allow_expected_event_override="${ALLOW_EXPECTED_EVENT_OVERRIDE:-0}"
expected_head="${EXPECTED_HEAD_SHA:-$(git rev-parse HEAD)}"
expected_jobs=(Check Test Format Clippy Docs)
run_id="${GITHUB_RUN_ID:-}"
json_output=0
run_json="$(mktemp "${TMPDIR:-/tmp}/engram-hosted-ci-run.XXXXXX")"

cleanup() {
    rm -f "$run_json"
}
trap cleanup EXIT

fail() {
    printf 'error: %s\n' "$*" >&2
    exit 1
}

require_tool() {
    local tool="$1"
    command -v "$tool" >/dev/null 2>&1 || fail "required tool is missing: $tool"
}

usage() {
    cat <<'USAGE'
Usage: scripts/verify-hosted-ci-prestep-blocker.sh [options] [run-id]

Verify that a hosted CI run failed before workflow steps ran.

Options:
  --event <event>  Expected GitHub Actions event (default: EXPECTED_EVENT or pull_request)
  --json        Emit machine-readable JSON instead of text on success
  -h, --help    Show this help

Environment overrides:
  EXPECTED_HEAD_SHA       Expected run head (default: git rev-parse HEAD)
  EXPECTED_WORKFLOW_NAME  Expected workflow name (default: CI)
  ALLOW_EXPECTED_WORKFLOW_NAME_OVERRIDE
                          Allow non-CI workflow names for explicit local rehearsals
  EXPECTED_EVENT          Expected run event (default: pull_request)
  ALLOW_EXPECTED_EVENT_OVERRIDE
                          Allow non-pull_request events for explicit local rehearsals
  GITHUB_RUN_ID           Run ID when no positional run-id is supplied

This script is evidence only. It does not accept a hosted-CI fallback,
mark a PR ready, merge, tag, publish, or perform release actions.
USAGE
}

while [[ $# -gt 0 ]]; do
    case "$1" in
        --event)
            [[ $# -ge 2 ]] || fail "--event requires an event name"
            expected_event="$2"
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
        -*)
            fail "unknown option: $1"
            ;;
        *)
            [[ -z "$run_id" ]] || fail "run id provided more than once"
            run_id="$1"
            shift
            ;;
    esac
done

if [[ -n "$run_id" && ! "$run_id" =~ ^[0-9]+$ ]]; then
    fail "GITHUB_RUN_ID/positional run id must be a numeric GitHub Actions run id, got $run_id"
fi
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
event_name_pattern='^[A-Za-z0-9_]+$'
case "$allow_expected_event_override" in
    0 | 1) ;;
    *) fail "ALLOW_EXPECTED_EVENT_OVERRIDE must be 0 or 1, got $allow_expected_event_override" ;;
esac
if [[ -z "$expected_event" ]]; then
    fail "EXPECTED_EVENT/--event must not be empty"
fi
if [[ ! "$expected_event" =~ $event_name_pattern ]]; then
    expected_event_error="EXPECTED_EVENT/--event must be a GitHub event name token"
    expected_event_error+=", got $expected_event"
    fail "$expected_event_error"
fi
if [[ "$expected_event" != "$default_expected_event" &&
    "$allow_expected_event_override" != "1" ]]; then
    printf 'error: EXPECTED_EVENT override requires explicit approval\n' >&2
    printf 'expected default: %s\n' "$default_expected_event" >&2
    printf 'got: %s\n' "$expected_event" >&2
    printf 'hint: set ALLOW_EXPECTED_EVENT_OVERRIDE=1 only for local rehearsals\n' >&2
    exit 1
fi
if [[ ! "$expected_head" =~ ^[0-9a-f]{40}$ ]]; then
    fail "EXPECTED_HEAD_SHA must be a 40-character Git SHA, got $expected_head"
fi

require_tool gh
require_tool jq

if [[ -z "$run_id" ]]; then
    current_branch="$(git branch --show-current)"
    [[ -n "$current_branch" ]] || fail "could not determine current branch"

    run_id="$(
        gh run list \
            --workflow "$expected_workflow" \
            --branch "$current_branch" \
            --event "$expected_event" \
            --limit 1 \
            --json databaseId \
            --jq '.[0].databaseId // empty'
    )"
fi

[[ -n "$run_id" ]] || fail "no hosted CI run id was provided or discovered"
if [[ ! "$run_id" =~ ^[0-9]+$ ]]; then
    fail "hosted run id must be a numeric GitHub Actions run id, got $run_id"
fi

gh run view "$run_id" \
    --json databaseId,headSha,status,conclusion,workflowName,event,jobs,url >"$run_json"

actual_run_id="$(jq -r '.databaseId // empty' "$run_json")"
actual_head="$(jq -r '.headSha // empty' "$run_json")"
actual_status="$(jq -r '.status // empty' "$run_json")"
actual_conclusion="$(jq -r '.conclusion // empty' "$run_json")"
actual_workflow="$(jq -r '.workflowName // empty' "$run_json")"
actual_event="$(jq -r '.event // empty' "$run_json")"
actual_url="$(jq -r '.url // empty' "$run_json")"

[[ "$actual_run_id" == "$run_id" ]] || fail "run id mismatch: expected $run_id, got $actual_run_id"
[[ "$actual_head" == "$expected_head" ]] || fail "run head mismatch: expected $expected_head, got $actual_head"
[[ "$actual_status" == "completed" ]] || fail "run is not completed: $actual_status"
[[ "$actual_conclusion" == "failure" ]] || fail "run conclusion is not failure: $actual_conclusion"
[[ "$actual_workflow" == "$expected_workflow" ]] ||
    fail "workflow mismatch: expected $expected_workflow, got $actual_workflow"
[[ "$actual_event" == "$expected_event" ]] ||
    fail "run event mismatch: expected $expected_event, got $actual_event"

actual_jobs="$(jq -r '.jobs[].name' "$run_json" | LC_ALL=C sort)"
expected_jobs_sorted="$(printf '%s\n' "${expected_jobs[@]}" | LC_ALL=C sort)"

if [[ "$actual_jobs" != "$expected_jobs_sorted" ]]; then
    {
        printf 'error: hosted CI jobs did not match expected release gate jobs\n'
        printf 'expected:\n%s\n' "$expected_jobs_sorted"
        printf 'actual:\n%s\n' "$actual_jobs"
    } >&2
    exit 1
fi

for job_name in "${expected_jobs[@]}"; do
    job_status="$(
        jq -r --arg name "$job_name" '.jobs[] | select(.name == $name) | .status' "$run_json"
    )"
    job_conclusion="$(
        jq -r --arg name "$job_name" '.jobs[] | select(.name == $name) | .conclusion' "$run_json"
    )"
    step_count="$(
        jq -r --arg name "$job_name" '.jobs[] | select(.name == $name) | (.steps | length)' "$run_json"
    )"

    [[ "$job_status" == "completed" ]] ||
        fail "$job_name job is not completed: $job_status"
    [[ "$job_conclusion" == "failure" ]] ||
        fail "$job_name job conclusion is not failure: $job_conclusion"
    [[ "$step_count" == "0" ]] ||
        fail "$job_name job has workflow steps, so this is not a pre-step blocker: $step_count"
done

if [[ "$json_output" == "1" ]]; then
    expected_jobs_json="$(printf '%s\n' "${expected_jobs[@]}" | jq -R -s 'split("\n") | map(select(length > 0))')"
    jobs_json="$(
        jq '[.jobs[] | {
            name,
            status,
            conclusion,
            step_count: (.steps | length)
        }]' "$run_json"
    )"

    jq -n \
        --arg run_id "$actual_run_id" \
        --arg url "$actual_url" \
        --arg expected_head "$expected_head" \
        --arg head "$actual_head" \
        --arg status "$actual_status" \
        --arg conclusion "$actual_conclusion" \
        --arg expected_workflow "$expected_workflow" \
        --arg workflow "$actual_workflow" \
        --arg expected_event "$expected_event" \
        --arg event "$actual_event" \
        --argjson expected_jobs "$expected_jobs_json" \
        --argjson jobs "$jobs_json" \
        '{
            condition_verified: true,
            run: {
                id: ($run_id | tonumber),
                url: $url,
                expected_head: $expected_head,
                head: $head,
                status: $status,
                conclusion: $conclusion,
                expected_workflow: $expected_workflow,
                workflow: $workflow,
                expected_event: $expected_event,
                event: $event
            },
            expected_jobs: $expected_jobs,
            jobs: $jobs,
            condition: "all expected jobs completed with conclusion=failure and steps=[]",
            hosted_ci_fallback_accepted: false,
            release_actions_performed: false
        }'
else
    printf 'Hosted CI pre-step blocker verified:\n'
    printf '  run: %s\n' "$actual_run_id"
    printf '  url: %s\n' "$actual_url"
    printf '  head: %s\n' "$actual_head"
    printf '  workflow: %s\n' "$actual_workflow"
    printf '  event: %s\n' "$actual_event"
    printf '  jobs: %s\n' "${expected_jobs[*]}"
    printf '  condition: all expected jobs completed with conclusion=failure and steps=[]\n'
fi
