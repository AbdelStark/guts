#!/usr/bin/env bash
# =============================================================================
# Check Guts Simplex BFT Devnet Status
# =============================================================================
#
# Checks the status of all validators and consensus activity.
#
# Usage:
#   ./devnet-status.sh [--consensus]
#
# =============================================================================

set -euo pipefail

# Colors
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

SHOW_CONSENSUS=false

while [[ $# -gt 0 ]]; do
    case $1 in
        --consensus|-c)
            SHOW_CONSENSUS=true
            shift
            ;;
        --help|-h)
            echo "Usage: $0 [--consensus]"
            echo ""
            echo "Options:"
            echo "  --consensus, -c   Show consensus details"
            echo "  --help, -h        Show this help message"
            exit 0
            ;;
        *)
            shift
            ;;
    esac
done

VALIDATORS=(
    "8091:Validator 1"
    "8092:Validator 2"
    "8093:Validator 3"
    "8094:Validator 4"
)

echo ""
echo "============================================"
echo "   Guts Simplex BFT Devnet Status"
echo "============================================"
echo ""

healthy=0
total=${#VALIDATORS[@]}

for node_info in "${VALIDATORS[@]}"; do
    port="${node_info%%:*}"
    name="${node_info##*:}"

    response=$(curl -sf "http://localhost:$port/health" 2>/dev/null || echo "")

    if [[ -n "$response" ]]; then
        status=$(echo "$response" | jq -r '.status' 2>/dev/null || echo "unknown")
        version=$(echo "$response" | jq -r '.version' 2>/dev/null || echo "unknown")

        if [[ "$status" == "up" ]]; then
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
echo "  Healthy: $healthy / $total validators"
echo "============================================"

if [[ "$SHOW_CONSENSUS" == "true" ]]; then
    echo ""
    echo -e "${BLUE}=== Consensus Status ===${NC}"
    echo ""

    # Get block heights from docker logs (strip ANSI codes for portability)
    for i in 1 2 3 4; do
        height=$(docker logs guts-validator$i 2>&1 | grep "finalized block" | tail -1 | sed 's/\x1b\[[0-9;]*m//g' | sed -n 's/.*height: \([0-9]*\).*/\1/p' 2>/dev/null || echo "0")
        [[ -z "$height" ]] && height=0
        echo "  Validator $i: height=$height"
    done

    # Count finalized blocks from validator1
    finalized=$(docker logs guts-validator1 2>&1 | grep "finalized block" | wc -l | tr -d ' ' || echo "0")
    echo ""
    echo "  Total finalized blocks: $finalized"
fi

echo ""

if [[ $healthy -eq $total ]]; then
    exit 0
else
    exit 1
fi
