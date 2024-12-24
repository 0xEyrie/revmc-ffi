use alloy_primitives::{Address, Bytes, StorageKey, TxKind, B256, U256};
use prost::{DecodeError, Message};
use revm::primitives::{
    AccessListItem, Authorization, AuthorizationList, RecoveredAuthority, RecoveredAuthorization,
    SignedAuthorization, TxEnv,
};

use crate::{
    memory::ByteSliceView,
    v1::types::{authorization_list, Transaction},
};

#[derive(Clone, Debug, PartialEq)]
pub struct TransactionProto(Transaction);

impl From<Transaction> for TransactionProto {
    fn from(inner: Transaction) -> Self {
        Self(inner)
    }
}

impl TransactionProto {
    pub fn new(inner: Transaction) -> Self {
        Self(inner)
    }

    pub fn into_inner(self) -> Transaction {
        self.0
    }
}

impl From<TransactionProto> for TxEnv {
    fn from(transaction: TransactionProto) -> Self {
        let transaction = transaction.into_inner();
        let transact_to = Address::from_slice(&transaction.transact_to);
        Self {
            chain_id: None,
            caller: Address::from_slice(&transaction.caller),
            gas_limit: transaction.gas_limit,
            gas_price: U256::from_be_slice(&transaction.gas_price),
            nonce: Some(transaction.nonce),
            transact_to: match transact_to {
                Address::ZERO => TxKind::Create,
                _ => TxKind::Call(transact_to),
            },
            value: U256::from_be_slice(&transaction.value),
            data: Bytes::from(transaction.data),
            gas_priority_fee: Some(U256::from_be_slice(&transaction.gas_priority_fee)),
            access_list: transaction
                .access_list
                .iter()
                .map(|item| AccessListItem {
                    address: Address::from_slice(&item.address),
                    storage_keys: item
                        .storage_keys
                        .iter()
                        .map(|key| StorageKey::from_slice(&key.value))
                        .collect(),
                })
                .collect(),
            blob_hashes: transaction
                .blob_hashes
                .iter()
                .map(|hash| B256::from_slice(hash))
                .collect(),
            max_fee_per_blob_gas: Some(U256::from_be_slice(&transaction.max_fee_per_blob_gas)),
            authorization_list: if let Some(list) = transaction.authorization_list {
                match list.authorization_list {
                    Some(al) => match al {
                        authorization_list::AuthorizationList::Signed(
                            signed_authorization_list,
                        ) => Some(AuthorizationList::Signed(
                            signed_authorization_list
                                .signed
                                .iter()
                                .map(|sa| {
                                    let inner = sa.inner.clone().unwrap();
                                    SignedAuthorization::new_unchecked(
                                        Authorization {
                                            chain_id: inner.chain_id,
                                            address: Address::from_slice(&inner.address),
                                            nonce: inner.chain_id,
                                        },
                                        u8::from_be_bytes(sa.y_parity.clone().try_into().unwrap()),
                                        U256::from_be_slice(&sa.r),
                                        U256::from_be_slice(&sa.s),
                                    )
                                })
                                .collect(),
                        )),
                        authorization_list::AuthorizationList::Recovered(
                            recovered_authorization_list,
                        ) => Some(AuthorizationList::Recovered(
                            recovered_authorization_list
                                .recovered
                                .iter()
                                .map(|ra| {
                                    let inner = ra.inner.clone().unwrap();
                                    let authority = if ra.authority.is_empty() {
                                        RecoveredAuthority::Invalid
                                    } else {
                                        RecoveredAuthority::Valid(Address::from_slice(
                                            &ra.authority,
                                        ))
                                    };
                                    RecoveredAuthorization::new_unchecked(
                                        Authorization {
                                            chain_id: inner.chain_id,
                                            address: Address::from_slice(&inner.address),
                                            nonce: inner.nonce,
                                        },
                                        authority,
                                    )
                                })
                                .collect(),
                        )),
                    },

                    None => None,
                }
            } else {
                None
            },
        }
    }
}

impl TryFrom<ByteSliceView> for TxEnv {
    type Error = DecodeError;
    fn try_from(value: ByteSliceView) -> Result<Self, Self::Error> {
        let tx_bytes = value.read().unwrap();
        Ok(TxEnv::from(TransactionProto::from(Transaction::decode(tx_bytes).unwrap())))
    }
}
