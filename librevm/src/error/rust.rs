use errno::{set_errno, Errno};
use revm::primitives::{EVMError, InvalidTransaction};

use crate::memory::UnmanagedVector;

use super::BackendError;

#[repr(i32)]
pub enum ErrnoValue {
    #[allow(dead_code)]
    Success = 0,
    Other = 1,
}

pub fn set_error(err: EVMError<BackendError>, error_msg: Option<&mut UnmanagedVector>) {
    if let Some(error_msg) = error_msg {
        let msg: Vec<u8> = match err {
            EVMError::Transaction(err) => match err {
                InvalidTransaction::PriorityFeeGreaterThanMaxFee => {
                    b"Priority fee is greater than max fee".to_vec()
                }
                InvalidTransaction::GasPriceLessThanBasefee => {
                    b"Gas price is less than base fee".to_vec()
                }
                InvalidTransaction::CallerGasLimitMoreThanBlock => {
                    b"Caller gas limit is more than block gas limit".to_vec()
                }
                InvalidTransaction::CallGasCostMoreThanGasLimit => {
                    b"Call gas cost is more than gas limit".to_vec()
                }
                InvalidTransaction::RejectCallerWithCode => b"Caller rejected with code".to_vec(),
                InvalidTransaction::LackOfFundForMaxFee { fee, balance } => {
                    format!("Lack of fund for max fee: fee = {}, balance = {}", fee, balance)
                        .into_bytes()
                }
                InvalidTransaction::OverflowPaymentInTransaction => {
                    b"Overflow payment in transaction".to_vec()
                }
                InvalidTransaction::NonceOverflowInTransaction => {
                    b"Nonce overflow in transaction".to_vec()
                }
                InvalidTransaction::NonceTooHigh { tx, state } => {
                    format!("Nonce too high: tx = {}, state = {}", tx, state).into_bytes()
                }
                InvalidTransaction::NonceTooLow { tx, state } => {
                    format!("Nonce too low: tx = {}, state = {}", tx, state).into_bytes()
                }
                InvalidTransaction::CreateInitCodeSizeLimit => {
                    b"Create init code size limit exceeded".to_vec()
                }
                InvalidTransaction::InvalidChainId => b"Invalid chain ID".to_vec(),
                InvalidTransaction::AccessListNotSupported => b"Access list not supported".to_vec(),
                InvalidTransaction::MaxFeePerBlobGasNotSupported => {
                    b"Max fee per blob gas not supported".to_vec()
                }
                InvalidTransaction::BlobVersionedHashesNotSupported => {
                    b"Blob versioned hashes not supported".to_vec()
                }
                InvalidTransaction::BlobGasPriceGreaterThanMax => {
                    b"Blob gas price is greater than max".to_vec()
                }
                InvalidTransaction::EmptyBlobs => b"Empty blobs".to_vec(),
                InvalidTransaction::BlobCreateTransaction => b"Blob create transaction".to_vec(),
                InvalidTransaction::TooManyBlobs { max, have } => {
                    format!("Too many blobs: max = {}, have = {}", max, have).into_bytes()
                }
                InvalidTransaction::BlobVersionNotSupported => {
                    b"Blob version not supported".to_vec()
                }
                InvalidTransaction::EofCrateShouldHaveToAddress => {
                    b"EOF crate should have to address".to_vec()
                }
                InvalidTransaction::AuthorizationListNotSupported => {
                    b"Authorization list not supported".to_vec()
                }
                InvalidTransaction::AuthorizationListInvalidFields => {
                    b"Authorization list has invalid fields".to_vec()
                }
                InvalidTransaction::EmptyAuthorizationList => b"Empty authorization list".to_vec(),
            },
            EVMError::Header(invalid_header) => invalid_header.to_string().into(),
            EVMError::Database(err) => err.to_string().into(),
            EVMError::Custom(err) => err.into(),
            EVMError::Precompile(err) => err.into(),
        };
        *error_msg = UnmanagedVector::new(Some(msg));
    } else {
        // The caller provided a nil pointer for the error message.
        // That's not nice but we can live with it.
    }

    set_errno(Errno(ErrnoValue::Other as i32));
}
