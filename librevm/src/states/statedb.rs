use alloy_primitives::{Address, BlockHash, Bytes, B256, U256};
use revm::primitives::{Account, AccountInfo, Bytecode, HashMap};
use revm::{Database, DatabaseCommit};

use crate::error::{BackendError, GoError};
use crate::memory::{U8SliceView, UnmanagedVector};
use crate::types::{Code, DeletedAccounts, UpdatedAccounts, UpdatedCodes, UpdatedStorages};

use super::vtable::Db;

pub struct StateDB<'r> {
    pub db: &'r Db,
}

impl<'r> StateDB<'r> {
    pub fn new(db: &'r Db) -> Self {
        StateDB { db }
    }
}

impl<'db> Database for StateDB<'db> {
    type Error = BackendError;

    #[doc = " Get basic account information."]
    fn basic(&mut self, address: Address) -> Result<Option<AccountInfo>, BackendError> {
        let mut error_msg = UnmanagedVector::default();
        let mut output = UnmanagedVector::default();
        let go_error: GoError = (self.db.vtable.get_account)(
            self.db.state,
            U8SliceView::new(Some(address.as_slice())),
            &mut output as *mut UnmanagedVector,
            &mut error_msg as *mut UnmanagedVector,
        )
        .into();
        unsafe {
            go_error
                .into_result(error_msg, || "Failed to get account info from the db".to_owned())?;
        }
        let account_info: AccountInfo = output.try_into().unwrap();
        Ok(Some(account_info))
    }

    #[doc = " Get account code by its hash."]
    fn code_by_hash(&mut self, code_hash: B256) -> Result<Bytecode, Self::Error> {
        let mut error_msg = UnmanagedVector::default();
        let mut output = UnmanagedVector::default();
        let go_error: GoError = (self.db.vtable.get_code_by_hash)(
            self.db.state,
            U8SliceView::new(Some(code_hash.as_slice())),
            &mut output as *mut UnmanagedVector,
            &mut error_msg as *mut UnmanagedVector,
        )
        .into();
        unsafe {
            go_error.into_result(error_msg, || "Failed to get code from the db".to_owned())?;
        }
        let bytecode_bytes = output.consume().unwrap();
        let bytecode = Bytecode::new_raw(Bytes::from(bytecode_bytes));
        Ok(bytecode)
    }

    #[doc = " Get storage value of address at index."]
    fn storage(&mut self, address: Address, index: U256) -> Result<U256, Self::Error> {
        let mut error_msg = UnmanagedVector::default();
        let mut output = UnmanagedVector::default();
        let go_error: GoError = (self.db.vtable.get_storage)(
            self.db.state,
            U8SliceView::new(Some(address.as_slice())),
            U8SliceView::new(Some(&index.to_be_bytes_vec())),
            &mut output as *mut UnmanagedVector,
            &mut error_msg as *mut UnmanagedVector,
        )
        .into();
        unsafe {
            go_error.into_result(error_msg, || "Failed to get storage from the db".to_owned())?;
        }
        let value_bytes = output.consume().unwrap();
        let value = U256::from_be_slice(value_bytes.as_slice());
        Ok(value)
    }

    #[doc = " Get block hash by block number."]
    fn block_hash(&mut self, number: u64) -> Result<BlockHash, Self::Error> {
        let mut error_msg = UnmanagedVector::default();
        let mut output = UnmanagedVector::default();
        let go_error: GoError = (self.db.vtable.get_block_hash)(
            self.db.state,
            number,
            &mut output as *mut UnmanagedVector,
            &mut error_msg as *mut UnmanagedVector,
        )
        .into();

        unsafe {
            go_error
                .into_result(error_msg, || "Failed to get block hash from the db".to_owned())?;
        }

        let block_hash = BlockHash::from_slice(&output.consume().unwrap());
        Ok(block_hash)
    }
}

impl<'a> DatabaseCommit for StateDB<'a> {
    #[doc = " Commit changes to the database."]
    fn commit(&mut self, changes: HashMap<Address, Account>) {
        let mut updated_codes: UpdatedCodes = HashMap::new();
        let mut updated_storages: UpdatedStorages = HashMap::new();
        let mut updated_accounts: UpdatedAccounts = HashMap::new();
        let mut deleted_accounts: DeletedAccounts = Vec::new();

        for (address, account) in changes {
            if !account.is_touched() {
                continue;
            }
            if account.is_selfdestructed() {
                // Update Deleted Accounts
                deleted_accounts.push(address);
                continue;
            }
            let is_newly_created = account.is_created();
            // Update Codes
            if is_newly_created && !account.info.is_empty_code_hash() {
                updated_codes.insert(
                    account.info.code_hash,
                    account.info.code.clone().unwrap().original_byte_slice().to_vec(),
                );
            }

            // Update Accounts
            updated_accounts.insert(address, account.clone().info.copy_without_code());

            // Update Storages
            let mut updated_storages_by_address = HashMap::new();
            for (key, evm_storage_slot) in account.storage {
                if evm_storage_slot.original_value != evm_storage_slot.present_value {
                    updated_storages_by_address.insert(key, evm_storage_slot.present_value);
                }
            }
            updated_storages.insert(address, updated_storages_by_address);
        }
        // Commited by ffi call in state database
        let mut error_msg = UnmanagedVector::default();
        let go_error: GoError = (self.db.vtable.commit)(
            self.db.state,
            updated_codes.try_into().unwrap(),
            updated_storages.try_into().unwrap(),
            updated_accounts.try_into().unwrap(),
            deleted_accounts.try_into().unwrap(),
            &mut error_msg as *mut UnmanagedVector,
        )
        .into();

        unsafe {
            let _ = go_error
                .into_result(error_msg, || "Failed to commit changes in the state db".to_owned());
        }
    }
}
