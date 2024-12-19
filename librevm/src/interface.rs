use crate::{
    error::set_error,
    memory::{ ByteSliceView, UnmanagedVector },
    states::{ Db, StateDB },
    types::TryIntoVec,
};
use alloy_primitives::B256;
use once_cell::sync::OnceCell;
use revm::{ primitives::SpecId, Evm, EvmBuilder };
use std::sync::{ Arc, RwLock };

// pub static SLED_DB: OnceCell<Arc<RwLock<SledDB<B256>>>> = OnceCell::new();

// #[allow(non_camel_case_types)]
// #[repr(C)]
// pub struct compiler_t {}

// pub fn to_compiler(ptr: *mut compiler_t) -> Option<&'static mut CompileWorker> {
//     if ptr.is_null() {
//         None
//     } else {
//         let compiler = unsafe { &mut *(ptr as *mut CompileWorker) };
//         Some(compiler)
//     }
// }

// #[no_mangle]
// pub extern "C" fn new_compiler(threshold: u64) -> *mut compiler_t {
//     let rt = tokio::runtime::Runtime::new().unwrap();
//     rt.block_on(async {
//         let sled_db = SLED_DB.get_or_init(|| Arc::new(RwLock::new(SledDB::init())));
//         let compiler = CompileWorker::new(threshold, Arc::clone(sled_db));
//         let compiler = Box::into_raw(Box::new(compiler));
//         compiler as *mut compiler_t
//     })
// }

// #[no_mangle]
// pub extern "C" fn free_compiler(compiler: *mut compiler_t) {
//     if !compiler.is_null() {
//         // this will free cache when it goes out of scope
//         let _ = unsafe { Box::from_raw(compiler as *mut CompileWorker) };
//     }
// }

// byte slice view: golang data type
// unamangedvector: ffi safe vector data type compliants with rust's ownership and data types, for returning optional error value
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
pub extern "C" fn new_vm_with_compiler(default_spec_id: u8, thershold: u64) -> *mut evm_t {
    let db = Db::default();
    let state_db = StateDB::new(&db);
    let spec = SpecId::try_from_u8(default_spec_id).unwrap_or(SpecId::OSAKA);
    let builder = EvmBuilder::default();

    let evm = {
        let compiler = unsafe { &mut *(compiler as *mut CompileWorker) };
        let ext = EXTComp::new(compiler);
        builder
            .with_db(state_db)
            .with_spec_id(spec)
            .with_external_context::<ExternalContext>(ext)
            .append_handler_register(register_handler)
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
            let _ = unsafe { Box::from_raw(vm as *mut Evm<ExternalContext, StateDB>) };
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
    errmsg: Option<&mut UnmanagedVector>
) -> UnmanagedVector {
    let data = if aot {
        execute::<ExternalContext>(vm_ptr, db, block, tx, errmsg)
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
    errmsg: Option<&mut UnmanagedVector>
) -> UnmanagedVector {
    let data = if aot {
        simulate::<ExternalContext>(vm_ptr, db, block, tx, errmsg)
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
    errmsg: Option<&mut UnmanagedVector>
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
    errmsg: Option<&mut UnmanagedVector>
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
