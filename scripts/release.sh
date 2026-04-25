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
git tag -s "v$version" -m "Release $version"
git push origin main "v$version"

echo "Released $version"

# Update Homebrew tap
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
TAP_DIR="$(cd "$SCRIPT_DIR/../.." && pwd)/homebrew-hellbox"

if [[ ! -d "$TAP_DIR" ]]; then
  echo "Warning: homebrew tap not found at $TAP_DIR, skipping tap update"
  exit 0
fi

echo "Waiting for GitHub Actions to publish release assets..."
for i in $(seq 1 40); do
  ready=$(gh release view "v$version" --repo hellboxpy/hell --json assets \
    --jq '[.assets[].name] | contains([
      "hell-aarch64-apple-darwin.sha256",
      "hell-x86_64-apple-darwin.sha256",
      "hell-aarch64-unknown-linux-musl.sha256",
      "hell-x86_64-unknown-linux-musl.sha256"
    ])' 2>/dev/null || echo false)
  [[ "$ready" == "true" ]] && break
  printf "  (%d/40) waiting...\n" "$i"
  sleep 15
done

get_sha256() {
  gh release download "v$version" --repo hellboxpy/hell --pattern "$1.sha256" --output - | awk '{print $1}'
}

sha_aarch64_darwin=$(get_sha256 "hell-aarch64-apple-darwin")
sha_x86_64_darwin=$(get_sha256 "hell-x86_64-apple-darwin")
sha_aarch64_linux=$(get_sha256 "hell-aarch64-unknown-linux-musl")
sha_x86_64_linux=$(get_sha256 "hell-x86_64-unknown-linux-musl")

cat > "$TAP_DIR/Formula/hell.rb" << FORMULA
class Hell < Formula
  desc "Lightweight wrapper around uv for running the Hellbox toolchain"
  homepage "https://github.com/hellboxpy/hell"
  version "$version"
  license "MIT"

  on_macos do
    on_arm do
      url "https://github.com/hellboxpy/hell/releases/download/v#{version}/hell-aarch64-apple-darwin"
      sha256 "$sha_aarch64_darwin"
    end
    on_intel do
      url "https://github.com/hellboxpy/hell/releases/download/v#{version}/hell-x86_64-apple-darwin"
      sha256 "$sha_x86_64_darwin"
    end
  end

  on_linux do
    on_arm do
      url "https://github.com/hellboxpy/hell/releases/download/v#{version}/hell-aarch64-unknown-linux-musl"
      sha256 "$sha_aarch64_linux"
    end
    on_intel do
      url "https://github.com/hellboxpy/hell/releases/download/v#{version}/hell-x86_64-unknown-linux-musl"
      sha256 "$sha_x86_64_linux"
    end
  end

  def install
    os   = OS.mac? ? "apple-darwin" : "unknown-linux-musl"
    arch = Hardware::CPU.intel? ? "x86_64" : "aarch64"
    bin.install "hell-#{arch}-#{os}" => "hell"
  end

  test do
    assert_match version.to_s, shell_output("#{bin}/hell environment 2>&1")
  end
end
FORMULA

git -C "$TAP_DIR" add Formula/hell.rb
git -C "$TAP_DIR" commit -m "Update hell to $version"
git -C "$TAP_DIR" push origin main

echo "Homebrew tap updated to $version"
