#!/bin/bash

export PATH="$PATH:$(go env GOPATH)/bin"
cd proto
PROTO_SRC_DIR="$(pwd)/evm/v1"
OUT_DIR="$(pwd)/../types"

mkdir -p $OUT_DIR

for proto_file in $PROTO_SRC_DIR/*.proto; do
  protoc -I=$PROTO_SRC_DIR --go_out=$OUT_DIR $proto_file
done

echo "Protobuf files generated in $OUT_DIR"
