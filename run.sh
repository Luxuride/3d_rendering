#!/usr/bin/env bash
set -e

# Force software rendering if needed
# Uncomment if you're having issues with hardware acceleration
#export LIBGL_ALWAYS_SOFTWARE=1 

# Build and run the application
echo "Building and running OpenGL Triangle demo..."
cargo run --release 