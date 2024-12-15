use alloy_primitives::Address;
use prost::{EncodeError, Message};
use revm::primitives::ExecutionResult;

use crate::v1::types::{
    evm_result::Result as ResultType, Call, Create, EvmResult, Halt, HaltReasonEnum, Log, LogData,
    Output, Revert, Success, SuccessReasonEnum, Topic,
};

pub trait TryIntoVec {
    type Error;

    fn try_into_vec(self) -> Result<Vec<u8>, Self::Error>;
}

impl TryIntoVec for ExecutionResult {
    type Error = EncodeError;

    fn try_into_vec(self) -> Result<Vec<u8>, Self::Error> {
        let evm_result = EvmResult {
            result: match self {
                ExecutionResult::Success {
                    reason,
                    gas_used,
                    gas_refunded,
                    logs,
                    output,
                } => Some(ResultType::Success(Success {
                    reason: match reason {
                        revm::primitives::SuccessReason::Stop => SuccessReasonEnum::Stop.into(),
                        revm::primitives::SuccessReason::Return => SuccessReasonEnum::Return.into(),
                        revm::primitives::SuccessReason::SelfDestruct => {
                            SuccessReasonEnum::SelfDestruct.into()
                        }
                        revm::primitives::SuccessReason::EofReturnContract => {
                            SuccessReasonEnum::EofReturnContract.into()
                        }
                    },
                    gas_used,
                    gas_refunded,
                    logs: logs
                        .into_iter()
                        .map(|log| Log {
                            address: log.address.to_vec(),
                            data: Some(LogData {
                                topics: log
                                    .data
                                    .topics()
                                    .iter()
                                    .map(|topic| Topic {
                                        value: topic.to_vec(),
                                    })
                                    .collect(),
                                data: log.data.data.to_vec(),
                            }),
                        })
                        .collect(),
                    output: match output {
                        revm::primitives::Output::Call(bytes) => Some(Output {
                            output: Some(crate::v1::types::output::Output::Call(Call {
                                call: bytes.to_vec(),
                            })),
                        }),
                        revm::primitives::Output::Create(bytes, address) => Some(Output {
                            output: Some(crate::v1::types::output::Output::Create(Create {
                                create: bytes.to_vec(),
                                created_address: match address {
                                    Some(addr) => addr.to_vec(),
                                    None => Address::ZERO.to_vec(),
                                },
                            })),
                        }),
                    },
                })),
                ExecutionResult::Revert { gas_used, output } => Some(ResultType::Revert(Revert {
                    gas_used,
                    output: output.to_vec(),
                })),
                ExecutionResult::Halt { reason, gas_used } => Some(ResultType::Halt(Halt {
                    reason: match reason {
                        revm::primitives::HaltReason::OutOfGas(out_of_gas_error) => {
                            match out_of_gas_error {
                                revm::primitives::OutOfGasError::Basic => {
                                    HaltReasonEnum::OutOfGasBasic.into()
                                }
                                revm::primitives::OutOfGasError::MemoryLimit => {
                                    HaltReasonEnum::OutOfGasMemoryLimit.into()
                                }
                                revm::primitives::OutOfGasError::Memory => {
                                    HaltReasonEnum::OutOfGasMemory.into()
                                }
                                revm::primitives::OutOfGasError::Precompile => {
                                    HaltReasonEnum::OutOfGasPrecompile.into()
                                }
                                revm::primitives::OutOfGasError::InvalidOperand => {
                                    HaltReasonEnum::OutOfGasInvalidOperand.into()
                                }
                            }
                        }
                        revm::primitives::HaltReason::OpcodeNotFound => {
                            HaltReasonEnum::OpcodeNotFound.into()
                        }
                        revm::primitives::HaltReason::InvalidFEOpcode => {
                            HaltReasonEnum::InvalidFeOpcode.into()
                        }
                        revm::primitives::HaltReason::InvalidJump => {
                            HaltReasonEnum::InvalidJump.into()
                        }
                        revm::primitives::HaltReason::NotActivated => {
                            HaltReasonEnum::NotActivated.into()
                        }
                        revm::primitives::HaltReason::StackUnderflow => {
                            HaltReasonEnum::StackUnderflow.into()
                        }
                        revm::primitives::HaltReason::StackOverflow => {
                            HaltReasonEnum::StackOverflow.into()
                        }
                        revm::primitives::HaltReason::OutOfOffset => {
                            HaltReasonEnum::OutOfOffset.into()
                        }
                        revm::primitives::HaltReason::CreateCollision => {
                            HaltReasonEnum::CreateCollision.into()
                        }
                        revm::primitives::HaltReason::PrecompileError => {
                            HaltReasonEnum::PrecompileError.into()
                        }
                        revm::primitives::HaltReason::NonceOverflow => {
                            HaltReasonEnum::NonceOverflow.into()
                        }
                        revm::primitives::HaltReason::CreateContractSizeLimit => {
                            HaltReasonEnum::CreateContractSizeLimit.into()
                        }
                        revm::primitives::HaltReason::CreateContractStartingWithEF => {
                            HaltReasonEnum::CreateContractStartingWithEf.into()
                        }
                        revm::primitives::HaltReason::CreateInitCodeSizeLimit => {
                            HaltReasonEnum::CreateInitCodeSizeLimit.into()
                        }
                        revm::primitives::HaltReason::OverflowPayment => {
                            HaltReasonEnum::OverflowPayment.into()
                        }
                        revm::primitives::HaltReason::StateChangeDuringStaticCall => {
                            HaltReasonEnum::StateChangeDuringStaticCall.into()
                        }
                        revm::primitives::HaltReason::CallNotAllowedInsideStatic => {
                            HaltReasonEnum::CallNotAllowedInsideStatic.into()
                        }
                        revm::primitives::HaltReason::OutOfFunds => {
                            HaltReasonEnum::OutOfFunds.into()
                        }
                        revm::primitives::HaltReason::CallTooDeep => {
                            HaltReasonEnum::CallTooDeep.into()
                        }
                        revm::primitives::HaltReason::EofAuxDataOverflow => {
                            HaltReasonEnum::EofAuxDataOverflow.into()
                        }
                        revm::primitives::HaltReason::EofAuxDataTooSmall => {
                            HaltReasonEnum::EofAuxDataTooSmall.into()
                        }
                        revm::primitives::HaltReason::EOFFunctionStackOverflow => {
                            HaltReasonEnum::EofFunctionStackOverflow.into()
                        }
                        revm::primitives::HaltReason::InvalidEXTCALLTarget => {
                            HaltReasonEnum::InvalidExtcallTarget.into()
                        }
                    },
                    gas_used,
                })),
            },
        };
        let mut buf = Vec::new();
        evm_result.encode(&mut buf).unwrap();
        Ok(buf)
    }
}
