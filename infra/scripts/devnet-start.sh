#!/usr/bin/env bash
# =============================================================================
# Start Guts Devnet
# =============================================================================
#
# Starts a 5-node Guts devnet for local development and testing.
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

COMPOSE_ARGS="-f docker-compose.devnet.yml up"

if [[ "$BUILD" == "true" ]]; then
    COMPOSE_ARGS="$COMPOSE_ARGS --build"
fi

if [[ "$DETACH" == "true" ]]; then
    COMPOSE_ARGS="$COMPOSE_ARGS -d"
fi

echo "Starting Guts devnet (5 nodes)..."
echo ""
echo "Node endpoints:"
echo "  Node 1: http://localhost:8081"
echo "  Node 2: http://localhost:8082"
echo "  Node 3: http://localhost:8083"
echo "  Node 4: http://localhost:8084"
echo "  Node 5: http://localhost:8085"
echo ""

docker compose $COMPOSE_ARGS
