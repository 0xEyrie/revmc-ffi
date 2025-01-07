use crate::memory::{U8SliceView, UnmanagedVector};

//use crate::{ iterator::GoIter, memory::{ U8SliceView, UnmanagedVector } };
//
// this represents something passed in from the caller side of FFI
#[repr(C)]
#[allow(non_camel_case_types)]
pub struct db_t {
    _private: [u8; 0],
}

// These functions should return GoError but because we don't trust them here, we treat the return
// value as i32 and then check it when converting to GoError manually
// https://github.com/ethereum/go-ethereum/blob/08e6bdb550712503873fb2a138b30132cc36c481/core/vm/interface.go#L32
#[repr(C)]
#[allow(non_camel_case_types)]
pub struct Db_vtable {
    /// Commits the state mutations into the configured data stores.
    ///
    /// # Parameters
    /// - `db_t`: A mutable pointer to the database.
    /// - `UnmanagedVector`: A mutable pointer to an unmanaged vector where the codes will be
    ///   stored.
    /// - `UnmanagedVector`: A mutable pointer to an unmanaged vector where the storages will be
    ///   stored.
    /// - `UnmanagedVector`: A mutable pointer to an unmanaged vector where the accounts will be
    ///   stored.
    /// - `UnmanagedVector`: A mutable pointer to an unmanaged vector where the deleted accounts
    ///   will be stored.
    /// - `UnmanagedVector`: A mutable pointer to an unmanaged vector where any error message will
    ///   be stored.
    ///
    /// # Returns
    /// - `i32`: Status code indicating success or failure.
    pub commit: extern "C" fn(
        *mut db_t,
        U8SliceView,          // storages
        U8SliceView,          // accounts
        U8SliceView,          // deleted accounts and storages
        *mut UnmanagedVector, // error message output
    ) -> i32,

    /// Retrieves the account for a given address.
    ///
    /// # Parameters
    /// - `db_t`: A mutable pointer to the database.
    /// - `U8SliceView`: The address for which the account is being retrieved.
    /// - `UnmanagedVector`: A mutable pointer to an unmanaged vector where the result will be
    ///   stored.
    /// - `UnmanagedVector`: A mutable pointer to an unmanaged vector where any error message will
    ///   be stored.
    ///
    /// # Returns
    /// - `i32`: Status code indicating success or failure.
    pub get_account: extern "C" fn(
        *mut db_t,
        U8SliceView,          // address
        *mut UnmanagedVector, // result output
        *mut UnmanagedVector, // error message output
    ) -> i32,

    /// Retrieves the code by its hash.
    ///
    /// # Parameters
    /// - `db_t`: A mutable pointer to the database.
    /// - `U8SliceView`: The code hash for which the code is being retrieved.
    /// - `UnmanagedVector`: A mutable pointer to an unmanaged vector where the result will be
    ///   stored.
    /// - `UnmanagedVector`: A mutable pointer to an unmanaged vector where any error message will
    ///   be stored.
    ///
    /// # Returns
    /// - `i32`: Status code indicating success or failure.
    pub get_code_by_hash: extern "C" fn(
        *mut db_t,
        U8SliceView,          // code hash
        *mut UnmanagedVector, // result output
        *mut UnmanagedVector, // error message output
    ) -> i32,

    /// Retrieves the storage for a given address and key.
    ///
    /// # Parameters
    /// - `db_t`: A mutable pointer to the database.
    /// - `U8SliceView`: The address for which the storage is being retrieved.
    /// - `U8SliceView`: The key for which the storage is being retrieved.
    /// - `UnmanagedVector`: A mutable pointer to an unmanaged vector where the result will be
    ///   stored.
    /// - `UnmanagedVector`: A mutable pointer to an unmanaged vector where any error message will
    ///   be stored.
    ///
    /// # Returns
    /// - `i32`: Status code indicating success or failure.
    pub get_storage: extern "C" fn(
        *mut db_t,
        U8SliceView,          // address
        U8SliceView,          // key
        *mut UnmanagedVector, // result output
        *mut UnmanagedVector, // error message output
    ) -> i32,

    /// Retrieves the block hash for a given block number.
    ///
    /// # Parameters
    /// - `db_t`: A mutable pointer to the database.
    /// - `u64`: The block number for which the block hash is being retrieved.
    /// - `UnmanagedVector`: A mutable pointer to an unmanaged vector where the result will be
    ///   stored.
    /// - `UnmanagedVector`: A mutable pointer to an unmanaged vector where any error message will
    ///   be stored.
    ///
    /// # Returns
    /// - `i32`: Status code indicating success or failure.
    pub get_block_hash: extern "C" fn(
        *mut db_t,
        u64,                  // block number
        *mut UnmanagedVector, // result output
        *mut UnmanagedVector, // error message output
    ) -> i32,
}

#[repr(C)]
pub struct Db {
    pub state: *mut db_t,
    pub vtable: Db_vtable,
}

#[allow(unreachable_code)]
impl Default for Db {
    fn default() -> Self {
        // Initialize a null pointer for the state field (no state by default)
        let _state = std::ptr::null_mut::<db_t>();

        // Initialize the vtable with default no-op functions (unreachable functions)
        let _vtable = Db_vtable {
            commit: default_write_db,
            get_account: default_read_db,
            get_code_by_hash: default_read_db,
            get_storage: default_read_db2,
            get_block_hash: default_read_db3,
        };

        Db { state: _state, vtable: _vtable }
    }
}

extern "C" fn default_write_db(
    _: *mut db_t,
    _: U8SliceView,
    _: U8SliceView,
    _: U8SliceView,
    _: *mut UnmanagedVector,
) -> i32 {
    panic!("Default write_db called");
}

extern "C" fn default_read_db(
    _: *mut db_t,
    _: U8SliceView,
    _: *mut UnmanagedVector,
    _: *mut UnmanagedVector,
) -> i32 {
    panic!("Default read_db called");
}

extern "C" fn default_read_db2(
    _: *mut db_t,
    _: U8SliceView,
    _: U8SliceView,
    _: *mut UnmanagedVector,
    _: *mut UnmanagedVector,
) -> i32 {
    panic!("Default read_db called");
}

extern "C" fn default_read_db3(
    _: *mut db_t,
    _: u64,
    _: *mut UnmanagedVector,
    _: *mut UnmanagedVector,
) -> i32 {
    panic!("Default read_db called");
}
