#!/usr/bin/env bash
# =============================================================================
# Guts Devnet Load Test Script
# =============================================================================
#
# This script performs load and stress testing on the Guts devnet:
#   - Concurrent repository creation
#   - Concurrent issue creation
#   - Concurrent PR creation
#   - High-throughput read operations
#   - Mixed workload simulation
#   - Response time measurement
#
# Usage:
#   ./devnet-load-test.sh [OPTIONS]
#
# Options:
#   --nodes URL1,URL2,...   Comma-separated node URLs
#   --repos N               Number of repos to create (default: 50)
#   --issues N              Number of issues to create (default: 100)
#   --reads N               Number of concurrent reads (default: 200)
#   --output FILE           Output file for results (JSON)
#   --verbose, -v           Enable verbose output
#   --help, -h              Show this help
#
# =============================================================================

set -euo pipefail

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

declare -a NODES=(
    "http://localhost:8081"
    "http://localhost:8082"
    "http://localhost:8083"
    "http://localhost:8084"
    "http://localhost:8085"
)

NUM_REPOS=50
NUM_ISSUES=100
NUM_READS=200
OUTPUT_FILE=""
VERBOSE=false

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m'

# Results
declare -A RESULTS

# =============================================================================
# Utility Functions
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

random_node() {
    echo "${NODES[$((RANDOM % ${#NODES[@]}))]}"
}

measure_time() {
    local start end duration
    start=$(date +%s.%N)
    eval "$1"
    end=$(date +%s.%N)
    duration=$(echo "$end - $start" | bc 2>/dev/null || echo "0")
    echo "$duration"
}

parse_args() {
    while [[ $# -gt 0 ]]; do
        case $1 in
            --nodes)
                IFS=',' read -ra NODES <<< "$2"
                shift 2
                ;;
            --repos)
                NUM_REPOS="$2"
                shift 2
                ;;
            --issues)
                NUM_ISSUES="$2"
                shift 2
                ;;
            --reads)
                NUM_READS="$2"
                shift 2
                ;;
            --output)
                OUTPUT_FILE="$2"
                shift 2
                ;;
            --verbose|-v)
                VERBOSE=true
                shift
                ;;
            --help|-h)
                show_help
                exit 0
                ;;
            *)
                log_error "Unknown option: $1"
                exit 1
                ;;
        esac
    done
}

show_help() {
    cat << EOF
Guts Devnet Load Test Script

Usage: $0 [OPTIONS]

Options:
  --nodes URL1,URL2,...   Comma-separated node URLs
  --repos N               Number of repos to create (default: 50)
  --issues N              Number of issues to create (default: 100)
  --reads N               Number of concurrent reads (default: 200)
  --output FILE           Output file for results (JSON)
  --verbose, -v           Enable verbose output
  --help, -h              Show this help

Examples:
  $0 --repos 100 --issues 500 --reads 1000
  $0 --nodes http://localhost:8081,http://localhost:8082 --output results.json
EOF
}

# =============================================================================
# Load Tests
# =============================================================================

test_concurrent_repo_creation() {
    log_info "=== Load Test: Concurrent Repository Creation ==="
    log_info "Creating $NUM_REPOS repositories across ${#NODES[@]} nodes..."

    local start_time
    start_time=$(date +%s.%N)

    local success_count=0
    local fail_count=0

    # Create repos in parallel
    for i in $(seq 1 "$NUM_REPOS"); do
        local node
        node=$(random_node)
        (
            local response
            response=$(curl -sf -X POST "$node/api/repos" \
                -H "Content-Type: application/json" \
                -d "{\"owner\": \"load-test-org\", \"name\": \"load-repo-$i\"}" 2>/dev/null || echo "")

            if [[ -n "$response" ]] && echo "$response" | jq -e '.name' > /dev/null 2>&1; then
                echo "success"
            else
                echo "fail"
            fi
        ) &
    done

    # Wait for all background jobs
    local results
    results=$(wait)

    local end_time
    end_time=$(date +%s.%N)
    local duration
    duration=$(echo "$end_time - $start_time" | bc)

    # Count actual repos created
    local actual_count
    actual_count=$(curl -sf "${NODES[0]}/api/repos" | jq '[.[] | select(.owner == "load-test-org")] | length' 2>/dev/null || echo "0")

    local throughput
    throughput=$(echo "scale=2; $actual_count / $duration" | bc 2>/dev/null || echo "0")

    RESULTS["repo_creation_count"]="$actual_count"
    RESULTS["repo_creation_duration"]="$duration"
    RESULTS["repo_creation_throughput"]="$throughput"

    log_success "Created $actual_count/$NUM_REPOS repos in ${duration}s (${throughput} repos/s)"
}

