#!/bin/bash
set -o errexit -o nounset -o pipefail
mkdir -p artifacts
prefix=$(llvm-config-18 --prefix)
export LLVM_SYS_180_PREFIX=$prefix
export CARGO_REGISTRIES_CRATES_IO_PROTOCOL=sparse
export DYLD_LIBRARY_PATH="./api"

# No stripping implemented (see https://github.com/CosmWasm/wasmvm/issues/222#issuecomment-2260007943).
echo "Starting x86_64-unknown-linux-gnu build"
export CC=clang
export CXX=clang++
(cd librevm && cargo build --release --target x86_64-unknown-linux-gnu)
cp "./target/x86_64-unknown-linux-gnu/release/librevmapi.so" artifacts/librevmapi.x86_64.so
