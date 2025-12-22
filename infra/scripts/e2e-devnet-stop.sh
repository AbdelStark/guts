#!/usr/bin/env bash
# =============================================================================
# Stop E2E Devnet
# =============================================================================
#
# This script stops the E2E devnet and optionally cleans up data.
#
# Usage:
#   ./e2e-devnet-stop.sh [--clean]
#
# Options:
#   --clean   Remove volumes and orphan containers
#
# =============================================================================

set -eo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
DOCKER_DIR="$SCRIPT_DIR/../docker"

CLEAN=false

# Parse arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --clean|-c)
            CLEAN=true
            shift
            ;;
        --help|-h)
            echo "Usage: $0 [--clean]"
            echo ""
            echo "Options:"
            echo "  --clean   Remove volumes and orphan containers"
            exit 0
            ;;
        *)
            shift
            ;;
    esac
done

echo ""
echo "═══════════════════════════════════════════════════════════════"
echo "          GUTS E2E DEVNET SHUTDOWN"
echo "═══════════════════════════════════════════════════════════════"
echo ""

cd "$DOCKER_DIR"

if [[ "$CLEAN" == "true" ]]; then
    echo "[INFO] Stopping and cleaning up E2E devnet..."
    docker compose -f docker-compose.e2e.yml down -v --remove-orphans
    echo "[OK] Devnet stopped and cleaned"
else
    echo "[INFO] Stopping E2E devnet (keeping volumes)..."
    docker compose -f docker-compose.e2e.yml down
    echo "[OK] Devnet stopped"
fi

echo ""