test_concurrent_issue_creation() {
    log_info "=== Load Test: Concurrent Issue Creation ==="

    # First ensure we have a target repo
    curl -sf -X POST "${NODES[0]}/api/repos" \
        -H "Content-Type: application/json" \
        -d '{"owner": "issue-load-org", "name": "issue-target"}' > /dev/null 2>&1 || true

    log_info "Creating $NUM_ISSUES issues across ${#NODES[@]} nodes..."

    local start_time
    start_time=$(date +%s.%N)

    # Create issues in parallel
    for i in $(seq 1 "$NUM_ISSUES"); do
        local node
        node=$(random_node)
        (
            curl -sf -X POST "$node/api/repos/issue-load-org/issue-target/issues" \
                -H "Content-Type: application/json" \
                -d "{\"title\": \"Load Test Issue $i\", \"description\": \"Performance test\", \"author\": \"load-tester\"}" \
                > /dev/null 2>&1 || true
        ) &
    done

    wait

    local end_time
    end_time=$(date +%s.%N)
    local duration
    duration=$(echo "$end_time - $start_time" | bc)

    # Count actual issues
    local actual_count
    actual_count=$(curl -sf "${NODES[0]}/api/repos/issue-load-org/issue-target/issues" | jq 'length' 2>/dev/null || echo "0")

    local throughput
    throughput=$(echo "scale=2; $actual_count / $duration" | bc 2>/dev/null || echo "0")

    RESULTS["issue_creation_count"]="$actual_count"
    RESULTS["issue_creation_duration"]="$duration"
    RESULTS["issue_creation_throughput"]="$throughput"

    log_success "Created $actual_count/$NUM_ISSUES issues in ${duration}s (${throughput} issues/s)"
}

test_concurrent_reads() {
    log_info "=== Load Test: Concurrent Read Operations ==="
    log_info "Performing $NUM_READS concurrent reads across ${#NODES[@]} nodes..."

    local start_time
    start_time=$(date +%s.%N)

    local success_count=0

    for i in $(seq 1 "$NUM_READS"); do
        local node
        node=$(random_node)
        (
            # Mix of different read operations
            case $((i % 4)) in
                0) curl -sf "$node/api/repos" > /dev/null 2>&1 ;;
                1) curl -sf "$node/health" > /dev/null 2>&1 ;;
                2) curl -sf "$node/api/consensus/status" > /dev/null 2>&1 ;;
                3) curl -sf "$node/api/orgs" > /dev/null 2>&1 ;;
            esac
        ) &
    done

    wait

    local end_time
    end_time=$(date +%s.%N)
    local duration
    duration=$(echo "$end_time - $start_time" | bc)

    local throughput
    throughput=$(echo "scale=2; $NUM_READS / $duration" | bc 2>/dev/null || echo "0")

    RESULTS["read_count"]="$NUM_READS"
    RESULTS["read_duration"]="$duration"
    RESULTS["read_throughput"]="$throughput"

    log_success "Completed $NUM_READS reads in ${duration}s (${throughput} reads/s)"
}

