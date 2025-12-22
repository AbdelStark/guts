#!/usr/bin/env bash
# =============================================================================
# Start E2E Devnet (Consensus-Enabled 4 Validators + 1 Observer)
# =============================================================================
#
# This script starts the E2E devnet with full consensus enabled.
#
# Usage:
#   ./e2e-devnet-start.sh [--build] [--detach]
#
# Options:
#   --build   Rebuild Docker images before starting
#   --detach  Run in background (detached mode)
#
# Endpoints after startup:
#   Validator 1: http://localhost:8081
#   Validator 2: http://localhost:8082
#   Validator 3: http://localhost:8083
#   Validator 4: http://localhost:8084
#   Observer:    http://localhost:8085
#
# =============================================================================

set -eo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
DOCKER_DIR="$SCRIPT_DIR/../docker"

BUILD=false
DETACH=false

# Parse arguments
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
            echo "  --build   Rebuild Docker images"
            echo "  --detach  Run in background"
            exit 0
            ;;
        *)
            shift
            ;;
    esac
done

echo ""
echo "═══════════════════════════════════════════════════════════════"
echo "          GUTS E2E DEVNET STARTUP"
echo "═══════════════════════════════════════════════════════════════"
echo ""

cd "$DOCKER_DIR"

# Build if requested
if [[ "$BUILD" == "true" ]]; then
    echo "[INFO] Building Docker images..."
    docker compose -f docker-compose.e2e.yml build
    echo ""
fi

# Start the network
echo "[INFO] Starting E2E devnet (4 validators + 1 observer)..."

COMPOSE_ARGS="-f docker-compose.e2e.yml up"
if [[ "$DETACH" == "true" ]]; then
    COMPOSE_ARGS="$COMPOSE_ARGS -d"
fi

docker compose $COMPOSE_ARGS

if [[ "$DETACH" == "true" ]]; then
    echo ""
    echo "[INFO] Waiting for network to become healthy..."

    max_wait=120
    waited=0

    while [[ $waited -lt $max_wait ]]; do
        healthy=0
        for port in 8081 8082 8083 8084 8085; do
            if curl -sf http://localhost:$port/health > /dev/null 2>&1; then
                healthy=$((healthy + 1))
            fi
        done

        if [[ $healthy -eq 5 ]]; then
            echo "[OK] All 5 nodes are healthy!"
            break
        fi

        echo "[INFO] Healthy nodes: $healthy/5, waiting..."
        sleep 2
        waited=$((waited + 2))
    done

    if [[ $healthy -lt 5 ]]; then
        echo "[WARN] Not all nodes became healthy within ${max_wait}s"
    fi

    echo ""
    echo "═══════════════════════════════════════════════════════════════"
    echo "                    DEVNET RUNNING"
    echo "═══════════════════════════════════════════════════════════════"
    echo ""
    echo "  Endpoints:"
    echo "    Validator 1: http://localhost:8081"
    echo "    Validator 2: http://localhost:8082"
    echo "    Validator 3: http://localhost:8083"
    echo "    Validator 4: http://localhost:8084"
    echo "    Observer:    http://localhost:8085"
    echo ""
    echo "  Consensus Dashboard: http://localhost:8081/consensus"
    echo ""
    echo "  To run tests:  ./infra/scripts/devnet-consensus-test.sh"
    echo "  To view logs:  docker compose -f docker-compose.e2e.yml logs -f"
    echo "  To stop:       ./infra/scripts/e2e-devnet-stop.sh"
    echo ""
fi
