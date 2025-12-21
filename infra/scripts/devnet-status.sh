#!/usr/bin/env bash
# =============================================================================
# Check Guts Devnet Status
# =============================================================================
#
# Checks the status and health of all devnet nodes.
#
# Usage:
#   ./devnet-status.sh
#
# =============================================================================

set -euo pipefail

# Colors
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m'

NODES=(
    "8081:Node 1"
    "8082:Node 2"
    "8083:Node 3"
    "8084:Node 4"
    "8085:Node 5"
)

echo "============================================"
echo "        Guts Devnet Status"
echo "============================================"
echo ""

healthy=0
total=${#NODES[@]}

for node_info in "${NODES[@]}"; do
    port="${node_info%%:*}"
    name="${node_info##*:}"

    response=$(curl -sf "http://localhost:$port/health" 2>/dev/null || echo "")

    if [[ -n "$response" ]]; then
        status=$(echo "$response" | jq -r '.status' 2>/dev/null || echo "unknown")
        version=$(echo "$response" | jq -r '.version' 2>/dev/null || echo "unknown")

        if [[ "$status" == "ok" ]]; then
            echo -e "${GREEN}[OK]${NC} $name (port $port) - v$version"
            healthy=$((healthy + 1))
        else
            echo -e "${YELLOW}[WARN]${NC} $name (port $port) - status: $status"
        fi
    else
        echo -e "${RED}[DOWN]${NC} $name (port $port) - not responding"
    fi
done

echo ""
echo "============================================"
echo "  Healthy: $healthy / $total nodes"
echo "============================================"

if [[ $healthy -eq $total ]]; then
    exit 0
else
    exit 1
fi
