mod error;
mod interface;
mod memory;
mod states;
mod types;

mod evm {
    pub mod v1 {
        pub mod types {
            include!(concat!(env!("OUT_DIR"), "/evm.v1.rs"));
        }
    }
}

pub use evm::*;
pub use interface::*;
