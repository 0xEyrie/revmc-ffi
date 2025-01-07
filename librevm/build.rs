use std::env;

fn main() {
    let crate_dir = env::var("CARGO_MANIFEST_DIR").unwrap();

    cbindgen::generate(crate_dir).expect("Unable to generate bindings").write_to_file("bindings.h");

    prost_build::compile_protos(
        &[
            "../proto/evm/v1/transaction.proto",
            "../proto/evm/v1/block.proto",
            "../proto/evm/v1/result.proto",
            "../proto/evm/v1/state.proto",
        ],
        &["../proto/evm/v1/"],
    )
    .expect("Failed to compile protos");
}
