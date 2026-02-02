#!/bin/bash

# Script to publish platform-specific packages
# This should be run after all binaries are built

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PACKAGE_DIR="$(dirname "$SCRIPT_DIR")"
NPM_DIR="$PACKAGE_DIR/npm"

echo "Publishing platform-specific packages..."

# Array of platforms
PLATFORMS=(
  "darwin-x64"
  "darwin-arm64"
  "darwin-universal"
  "linux-x64-gnu"
  "linux-arm64-gnu"
  "win32-x64-msvc"
  "freebsd-x64"
)

# Function to publish a platform package
publish_platform() {
  local platform=$1
  local platform_dir="$NPM_DIR/$platform"
  
  echo "Publishing $platform..."
  
  if [ ! -d "$platform_dir" ]; then
    echo "Error: Platform directory not found: $platform_dir"
    return 1
  fi
  
  # Find the .node file in the package directory
  local node_file=$(find "$PACKAGE_DIR" -maxdepth 1 -name "*.$platform.node" | head -n 1)
  
  if [ -z "$node_file" ]; then
    echo "Warning: No .node file found for $platform, skipping..."
    return 0
  fi
  
  # Copy the .node file to the platform directory
  cp "$node_file" "$platform_dir/"
  
  # Publish the package
  cd "$platform_dir"
  npm publish --access public
  
  echo "Successfully published $platform"
}

# Publish each platform
for platform in "${PLATFORMS[@]}"; do
  publish_platform "$platform" || echo "Failed to publish $platform"
done

echo "All platform packages published!"
