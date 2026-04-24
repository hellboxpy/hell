#!/usr/bin/env bash
set -euo pipefail

[ -f "$HOME/.cargo/env" ] && source "$HOME/.cargo/env"

usage() {
  echo "Usage: $0 [--major | --minor | --patch]"
  exit 1
}

[[ $# -eq 1 ]] || usage

current=$(grep '^version = ' Cargo.toml | head -1 | sed 's/version = "\(.*\)"/\1/')
IFS='.' read -r major minor patch <<< "$current"

case "$1" in
  --major) major=$((major + 1)); minor=0; patch=0 ;;
  --minor) minor=$((minor + 1)); patch=0 ;;
  --patch) patch=$((patch + 1)) ;;
  *) usage ;;
esac

version="$major.$minor.$patch"

tmp=$(mktemp)
sed "s/^version = \"$current\"/version = \"$version\"/" Cargo.toml > "$tmp"
mv "$tmp" Cargo.toml

cargo fetch --quiet 2>/dev/null

git add Cargo.toml Cargo.lock
git commit -m "Release $version"
git tag "v$version"
git push origin main "v$version"

echo "Released $version"
