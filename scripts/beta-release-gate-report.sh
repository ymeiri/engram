#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repo_root"

pr_number="${PR_NUMBER:-3}"
hosted_run_id="${HOSTED_RUN_ID:-}"
run_local_ci=1
run_package_smoke=1
allow_tracked_changes=0

usage() {
    cat <<'USAGE'
Usage: scripts/beta-release-gate-report.sh [options]

Collect release-owner evidence for the scoped local/Codex beta gate.

Options:
  --pr <number>                 GitHub PR number to inspect (default: PR_NUMBER or 3)
  --hosted-run <id>             Hosted CI run ID to use for pre-step-blocker verification
  --quick                       Skip local CI and package/install smoke
  --skip-local-ci               Skip ./scripts/local-ci.sh
  --skip-package-smoke          Skip ./scripts/package-install-smoke.sh
  --allow-tracked-changes       Allow tracked working-tree/index changes during development
  -h, --help                    Show this help

This script is evidence only. It does not accept the hosted-CI fallback, mark a
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
    printf '\n==> %s\n' "$name"
    "$@"
}

while [[ $# -gt 0 ]]; do
    case "$1" in
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
        --quick)
            run_local_ci=0
            run_package_smoke=0
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
        --allow-tracked-changes)
            allow_tracked_changes=1
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
require_tool gh
require_tool jq

pr_json="$(mktemp "${TMPDIR:-/tmp}/engram-beta-pr.XXXXXX")"
checks_file="$(mktemp "${TMPDIR:-/tmp}/engram-beta-checks.XXXXXX")"
cleanup() {
    rm -f "$pr_json" "$checks_file"
}
trap cleanup EXIT

branch="$(git branch --show-current)"
[[ -n "$branch" ]] || fail "could not determine current branch"
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

expected_jobs_sorted="$(printf '%s\n' Check Test Format Clippy Docs | LC_ALL=C sort)"
actual_jobs_sorted="$(awk -F '\t' '{ print $1 }' "$checks_file" | LC_ALL=C sort)"
hosted_ci_state="unknown"

if [[ "$actual_jobs_sorted" == "$expected_jobs_sorted" ]] &&
    awk -F '\t' 'BEGIN { ok = 1 } $2 != "COMPLETED" || $3 != "SUCCESS" { ok = 0 } END { exit ok ? 0 : 1 }' \
        "$checks_file"; then
    hosted_ci_state="passing"
else
    verify_args=()
    if [[ -n "$hosted_run_id" ]]; then
        verify_args+=("$hosted_run_id")
    fi

    run_step "verify hosted CI pre-step fallback" env EXPECTED_HEAD_SHA="$head_sha" \
        "$repo_root/scripts/verify-hosted-ci-prestep-blocker.sh" "${verify_args[@]}"
    hosted_ci_state="pre_step_blocker_verified"
fi

if [[ "$run_local_ci" == "1" ]]; then
    run_step "local CI-equivalent validation" "$repo_root/scripts/local-ci.sh"
else
    printf '\n==> local CI-equivalent validation\nskipped\n'
fi

if [[ "$run_package_smoke" == "1" ]]; then
    run_step "package/install smoke validation" "$repo_root/scripts/package-install-smoke.sh"
else
    printf '\n==> package/install smoke validation\nskipped\n'
fi

printf '\nBeta release gate evidence collected:\n'
printf '  branch: %s\n' "$branch"
printf '  upstream: %s (ahead=%s behind=%s)\n' "$upstream" "$ahead_count" "$behind_count"
printf '  head: %s\n' "$head_sha"
printf '  tracked_changes_present: %s\n' "$tracked_changes_present"
printf '  pr: #%s %s\n' "$pr_number" "$pr_url"
printf '  pr_draft: %s\n' "$pr_draft"
printf '  pr_merge_state: %s\n' "$pr_merge_state"
printf '  hosted_ci_state: %s\n' "$hosted_ci_state"
printf '  local_ci: %s\n' "$([[ "$run_local_ci" == "1" ]] && printf 'passed' || printf 'skipped')"
printf '  package_install_smoke: %s\n' \
    "$([[ "$run_package_smoke" == "1" ]] && printf 'passed' || printf 'skipped')"
printf '  release_owner_decision_required: true\n'
printf '  release_actions_performed: false\n'
