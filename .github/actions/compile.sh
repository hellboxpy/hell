NAME=hell

# Set an output prefix, which is the local directory if not specified
PREFIX=$(pwd)

# Set the build dir, where built cross-compiled binaries will be output
BUILDDIR=${PREFIX}/cross

# These are chosen from: https://doc.rust-lang.org/nightly/rustc/platform-support.html
if [[ $(uname) = "Darwin" ]]; then
	CROSS_TARGETS=("x86_64-apple-darwin" "aarch64-apple-darwin")
else
	CROSS_TARGETS=("x86_64-pc-windows-gnu" "x86_64-unknown-linux-musl" "aarch64-unknown-linux-musl")
fi

mkdir -p "$BUILDDIR"

compile() {
  rustup target add $1
  cargo build --release --target $1 || cross build --release --target $1
  mv "./target/$1/release/$NAME" "$BUILDDIR/$NAME-$1" || mv "./target/$1/release/$NAME.exe" "$BUILDDIR/$NAME-$1"
  md5sum "$BUILDDIR/$NAME-$1" > "$BUILDDIR/$NAME-$1.md5"
  sha256sum "$BUILDDIR/$NAME-$1" > "$BUILDDIR/$NAME-$1.sha256"
}

for target in "${CROSS_TARGETS[@]}"
do
  compile $target
done