test_mixed_workload() {
    log_info "=== Load Test: Mixed Workload Simulation ==="

    local duration=10
    log_info "Running mixed workload for ${duration}s..."

    local start_time
    start_time=$(date +%s)
    local end_time=$((start_time + duration))

    local write_ops=0
    local read_ops=0

    while [[ $(date +%s) -lt $end_time ]]; do
        local node
        node=$(random_node)

        # 80% reads, 20% writes
        if [[ $((RANDOM % 5)) -eq 0 ]]; then
            # Write operation
            local op_type=$((RANDOM % 3))
            case $op_type in
                0)
                    curl -sf -X POST "$node/api/repos" \
                        -H "Content-Type: application/json" \
                        -d "{\"owner\": \"mixed-org\", \"name\": \"mixed-repo-$RANDOM\"}" \
                        > /dev/null 2>&1 &
                    ;;
                1)
                    curl -sf -X POST "$node/api/repos/issue-load-org/issue-target/issues" \
                        -H "Content-Type: application/json" \
                        -d "{\"title\": \"Mixed Issue $RANDOM\", \"description\": \"Mixed test\", \"author\": \"tester\"}" \
                        > /dev/null 2>&1 &
                    ;;
                2)
                    curl -sf -X POST "$node/api/orgs" \
                        -H "Content-Type: application/json" \
                        -d "{\"name\": \"mixed-org-$RANDOM\", \"display_name\": \"Mixed Org\", \"creator\": \"admin\"}" \
                        > /dev/null 2>&1 &
                    ;;
            esac
            write_ops=$((write_ops + 1))
        else
            # Read operation
            curl -sf "$node/api/repos" > /dev/null 2>&1 &
            read_ops=$((read_ops + 1))
        fi

        # Small delay to prevent overwhelming
        sleep 0.01
    done

    wait

    local total_ops=$((read_ops + write_ops))
    local ops_per_sec=$((total_ops / duration))

    RESULTS["mixed_total_ops"]="$total_ops"
    RESULTS["mixed_read_ops"]="$read_ops"
    RESULTS["mixed_write_ops"]="$write_ops"
    RESULTS["mixed_ops_per_sec"]="$ops_per_sec"

    log_success "Mixed workload: ${total_ops} ops (${read_ops} reads, ${write_ops} writes) = ${ops_per_sec} ops/s"
}

test_response_times() {
    log_info "=== Load Test: Response Time Measurement ==="

    local samples=20
    declare -a times=()

    for i in $(seq 1 $samples); do
        local node
        node=$(random_node)

        local start
        start=$(date +%s.%N)

        curl -sf "$node/api/repos" > /dev/null 2>&1 || true

        local end
        end=$(date +%s.%N)

        local duration
        duration=$(echo "($end - $start) * 1000" | bc 2>/dev/null || echo "0")
        times+=("$duration")
    done

    # Calculate stats
    local sum=0
    local min=${times[0]}
    local max=${times[0]}

    for t in "${times[@]}"; do
        sum=$(echo "$sum + $t" | bc 2>/dev/null || echo "0")
        if [[ $(echo "$t < $min" | bc 2>/dev/null || echo "0") -eq 1 ]]; then
            min=$t
        fi
        if [[ $(echo "$t > $max" | bc 2>/dev/null || echo "0") -eq 1 ]]; then
            max=$t
        fi
    done

    local avg
    avg=$(echo "scale=2; $sum / $samples" | bc 2>/dev/null || echo "0")

    RESULTS["response_time_avg_ms"]="$avg"
    RESULTS["response_time_min_ms"]="$min"
    RESULTS["response_time_max_ms"]="$max"

    log_success "Response times: avg=${avg}ms, min=${min}ms, max=${max}ms"
}

# =============================================================================
# Reporting
# =============================================================================

