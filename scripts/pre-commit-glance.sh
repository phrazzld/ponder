#!/bin/bash

# Pre-commit hook script for running glance and handling file modifications
# This script runs glance to update documentation and stages any modified files

set -e  # Exit on any error

echo "🔍 Running glance to update documentation..."

# Capture the current git status to detect changes
before_status=$(git status --porcelain)

# Run glance
glance .

# Capture the git status after running glance
after_status=$(git status --porcelain)

# Check if glance modified any files
if [[ "$before_status" != "$after_status" ]]; then
    echo "📝 Glance updated documentation files. Staging changes..."
    
    # Stage any modified glance.md files
    git add -A "*.md"
    
    echo "✅ Documentation updates have been staged for this commit."
    echo "📋 Files modified by glance:"
    git diff --cached --name-only | grep -E '\.(md)$' || true
else
    echo "✅ Documentation is already up to date."
fi

echo "🎉 Glance completed successfully!"