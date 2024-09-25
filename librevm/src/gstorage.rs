use alloy_primitives::aliases::{B32, B8};
use alloy_primitives::{Bytes, Uint};
use revm::{Database, DatabaseCommit};
use revm_primitives::{AccountInfo, Address, Bytecode, B256, U256};
use types::BackendError;

use crate::db::Db;
use crate::error::GoError;
use crate::memory::{U8SliceView, UnmanagedVector};
/// Access to the VM's backend storage, i.e. the chain
pub trait Storage {
    #[allow(dead_code)]
    /// Returns Err on error.
    /// Returns Ok(None) when key does not exist.
    /// Returns Ok(Some(Vec<u8>)) when key exists.
    ///
    /// Note: Support for differentiating between a non-existent key and a key with empty value
    /// is not great yet and might not be possible in all backends. But we're trying to get there.
    fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>, BackendError>;

    fn set(&mut self, key: &[u8], value: &[u8]) -> Result<(), BackendError>;

    /// Removes a database entry at `key`.
    ///
    /// The current interface does not allow to differentiate between a key that existed
    /// before and one that didn't exist. See https://github.com/CosmWasm/cosmwasm/issues/290
    fn remove(&mut self, key: &[u8]) -> Result<(), BackendError>;
}

pub struct GoStorage<'r> {
    db: &'r Db,
}

impl<'r> GoStorage<'r> {
    pub fn new(db: &'r Db) -> Self {
        GoStorage { db }
    }
}
// KVStore
// TODO: key padding to query
// ACCOUNT_PREFIX(B1) + {address(B20)} => ACCOUNT INFO {balance(B64)(0) | nonce(B256)(1) | code_hash(B256)(2)}
// CODE_PREFIX(B1) + {code_hash(B32)} => vm bytecode
// STORAGE_PREFIX(B1) + {address(B20)} + {index(B32)} => [32]byte(value)
// BLOCK_PREFIX(B1) + block_num(B8) => block_hash

enum EvmStoreKeyPrefix {
    AccountPrefix,
    CodePrefix,
    StoragePrefix,
    BlockPrefix,
}

impl From<EvmStoreKeyPrefix> for u8 {
    fn from(value: EvmStoreKeyPrefix) -> Self {
        match value {
            EvmStoreKeyPrefix::AccountPrefix => 1,
            EvmStoreKeyPrefix::CodePrefix => 2,
            EvmStoreKeyPrefix::StoragePrefix => 3,
            EvmStoreKeyPrefix::BlockPrefix => 4,
        }
    }
}

type CodeHash = B256;
type StorageIndex = U256;
type BlockNum = u64;

enum EvmStoreKey {
    Account(Address),
    Code(CodeHash),
    Storage(Address, StorageIndex),
    Block(BlockNum),
}

impl EvmStoreKey {
    fn key(self) -> Vec<u8> {
        match self {
            Self::Account(addr) => {
                let mut result: Vec<u8> = vec![EvmStoreKeyPrefix::AccountPrefix.into()];
                result.append(&mut addr.to_vec());
                result
            }
            Self::Code(addr) => {
                let mut result = vec![EvmStoreKeyPrefix::CodePrefix.into()];
                result.append(&mut addr.to_vec());
                result
            }
            Self::Storage(addr, idx) => {
                let mut result = vec![EvmStoreKeyPrefix::StoragePrefix.into()];
                result.append(&mut addr.to_vec());
                //result.append(&mut idx);
                result
            }
            Self::Block(block_num) => {
                let mut result = vec![EvmStoreKeyPrefix::BlockPrefix.into()];
                //result.append(&mut block_num);
                result
            }
        }
    }
}

impl<'DB> Database for GoStorage<'DB> {
    type Error = BackendError;

    fn basic(
        &mut self,
        address: revm_primitives::Address,
    ) -> Result<Option<revm_primitives::AccountInfo>, Self::Error> {
        let mut output = UnmanagedVector::default();
        let mut error_msg = UnmanagedVector::default();

        let account_key = EvmStoreKey::Account(address).key();
        let account_key_slice = account_key.as_slice();

        let _go_error: GoError = (self.db.vtable.read_db)(
            self.db.state,
            U8SliceView::new(Some(account_key_slice)),
            &mut output as *mut UnmanagedVector,
            &mut error_msg as *mut UnmanagedVector,
        )
        .into();

        let maybe_output = output.consume();
        let default = || format!("Failed to read an address in the db: {}", address);
        // TODO: parsing
        Ok(maybe_output.map(|v| v.into()))
    }

    fn storage(
        &mut self,
        address: revm_primitives::Address,
        index: revm_primitives::U256,
    ) -> Result<revm_primitives::U256, Self::Error> {
        let mut output = UnmanagedVector::default();
        let mut error_msg = UnmanagedVector::default();

        let storage_key = EvmStoreKey::Storage(address, index).key();
        let storage_key_slice = storage_key.as_slice();

        let _go_error: GoError = (self.db.vtable.read_db)(
            self.db.state,
            U8SliceView::new(Some(storage_key_slice)),
            &mut output as *mut UnmanagedVector,
            &mut error_msg as *mut UnmanagedVector,
        )
        .into();

        let maybe_output = output.consume();
        let output = maybe_output.unwrap();

        Ok(Uint::from_be_slice(&output))
    }

    fn block_hash(&mut self, number: u64) -> Result<revm_primitives::B256, Self::Error> {
        let mut output = UnmanagedVector::default();
        let mut error_msg = UnmanagedVector::default();

        let block_key = EvmStoreKey::Block(number).key();
        let block_key_slice = block_key.as_slice();

        let _go_error: GoError = (self.db.vtable.read_db)(
            self.db.state,
            U8SliceView::new(Some(block_key_slice)),
            &mut output as *mut UnmanagedVector,
            &mut error_msg as *mut UnmanagedVector,
        )
        .into();

        let maybe_output = output.consume();
        let output = maybe_output.unwrap();

        Ok(B256::from_slice(&output))
    }

    fn code_by_hash(
        &mut self,
        code_hash: revm_primitives::B256,
    ) -> Result<revm_primitives::Bytecode, Self::Error> {
        let mut output = UnmanagedVector::default();
        let mut error_msg = UnmanagedVector::default();

        let code_key = EvmStoreKey::Code(code_hash).key();
        let code_key_slice = code_key.as_slice();

        let _go_error: GoError = (self.db.vtable.read_db)(
            self.db.state,
            U8SliceView::new(Some(code_key_slice)),
            &mut output as *mut UnmanagedVector,
            &mut error_msg as *mut UnmanagedVector,
        )
        .into();

        let maybe_output = output.consume();
        let output = maybe_output.unwrap();

        Ok(Bytecode::LegacyRaw(Bytes::from(output)))
    }
}

impl<'a> DatabaseCommit for GoStorage<'a> {
    fn commit(&mut self, changes: std::collections::HashMap<Address, revm_primitives::Account>) {
        todo!();
    }
}