generate_report() {
    echo ""
    echo "═══════════════════════════════════════════════════════════════"
    echo "                LOAD TEST RESULTS SUMMARY"
    echo "═══════════════════════════════════════════════════════════════"
    echo ""
    echo "Configuration:"
    echo "  Nodes:  ${#NODES[@]}"
    echo "  Repos:  $NUM_REPOS"
    echo "  Issues: $NUM_ISSUES"
    echo "  Reads:  $NUM_READS"
    echo ""
    echo "───────────────────────────────────────────────────────────────"
    echo "Repository Creation:"
    echo "  Created:    ${RESULTS[repo_creation_count]:-0} repos"
    echo "  Duration:   ${RESULTS[repo_creation_duration]:-0}s"
    echo "  Throughput: ${RESULTS[repo_creation_throughput]:-0} repos/s"
    echo ""
    echo "Issue Creation:"
    echo "  Created:    ${RESULTS[issue_creation_count]:-0} issues"
    echo "  Duration:   ${RESULTS[issue_creation_duration]:-0}s"
    echo "  Throughput: ${RESULTS[issue_creation_throughput]:-0} issues/s"
    echo ""
    echo "Read Operations:"
    echo "  Count:      ${RESULTS[read_count]:-0} reads"
    echo "  Duration:   ${RESULTS[read_duration]:-0}s"
    echo "  Throughput: ${RESULTS[read_throughput]:-0} reads/s"
    echo ""
    echo "Mixed Workload (10s):"
    echo "  Total Ops:  ${RESULTS[mixed_total_ops]:-0}"
    echo "  Read Ops:   ${RESULTS[mixed_read_ops]:-0}"
    echo "  Write Ops:  ${RESULTS[mixed_write_ops]:-0}"
    echo "  Rate:       ${RESULTS[mixed_ops_per_sec]:-0} ops/s"
    echo ""
    echo "Response Times:"
    echo "  Average:    ${RESULTS[response_time_avg_ms]:-0}ms"
    echo "  Min:        ${RESULTS[response_time_min_ms]:-0}ms"
    echo "  Max:        ${RESULTS[response_time_max_ms]:-0}ms"
    echo ""
    echo "═══════════════════════════════════════════════════════════════"

    # Generate JSON output if requested
    if [[ -n "$OUTPUT_FILE" ]]; then
        cat > "$OUTPUT_FILE" << EOF
{
    "timestamp": "$(date -Iseconds)",
    "configuration": {
        "nodes": ${#NODES[@]},
        "target_repos": $NUM_REPOS,
        "target_issues": $NUM_ISSUES,
        "target_reads": $NUM_READS
    },
    "results": {
        "repo_creation": {
            "count": ${RESULTS[repo_creation_count]:-0},
            "duration_secs": ${RESULTS[repo_creation_duration]:-0},
            "throughput_per_sec": ${RESULTS[repo_creation_throughput]:-0}
        },
        "issue_creation": {
            "count": ${RESULTS[issue_creation_count]:-0},
            "duration_secs": ${RESULTS[issue_creation_duration]:-0},
            "throughput_per_sec": ${RESULTS[issue_creation_throughput]:-0}
        },
        "read_operations": {
            "count": ${RESULTS[read_count]:-0},
            "duration_secs": ${RESULTS[read_duration]:-0},
            "throughput_per_sec": ${RESULTS[read_throughput]:-0}
        },
        "mixed_workload": {
            "total_ops": ${RESULTS[mixed_total_ops]:-0},
            "read_ops": ${RESULTS[mixed_read_ops]:-0},
            "write_ops": ${RESULTS[mixed_write_ops]:-0},
            "ops_per_sec": ${RESULTS[mixed_ops_per_sec]:-0}
        },
        "response_times_ms": {
            "average": ${RESULTS[response_time_avg_ms]:-0},
            "min": ${RESULTS[response_time_min_ms]:-0},
            "max": ${RESULTS[response_time_max_ms]:-0}
        }
    }
}
EOF
        log_info "Results saved to: $OUTPUT_FILE"
    fi
}

# =============================================================================
# Main
# =============================================================================

main() {
    parse_args "$@"

    echo ""
    echo -e "${CYAN}═══════════════════════════════════════════════════════════════${NC}"
    echo -e "${CYAN}           GUTS DEVNET LOAD TEST SUITE${NC}"
    echo -e "${CYAN}═══════════════════════════════════════════════════════════════${NC}"
    echo ""

    # Verify at least one node is accessible
    local accessible=false
    for node in "${NODES[@]}"; do
        if curl -sf "$node/health" > /dev/null 2>&1; then
            accessible=true
            break
        fi
    done

    if [[ "$accessible" != "true" ]]; then
        log_error "No nodes are accessible. Please ensure the devnet is running."
        exit 1
    fi

    log_info "Starting load tests..."
    echo ""

    test_concurrent_repo_creation
    echo ""

    test_concurrent_issue_creation
    echo ""

    test_concurrent_reads
    echo ""

    test_mixed_workload
    echo ""

    test_response_times
    echo ""

    generate_report

    log_success "Load tests completed successfully!"
}

main "$@"
