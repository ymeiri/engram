#!/bin/bash
# =============================================================================
# Layer 5: Session Coordination - CLI Scenario Tests
# =============================================================================
#
# This script tests the coordination system by simulating multiple AI agents
# working on the same project and detecting conflicts.
#
# Usage: ./scripts/test-coordination.sh
#
# =============================================================================

set -e  # Exit on error

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Path to engram binary
ENGRAM="${ENGRAM:-./target/debug/engram}"

# Test counters
TESTS_RUN=0
TESTS_PASSED=0
TESTS_FAILED=0

# =============================================================================
# Helper Functions
# =============================================================================

log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[PASS]${NC} $1"
    TESTS_PASSED=$((TESTS_PASSED + 1))
}

log_failure() {
    echo -e "${RED}[FAIL]${NC} $1"
    TESTS_FAILED=$((TESTS_FAILED + 1))
}

log_test() {
    echo -e "\n${YELLOW}━━━ Test: $1 ━━━${NC}"
    TESTS_RUN=$((TESTS_RUN + 1))
}

# Generate a random UUID
gen_uuid() {
    uuidgen | tr '[:upper:]' '[:lower:]'
}

# Run engram command and capture output
run_engram() {
    $ENGRAM "$@" 2>&1
}

# Assert output contains expected string
assert_contains() {
    local output="$1"
    local expected="$2"
    local test_name="$3"

    if echo "$output" | grep -q "$expected"; then
        log_success "$test_name"
        return 0
    else
        log_failure "$test_name - expected to contain: '$expected'"
        echo "  Actual output: $output"
        return 1
    fi
}

# Assert output does NOT contain string
assert_not_contains() {
    local output="$1"
    local unexpected="$2"
    local test_name="$3"

    if echo "$output" | grep -q "$unexpected"; then
        log_failure "$test_name - should not contain: '$unexpected'"
        echo "  Actual output: $output"
        return 1
    else
        log_success "$test_name"
        return 0
    fi
}

# =============================================================================
# Setup
# =============================================================================

echo -e "\n${BLUE}╔═══════════════════════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║     Layer 5: Session Coordination - CLI Scenario Tests    ║${NC}"
echo -e "${BLUE}╚═══════════════════════════════════════════════════════════╝${NC}\n"

# Check engram binary exists
if [[ ! -x "$ENGRAM" ]]; then
    echo -e "${RED}Error: engram binary not found at $ENGRAM${NC}"
    echo "Run 'cargo build' first or set ENGRAM=/path/to/engram"
    exit 1
fi

log_info "Using engram binary: $ENGRAM"
log_info "Starting tests...\n"

# =============================================================================
# Test 1: Basic Registration and Unregistration
# =============================================================================

log_test "Basic Registration and Unregistration"

SESSION1=$(gen_uuid)
log_info "Generated session ID: $SESSION1"

# Register session
output=$(run_engram coord register "$SESSION1" -a claude-code -p test-project -g "Testing coordination")
assert_contains "$output" "Session registered" "Register session"
assert_contains "$output" "claude-code" "Agent name in output"

# List should show the session
output=$(run_engram coord list)
assert_contains "$output" "$SESSION1" "Session appears in list"
assert_contains "$output" "test-project" "Project in list"

# Unregister
output=$(run_engram coord unregister "$SESSION1")
assert_contains "$output" "unregistered" "Unregister session"

# List should be empty (or not contain our session)
output=$(run_engram coord list)
assert_not_contains "$output" "$SESSION1" "Session removed from list"

# =============================================================================
# Test 2: Component-Based Conflict Detection
# =============================================================================

log_test "Component-Based Conflict Detection"

SESSION_A=$(gen_uuid)
SESSION_B=$(gen_uuid)
log_info "Session A: $SESSION_A"
log_info "Session B: $SESSION_B"

# Register Session A with components: auth-service, user-api
run_engram coord register "$SESSION_A" -a claude-code -p my-app -g "Implementing auth" \
    -c auth-service -c user-api > /dev/null

# Register Session B with overlapping component: auth-service, billing
output=$(run_engram coord register "$SESSION_B" -a cursor -p my-app -g "Adding billing")
assert_contains "$output" "Session registered" "Register session B"

# Now set components on Session B that overlap with Session A
output=$(run_engram coord set-components "$SESSION_B" -c auth-service -c billing)
assert_contains "$output" "Components set" "Set components on session B"

