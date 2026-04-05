#!/usr/bin/env bash
set -euo pipefail

if [[ $# -lt 3 ]]; then
  echo "Usage: $0 <version> <sha256> <github-owner/repo>" >&2
  echo "Example: $0 1.0.0 aabbcc... yourname/look" >&2
  exit 1
fi

VERSION="$1"
SHA256="$2"
REPO_SLUG="$3"

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
OUT_DIR="$ROOT_DIR/.build/homebrew-tap/Casks"
OUT_FILE="$OUT_DIR/look.rb"

mkdir -p "$OUT_DIR"

cat > "$OUT_FILE" <<EOF
cask "look" do
  version "${VERSION}"
  sha256 "${SHA256}"

  url "https://github.com/${REPO_SLUG}/releases/download/v#{version}/Look-#{version}-macOS.zip"
  name "look"
  desc "Keyboard-first local launcher for macOS"
  homepage "https://github.com/${REPO_SLUG}"

  app "Look.app"
end
EOF

echo "Generated cask: $OUT_FILE"
