#!/usr/bin/env bash

set -e

echo "Downloading Solana v${SOLANA_VERSION}"

TARBALL="solana-release-x86_64-unknown-linux-gnu.tar.bz2"
URL="https://github.com/anza-xyz/agave/releases/download/v${SOLANA_VERSION}/${TARBALL}"

wget "$URL"
tar -xjf "$TARBALL"

EXTRACTED_DIR="solana-release-x86_64-unknown-linux-gnu"

# Move to a stable PATH location
mkdir -p ~/.local/share/solana/install/active_release/bin
mv ${EXTRACTED_DIR}/bin/* ~/.local/share/solana/install/active_release/bin/

echo "$HOME/.local/share/solana/install/active_release/bin" >>$GITHUB_PATH

# Clean up
rm -rf "$EXTRACTED_DIR" "$TARBALL"
