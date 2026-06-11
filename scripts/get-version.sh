#!/usr/bin/env bash
set -euo pipefail

# Script to extract version from the workspace Cargo.toml
# Usage: ./scripts/get-version.sh

if [ ! -f "Cargo.toml" ]; then
  echo "Error: Cargo.toml not found" >&2
  exit 1
fi

# Version is managed centrally in [workspace.package]
VERSION=$(grep '^version = ' Cargo.toml | head -1 | cut -d'"' -f2)

if [ -z "$VERSION" ]; then
  echo "Error: Could not extract version from Cargo.toml" >&2
  exit 1
fi

echo "$VERSION"
