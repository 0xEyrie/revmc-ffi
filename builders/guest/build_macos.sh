#!/bin/bash
set -o errexit -o nounset -o pipefail
# create artifacts directory
mkdir -p artifacts

prefix=$(llvm-config --prefix)
export LLVM_SYS_180_PREFIX=$prefix
export CARGO_REGISTRIES_CRATES_IO_PROTOCOL=sparse
export DYLD_LIBRARY_PATH="./api"
# ref: https://wapl.es/rust/2019/02/17/rust-cross-compile-linux-to-macos.html
export PATH="/opt/osxcross/target/bin:$PATH"
export LIBZ_SYS_STATIC=1


echo "Starting aarch64-apple-darwin build"
export CC=aarch64-apple-darwin20.4-clang
export CXX=aarch64-apple-darwin20.4-clang++

cargo build --release --target aarch64-apple-darwin

echo "Starting x86_64-apple-darwin build"
export CC=o64-clang
export CXX=o64-clang++

cargo build --release --target x86_64-apple-darwin

# Create a universal library with both archs
lipo -output artifacts/librevmapi.dylib -create \
  "./target/x86_64-apple-darwin/release/deps/librevmapi.dylib" \
  "./target/aarch64-apple-darwin/release/deps/librevmapi.dylib"
