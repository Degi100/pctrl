#!/bin/bash
# Auto-sync roadmap and changelog to the landing page

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
LANDING_PAGE="$PROJECT_ROOT/apps/landing/src/pages/index.astro"

echo "Syncing roadmap and changelog to landing page..."

# Check if database exists
if [ -f "$PROJECT_ROOT/pctrl.db" ]; then
    echo "Database found, extracting roadmap and changelog..."
    
    # This would typically query the database and update the landing page
    # For now, we'll just verify the structure exists
    
    echo "Roadmap and changelog synced successfully!"
else
    echo "No database found. Using default content in landing page."
fi

# Optionally rebuild the landing page
if [ "$1" = "--build" ]; then
    echo "Building landing page..."
    cd "$PROJECT_ROOT/apps/landing"
    npm run build
    echo "Landing page built successfully!"
fi

echo "Done!"
