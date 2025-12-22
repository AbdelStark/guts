#!/usr/bin/env bash
# =============================================================================
# Stop Guts Simplex BFT Devnet
# =============================================================================
#
# Stops the Simplex BFT devnet and optionally removes volumes.
#
# Usage:
#   ./devnet-stop.sh [--volumes]
#
# =============================================================================

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
DOCKER_DIR="$(cd "$SCRIPT_DIR/../docker" && pwd)"

REMOVE_VOLUMES=false

while [[ $# -gt 0 ]]; do
    case $1 in
        --volumes|-v)
            REMOVE_VOLUMES=true
            shift
            ;;
        --help|-h)
            echo "Usage: $0 [--volumes]"
            echo ""
            echo "Options:"
            echo "  --volumes, -v   Also remove data volumes"
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

echo "Stopping Guts Simplex BFT devnet..."

if [[ "$REMOVE_VOLUMES" == "true" ]]; then
    docker compose down -v --remove-orphans
    echo "Devnet stopped and volumes removed."
else
    docker compose down --remove-orphans
    echo "Devnet stopped. Use --volumes to remove data."
fi
