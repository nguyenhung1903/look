#!/usr/bin/env bash
set -euo pipefail

REPO="${LOOK_REPO:-kunkka19xx/look}"
VERSION="${LOOK_VERSION:-}"
DOWNLOAD_URL="${LOOK_DOWNLOAD_URL:-}"
APP_NAME="Look.app"

resolve_latest_version() {
  local api_url
  api_url="https://api.github.com/repos/${REPO}/releases/latest"

  curl -fsSL "$api_url" | /usr/bin/python3 -c 'import json,sys; data=json.load(sys.stdin); tag=data.get("tag_name", ""); print(tag[1:] if tag.startswith("v") else tag)'
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --repo)
      REPO="$2"
      shift 2
      ;;
    --version)
      VERSION="$2"
      shift 2
      ;;
    --url)
      DOWNLOAD_URL="$2"
      shift 2
      ;;
    *)
      echo "Unknown argument: $1" >&2
      exit 1
      ;;
  esac
done

if [[ "$(uname -s)" != "Darwin" ]]; then
  echo "This installer supports macOS only." >&2
  exit 1
fi

if [[ -z "$DOWNLOAD_URL" ]]; then
  if [[ -z "$VERSION" ]]; then
    VERSION="$(resolve_latest_version || true)"
    if [[ -z "$VERSION" ]]; then
      echo "Unable to resolve latest version from GitHub. Set LOOK_VERSION or pass --version <x.y.z>." >&2
      exit 1
    fi
    echo "Using latest release version: $VERSION"
  fi
  DOWNLOAD_URL="https://github.com/${REPO}/releases/download/v${VERSION}/Look-${VERSION}-macOS.zip"
fi

TMP_DIR="$(mktemp -d)"
ZIP_PATH="$TMP_DIR/look.zip"
EXTRACT_DIR="$TMP_DIR/extracted"

cleanup() {
  rm -rf "$TMP_DIR"
}
trap cleanup EXIT

echo "Downloading: $DOWNLOAD_URL"
curl -fL "$DOWNLOAD_URL" -o "$ZIP_PATH"

mkdir -p "$EXTRACT_DIR"
ditto -x -k "$ZIP_PATH" "$EXTRACT_DIR"

if [[ ! -d "$EXTRACT_DIR/$APP_NAME" ]]; then
  echo "Downloaded archive does not contain $APP_NAME" >&2
  exit 1
fi

TARGET_DIR="/Applications"
if [[ ! -w "$TARGET_DIR" ]]; then
  TARGET_DIR="$HOME/Applications"
  mkdir -p "$TARGET_DIR"
fi

TARGET_APP="$TARGET_DIR/$APP_NAME"
if [[ -d "$TARGET_APP" ]]; then
  rm -rf "$TARGET_APP"
fi

echo "Installing to: $TARGET_DIR"
ditto "$EXTRACT_DIR/$APP_NAME" "$TARGET_APP"

if command -v xattr >/dev/null 2>&1; then
  xattr -dr com.apple.quarantine "$TARGET_APP" 2>/dev/null || true
fi

echo "Installed $APP_NAME"
echo "Launch it from Finder or run: open \"$TARGET_APP\""
