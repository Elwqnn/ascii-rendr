#!/bin/bash
# Setup script to install git hooks

# Colors
GREEN='\033[0;32m'
NC='\033[0m' # No Color

echo "Setting up git hooks..."

# Configure git to use .githooks directory
git config core.hooksPath .githooks

echo -e "${GREEN}SUCCESS${NC}: Git hooks configured!"
echo "Pre-commit hook will now run 'cargo fmt' and 'cargo clippy' before each commit."
echo ""
echo "To bypass the hook temporarily, use: git commit --no-verify"
