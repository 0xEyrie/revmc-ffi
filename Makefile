.PHONY: all build build-rust build-go test precompile

# Builds the Rust library librevm
BUILDERS_PREFIX := rethmint/librevm-builder:0001
CONTRACTS_DIR = ./contracts
USER_ID := $(shell id -u)
USER_GROUP = $(shell id -g)

SHARED_LIB_SRC = "" # File name of the shared library as created by the Rust build system
SHARED_LIB_DST = "" # File name of the shared library that we store
ifeq ($(OS),Windows_NT)
	SHARED_LIB_SRC = librevmapi.dll
	SHARED_LIB_DST = librevmapi.dll
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

# Use debug build for quick testing.
# In order to use "--features backtraces" here we need a Rust nightly toolchain, which we don't have by default
build-rust-debug:
	cargo build
	cp -fp target/debug/$(SHARED_LIB_SRC) api/$(SHARED_LIB_DST)
	make update-bindings

build-rust-release:
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

# Creates a release build in a containerized build environment of the static library for Alpine Linux (.a)
release-build-alpine:
	rm -rf target/release
	# build the muslc *.a file
	mkdir -p artifacts
	docker run --rm -u $(USER_ID):$(USER_GROUP)  \
		-v $(shell pwd):/code/ \
		$(BUILDERS_PREFIX)-alpine
	cp artifacts/librevmapi_muslc.x86_64.a api
	cp artifacts/librevmapi_muslc.aarch64.a api
	make update-bindings

# Creates a release build in a containerized build environment of the shared library for glibc Linux (.so)
release-build-linux:
	mkdir -p artifacts
	docker run --rm -v $(shell pwd)/librevmapi:/code $(BUILDERS_PREFIX)-debian build_gnu_x86_64.sh
	docker run --rm -v $(shell pwd)/librevmapi:/code $(BUILDERS_PREFIX)-debian build_gnu_aarch64.sh
	cp librevmapi/artifacts/librevmapi.x86_64.so internal/api
	cp librevmapi/artifacts/librevmapi.aarch64.so internal/api
	make update-bindings

# Creates a release build in a containerized build environment of the shared library for macOS (.dylib)
release-build-macos:
	mkdir -p artifacts
	rm -rf target/x86_64-apple-darwin/release
	rm -rf target/aarch64-apple-darwin/release
	docker run --rm -u $(USER_ID):$(USER_GROUP) \
		-v $(shell pwd):/code/ \
		$(BUILDERS_PREFIX)-cross build_macos.sh
	cp artifacts/librevmapi.dylib api
	make update-bindings

# Creates a release build in a containerized build environment of the shared library for Windows (.dll)
release-build-windows:
	mkdir -p artifacts
	docker run --rm -v $(shell pwd)/librevmapi:/code $(BUILDERS_PREFIX)-cross build_windows.sh
	cp librevmapi/artifacts/revmapi.dll internal/api
	make update-bindings

release-build:
	# Write like this because those must not run in parallel
	make release-build-alpine
	make release-build-linux
	make release-build-macos

flatbuffer-gen:
	@bash ./scripts/flatbuffer-gen.sh
	cargo fix --allow-dirty
	

