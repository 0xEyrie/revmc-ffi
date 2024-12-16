use alloy_primitives::{Address, B256, U256};
use prost::{DecodeError, Message};
use revm::primitives::{BlobExcessGasAndPrice, BlockEnv};

use crate::{memory::ByteSliceView, v1::types::Block};

#[derive(Clone, Debug, PartialEq)]
pub struct BlockProto(Block);

impl From<Block> for BlockProto {
    fn from(inner: Block) -> Self {
        Self(inner)
    }
}

impl BlockProto {
    pub fn new(inner: Block) -> Self {
        Self(inner)
    }

    pub fn into_inner(self) -> Block {
        self.0
    }
}

impl TryFrom<BlockProto> for BlockEnv {
    type Error = Vec<u8>;
    fn try_from(block: BlockProto) -> Result<Self, Self::Error> {
        let block = block.into_inner();
        let blob_excess_gas_and_price = block.blob_excess_gas_and_price;
        let prevrandao = B256::from_slice(&block.prevrandao);
        let number = U256::from_be_slice(&block.number);
        Ok(Self {
            number,
            coinbase: Address::from_slice(&block.coinbase),
            timestamp: U256::from_be_slice(&block.timestamp),
            gas_limit: U256::from_be_slice(&block.gas_limit),
            basefee: U256::from_be_slice(&block.basefee),
            difficulty: U256::from_be_slice(&block.difficulty),
            prevrandao: match prevrandao {
                B256::ZERO => None,
                _ => Some(prevrandao),
            },
            blob_excess_gas_and_price: if let Some(blob_excess_gas_and_price) =
                blob_excess_gas_and_price
            {
                if blob_excess_gas_and_price.excess_blob_gas == 0
                    && blob_excess_gas_and_price
                        .blob_gasprice
                        .iter()
                        .all(|&b| b == 0)
                {
                    None
                } else {
                    Some(BlobExcessGasAndPrice {
                        excess_blob_gas: blob_excess_gas_and_price.excess_blob_gas,
                        blob_gasprice: u128::from_be_bytes(
                            blob_excess_gas_and_price.blob_gasprice.try_into().unwrap(),
                        ),
                    })
                }
            } else {
                None
            },
        })
    }
}

impl TryFrom<ByteSliceView> for BlockEnv {
    type Error = DecodeError;

    fn try_from(value: ByteSliceView) -> Result<Self, Self::Error> {
        let block_bytes = value.read().unwrap();
        Ok(BlockEnv::try_from(BlockProto::from(Block::decode(block_bytes).unwrap())).unwrap())
    }
}
