use alloy_primitives::{B256, U256};
use prost::{DecodeError, Message};
use revm::primitives::AccountInfo;

use crate::{memory::UnmanagedVector, v1::types::Account};

impl TryFrom<UnmanagedVector> for AccountInfo {
    type Error = DecodeError;

    fn try_from(value: UnmanagedVector) -> Result<Self, Self::Error> {
        let account_bytes = value.consume().unwrap();
        let account = Account::decode(account_bytes.as_slice()).unwrap();
        Ok(AccountInfo {
            balance: U256::from_be_slice(&account.balance),
            nonce: account.nonce,
            code_hash: B256::from_slice(&account.code_hash),
            code: None,
        })
    }
}
