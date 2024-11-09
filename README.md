#  Rethmint Revm

## Structure

This repo contains both Rust and Go codes. The rust code is compiled into a dll/so to be linked via cgo and wrapped with a pleasant Go API. The full build step involves compiling rust -> C library, and linking that library to the Go code. For ergonomics of the user, we will include pre-compiled libraries to easily link with, and Go developers should just be able to import this directly.

## Supported Platform

Requires **Rust 1.77+ and Go 1.22+.**

The Rust implementation of the VM is compiled to a library called libmrevm. This is then linked to the Go code when the final binary is built. For that reason not all systems supported by Go are supported by this project.

Linux (tested on CentOS7 and Alpine) and macOS is supported.

### Builds of libmrevm

Our system currently supports the following builds. In general we can only support targets that are supported by Move's singlepass backend, which for example excludes all 32 bit systems.

| OS family       | Arch    | Linking | Supported                     | Note    |
| --------------- | ------- | ------- | ----------------------------- | ------- |
| Linux (glibc)   | x86_64  | shared  | ✅​libmrevmapi.x86_64.so         |  |
| Linux (glibc)   | x86_64  | static  | 🚫​                            | Would link libmrevm statically but glibc dynamically as static glibc linking is not recommended. Potentially interesting for Osmosis. |
| Linux (glibc)   | aarch64 | shared  | ✅​libmrevmapi.aarch64.so        |  |
| Linux (glibc)   | aarch64 | static  | 🚫​                            |  |
| Linux (musl)    | x86_64  | shared  | 🚫​                            | Possible but not needed |
| Linux (musl)    | x86_64  | static  | ✅​libmrevmapi_muslc.x86_64.a    |  |
| Linux (musl)    | aarch64 | shared  | 🚫​                            | Possible but not needed |
| Linux (musl)    | aarch64 | static  | ✅​libmrevmapi_muslc.aarch64.a   |  |
| macOS           | x86_64  | shared  | ✅​libmrevmapi.dylib             |  |
| macOS           | x86_64  | static  | 🚫​                            |  |
| macOS           | aarch64 | shared  | ✅​libmrevmapi.dylib             |  |
| macOS           | aarch64 | static  | 🚫​                            |  |

## Development

There are two parts to this code - go and rust. The first step is to ensure that there is a proper dll built for your platform. This should be api/libmrevm.X, where X is:

- `aarch64.so` or `x86_64.so` for Linux systems
- `dylib` for MacOS

If this is present, then `make test` will run the Go test suite and you can import this code freely. If it is not present you will have to build it for your system, and ideally add it to this repo with a PR (on your fork). We will set up a proper CI system for building these binaries, but we are not there yet.

To build the rust side, try make `build-rust` and wait for it to compile. This depends on `cargo` being installed with rustc version 1.77+. Generally, you can just use rustup to install all this with no problems.
