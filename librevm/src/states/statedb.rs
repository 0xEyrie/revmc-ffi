use alloy_primitives::{Address, BlockHash, Bytes, B256, U256};
use revm::{
    primitives::{Account, AccountInfo, Bytecode, HashMap},
    Database, DatabaseCommit,
};

use crate::{
    error::{BackendError, GoError},
    memory::{U8SliceView, UnmanagedVector},
    types::{DeletedAccounts, UpdatedAccounts, UpdatedStorages},
};

use super::vtable::Db;

pub struct StateDB<'r> {
    pub db: &'r Db,
}

impl<'r> StateDB<'r> {
    pub fn new(db: &'r Db) -> Self {
        StateDB { db }
    }
}

impl Database for StateDB<'_> {
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

impl DatabaseCommit for StateDB<'_> {
    #[doc = " Commit changes to the database."]
    fn commit(&mut self, changes: HashMap<Address, Account>) {
        let mut updated_storages: UpdatedStorages = HashMap::default();
        let mut updated_accounts: UpdatedAccounts = HashMap::default();
        let mut deleted_accounts: DeletedAccounts = Vec::default();

        for (address, account) in changes {
            if !account.is_touched() {
                continue;
            }
            if account.is_selfdestructed() {
                // Update Deleted Accounts
                deleted_accounts.push(address);
                continue;
            }
            let mut info = account.clone().info;
            if info.code.is_none() {
                info.code = Some(self.code_by_hash(info.code_hash).unwrap());
            }
            // Update Accounts
            updated_accounts.insert(address, info);

            // Update Storages
            let mut updated_storages_by_address = HashMap::default();
            for (key, evm_storage_slot) in account.storage {
                if evm_storage_slot.original_value != evm_storage_slot.present_value {
                    updated_storages_by_address.insert(key, evm_storage_slot.present_value);
                }
            }
            updated_storages.insert(address, updated_storages_by_address);
        }
        // Commited by ffi call in extended state database
        let mut error_msg = UnmanagedVector::default();
        let go_error: GoError = (self.db.vtable.commit)(
            self.db.state,
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
