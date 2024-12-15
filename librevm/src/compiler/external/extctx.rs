use crate::{
    compiler::{path::aot_store_path, CompileWorker},
    error::ExtError,
};
use alloy_primitives::B256;
use revmc::{primitives::SpecId, EvmCompilerFn};

pub struct ExternalContext {
    compile_worker: &'static mut CompileWorker,
}

impl ExternalContext {
    pub fn new(compile_worker: &'static mut CompileWorker) -> Self {
        Self { compile_worker }
    }

    pub fn get_function(
        &self,
        code_hash: B256,
    ) -> Result<Option<(EvmCompilerFn, libloading::Library)>, ExtError> {
        let label = code_hash.to_string();
        let so_file = aot_store_path().join(label).join("a.so");
        let exist: bool = so_file.try_exists().unwrap_or(false);
        if exist {
            let lib;
            let f = {
                lib = (unsafe { libloading::Library::new(&so_file) }).map_err(|err| {
                    ExtError::LibLoadingError {
                        err: err.to_string(),
                    }
                })?;
                let f: libloading::Symbol<'_, revmc::EvmCompilerFn> = unsafe {
                    lib.get(code_hash.to_string().as_ref()).map_err(|err| {
                        ExtError::GetSymbolError {
                            err: err.to_string(),
                        }
                    })?
                };
                *f
            };

            return Ok(Some((f, lib)));
        }

        Ok(None)
    }

    pub fn work(&mut self, spec_id: SpecId, code_hash: B256, bytecode: revm::primitives::Bytes) {
        self.compile_worker.work(spec_id, code_hash, bytecode);
    }
}
