use alloy_primitives::{ Address, B256, U256 };
use prost::{ DecodeError, Message };
use revm::primitives::{ BlobExcessGasAndPrice, BlockEnv };

use crate::{ memory::ByteSliceView, v1::types::Block };

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
            blob_excess_gas_and_price: if let Some(excess_blob_gas) = block.excess_blob_gas {
                if excess_blob_gas == 0 {
                    None
                } else {
                    Some(BlobExcessGasAndPrice::new(excess_blob_gas))
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
