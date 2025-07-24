#!/bin/bash
set -e

echo "Fixing Homebrew tap formula..."

# Get the current version and latest commit
VERSION=$(cargo metadata --no-deps --format-version 1 | jq -r '.packages[0].version')
COMMIT=$(git rev-parse HEAD)
TAG="v$VERSION"

echo "Version: $VERSION"
echo "Tag: $TAG"
echo "Commit: $COMMIT"

# Clone the tap repo
TEMP_DIR=$(mktemp -d)
cd "$TEMP_DIR"

echo "Cloning tap repository..."
git clone https://github.com/lrangell/homebrew-karabiner-pkl.git
cd homebrew-karabiner-pkl

# Create the corrected formula
mkdir -p Formula
cat > Formula/karabiner-pkl.rb << EOF
class KarabinerPkl < Formula
  desc "Configuration tool for Karabiner-Elements using Pkl"
  homepage "https://github.com/lrangell/karabiner-pkl"
  url "https://github.com/lrangell/karabiner-pkl.git",
      tag: "$TAG",
      revision: "$COMMIT"
  version "$VERSION"
  license "MIT"
  
  depends_on "rust" => :build
  depends_on "pkl"
  
  def install
    system "cargo", "install", "--locked", "--root", prefix, "--path", "."
  end
  
  test do
    system "#{bin}/karabiner-pkl", "--version"
  end
end
EOF

echo "Formula created. Please review:"
cat Formula/karabiner-pkl.rb

echo ""
echo "To commit and push these changes:"
echo "cd $TEMP_DIR/homebrew-karabiner-pkl"
echo "git add Formula/karabiner-pkl.rb"
echo "git commit -m 'Fix formula with correct version $VERSION'"
echo "git push origin main"