# Check conflicts for Session B
output=$(run_engram coord conflicts "$SESSION_B")
assert_contains "$output" "Conflicts detected" "Conflict detected"
assert_contains "$output" "auth-service" "Overlapping component identified"
assert_contains "$output" "$SESSION_A" "Conflicting session identified"
assert_contains "$output" "claude-code" "Conflicting agent identified"

# Session A should also see the conflict
output=$(run_engram coord conflicts "$SESSION_A")
assert_contains "$output" "Conflicts detected" "Session A also sees conflict"
assert_contains "$output" "$SESSION_B" "Session A sees Session B as conflicting"

# Cleanup
run_engram coord unregister "$SESSION_A" > /dev/null
run_engram coord unregister "$SESSION_B" > /dev/null

# =============================================================================
# Test 3: File-Based Conflict Detection
# =============================================================================

log_test "File-Based Conflict Detection"

SESSION_X=$(gen_uuid)
SESSION_Y=$(gen_uuid)
log_info "Session X: $SESSION_X"
log_info "Session Y: $SESSION_Y"

# Register both sessions
run_engram coord register "$SESSION_X" -a claude-code -p webapp -g "Fixing auth bug" > /dev/null
run_engram coord register "$SESSION_Y" -a cursor -p webapp -g "Adding tests" > /dev/null

# Session X starts editing auth.go
output=$(run_engram coord set-file "$SESSION_X" -f src/auth/auth.go)
assert_contains "$output" "Current file set" "Session X set file"
assert_not_contains "$output" "Conflicts" "No conflict initially"

# Session Y also starts editing auth.go
output=$(run_engram coord set-file "$SESSION_Y" -f src/auth/auth.go)
assert_contains "$output" "Current file set" "Session Y set file"
assert_contains "$output" "Conflicts detected" "File conflict detected"
assert_contains "$output" "$SESSION_X" "Conflicting session identified"
assert_contains "$output" "also editing this file" "Clear conflict message"

# Verify in list that both show the file
output=$(run_engram coord list)
assert_contains "$output" "Editing: src/auth/auth.go" "File shown in list"

# Session X clears file - conflict should resolve
run_engram coord set-file "$SESSION_X" > /dev/null  # Clear file (no -f flag)

# Now Session Y should have no file conflicts
output=$(run_engram coord conflicts "$SESSION_Y")
# With no component overlap and cleared file, should be no conflicts
# (depends on implementation - might still be component conflicts)

# Cleanup
run_engram coord unregister "$SESSION_X" > /dev/null
run_engram coord unregister "$SESSION_Y" > /dev/null

# =============================================================================
# Test 4: No Conflicts When Components Don't Overlap
# =============================================================================

log_test "No Conflicts When Components Don't Overlap"

SESSION_P=$(gen_uuid)
SESSION_Q=$(gen_uuid)
log_info "Session P: $SESSION_P"
log_info "Session Q: $SESSION_Q"

# Register with non-overlapping components
run_engram coord register "$SESSION_P" -a claude-code -p big-project -g "Frontend work" \
    -c frontend -c ui-components > /dev/null
run_engram coord register "$SESSION_Q" -a cursor -p big-project -g "Backend work" \
    -c backend -c database > /dev/null

# Neither should see conflicts
output=$(run_engram coord conflicts "$SESSION_P")
assert_contains "$output" "No conflicts" "Session P has no conflicts"

output=$(run_engram coord conflicts "$SESSION_Q")
assert_contains "$output" "No conflicts" "Session Q has no conflicts"

# Cleanup
run_engram coord unregister "$SESSION_P" > /dev/null
run_engram coord unregister "$SESSION_Q" > /dev/null

# =============================================================================
# Test 5: Heartbeat Updates Session
# =============================================================================

log_test "Heartbeat Mechanism"

SESSION_H=$(gen_uuid)
log_info "Session H: $SESSION_H"

# Register session
run_engram coord register "$SESSION_H" -a claude-code -p heartbeat-test -g "Testing heartbeat" > /dev/null

# Send heartbeat
output=$(run_engram coord heartbeat "$SESSION_H")
assert_contains "$output" "Heartbeat recorded" "Heartbeat acknowledged"

# Session should still be active
output=$(run_engram coord list)
assert_contains "$output" "$SESSION_H" "Session still active after heartbeat"

# Cleanup
run_engram coord unregister "$SESSION_H" > /dev/null

# =============================================================================
# Test 6: Statistics
# =============================================================================

log_test "Statistics Tracking"

