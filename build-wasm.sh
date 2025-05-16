#!/bin/bash
set -e

# Current directory - project root
ROOT_DIR="$(pwd)"
WASM_SRC_DIR="$ROOT_DIR/programs/range-bet-program/math-core"
WASM_PKG_DIR="$ROOT_DIR/programs/range-bet-program/pkg-wasm"

# Store current version before build (in case we're updating)
if [ -f "$WASM_PKG_DIR/package.json" ]; then
  CURRENT_VERSION=$(node -p "require('$WASM_PKG_DIR/package.json').version")
else
  CURRENT_VERSION="0.1.0"
fi

echo "Current package version: $CURRENT_VERSION"

# Create temporary files for README and package.json
echo "Creating backup files..."
TEMP_DIR="$ROOT_DIR/temp"
mkdir -p "$TEMP_DIR"

# Store current npm README if exists
if [ -f "$WASM_PKG_DIR/README.md" ]; then
  cp "$WASM_PKG_DIR/README.md" "$TEMP_DIR/README.md"
else
  # Create a default npm README if it doesn't exist
  echo "# range-bet-math-core" > "$TEMP_DIR/README.md"
  echo "" >> "$TEMP_DIR/README.md"
  echo "WebAssembly-powered library for Signals Breakout Programs protocol" >> "$TEMP_DIR/README.md"
fi

# Create a package.json template with updated version
cat > "$TEMP_DIR/package.json" << EOF
{
  "name": "range-bet-math-core",
  "type": "module",
  "version": "$CURRENT_VERSION",
  "description": "Mathematical core library for Signals Breakout Programs protocol",
  "repository": {
    "type": "git",
    "url": "https://github.com/signals-protocol/signals-breakout-programs/tree/main/programs/range-bet-program/math-core"
  },
  "keywords": [
    "solana",
    "math",
    "wasm",
    "breakout",
    "signals"
  ],
  "author": "Signals Team",
  "license": "ISC",
  "files": [
    "range_bet_math_core_bg.wasm",
    "range_bet_math_core.js",
    "range_bet_math_core_bg.js",
    "range_bet_math_core.d.ts",
    "README.md"
  ],
  "main": "range_bet_math_core.js",
  "types": "range_bet_math_core.d.ts",
  "sideEffects": [
    "./range_bet_math_core.js",
    "./snippets/*"
  ]
}
EOF

# Run the build
echo "Building WASM package..."
cd "$WASM_SRC_DIR"
wasm-pack build --target bundler --out-dir ../pkg-wasm --features wasm

# Restore the README and package.json after build
echo "Restoring README and package.json..."
cp "$TEMP_DIR/README.md" "$WASM_PKG_DIR/README.md"
cp "$TEMP_DIR/package.json" "$WASM_PKG_DIR/package.json"

# Remove any .gitignore file that wasm-pack might have created
if [ -f "$WASM_PKG_DIR/.gitignore" ]; then
  echo "Removing .gitignore file from pkg-wasm directory..."
  rm "$WASM_PKG_DIR/.gitignore"
fi

# Prompt for version update
echo "Current version is $CURRENT_VERSION"
read -p "Do you want to update the version? (y/n): " UPDATE_VERSION

if [ "$UPDATE_VERSION" = "y" ] || [ "$UPDATE_VERSION" = "Y" ]; then
  read -p "Enter new version (x.y.z format): " NEW_VERSION
  if [[ $NEW_VERSION =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
    # Update version in package.json
    sed -i.bak "s/\"version\": \"$CURRENT_VERSION\"/\"version\": \"$NEW_VERSION\"/" "$WASM_PKG_DIR/package.json"
    rm "$WASM_PKG_DIR/package.json.bak"
    echo "Version updated to $NEW_VERSION"
  else
    echo "Invalid version format. Keeping current version $CURRENT_VERSION."
  fi
fi

# Clean up
echo "Cleaning up temporary files..."
rm -rf "$TEMP_DIR"

echo "WASM package built successfully!"
echo ""
echo "To publish the package, run: npm run publish:wasm"
echo "or go to the package directory and run: npm publish --access public" 