use crate::{
    error::set_error,
    memory::{ByteSliceView, UnmanagedVector},
    states::{Db, StateDB},
    types::TryIntoVec,
};
use revm::{primitives::SpecId, Evm, EvmBuilder};
use revmc_worker::{register_handler, EXTCompileWorker};

// byte slice view: golang data type
// unamangedvector: ffi safe vector data type compliants with rust's ownership and data types, for
// returning optional error value
#[allow(non_camel_case_types)]
#[repr(C)]
pub struct evm_t {}

pub fn to_evm<'a, EXT>(ptr: *mut evm_t) -> Option<&'a mut Evm<'a, EXT, StateDB<'a>>> {
    if ptr.is_null() {
        None
    } else {
        let evm = unsafe { &mut *(ptr as *mut Evm<'a, EXT, StateDB<'a>>) };
        Some(evm)
    }
}

#[no_mangle]
pub extern "C" fn new_vm(default_spec_id: u8) -> *mut evm_t {
    let db = Db::default();
    let state_db = StateDB::new(&db);
    let spec = SpecId::try_from_u8(default_spec_id).unwrap_or(SpecId::OSAKA);
    let builder = EvmBuilder::default();
    let evm = builder.with_db(state_db).with_spec_id(spec).build();

    let vm = Box::into_raw(Box::new(evm));
    vm as *mut evm_t
}

#[no_mangle]
pub extern "C" fn new_vm_with_compiler(
    default_spec_id: u8,
    thershold: u64,
    max_concurrent_size: usize,
) -> *mut evm_t {
    let db = Db::default();
    let state_db = StateDB::new(&db);
    let spec = SpecId::try_from_u8(default_spec_id).unwrap_or(SpecId::OSAKA);
    let builder = EvmBuilder::default();

    let evm = {
        let ext = EXTCompileWorker::new(thershold, max_concurrent_size);
        builder
            .with_db(state_db)
            .with_spec_id(spec)
            .with_external_context::<EXTCompileWorker>(ext)
            .append_handler_register(register_handler::<StateDB>)
            .build()
    };

    let vm = Box::into_raw(Box::new(evm));
    vm as *mut evm_t
}

#[no_mangle]
pub extern "C" fn free_vm(vm: *mut evm_t, aot: bool) {
    if !vm.is_null() {
        // this will free cache when it goes out of scope
        if aot {
            let _ = unsafe { Box::from_raw(vm as *mut Evm<EXTCompileWorker, StateDB>) };
        } else {
            let _ = unsafe { Box::from_raw(vm as *mut Evm<(), StateDB>) };
        }
    }
}

#[no_mangle]
pub extern "C" fn execute_tx(
    vm_ptr: *mut evm_t,
    aot: bool,
    db: Db,
    block: ByteSliceView,
    tx: ByteSliceView,
    errmsg: Option<&mut UnmanagedVector>,
) -> UnmanagedVector {
    let data = if aot {
        execute::<EXTCompileWorker>(vm_ptr, db, block, tx, errmsg)
    } else {
        execute::<()>(vm_ptr, db, block, tx, errmsg)
    };

    UnmanagedVector::new(Some(data))
}

#[no_mangle]
pub extern "C" fn simulate_tx(
    vm_ptr: *mut evm_t,
    aot: bool,
    db: Db,
    block: ByteSliceView,
    tx: ByteSliceView,
    errmsg: Option<&mut UnmanagedVector>,
) -> UnmanagedVector {
    let data = if aot {
        simulate::<EXTCompileWorker>(vm_ptr, db, block, tx, errmsg)
    } else {
        simulate::<()>(vm_ptr, db, block, tx, errmsg)
    };

    UnmanagedVector::new(Some(data))
}

fn execute<EXT>(
    vm_ptr: *mut evm_t,
    db: Db,
    block: ByteSliceView,
    tx: ByteSliceView,
    errmsg: Option<&mut UnmanagedVector>,
) -> Vec<u8> {
    let evm = match to_evm::<EXT>(vm_ptr) {
        Some(vm) => vm,
        None => {
            panic!("Failed to get VM");
        }
    };

    let statedb = StateDB::new(&db);
    // TODO: check is it safe way to set evm
    evm.context.evm.db = statedb;
    evm.context.evm.inner.env.block = block.try_into().unwrap();
    evm.context.evm.inner.env.tx = tx.try_into().unwrap();

    let result = evm.transact_commit();
    match result {
        Ok(res) => res.try_into_vec().unwrap(),
        Err(err) => {
            set_error(err, errmsg);
            Vec::new()
        }
    }
}

fn simulate<EXT>(
    vm_ptr: *mut evm_t,
    db: Db,
    block: ByteSliceView,
    tx: ByteSliceView,
    errmsg: Option<&mut UnmanagedVector>,
) -> Vec<u8> {
    let evm = match to_evm::<EXT>(vm_ptr) {
        Some(vm) => vm,
        None => {
            panic!("Failed to get VM");
        }
    };
    let state_db = StateDB::new(&db);
    // TODO: check is it safe way to set evm
    evm.context.evm.db = state_db;
    evm.context.evm.inner.env.block = block.try_into().unwrap();
    evm.context.evm.inner.env.tx = tx.try_into().unwrap();

    // transact witout verification
    let result = evm.transact_preverified();
    match result {
        Ok(res) => res.result.try_into_vec().unwrap(),
        Err(err) => {
            set_error(err, errmsg);
            Vec::new()
        }
    }
}