# Start with clean state - check initial stats
output=$(run_engram coord stats)
assert_contains "$output" "Active sessions:" "Stats shows active sessions count"

# Register a few sessions
S1=$(gen_uuid)
S2=$(gen_uuid)
S3=$(gen_uuid)

run_engram coord register "$S1" -a agent1 -p proj1 -g "Goal 1" > /dev/null
run_engram coord register "$S2" -a agent2 -p proj1 -g "Goal 2" > /dev/null
run_engram coord register "$S3" -a agent3 -p proj2 -g "Goal 3" > /dev/null

# Stats should show 3 active sessions (plus any from other tests)
output=$(run_engram coord stats)
log_info "Stats output: $output"
# We can't assert exact count due to potential leftover sessions, but structure should be there
assert_contains "$output" "Active sessions:" "Stats structure correct"

# List filtered by project
output=$(run_engram coord list -p proj1)
assert_contains "$output" "$S1" "Filter by project works - S1"
assert_contains "$output" "$S2" "Filter by project works - S2"
assert_not_contains "$output" "$S3" "Filter excludes other project"

# Cleanup
run_engram coord unregister "$S1" > /dev/null
run_engram coord unregister "$S2" > /dev/null
run_engram coord unregister "$S3" > /dev/null

# =============================================================================
# Test 7: Error Handling - Invalid Session ID
# =============================================================================

log_test "Error Handling - Invalid Session ID"

# Try to operate on invalid session ID
output=$(run_engram coord heartbeat "not-a-valid-uuid" 2>&1 || true)
assert_contains "$output" "invalid" "Rejects invalid UUID format"

# Try to check conflicts on non-existent session
FAKE_UUID=$(gen_uuid)
output=$(run_engram coord conflicts "$FAKE_UUID" 2>&1 || true)
# Should return error about session not being registered
assert_contains "$output" "not registered\|Not found\|Error\|not found" "Handles non-existent session"

# =============================================================================
# Test 8: Real-World Scenario - Two Agents on Same Feature
# =============================================================================

log_test "Real-World Scenario - Two Agents on Same Feature"

CLAUDE_SESSION=$(gen_uuid)
CURSOR_SESSION=$(gen_uuid)

log_info "Simulating: Claude Code and Cursor both working on authentication"
log_info "Claude session: $CLAUDE_SESSION"
log_info "Cursor session: $CURSOR_SESSION"

# Claude Code starts working on auth
run_engram coord register "$CLAUDE_SESSION" -a claude-code -p my-startup \
    -g "Implement OAuth login" -c auth -c user-service > /dev/null
run_engram coord set-file "$CLAUDE_SESSION" -f src/auth/oauth.go > /dev/null

# Cursor starts working on related feature
output=$(run_engram coord register "$CURSOR_SESSION" -a cursor -p my-startup \
    -g "Add social login buttons" -c auth -c frontend)
assert_contains "$output" "Session registered" "Cursor registers"

# Cursor sets file and gets both component AND file alerts
output=$(run_engram coord set-file "$CURSOR_SESSION" -f src/auth/oauth.go)
assert_contains "$output" "Conflicts" "Cursor warned about file conflict"

# Full conflict check
output=$(run_engram coord conflicts "$CURSOR_SESSION")
assert_contains "$output" "Conflicts detected" "Full conflict check works"
assert_contains "$output" "claude-code" "Identifies Claude as conflicting agent"
assert_contains "$output" "auth" "Shows overlapping component"

# Show the overall state
log_info "Current state of all sessions:"
run_engram coord list

# Cleanup
run_engram coord unregister "$CLAUDE_SESSION" > /dev/null
run_engram coord unregister "$CURSOR_SESSION" > /dev/null

# =============================================================================
# Summary
# =============================================================================

echo -e "\n${BLUE}═══════════════════════════════════════════════════════════${NC}"
echo -e "${BLUE}                      TEST SUMMARY                          ${NC}"
echo -e "${BLUE}═══════════════════════════════════════════════════════════${NC}"

echo -e "\nTests run:    $TESTS_RUN"
echo -e "Tests passed: ${GREEN}$TESTS_PASSED${NC}"
echo -e "Tests failed: ${RED}$TESTS_FAILED${NC}"

if [[ $TESTS_FAILED -eq 0 ]]; then
    echo -e "\n${GREEN}✅ All tests passed!${NC}\n"
    exit 0
else
    echo -e "\n${RED}❌ Some tests failed!${NC}\n"
    exit 1
fi
