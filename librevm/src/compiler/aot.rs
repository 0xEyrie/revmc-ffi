use revmc::primitives::SpecId;
use revmc::OptimizationLevel;
use revmc::{EvmCompiler, EvmLlvmBackend};

use crate::error::CompilerError;

use super::path::aot_store_path;

pub struct RuntimeAot {
    pub cfg: AotCfg,
}

impl RuntimeAot {
    pub fn new(cfg: AotCfg) -> Self {
        Self { cfg }
    }

    pub fn compile(
        &self,
        name: &'static str,
        bytecode: &[u8],
        spec_id: SpecId,
    ) -> Result<(), CompilerError> {
        let _ = color_eyre::install();

        let context = revmc::llvm::inkwell::context::Context::create();
        let backend = EvmLlvmBackend::new_for_target(
            &context,
            self.cfg.aot,
            self.cfg.opt_level,
            &revmc::Target::Native,
        )
        .map_err(|err| CompilerError::BackendInit {
            err: err.to_string(),
        })?;

        let mut compiler = EvmCompiler::new(backend);

        let out_dir = aot_store_path();
        std::fs::create_dir_all(&out_dir).map_err(|err| CompilerError::FileIO {
            err: err.to_string(),
        })?;

        compiler.set_dump_to(Some(out_dir.clone()));
        compiler.gas_metering(self.cfg.no_gas);

        unsafe {
            compiler.stack_bound_checks(self.cfg.no_len_checks);
        }

        compiler.frame_pointers(true);
        compiler.debug_assertions(self.cfg.debug_assertions);
        compiler.set_module_name(name);
        compiler.validate_eof(true);

        compiler.inspect_stack_length(true);
        let _f_id = compiler.translate(name, bytecode, spec_id).map_err(|err| {
            CompilerError::BytecodeTranslation {
                err: err.to_string(),
            }
        })?;

        let module_out_dir = out_dir.join(name);
        std::fs::create_dir_all(&module_out_dir).map_err(|err| CompilerError::FileIO {
            err: err.to_string(),
        })?;

        // Compile.
        let obj = module_out_dir.join("a.o");
        compiler
            .write_object_to_file(&obj)
            .map_err(|err| CompilerError::FileIO {
                err: err.to_string(),
            })?;

        // Link.
        let so_path = module_out_dir.join("a.so");
        let linker = revmc::Linker::new();
        linker
            .link(&so_path, [obj.to_str().unwrap()])
            .map_err(|err| CompilerError::Link {
                err: err.to_string(),
            })?;

        Ok(())
    }
}

pub struct AotCfg {
    pub aot: bool,
    pub opt_level: OptimizationLevel,
    pub no_gas: bool,
    pub no_len_checks: bool,
    pub debug_assertions: bool,
}

impl Default for AotCfg {
    fn default() -> Self {
        AotCfg {
            aot: true,
            opt_level: OptimizationLevel::Aggressive,
            no_gas: true,
            no_len_checks: true,
            debug_assertions: true,
        }
    }
}
