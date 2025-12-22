#!/usr/bin/env bash
# =============================================================================
# Guts Devnet Log Collection Script
# =============================================================================
#
# This script collects logs and diagnostic information from all devnet nodes.
# It gathers:
#   - Container logs from all nodes
#   - Current consensus status from all nodes
#   - System metrics and resource usage
#   - Network diagnostics
#
# Usage:
#   ./devnet-collect-logs.sh [--output-dir DIR] [--compose-file FILE]
#
# =============================================================================

set -euo pipefail

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
DOCKER_DIR="$SCRIPT_DIR/../docker"
OUTPUT_DIR="${OUTPUT_DIR:-$SCRIPT_DIR/../../e2e-results/logs}"
COMPOSE_FILE="${COMPOSE_FILE:-docker-compose.e2e.yml}"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)

# Node endpoints
declare -a NODES=(
    "http://localhost:8081"
    "http://localhost:8082"
    "http://localhost:8083"
    "http://localhost:8084"
    "http://localhost:8085"
)

# Container names
declare -a CONTAINERS=(
    "guts-e2e-validator1"
    "guts-e2e-validator2"
    "guts-e2e-validator3"
    "guts-e2e-validator4"
    "guts-e2e-observer"
)

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[OK]${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

parse_args() {
    while [[ $# -gt 0 ]]; do
        case $1 in
            --output-dir)
                OUTPUT_DIR="$2"
                shift 2
                ;;
            --compose-file)
                COMPOSE_FILE="$2"
                shift 2
                ;;
            --help|-h)
                echo "Usage: $0 [--output-dir DIR] [--compose-file FILE]"
                exit 0
                ;;
            *)
                shift
                ;;
        esac
    done
}

collect_container_logs() {
    log_info "Collecting container logs..."

    for container in "${CONTAINERS[@]}"; do
        if docker ps -a --format '{{.Names}}' | grep -q "^${container}$"; then
            log_info "  Collecting logs from $container..."
            docker logs "$container" > "$OUTPUT_DIR/${container}.log" 2>&1 || true
            docker logs --since=5m "$container" > "$OUTPUT_DIR/${container}_recent.log" 2>&1 || true
            log_success "  $container logs collected"
        else
            log_warning "  Container $container not found"
        fi
    done
}

collect_compose_logs() {
    log_info "Collecting docker-compose logs..."

    cd "$DOCKER_DIR"
    if [[ -f "$COMPOSE_FILE" ]]; then
        docker compose -f "$COMPOSE_FILE" logs --no-color --timestamps > "$OUTPUT_DIR/compose_all.log" 2>&1 || true
        log_success "  Compose logs collected"
    else
        log_warning "  Compose file not found: $COMPOSE_FILE"
    fi
}

collect_container_stats() {
    log_info "Collecting container statistics..."

    docker stats --no-stream --format "table {{.Name}}\t{{.CPUPerc}}\t{{.MemUsage}}\t{{.NetIO}}\t{{.BlockIO}}" \
        "${CONTAINERS[@]}" 2>/dev/null > "$OUTPUT_DIR/container_stats.txt" || true

    docker ps -a --format "table {{.Names}}\t{{.Status}}\t{{.Ports}}" \
        --filter "name=guts-e2e" > "$OUTPUT_DIR/container_status.txt" 2>/dev/null || true

    log_success "  Container stats collected"
}

collect_api_status() {
    log_info "Collecting API status from nodes..."

    for i in "${!NODES[@]}"; do
        local node="${NODES[$i]}"
        local container="${CONTAINERS[$i]}"
        local status_file="$OUTPUT_DIR/${container}_api_status.json"

        {
            echo "{"
            echo '  "timestamp": "'$(date -Iseconds)'",'
            echo '  "node": "'$node'",'

            # Health
            echo '  "health": '
            curl -sf "$node/health" 2>/dev/null || echo '{"error": "unreachable"}'
            echo ','

            # Consensus status
            echo '  "consensus": '
            curl -sf "$node/api/consensus/status" 2>/dev/null || echo '{"error": "unreachable"}'
            echo ','

            # Mempool
            echo '  "mempool": '
            curl -sf "$node/api/consensus/mempool" 2>/dev/null || echo '{"error": "unreachable"}'
            echo ','

            # Validators
            echo '  "validators": '
            curl -sf "$node/api/consensus/validators" 2>/dev/null || echo '{"error": "unreachable"}'

            echo "}"
        } > "$status_file" 2>/dev/null || true

        log_success "  API status from $container collected"
    done
}

