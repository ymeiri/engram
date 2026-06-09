#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repo_root"

pr_number="${PR_NUMBER:-3}"
hosted_run_id="${HOSTED_RUN_ID:-}"
run_local_ci=1
run_package_smoke=1
allow_tracked_changes=0
json_output=0

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
  --json                        Emit final evidence as machine-readable JSON
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
require_tool gh
require_tool jq

pr_json="$(mktemp "${TMPDIR:-/tmp}/engram-beta-pr.XXXXXX")"
checks_file="$(mktemp "${TMPDIR:-/tmp}/engram-beta-checks.XXXXXX")"
hosted_verifier_file="$(mktemp "${TMPDIR:-/tmp}/engram-beta-hosted-verifier.XXXXXX")"
cleanup() {
    rm -f "$pr_json" "$checks_file" "$hosted_verifier_file"
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
hosted_ci_verifier_json="null"

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
            "$repo_root/scripts/verify-hosted-ci-prestep-blocker.sh" --json \
            "${verify_args[@]}" >"$hosted_verifier_file"
        jq -e '.condition_verified == true' "$hosted_verifier_file" >/dev/null ||
            fail "hosted CI pre-step verifier did not emit verified JSON"
        hosted_ci_verifier_json="$(jq -c '.' "$hosted_verifier_file")"
        if [[ -z "$hosted_run_id" ]]; then
            hosted_run_id="$(jq -r '.run.id // empty' "$hosted_verifier_file")"
        fi
    else
        run_step "verify hosted CI pre-step fallback" env EXPECTED_HEAD_SHA="$head_sha" \
            "$repo_root/scripts/verify-hosted-ci-prestep-blocker.sh" "${verify_args[@]}"
    fi
    hosted_ci_state="pre_step_blocker_verified"
fi

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
    run_step "package/install smoke validation" "$repo_root/scripts/package-install-smoke.sh"
else
    if [[ "$json_output" == "1" ]]; then
        printf '\n==> package/install smoke validation\nskipped\n' >&2
    else
        printf '\n==> package/install smoke validation\nskipped\n'
    fi
fi

local_ci_state="$([[ "$run_local_ci" == "1" ]] && printf 'passed' || printf 'skipped')"
package_smoke_state="$([[ "$run_package_smoke" == "1" ]] && printf 'passed' || printf 'skipped')"

release_gate_state="evidence_incomplete"
ready_for_release_owner_review=false
hosted_ci_fallback_decision_required=false

if [[ "$local_ci_state" == "passed" && "$package_smoke_state" == "passed" ]]; then
    if [[ "$hosted_ci_state" == "passing" ]]; then
        release_gate_state="hosted_ci_passing_release_owner_review_required"
        ready_for_release_owner_review=true
    elif [[ "$hosted_ci_state" == "pre_step_blocker_verified" ]]; then
        release_gate_state="fallback_release_owner_decision_required"
        ready_for_release_owner_review=true
        hosted_ci_fallback_decision_required=true
    fi
fi

if [[ "$json_output" == "1" ]]; then
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

    remaining_release_actions_json="$(
        jq -n \
            --arg state "$release_gate_state" \
            'if $state == "fallback_release_owner_decision_required" then
                [
                    "release_owner_accept_hosted_ci_fallback_or_restore_hosted_ci",
                    "mark_pr_ready",
                    "merge_pr",
                    "tag_v0.2.0-beta.1",
                    "publish_release_artifacts",
                    "verify_published_install"
                ]
            elif $state == "hosted_ci_passing_release_owner_review_required" then
                [
                    "release_owner_approve_release",
                    "mark_pr_ready",
                    "merge_pr",
                    "tag_v0.2.0-beta.1",
                    "publish_release_artifacts",
                    "verify_published_install"
                ]
            else
                [
                    "run_full_beta_release_gate_report_with_local_ci_and_package_smoke"
                ]
            end'
    )"

    jq -n \
        --arg branch "$branch" \
        --arg upstream "$upstream" \
        --arg ahead "$ahead_count" \
        --arg behind "$behind_count" \
        --arg head "$head_sha" \
        --arg tracked "$tracked_changes_present" \
        --arg pr_number "$pr_number" \
        --arg pr_url "$pr_url" \
        --arg pr_draft "$pr_draft" \
        --arg pr_merge_state "$pr_merge_state" \
        --arg hosted_ci_state "$hosted_ci_state" \
        --arg hosted_run_id "$hosted_run_id" \
        --arg local_ci "$local_ci_state" \
        --arg package_install_smoke "$package_smoke_state" \
        --arg release_gate_state "$release_gate_state" \
        --arg ready_for_review "$ready_for_release_owner_review" \
        --arg fallback_decision "$hosted_ci_fallback_decision_required" \
        --argjson checks "$checks_json" \
        --argjson hosted_ci_verifier "$hosted_ci_verifier_json" \
        --argjson remaining_release_actions "$remaining_release_actions_json" \
        '{
            branch: $branch,
            upstream: {
                name: $upstream,
                ahead: ($ahead | tonumber),
                behind: ($behind | tonumber)
            },
            head: $head,
            tracked_changes_present: ($tracked == "true"),
            pr: {
                number: ($pr_number | tonumber),
                url: $pr_url,
                draft: ($pr_draft == "true"),
                merge_state: $pr_merge_state,
                checks: $checks
            },
            hosted_ci: {
                state: $hosted_ci_state,
                run_id: (if $hosted_run_id == "" then null else $hosted_run_id end),
                verifier: $hosted_ci_verifier
            },
            local_ci: $local_ci,
            package_install_smoke: $package_install_smoke,
            release_gate_state: $release_gate_state,
            ready_for_release_owner_review: ($ready_for_review == "true"),
            hosted_ci_fallback_decision_required: ($fallback_decision == "true"),
            remaining_release_actions: $remaining_release_actions,
            release_owner_decision_required: true,
            release_actions_performed: false
        }'
else
    printf '\nBeta release gate evidence collected:\n'
    printf '  branch: %s\n' "$branch"
    printf '  upstream: %s (ahead=%s behind=%s)\n' "$upstream" "$ahead_count" "$behind_count"
    printf '  head: %s\n' "$head_sha"
    printf '  tracked_changes_present: %s\n' "$tracked_changes_present"
    printf '  pr: #%s %s\n' "$pr_number" "$pr_url"
    printf '  pr_draft: %s\n' "$pr_draft"
    printf '  pr_merge_state: %s\n' "$pr_merge_state"
    printf '  hosted_ci_state: %s\n' "$hosted_ci_state"
    printf '  local_ci: %s\n' "$local_ci_state"
    printf '  package_install_smoke: %s\n' "$package_smoke_state"
    printf '  release_gate_state: %s\n' "$release_gate_state"
    printf '  ready_for_release_owner_review: %s\n' "$ready_for_release_owner_review"
    printf '  hosted_ci_fallback_decision_required: %s\n' \
        "$hosted_ci_fallback_decision_required"
    printf '  release_owner_decision_required: true\n'
    printf '  release_actions_performed: false\n'
fi
