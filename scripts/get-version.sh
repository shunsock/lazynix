#!/usr/bin/env bash
set -euo pipefail

# Script to extract version from Cargo.toml
# Usage: ./scripts/get-version.sh

if [ ! -f "src/cli/Cargo.toml" ]; then
  echo "Error: src/cli/Cargo.toml not found" >&2
  exit 1
fi

VERSION=$(grep '^version = ' src/cli/Cargo.toml | head -1 | cut -d'"' -f2)

if [ -z "$VERSION" ]; then
  echo "Error: Could not extract version from Cargo.toml" >&2
  exit 1
fi

echo "$VERSION"