collect_network_info() {
    log_info "Collecting network information..."

    # Docker network info
    docker network inspect guts-e2e-network > "$OUTPUT_DIR/network_info.json" 2>/dev/null || \
        docker network inspect guts-e2e-devnet_guts-e2e-network > "$OUTPUT_DIR/network_info.json" 2>/dev/null || true

    # Container network info
    for container in "${CONTAINERS[@]}"; do
        if docker ps -a --format '{{.Names}}' | grep -q "^${container}$"; then
            docker inspect "$container" --format '{{json .NetworkSettings}}' \
                > "$OUTPUT_DIR/${container}_network.json" 2>/dev/null || true
        fi
    done

    log_success "  Network info collected"
}

collect_system_info() {
    log_info "Collecting system information..."

    {
        echo "=== Docker Version ==="
        docker version 2>/dev/null || true
        echo ""
        echo "=== Docker Info ==="
        docker info 2>/dev/null | head -30 || true
        echo ""
        echo "=== Docker Compose Version ==="
        docker compose version 2>/dev/null || true
        echo ""
        echo "=== Disk Usage ==="
        docker system df 2>/dev/null || true
    } > "$OUTPUT_DIR/system_info.txt" 2>/dev/null || true

    log_success "  System info collected"
}

generate_summary() {
    log_info "Generating log summary..."

    cat > "$OUTPUT_DIR/SUMMARY.md" << EOF
# Guts Devnet Log Collection

**Timestamp:** $(date -Iseconds)
**Collection ID:** $TIMESTAMP

## Files Collected

### Container Logs
EOF

    for container in "${CONTAINERS[@]}"; do
        if [[ -f "$OUTPUT_DIR/${container}.log" ]]; then
            local size
            size=$(wc -l < "$OUTPUT_DIR/${container}.log" 2>/dev/null || echo "0")
            echo "- \`${container}.log\` ($size lines)" >> "$OUTPUT_DIR/SUMMARY.md"
        fi
    done

    cat >> "$OUTPUT_DIR/SUMMARY.md" << EOF

### API Status
EOF

    for container in "${CONTAINERS[@]}"; do
        if [[ -f "$OUTPUT_DIR/${container}_api_status.json" ]]; then
            echo "- \`${container}_api_status.json\`" >> "$OUTPUT_DIR/SUMMARY.md"
        fi
    done

    cat >> "$OUTPUT_DIR/SUMMARY.md" << EOF

### Other Files
- \`compose_all.log\` - Combined docker-compose logs
- \`container_stats.txt\` - Resource usage statistics
- \`container_status.txt\` - Container status
- \`network_info.json\` - Docker network configuration
- \`system_info.txt\` - Docker system information

## Quick Analysis

### Errors in logs:
\`\`\`
EOF

    for container in "${CONTAINERS[@]}"; do
        if [[ -f "$OUTPUT_DIR/${container}.log" ]]; then
            local errors
            errors=$(grep -ic "error\|panic\|fatal" "$OUTPUT_DIR/${container}.log" 2>/dev/null || echo "0")
            echo "$container: $errors error(s)" >> "$OUTPUT_DIR/SUMMARY.md"
        fi
    done

    echo '```' >> "$OUTPUT_DIR/SUMMARY.md"

    log_success "  Summary generated"
}

create_archive() {
    log_info "Creating log archive..."

    local archive_name="guts-e2e-logs-${TIMESTAMP}.tar.gz"
    local archive_path
    archive_path=$(dirname "$OUTPUT_DIR")/"$archive_name"

    tar -czf "$archive_path" -C "$(dirname "$OUTPUT_DIR")" "$(basename "$OUTPUT_DIR")" 2>/dev/null || true

    if [[ -f "$archive_path" ]]; then
        log_success "  Archive created: $archive_path"
        echo ""
        echo "Log archive: $archive_path"
    fi
}

main() {
    parse_args "$@"

    echo ""
    echo "═══════════════════════════════════════════════════════════════"
    echo "         GUTS DEVNET LOG COLLECTION"
    echo "═══════════════════════════════════════════════════════════════"
    echo ""

    # Create output directory
    mkdir -p "$OUTPUT_DIR"
    log_info "Output directory: $OUTPUT_DIR"
    echo ""

    collect_container_logs
    echo ""

    collect_compose_logs
    echo ""

    collect_container_stats
    echo ""

    collect_api_status
    echo ""

    collect_network_info
    echo ""

    collect_system_info
    echo ""

    generate_summary
    echo ""

    create_archive
    echo ""

    echo "═══════════════════════════════════════════════════════════════"
    log_success "Log collection complete!"
    echo "═══════════════════════════════════════════════════════════════"
}

main "$@"
