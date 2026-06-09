#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repo_root"

expected_workflow="${EXPECTED_WORKFLOW_NAME:-CI}"
expected_head="${EXPECTED_HEAD_SHA:-$(git rev-parse HEAD)}"
expected_jobs=(Check Test Format Clippy Docs)
run_id="${1:-${GITHUB_RUN_ID:-}}"
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

require_tool gh
require_tool jq

if [[ -z "$run_id" ]]; then
    current_branch="$(git branch --show-current)"
    [[ -n "$current_branch" ]] || fail "could not determine current branch"

    run_id="$(
        gh run list \
            --workflow "$expected_workflow" \
            --branch "$current_branch" \
            --event pull_request \
            --limit 1 \
            --json databaseId \
            --jq '.[0].databaseId // empty'
    )"
fi

[[ -n "$run_id" ]] || fail "no hosted CI run id was provided or discovered"

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
[[ "$actual_event" == "pull_request" ]] || fail "run event is not pull_request: $actual_event"

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

printf 'Hosted CI pre-step blocker verified:\n'
printf '  run: %s\n' "$actual_run_id"
printf '  url: %s\n' "$actual_url"
printf '  head: %s\n' "$actual_head"
printf '  workflow: %s\n' "$actual_workflow"
printf '  jobs: %s\n' "${expected_jobs[*]}"
printf '  condition: all expected jobs completed with conclusion=failure and steps=[]\n'
