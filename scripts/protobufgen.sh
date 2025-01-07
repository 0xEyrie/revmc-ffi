#!/bin/bash

export PATH="$PATH:$(go env GOPATH)/bin"
OUT_DIR="$(pwd)/core/types"
cd proto
PROTO_SRC_DIR="$(pwd)/evm/v1"

for proto_file in $PROTO_SRC_DIR/*.proto; do
  protoc -I=$PROTO_SRC_DIR --go_out=$OUT_DIR $proto_file
done

echo "Protobuf files generated in $OUT_DIR"
