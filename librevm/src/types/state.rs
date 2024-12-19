use alloy_primitives::{ Address, B256, U256 };
use prost::{ EncodeError, Message };
use revm::primitives::{ AccountInfo, HashMap };

use crate::{
    memory::U8SliceView,
    v1::types::{ Account, Accounts, Codes, Deleted, Storage, Storages },
};

// Updated Accounts
pub type UpdatedAccounts = HashMap<Address, AccountInfo>;

impl TryFrom<UpdatedAccounts> for U8SliceView {
    type Error = EncodeError;

    fn try_from(value: UpdatedAccounts) -> Result<Self, Self::Error> {
        let accounts = Accounts {
            accounts: value
                .into_iter()
                .map(|(addr, acc)| {
                    let addr_str = addr.to_string();
                    (
                        addr_str,
                        Account {
                            balance: acc.balance.to_be_bytes_vec(),
                            nonce: acc.nonce,
                            code_hash: acc.code_hash.to_vec(),
                        },
                    )
                })
                .collect(),
        };
        // build proto message
        let mut buf = Vec::new();
        let _ = accounts.encode(&mut buf);
        Ok(U8SliceView::new(Some(&buf)))
    }
}

// Updated Codes
pub type UpdatedCodes = HashMap<B256, Code>;
pub type Code = Vec<u8>;

impl TryFrom<UpdatedCodes> for U8SliceView {
    type Error = EncodeError;

    fn try_from(value: UpdatedCodes) -> Result<Self, Self::Error> {
        let codes = Codes {
            codes: value
                .into_iter()
                .map(|(hash, code)| (hash.to_string(), code))
                .collect(),
        };
        // build proto message
        let mut buf = Vec::new();
        let _ = codes.encode(&mut buf);
        Ok(U8SliceView::new(Some(&buf)))
    }
}

// Storages
pub type UpdatedStorages = HashMap<Address, HashMap<U256, U256>>;

impl TryFrom<UpdatedStorages> for U8SliceView {
    type Error = EncodeError;

    fn try_from(value: UpdatedStorages) -> Result<Self, Self::Error> {
        let upated_storages = Storages {
            storages: value
                .into_iter()
                .map(|(addr, keys)| {
                    let addr_str = addr.to_string();
                    let storage = keys
                        .into_iter()
                        .map(|(key, value)| (key.to_string(), value.to_be_bytes_vec()))
                        .collect();
                    (addr_str, Storage { storage })
                })
                .collect(),
        };
        // build proto message
        let mut buf = Vec::new();
        upated_storages.encode(&mut buf).unwrap();
        Ok(U8SliceView::new(Some(&buf)))
    }
}

// Deleted Account
pub type DeletedAccounts = Vec<Address>;

impl TryFrom<DeletedAccounts> for U8SliceView {
    type Error = EncodeError;

    fn try_from(value: DeletedAccounts) -> Result<Self, Self::Error> {
        let deleted = Deleted {
            deleted: value
                .into_iter()
                .map(|addr| addr.to_vec())
                .collect(),
        };
        // build proto message
        let mut buf = Vec::new();
        deleted.encode(&mut buf).unwrap();
        Ok(U8SliceView::new(Some(&buf)))
    }
}
