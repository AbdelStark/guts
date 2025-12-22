#!/usr/bin/env bash
# =============================================================================
# Start Guts Simplex BFT Devnet
# =============================================================================
#
# Starts a 4-validator Simplex BFT devnet with real consensus.
#
# Usage:
#   ./devnet-start.sh [--build] [--detach]
#
# =============================================================================

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
DOCKER_DIR="$(cd "$SCRIPT_DIR/../docker" && pwd)"

BUILD=false
DETACH=false

while [[ $# -gt 0 ]]; do
    case $1 in
        --build|-b)
            BUILD=true
            shift
            ;;
        --detach|-d)
            DETACH=true
            shift
            ;;
        --help|-h)
            echo "Usage: $0 [--build] [--detach]"
            echo ""
            echo "Options:"
            echo "  --build, -b     Rebuild Docker images before starting"
            echo "  --detach, -d    Run in detached mode"
            echo "  --help, -h      Show this help message"
            exit 0
            ;;
        *)
            echo "Unknown option: $1"
            exit 1
            ;;
    esac
done

cd "$DOCKER_DIR"

COMPOSE_ARGS="up"

if [[ "$BUILD" == "true" ]]; then
    COMPOSE_ARGS="$COMPOSE_ARGS --build"
fi

if [[ "$DETACH" == "true" ]]; then
    COMPOSE_ARGS="$COMPOSE_ARGS -d"
fi

echo "Starting Guts Simplex BFT devnet (4 validators)..."
echo ""
echo "Validator endpoints:"
echo "  Validator 1: http://localhost:8091 (bootstrap)"
echo "  Validator 2: http://localhost:8092"
echo "  Validator 3: http://localhost:8093"
echo "  Validator 4: http://localhost:8094"
echo ""
echo "Consensus: Real Simplex BFT (f=1 Byzantine tolerance)"
echo ""

docker compose $COMPOSE_ARGS
