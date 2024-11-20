.PHONY: all build build-rust build-go test precompile clean-compile-sleddb

RETHMINT_DB_PATH := $(HOME)/.rethmint
# Builds the Rust library librevm
BUILDERS_PREFIX := rethmint/librevm-builder:0001
BENCHMARK_PREFIX := rethmint/benchmark:0001
CONTRACTS_DIR = ./contracts
USER_ID := $(shell id -u)
USER_GROUP = $(shell id -g)

SHARED_LIB_SRC = "" # File name of the shared library as created by the Rust build system
SHARED_LIB_DST = "" # File name of the shared library that we store
ifeq ($(OS),Windows_NT)
	# not supported
else
	UNAME_S := $(shell uname -s)
	ifeq ($(UNAME_S),Linux)
		SHARED_LIB_SRC = librevmapi.so
		SHARED_LIB_DST = librevmapi.$(shell rustc --print cfg | grep target_arch | cut  -d '"' -f 2).so
	endif
	ifeq ($(UNAME_S),Darwin)
		SHARED_LIB_SRC = librevmapi.dylib
		SHARED_LIB_DST = librevmapi.dylib
	endif
endif

fmt:
	cargo fmt

update-bindings:
	cp librevm/bindings.h api

lib-test:
	make build-rust-debug
	go clean -testcache
	go test -v -run Test_e2e_aot

go-test:
	go clean -testcache
	go test -v -run Test_e2e_aot

clean-compile-sleddb:
	@echo "clean the db: $(RETHMINT_DB_PATH)"
	@if [ -d "$(RETHMINT_DB_PATH)" ]; then \
		rm -rf "$(RETHMINT_DB_PATH)"; \
		echo "Directory $(RETHMINT_DB_PATH) removed successfully."; \
	else \
		echo "Directory $(RETHMINT_DB_PATH) does not exist."; \
	fi

# Use debug build for quick testing.
# In order to use "--features backtraces" here we need a Rust nightly toolchain, which we don't have by default
# build in macos to debug
build-rust-debug:
	@export LLVM_SYS_180_PREFIX=$(shell brew --prefix llvm@18);\
	export LIBRARY_PATH="/opt/homebrew/lib:$LIBRARY_PATH";\
	export LD_LIBRARY_PATH="/opt/homebrew/lib:$LD_LIBRARY_PATH";\
	cargo build
	@cp -fp target/debug/$(SHARED_LIB_SRC) api/$(SHARED_LIB_DST)
	@make update-bindings

build-rust-release:
	@export LLVM_SYS_180_PREFIX=$(shell brew --prefix llvm@18);\
	export LIBRARY_PATH="/opt/homebrew/lib:$LIBRARY_PATH";\
	export LD_LIBRARY_PATH="/opt/homebrew/lib:$LD_LIBRARY_PATH";\
	cargo build --release
	rm -f api/$(SHARED_LIB_DST)
	cp -fp target/release/$(SHARED_LIB_SRC) api/$(SHARED_LIB_DST)
	make update-bindings
	@ #this pulls out ELF symbols, 80% size reduction!

clean:
	cargo clean
	@-rm api/bindings.h
	@-rm librevm/bindings.h
	@-rm api/$(SHARED_LIB_DST)
	@echo cleaned.

# Creates a release build in a containerized build environment of the shared library for glibc Linux (.so)
release-build-linux:
	docker run --rm -v $(shell pwd):/code/ $(BUILDERS_PREFIX)-debian build_gnu_x86_64.sh
	docker run --rm -v $(shell pwd):/code/ $(BUILDERS_PREFIX)-debian build_gnu_aarch64.sh
	cp artifacts/librevmapi.x86_64.so api
	cp artifacts/librevmapi.aarch64.so api
	make update-bindings

# Creates a release build in a containerized build environment of the shared library for macOS (.dylib)
release-build-macos:
	rm -rf target/x86_64-apple-darwin/release
	rm -rf target/aarch64-apple-darwin/release
	docker run --rm -u $(USER_ID):$(USER_GROUP) \
		-v $(shell pwd):/code/ \
		$(BUILDERS_PREFIX)-cross build_macos.sh
	cp artifacts/librevmapi.dylib api
	make update-bindings

release-build:
	# Write like this because those must not run in parallel
	make release-build-linux
	make release-build-macos

flatbuffer-gen:
	@bash ./scripts/flatbuffer-gen.sh
	cargo fix --allow-dirty
	
contracts-gen:
	@bash ./scripts/contractsgen.sh


BENCHMARK_PREFIX := rethmint/benchmark:0001

.PHONY: docker-image-gevm
docker-image-gevm:
	docker build  --pull . -t $(BENCHMARK_PREFIX)-gevm -f ./benchmark/Dockerfile.gevm

.PHONY: docker-image-revmffi
docker-image-revmffi:
	docker build  --pull . -t $(BENCHMARK_PREFIX)-revmffi -f ./benchmark/Dockerfile.revmffi


.PHONY: docker-images
docker-images: docker-image-revmffi docker-image-gevm

.PHONY: docker-publish
docker-publish: docker-images
	docker push $(BENCHMARK_PREFIX)-revmffi
	docker push $(BENCHMARK_PREFIX)-gevm

.PHONY: profiling
profiling:
	@echo "Running Profiling..."
	@export BENCHMARK_PREFIX=$(BENCHMARK_PREFIX) && \
	export REV_CONTAINER_NAME=revmffi && \
	export GEV_CONTAINER_NAME=gevm && \
	docker-compose -f ./benchmark/docker-compose.yml up -d
	docker exec -it revmffi sh -c "cd /app/revmffi && go test -v --count 1000"
	docker exec -it gevm sh -c "cd /app/gevm && go test -v --count 1000"
