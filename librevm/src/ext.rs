use alloy_primitives::B256;
use revm::{handler::register::EvmHandler, Database};
use revmc::{eyre::Result, EvmCompilerFn};
use std::sync::{Arc, RwLock};

use crate::{
    aot::{Compiler, KeyPrefix, QueryKey, QueryKeySlice, SledDB},
    utils::{ivec_to_pathbuf, ivec_to_u64},
    SLED_DB,
};

pub struct ExternalContext {
    compiler: &'static mut Compiler,
}

impl ExternalContext {
    pub fn new(compiler: &'static mut Compiler) -> Self {
        Self { compiler }
    }

    fn get_function(
        &self,
        code_hash: B256,
    ) -> Result<Option<(EvmCompilerFn, libloading::Library)>> {
        let sled_db =
            SLED_DB.get_or_init(|| Arc::new(RwLock::new(SledDB::<QueryKeySlice>::init())));
        let key = QueryKey::with_prefix(code_hash, KeyPrefix::SOPath);

        let maybe_so_path = {
            let db_read = sled_db.read().expect("Failed to acquire read lock");
            db_read.get(*key.as_inner()).unwrap_or(None)
        };

        if let Some(so_path) = maybe_so_path {
            let so_path = ivec_to_pathbuf(&so_path).unwrap();

            let lib;
            let f = {
                lib = (unsafe { libloading::Library::new(&so_path) }).unwrap();
                let f: libloading::Symbol<'_, revmc::EvmCompilerFn> =
                    unsafe { lib.get(code_hash.to_string().as_ref()).unwrap() };
                *f
            };

            return Ok(Some((f, lib)));
        }

        Ok(None)
    }

    fn update_bytecode_reference(
        &mut self,
        code_hash: B256,
        bytecode: &revm::primitives::Bytecode,
    ) -> Result<()> {
        let sled_db =
            SLED_DB.get_or_init(|| Arc::new(RwLock::new(SledDB::<QueryKeySlice>::init())));
        let key = QueryKey::with_prefix(code_hash, KeyPrefix::Count);

        let count = {
            let db_read = match sled_db.read() {
                Ok(lock) => lock,
                Err(poisoned) => poisoned.into_inner(),
            };
            let count_bytes = db_read.get(*key.as_inner()).unwrap_or(None);
            count_bytes.and_then(|v| ivec_to_u64(&v)).unwrap_or(0)
        };

        let new_count = count + 1;
        {
            let db_write = sled_db.write().unwrap();
            db_write
                .put(*key.as_inner(), &new_count.to_be_bytes())
                .unwrap();
        }

        // if new count equals the threshold, push to queue
        if new_count == self.compiler.threshold {
            self.compiler.push_queue(code_hash, bytecode.clone());
        }
        Ok(())
    }
}

// This `+ 'static` bound is only necessary here because of an internal cfg feature.
pub fn register_handler<DB: Database>(handler: &mut EvmHandler<'_, ExternalContext, DB>) {
    let prev = handler.execution.execute_frame.clone();
    handler.execution.execute_frame = Arc::new(move |frame, memory, tables, context| {
        let interpreter = frame.interpreter_mut();
        let code_hash = interpreter.contract.hash.unwrap_or_default();

        let bytecode = context.evm.db.code_by_hash(code_hash).unwrap_or_default();
        context
            .external
            .update_bytecode_reference(code_hash, &bytecode)
            .unwrap();

        if let Some((f, _lib)) = context.external.get_function(code_hash).unwrap() {
            println!("Executing with AOT Compiled Fn\n");
            Ok(unsafe { f.call_with_interpreter_and_memory(interpreter, memory, context) })
        } else {
            prev(frame, memory, tables, context)
        }
    });
}
