#!/usr/bin/env bash
# =============================================================================
# Guts Devnet E2E Test Suite
# =============================================================================
#
# This script runs extensive end-to-end tests against a 5-node Guts devnet.
# It simulates 10 clients performing various operations on 5 different
# repositories.
#
# Usage:
#   ./devnet-e2e-test.sh [--skip-setup] [--verbose]
#
# Requirements:
#   - Docker Compose
#   - curl
#   - jq
#   - git
#
# =============================================================================

set -euo pipefail

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
DOCKER_DIR="$PROJECT_ROOT/infra/docker"

# Node endpoints
declare -a NODES=(
    "http://localhost:8081"
    "http://localhost:8082"
    "http://localhost:8083"
    "http://localhost:8084"
    "http://localhost:8085"
)

# Test configuration
NUM_CLIENTS=10
NUM_REPOS=5
SKIP_SETUP=false
VERBOSE=false

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Counters
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
}

log_error() {
    echo -e "${RED}[FAIL]${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

log_verbose() {
    if [[ "$VERBOSE" == "true" ]]; then
        echo -e "${BLUE}[DEBUG]${NC} $1"
    fi
}

assert_success() {
    local description="$1"
    local result="$2"
    TESTS_RUN=$((TESTS_RUN + 1))

    if [[ "$result" == "0" ]]; then
        TESTS_PASSED=$((TESTS_PASSED + 1))
        log_success "$description"
        return 0
    else
        TESTS_FAILED=$((TESTS_FAILED + 1))
        log_error "$description"
        return 1
    fi
}

assert_eq() {
    local description="$1"
    local expected="$2"
    local actual="$3"
    TESTS_RUN=$((TESTS_RUN + 1))

    if [[ "$expected" == "$actual" ]]; then
        TESTS_PASSED=$((TESTS_PASSED + 1))
        log_success "$description"
        return 0
    else
        TESTS_FAILED=$((TESTS_FAILED + 1))
        log_error "$description (expected: $expected, got: $actual)"
        return 1
    fi
}

assert_not_empty() {
    local description="$1"
    local value="$2"
    TESTS_RUN=$((TESTS_RUN + 1))

    if [[ -n "$value" ]]; then
        TESTS_PASSED=$((TESTS_PASSED + 1))
        log_success "$description"
        return 0
    else
        TESTS_FAILED=$((TESTS_FAILED + 1))
        log_error "$description (value is empty)"
        return 1
    fi
}

random_node() {
    echo "${NODES[$((RANDOM % ${#NODES[@]}))]}"
}

api_call() {
    local method="$1"
    local url="$2"
    local data="${3:-}"

    local curl_args=(-s -X "$method" "$url" -H "Content-Type: application/json")

    if [[ -n "$data" ]]; then
        curl_args+=(-d "$data")
    fi

    curl "${curl_args[@]}"
}

# =============================================================================
# Setup and Teardown
# =============================================================================

setup_devnet() {
    log_info "Starting 5-node devnet..."

    cd "$DOCKER_DIR"
    docker compose -f docker-compose.devnet.yml up -d --build

    log_info "Waiting for all nodes to be healthy..."

    local max_wait=120
    local waited=0

    while [[ $waited -lt $max_wait ]]; do
        local healthy=0
        for node in "${NODES[@]}"; do
            if curl -sf "$node/health" > /dev/null 2>&1; then
                healthy=$((healthy + 1))
            fi
        done

        if [[ $healthy -eq ${#NODES[@]} ]]; then
            log_success "All ${#NODES[@]} nodes are healthy"
            return 0
        fi

        log_verbose "Healthy nodes: $healthy/${#NODES[@]}, waiting..."
        sleep 2
        waited=$((waited + 2))
    done

    log_error "Timeout waiting for nodes to become healthy"
    return 1
}

teardown_devnet() {
    log_info "Stopping devnet..."
    cd "$DOCKER_DIR"
    docker compose -f docker-compose.devnet.yml down -v
}

# =============================================================================
# Test Cases
# =============================================================================

test_health_check() {
    log_info "=== Testing Health Checks ==="

    for i in "${!NODES[@]}"; do
        local node="${NODES[$i]}"
        local response
        response=$(curl -sf "$node/health" 2>/dev/null || echo "")

        if [[ -n "$response" ]]; then
            local status
            status=$(echo "$response" | jq -r '.status' 2>/dev/null || echo "")
            assert_eq "Node $((i+1)) health check" "ok" "$status" || true
        else
            assert_success "Node $((i+1)) health check" "1" || true
        fi
    done
}

test_repository_creation() {
    log_info "=== Testing Repository Creation ==="

    for repo_num in $(seq 1 $NUM_REPOS); do
        local owner="org${repo_num}"
        local name="project${repo_num}"
        local node=$(random_node)

        log_verbose "Creating repo $owner/$name on $node"

        local response
        response=$(api_call POST "$node/api/repos" \
            "{\"owner\": \"$owner\", \"name\": \"$name\"}")

        local created_name
        created_name=$(echo "$response" | jq -r '.name' 2>/dev/null || echo "")

        assert_eq "Create repository $owner/$name" "$name" "$created_name" || true
    done
}

test_repository_replication() {
    log_info "=== Testing Repository Replication ==="

    # Give time for replication
    sleep 3

    for repo_num in $(seq 1 $NUM_REPOS); do
        local owner="org${repo_num}"
        local name="project${repo_num}"

        for i in "${!NODES[@]}"; do
            local node="${NODES[$i]}"
            local response
            response=$(api_call GET "$node/api/repos/$owner/$name" 2>/dev/null || echo "")

            local found_name
            found_name=$(echo "$response" | jq -r '.name' 2>/dev/null || echo "")

            assert_eq "Repo $owner/$name replicated to Node $((i+1))" "$name" "$found_name" || true
        done
    done
}

test_pull_request_workflow() {
    log_info "=== Testing Pull Request Workflow ==="

    local owner="org1"
    local name="project1"

    for client in $(seq 1 $NUM_CLIENTS); do
        local node=$(random_node)
        local author="client${client}"
        local title="Feature from $author"

        log_verbose "Creating PR from $author on $node"

        # Create PR
        local pr_response
        pr_response=$(api_call POST "$node/api/repos/$owner/$name/pulls" \
            "{
                \"title\": \"$title\",
                \"description\": \"Description from $author\",
                \"author\": \"$author\",
                \"source_branch\": \"feature-$author\",
                \"target_branch\": \"main\",
                \"source_commit\": \"abc${client}123\",
                \"target_commit\": \"def456\"
            }")

        local pr_number
        pr_number=$(echo "$pr_response" | jq -r '.number' 2>/dev/null || echo "")

        assert_not_empty "Create PR #$pr_number from $author" "$pr_number" || true

        # Add comment
        if [[ -n "$pr_number" && "$pr_number" != "null" ]]; then
            local other_node=$(random_node)
            local comment_response
            comment_response=$(api_call POST "$other_node/api/repos/$owner/$name/pulls/$pr_number/comments" \
                "{\"author\": \"reviewer\", \"body\": \"LGTM from reviewer on PR #$pr_number\"}")

            local comment_id
            comment_id=$(echo "$comment_response" | jq -r '.id' 2>/dev/null || echo "")
            assert_not_empty "Add comment to PR #$pr_number" "$comment_id" || true

            # Add review
            local review_response
            review_response=$(api_call POST "$other_node/api/repos/$owner/$name/pulls/$pr_number/reviews" \
                "{
                    \"author\": \"reviewer\",
                    \"state\": \"approved\",
                    \"body\": \"Looks good!\",
                    \"commit_id\": \"abc${client}123\"
                }")

            local review_id
            review_id=$(echo "$review_response" | jq -r '.id' 2>/dev/null || echo "")
            assert_not_empty "Add review to PR #$pr_number" "$review_id" || true
        fi
    done
}

test_issue_workflow() {
    log_info "=== Testing Issue Workflow ==="

    local owner="org2"
    local name="project2"

    for client in $(seq 1 $NUM_CLIENTS); do
        local node=$(random_node)
        local author="client${client}"

        # Create issue
        local issue_response
        issue_response=$(api_call POST "$node/api/repos/$owner/$name/issues" \
            "{
                \"title\": \"Bug report from $author\",
                \"description\": \"Found an issue while testing\",
                \"author\": \"$author\",
                \"labels\": [\"bug\", \"help-wanted\"]
            }")

        local issue_number
        issue_number=$(echo "$issue_response" | jq -r '.number' 2>/dev/null || echo "")

        assert_not_empty "Create issue from $author" "$issue_number" || true

        # Add comment on different node
        if [[ -n "$issue_number" && "$issue_number" != "null" ]]; then
            local other_node=$(random_node)
            local comment_response
            comment_response=$(api_call POST "$other_node/api/repos/$owner/$name/issues/$issue_number/comments" \
                "{\"author\": \"helper\", \"body\": \"I can help with this issue\"}")

            local comment_id
            comment_id=$(echo "$comment_response" | jq -r '.id' 2>/dev/null || echo "")
            assert_not_empty "Add comment to issue #$issue_number" "$comment_id" || true
        fi
    done
}

test_organization_workflow() {
    log_info "=== Testing Organization Workflow ==="

    for org_num in $(seq 1 3); do
        local node=$(random_node)
        local org_name="testorg${org_num}"

        # Create organization
        local org_response
        org_response=$(api_call POST "$node/api/orgs" \
            "{
                \"name\": \"$org_name\",
                \"display_name\": \"Test Organization $org_num\",
                \"description\": \"Organization for testing\",
                \"creator\": \"admin\"
            }")

        local created_name
        created_name=$(echo "$org_response" | jq -r '.name' 2>/dev/null || echo "")

        assert_eq "Create organization $org_name" "$org_name" "$created_name" || true

        # Add members
        for member in $(seq 1 3); do
            local member_node=$(random_node)
            api_call POST "$member_node/api/orgs/$org_name/members" \
                "{\"user\": \"member${member}\", \"role\": \"member\"}" > /dev/null 2>&1 || true
        done

        # Create team
        local team_response
        team_response=$(api_call POST "$node/api/orgs/$org_name/teams" \
            "{
                \"name\": \"developers\",
                \"description\": \"Development team\",
                \"permission\": \"write\",
                \"creator\": \"admin\"
            }")

        local team_name
        team_name=$(echo "$team_response" | jq -r '.name' 2>/dev/null || echo "")
        assert_eq "Create team in $org_name" "developers" "$team_name" || true
    done
}

test_cross_node_consistency() {
    log_info "=== Testing Cross-Node Consistency ==="

    # Give time for replication
    sleep 5

    # Check that all nodes have the same repos
    local owner="org1"
    local name="project1"

    local baseline_prs=""
    for i in "${!NODES[@]}"; do
        local node="${NODES[$i]}"
        local prs_response
        prs_response=$(api_call GET "$node/api/repos/$owner/$name/pulls" 2>/dev/null || echo "[]")

        local pr_count
        pr_count=$(echo "$prs_response" | jq 'length' 2>/dev/null || echo "0")

        if [[ -z "$baseline_prs" ]]; then
            baseline_prs="$pr_count"
        fi

        assert_eq "Node $((i+1)) has consistent PR count" "$baseline_prs" "$pr_count" || true
    done
}

test_concurrent_operations() {
    log_info "=== Testing Concurrent Operations ==="

    local owner="org3"
    local name="project3"

    # Launch multiple concurrent requests
    local pids=()
    for i in $(seq 1 5); do
        (
            local node=$(random_node)
            api_call POST "$node/api/repos/$owner/$name/issues" \
                "{
                    \"title\": \"Concurrent issue $i\",
                    \"description\": \"Testing concurrent creation\",
                    \"author\": \"concurrent-client-$i\"
                }" > /dev/null 2>&1
        ) &
        pids+=($!)
    done

    # Wait for all
    local success=0
    for pid in "${pids[@]}"; do
        if wait "$pid" 2>/dev/null; then
            success=$((success + 1))
        fi
    done

    assert_eq "Concurrent issue creation" "5" "$success" || true
}

test_webhook_configuration() {
    log_info "=== Testing Webhook Configuration ==="

    local owner="org4"
    local name="project4"
    local node=$(random_node)

    # Create webhook
    local webhook_response
    webhook_response=$(api_call POST "$node/api/repos/$owner/$name/hooks" \
        "{
            \"url\": \"https://example.com/webhook\",
            \"events\": [\"push\", \"pull_request\", \"issues\"],
            \"active\": true
        }")

    local webhook_id
    webhook_id=$(echo "$webhook_response" | jq -r '.id' 2>/dev/null || echo "")

    assert_not_empty "Create webhook" "$webhook_id" || true

    # List webhooks
    local list_response
    list_response=$(api_call GET "$node/api/repos/$owner/$name/hooks")

    local webhook_count
    webhook_count=$(echo "$list_response" | jq 'length' 2>/dev/null || echo "0")

    assert_eq "Webhook count" "1" "$webhook_count" || true
}

test_branch_protection() {
    log_info "=== Testing Branch Protection ==="

    local owner="org5"
    local name="project5"
    local node=$(random_node)

    # Set branch protection
    local protection_response
    protection_response=$(api_call PUT "$node/api/repos/$owner/$name/branches/main/protection" \
        "{
            \"require_pr\": true,
            \"required_reviews\": 2,
            \"dismiss_stale_reviews\": true
        }")

    local require_pr
    require_pr=$(echo "$protection_response" | jq -r '.require_pr' 2>/dev/null || echo "")

    assert_eq "Branch protection require_pr" "true" "$require_pr" || true

    # Get branch protection
    local get_response
    get_response=$(api_call GET "$node/api/repos/$owner/$name/branches/main/protection")

    local required_reviews
    required_reviews=$(echo "$get_response" | jq -r '.required_reviews' 2>/dev/null || echo "0")

    assert_eq "Branch protection required_reviews" "2" "$required_reviews" || true
}

test_collaborator_management() {
    log_info "=== Testing Collaborator Management ==="

    local owner="org1"
    local name="project1"
    local node=$(random_node)

    # Add collaborator
    local collab_response
    collab_response=$(api_call PUT "$node/api/repos/$owner/$name/collaborators/external-dev" \
        "{\"permission\": \"write\"}")

    local permission
    permission=$(echo "$collab_response" | jq -r '.permission' 2>/dev/null || echo "")

    assert_eq "Add collaborator with write permission" "write" "$permission" || true

    # Check permission
    local perm_response
    perm_response=$(api_call GET "$node/api/repos/$owner/$name/permission/external-dev")

    local resolved_perm
    resolved_perm=$(echo "$perm_response" | jq -r '.permission' 2>/dev/null || echo "")

    assert_eq "Collaborator permission resolved" "write" "$resolved_perm" || true
}

# =============================================================================
# Main
# =============================================================================

parse_args() {
    while [[ $# -gt 0 ]]; do
        case $1 in
            --skip-setup)
                SKIP_SETUP=true
                shift
                ;;
            --verbose|-v)
                VERBOSE=true
                shift
                ;;
            --help|-h)
                echo "Usage: $0 [--skip-setup] [--verbose]"
                echo ""
                echo "Options:"
                echo "  --skip-setup    Skip devnet setup (assume already running)"
                echo "  --verbose, -v   Enable verbose output"
                echo "  --help, -h      Show this help message"
                exit 0
                ;;
            *)
                log_error "Unknown option: $1"
                exit 1
                ;;
        esac
    done
}

main() {
    parse_args "$@"

    echo ""
    echo "============================================================"
    echo "         Guts Devnet E2E Test Suite"
    echo "============================================================"
    echo ""
    echo "Configuration:"
    echo "  Nodes:    ${#NODES[@]}"
    echo "  Clients:  $NUM_CLIENTS"
    echo "  Repos:    $NUM_REPOS"
    echo ""

    # Setup
    if [[ "$SKIP_SETUP" != "true" ]]; then
        setup_devnet || exit 1
    else
        log_info "Skipping devnet setup (--skip-setup)"
    fi

    echo ""

    # Run tests
    test_health_check
    echo ""

    test_repository_creation
    echo ""

    test_repository_replication
    echo ""

    test_pull_request_workflow
    echo ""

    test_issue_workflow
    echo ""

    test_organization_workflow
    echo ""

    test_cross_node_consistency
    echo ""

    test_concurrent_operations
    echo ""

    test_webhook_configuration
    echo ""

    test_branch_protection
    echo ""

    test_collaborator_management
    echo ""

    # Summary
    echo "============================================================"
    echo "                     Test Summary"
    echo "============================================================"
    echo ""
    echo "  Total:   $TESTS_RUN"
    echo "  Passed:  $TESTS_PASSED"
    echo "  Failed:  $TESTS_FAILED"
    echo ""

    if [[ $TESTS_FAILED -eq 0 ]]; then
        echo -e "${GREEN}All tests passed!${NC}"
        exit 0
    else
        echo -e "${RED}$TESTS_FAILED test(s) failed${NC}"
        exit 1
    fi
}

# Cleanup on exit
cleanup() {
    if [[ "$SKIP_SETUP" != "true" ]]; then
        log_info "Cleaning up..."
        teardown_devnet 2>/dev/null || true
    fi
}

trap cleanup EXIT

main "$@"
