#!/bin/bash
# deploy.sh - Git push + Coolify redeploy
# Usage: ./deploy.sh [commit message]

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m'

# Coolify config (VPN required!)
COOLIFY_URL="http://10.0.0.1:8000"
COOLIFY_TOKEN="1|Stj4QFTkHVu8wiQeOwyAOesEVD7iS6bt5Q7gwNu9051fd5da"
DOCS_API_UUID="ysg4coc8cks40wg8oo0c4sk8"
LANDING_UUID="yocks840g04ok44o88044cgc"

echo -e "${CYAN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${CYAN}  pctrl Deploy Script${NC}"
echo -e "${CYAN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"

# 1. Check for changes
echo -e "\n${YELLOW}[1/4]${NC} Checking git status..."
if git diff --quiet && git diff --staged --quiet; then
    echo -e "${RED}No changes to commit.${NC}"
    exit 0
fi

# 2. Git operations
echo -e "\n${YELLOW}[2/4]${NC} Git add + commit + push..."

git add .

# Generate commit message
if [ -n "$1" ]; then
    COMMIT_MSG="$1"
else
    # Auto-generate based on changed files
    CHANGED_FILES=$(git diff --staged --name-only | head -5)
    if echo "$CHANGED_FILES" | grep -q "ROADMAP.md"; then
        COMMIT_MSG="docs: update roadmap"
    elif echo "$CHANGED_FILES" | grep -q "CHANGELOG.md"; then
        COMMIT_MSG="docs: update changelog"
    elif echo "$CHANGED_FILES" | grep -q "docs-api"; then
        COMMIT_MSG="feat(docs-api): update api"
    elif echo "$CHANGED_FILES" | grep -q "landing"; then
        COMMIT_MSG="feat(landing): update landing page"
    else
        COMMIT_MSG="chore: update $(date +%Y-%m-%d)"
    fi
fi

git commit -m "$COMMIT_MSG"
echo -e "${GREEN}✓ Committed: ${COMMIT_MSG}${NC}"

git push
echo -e "${GREEN}✓ Pushed to origin${NC}"

# 3. Redeploy docs-api
echo -e "\n${YELLOW}[3/4]${NC} Deploying docs-api..."
RESPONSE=$(curl -s -X POST "${COOLIFY_URL}/api/v1/deploy?uuid=${DOCS_API_UUID}&force=false" \
    -H "Authorization: Bearer ${COOLIFY_TOKEN}" \
    -H "Content-Type: application/json")

if echo "$RESPONSE" | grep -q "error"; then
    echo -e "${RED}✗ docs-api deploy failed: ${RESPONSE}${NC}"
else
    echo -e "${GREEN}✓ docs-api deployment triggered${NC}"
fi

# 4. Wait and deploy landing page
echo -e "\n${YELLOW}[4/4]${NC} Waiting 60s for docs-api, then deploying landing..."

sleep 60

RESPONSE=$(curl -s -X POST "${COOLIFY_URL}/api/v1/deploy?uuid=${LANDING_UUID}&force=false" \
    -H "Authorization: Bearer ${COOLIFY_TOKEN}" \
    -H "Content-Type: application/json")

if echo "$RESPONSE" | grep -q "error"; then
    echo -e "${RED}✗ landing deploy failed: ${RESPONSE}${NC}"
else
    echo -e "${GREEN}✓ landing deployment triggered${NC}"
fi

echo -e "\n${CYAN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${GREEN}  Deploy complete!${NC}"
echo -e "${CYAN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
