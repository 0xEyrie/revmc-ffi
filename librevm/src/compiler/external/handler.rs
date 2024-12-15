use std::{
    panic::{catch_unwind, AssertUnwindSafe},
    sync::Arc,
};

use revm::{handler::register::EvmHandler, Database};

use super::ExternalContext;

pub fn register_handler<DB: Database>(handler: &mut EvmHandler<'_, ExternalContext, DB>) {
    let prev = handler.execution.execute_frame.clone();
    handler.execution.execute_frame = Arc::new(move |frame, memory, tables, context| {
        let interpreter = frame.interpreter_mut();
        let code_hash = interpreter.contract.hash.unwrap_or_default();
        let spec_id = context.evm.inner.spec_id();

        match context.external.get_function(code_hash) {
            Ok(None) => {
                let bytecode = context.evm.db.code_by_hash(code_hash).unwrap_or_default();
                context
                    .external
                    .work(spec_id, code_hash, bytecode.original_bytes());

                prev(frame, memory, tables, context)
            }

            Ok(Some((f, _lib))) => {
                println!("Executing with AOT Compiled Fn\n");
                let res = catch_unwind(AssertUnwindSafe(|| unsafe {
                    f.call_with_interpreter_and_memory(interpreter, memory, context)
                }));

                if let Err(err) = &res {
                    tracing::error!(
                        "Extern Fn Call: with bytecode hash {} {:#?}",
                        code_hash,
                        err
                    );
                }

                Ok(res.unwrap())
            }

            Err(err) => {
                tracing::error!("Get function: with bytecode hash {} {:#?}", code_hash, err);
                prev(frame, memory, tables, context)
            }
        }
    });
}
