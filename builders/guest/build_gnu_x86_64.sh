#!/bin/bash
set -o errexit -o nounset -o pipefail
mkdir -p artifacts
prefix=$(llvm-config --prefix)
export LLVM_SYS_180_PREFIX=$prefix
echo $LLVM_SYS_180_PREFIX
export CARGO_REGISTRIES_CRATES_IO_PROTOCOL=sparse
export DYLD_LIBRARY_PATH="./core/vm"

echo "Starting x86_64-unknown-linux-gnu build"
export CC=clang
export CXX=clang++
(cd librevm && cargo build --release --target x86_64-unknown-linux-gnu)
cp "./target/x86_64-unknown-linux-gnu/release/librevmapi.so" artifacts/librevmapi.x86_64.so
