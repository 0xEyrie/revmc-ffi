mod cfg;
mod compiler;
mod key;
mod sled;

use color_eyre::Result;
use revmc::{eyre::ensure, EvmCompiler, EvmLlvmBackend};
use std::path::PathBuf;
use tempdir::TempDir;
use tokio::fs;

pub use cfg::*;
pub use compiler::*;
pub use key::*;
pub use sled::*;

pub struct RuntimeAot {
    pub cfg: AotCfg,
}

impl RuntimeAot {
    pub fn new(cfg: AotCfg) -> Self {
        Self { cfg }
    }

    pub async fn compile(&self, name: &'static str, bytecode: &[u8]) -> Result<PathBuf> {
        let _ = color_eyre::install();

        let context = revmc::llvm::inkwell::context::Context::create();
        let backend = EvmLlvmBackend::new_for_target(
            &context,
            self.cfg.aot,
            self.cfg.opt_level,
            &revmc::Target::Native,
        )?;

        let mut compiler = EvmCompiler::new(backend);

        let temp_dir = TempDir::new("aot_temp")?;
        let temp_path = temp_dir.path();
        fs::create_dir_all(&temp_path).await.unwrap();

        compiler.set_dump_to(Some(temp_path.to_path_buf()));
        compiler.gas_metering(self.cfg.no_gas);

        unsafe { compiler.stack_bound_checks(self.cfg.no_len_checks) };

        compiler.frame_pointers(true);
        compiler.debug_assertions(self.cfg.debug_assertions);
        compiler.set_module_name(name);
        compiler.validate_eof(true);

        let spec_id = self.cfg.spec_id.into();

        compiler.inspect_stack_length(true);
        let _f_id = compiler.translate(name, bytecode, spec_id)?;

        let out_dir = std::env::temp_dir().join(temp_dir).join(&name);
        std::fs::create_dir_all(&out_dir)?;

        // Compile.
        let obj = out_dir.join("a.o");
        compiler.write_object_to_file(&obj)?;
        ensure!(obj.exists(), "Failed to write object file");

        // Link.
        let so_path = out_dir.join("a.so");
        let linker = revmc::Linker::new();
        linker.link(&so_path, [obj.to_str().unwrap()])?;
        ensure!(so_path.exists(), "Failed to link object file");

        Ok(so_path)
    }
}