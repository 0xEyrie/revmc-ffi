/// Returns the length of the data after compression through FastLZ, based on
// https://github.com/Vectorized/solady/blob/5315d937d79b335c668896d7533ac603adac5315/js/solady.js
pub(crate) fn flz_compress_len(input: &[u8]) -> u32 {
    let mut idx: u32 = 2;

    let idx_limit: u32 = if input.len() < 13 {
        0
    } else {
        input.len() as u32 - 13
    };

    let mut anchor = 0;

    let mut size = 0;

    let mut htab = [0; 8192];

    while idx < idx_limit {
        let mut r: u32;
        let mut distance: u32;

        loop {
            let seq = u24(input, idx);
            let hash = hash(seq);
            r = htab[hash as usize];
            htab[hash as usize] = idx;
            distance = idx - r;
            if idx >= idx_limit {
                break;
            }
            idx += 1;
            if distance < 8192 && seq == u24(input, r) {
                break;
            }
        }

        if idx >= idx_limit {
            break;
        }

        idx -= 1;

        if idx > anchor {
            size = literals(idx - anchor, size);
        }

        let len = cmp(input, r + 3, idx + 3, idx_limit + 9);
        size = flz_match(len, size);

        idx = set_next_hash(&mut htab, input, idx + len);
        idx = set_next_hash(&mut htab, input, idx);
        anchor = idx;
    }

    literals(input.len() as u32 - anchor, size)
}

fn literals(r: u32, size: u32) -> u32 {
    let size = size + 0x21 * (r / 0x20);
    let r = r % 0x20;
    if r != 0 {
        size + r + 1
    } else {
        size
    }
}

fn cmp(input: &[u8], p: u32, q: u32, r: u32) -> u32 {
    let mut l = 0;
    let mut r = r - q;
    while l < r {
        if input[(p + l) as usize] != input[(q + l) as usize] {
            r = 0;
        }
        l += 1;
    }
    l
}

fn flz_match(l: u32, size: u32) -> u32 {
    let l = l - 1;
    let size = size + (3 * (l / 262));
    if l % 262 >= 6 {
        size + 3
    } else {
        size + 2
    }
}

fn set_next_hash(htab: &mut [u32; 8192], input: &[u8], idx: u32) -> u32 {
    htab[hash(u24(input, idx)) as usize] = idx;
    idx + 1
}

fn hash(v: u32) -> u16 {
    let hash = (v as u64 * 2654435769) >> 19;
    hash as u16 & 0x1fff
}

fn u24(input: &[u8], idx: u32) -> u32 {
    u32::from(input[idx as usize])
        + (u32::from(input[(idx + 1) as usize]) << 8)
        + (u32::from(input[(idx + 2) as usize]) << 16)
}

#[cfg(test)]
mod tests {
    use crate::wiring::OptimismEvmWiring;
    use crate::OpTransaction;

    use super::*;
    use alloy_sol_types::sol;
    use alloy_sol_types::SolCall;
    use database::BenchmarkDB;
    use revm::{
        bytecode::Bytecode,
        primitives::{address, bytes, Bytes, TxKind, U256},
        Evm,
    };
    use std::vec::Vec;

    use rstest::rstest;

    #[rstest]
    #[case::empty(&[], 0)]
    #[case::thousand_zeros(&[0; 1000], 21)]
    #[case::thousand_forty_twos(&[42; 1000], 21)]
    #[case::short_hex(&bytes!("FACADE"), 4)]
    #[case::sample_contract_call(&bytes!("02f901550a758302df1483be21b88304743f94f80e51afb613d764fa61751affd3313c190a86bb870151bd62fd12adb8e41ef24f3f000000000000000000000000000000000000000000000000000000000000006e000000000000000000000000af88d065e77c8cc2239327c5edb3a432268e5831000000000000000000000000000000000000000000000000000000000003c1e5000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000a000000000000000000000000000000000000000000000000000000000000000148c89ed219d02f1a5be012c689b4f5b731827bebe000000000000000000000000c001a033fd89cb37c31b2cba46b6466e040c61fc9b2a3675a7f5f493ebd5ad77c497f8a07cdf65680e238392693019b4092f610222e71b7cec06449cb922b93b6a12744e"), 202)]
    #[case::base_0x5dadeb52979f29fc7a7494c43fdabc5be1d8ff404f3aafe93d729fa8e5d00769(&bytes!("b9047c02f904788221050883036ee48409c6c87383037f6f941195cf65f83b3a5768f3c496d3a05ad6412c64b78644364c5bb000b90404d123b4d80000000000000000000000000000000000000000000000000000000000000040000000000000000000000000000000000000000000000000000000000000038000000000000000000000000000000000f6476f90447748c19248ccaa31e6b8bfda4eb9d830f5f47df7f0998f7c2123d9e6137761b75d3184efb0f788e3b14516000000000000000000000000000000000000000000000000000044364c5bb000000000000000000000000000f38e53bd45c8225a7c94b513beadaa7afe5d222d0000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000024000000000000000000000000000000000000000000000000000000000000002a000000000000000000000000000000000000000000000000000000000000002c000000000000000000000000000000000000000000000000000000000000002e0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000030000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000084d6574614d61736b0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000035697066733a2f2f516d656852577a743347745961776343347564745657557233454c587261436746434259416b66507331696f48610000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000001000000000000000000000000cd0d83d9e840f8e27d5c2e365fd365ff1c05b2480000000000000000000000000000000000000000000000000000000000000ce40000000000000000000000000000000000000000000000000000000000000041e4480d358dbae20880960a0a464d63b06565a0c9f9b1b37aa94b522247b23ce149c81359bf4239d1a879eeb41047ec710c15f5c0f67453da59a383e6abd742971c00000000000000000000000000000000000000000000000000000000000000c001a0b57f0ff8516ea29cb26a44ac5055a5420847d1e16a8e7b03b70f0c02291ff2d5a00ad3771e5f39ccacfff0faa8c5d25ef7a1c179f79e66e828ffddcb994c8b512e"), 471)]
    fn test_flz_compress_len(#[case] input: &[u8], #[case] expected: u32) {
        assert_eq!(flz_compress_len(input), expected);
    }

    #[test]
    fn test_flz_compress_len_no_repeats() {
        let mut input = Vec::new();
        let mut len = 0;

        for i in 0..256 {
            input.push(i as u8);
            let prev_len = len;
            len = flz_compress_len(&input);
            assert!(len > prev_len);
        }
    }

    #[rstest]
    #[case::short_hex(bytes!("FACADE"))]
    #[case::sample_contract_call(bytes!("02f901550a758302df1483be21b88304743f94f80e51afb613d764fa61751affd3313c190a86bb870151bd62fd12adb8e41ef24f3f000000000000000000000000000000000000000000000000000000000000006e000000000000000000000000af88d065e77c8cc2239327c5edb3a432268e5831000000000000000000000000000000000000000000000000000000000003c1e5000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000a000000000000000000000000000000000000000000000000000000000000000148c89ed219d02f1a5be012c689b4f5b731827bebe000000000000000000000000c001a033fd89cb37c31b2cba46b6466e040c61fc9b2a3675a7f5f493ebd5ad77c497f8a07cdf65680e238392693019b4092f610222e71b7cec06449cb922b93b6a12744e"))]
    #[case::base_0x5dadeb52979f29fc7a7494c43fdabc5be1d8ff404f3aafe93d729fa8e5d00769(bytes!("b9047c02f904788221050883036ee48409c6c87383037f6f941195cf65f83b3a5768f3c496d3a05ad6412c64b78644364c5bb000b90404d123b4d80000000000000000000000000000000000000000000000000000000000000040000000000000000000000000000000000000000000000000000000000000038000000000000000000000000000000000f6476f90447748c19248ccaa31e6b8bfda4eb9d830f5f47df7f0998f7c2123d9e6137761b75d3184efb0f788e3b14516000000000000000000000000000000000000000000000000000044364c5bb000000000000000000000000000f38e53bd45c8225a7c94b513beadaa7afe5d222d0000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000024000000000000000000000000000000000000000000000000000000000000002a000000000000000000000000000000000000000000000000000000000000002c000000000000000000000000000000000000000000000000000000000000002e0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000030000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000084d6574614d61736b0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000035697066733a2f2f516d656852577a743347745961776343347564745657557233454c587261436746434259416b66507331696f48610000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000001000000000000000000000000cd0d83d9e840f8e27d5c2e365fd365ff1c05b2480000000000000000000000000000000000000000000000000000000000000ce40000000000000000000000000000000000000000000000000000000000000041e4480d358dbae20880960a0a464d63b06565a0c9f9b1b37aa94b522247b23ce149c81359bf4239d1a879eeb41047ec710c15f5c0f67453da59a383e6abd742971c00000000000000000000000000000000000000000000000000000000000000c001a0b57f0ff8516ea29cb26a44ac5055a5420847d1e16a8e7b03b70f0c02291ff2d5a00ad3771e5f39ccacfff0faa8c5d25ef7a1c179f79e66e828ffddcb994c8b512e"))]
    #[case::base_0xfaada76a2dac09fc17f5a28d066aaabefc6d82ef6589b211ed8c9f766b070721(bytes!("b87602f873822105528304320f8409cfe5c98252089480c67432656d59144ceff962e8faf8926599bcf888011dfe52d06b633f80c001a08632f069f837aea7a28bab0affee14dda116956bd5a850a355c045d25afedd17a0084b8f273efffe17ece527116053e5781a4915ff89ab9c379f1e62c25b697687"))]
    #[case::base_0x112864e9b971af6a1dac840018833c5a5a659acc187cfdaba919ad1da013678d(bytes!("b8b302f8b0822105308304320f8409cfe5c9827496944ed4e862860bed51a9570b96d89af5e1b0efefed80b844095ea7b3000000000000000000000000000000000022d473030f116ddee9f6b43ac78ba300000000000000000000000000000000000000000000015e10fb0973595fffffc001a02020e39f07917c1a852feb131c857e12478c7e88a20772b91a8bf5cee38c5aeea06055981727f9aaa3471c1af800555b35a77916c154be3f9d02ad1a63029455ab"))]
    #[case::base_0x6905051352691641888d0c427fb137c5b95afb5870d5169ff014eff1d0952195(bytes!("b87202f86f8221058303dc6c8310db1f84068fa8d7838954409436af2ff952a7355c8045fcd5e88bc9f6c8257f7b8080c001a0b89e7ff3d7694109e73e7f4244e032581670313c36e48e485c9c94b853bd81d2a038ffaf8f10859ce21d1f7f7046c3d08027fb8aa15b69038f6102be97aaa1179a"))]
    #[case::base_0x6a38e9a26d7202a2268de69d2d47531c1a9829867579a483fb48d78e9e0b080d(bytes!("b9049b02f904978221058201618506fc23ac008506fc23ac008306ddd0943fc91a3afd70395cd496c647d5a6cc9d4b2b7fad80b904243593564c000000000000000000000000000000000000000000000000000000000000006000000000000000000000000000000000000000000000000000000000000000a0000000000000000000000000000000000000000000000000000000006641d67b00000000000000000000000000000000000000000000000000000000000000030a000c00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000003000000000000000000000000000000000000000000000000000000000000006000000000000000000000000000000000000000000000000000000000000001e000000000000000000000000000000000000000000000000000000000000003000000000000000000000000000000000000000000000000000000000000000160000000000000000000000000088487bd8c3222d64d1d0b3fa7098dcf9d94d79e000000000000000000000000ffffffffffffffffffffffffffffffffffffffff000000000000000000000000000000000000000000000000000000006669635d00000000000000000000000000000000000000000000000000000000000000000000000000000000000000003fc91a3afd70395cd496c647d5a6cc9d4b2b7fad000000000000000000000000000000000000000000000000000000006641d78900000000000000000000000000000000000000000000000000000000000000e000000000000000000000000000000000000000000000000000000000000000418661369ca026f92ff88347bd0e3625a7b5ed65071b366368c68ad7c55aed136c18659b34f9246e30a784227a53dd374fbd3d2124696808c678cd987c4e954a681b000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000549e5c020c764dbfffff00000000000000000000000000000000000000000000000002e5a629c093a2b600000000000000000000000000000000000000000000000000000000000000a00000000000000000000000000000000000000000000000000000000000000001000000000000000000000000000000000000000000000000000000000000002b088487bd8c3222d64d1d0b3fa7098dcf9d94d79e0027104200000000000000000000000000000000000006000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000004000000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000000c001a014a3acef764ff6d3bb9bd81e420bfa94171a5734ab997dfbc9b41b653ce018a4a01ff5fccb01ef5c60ba3aef67d4e74f3f47312dd78bfbbff9e5090fbf2d3d62bb"))]
    fn test_flz_native_evm_parity(#[case] input: Bytes) {
        // This bytecode and ABI is for a contract, which wraps the LibZip library for easier fuzz testing.
        // The source of this contract is here: https://github.com/danyalprout/fastlz/blob/main/src/FastLz.sol#L6-L10
        sol! {
            interface FastLz {
                function fastLz(bytes input) external view returns (uint256);
            }
        }

        let contract_bytecode = Bytecode::new_raw(bytes!("608060405234801561001057600080fd5b506004361061002b5760003560e01c8063920a769114610030575b600080fd5b61004361003e366004610374565b610055565b60405190815260200160405180910390f35b600061006082610067565b5192915050565b60606101e0565b818153600101919050565b600082840393505b838110156100a25782810151828201511860001a1590930292600101610081565b9392505050565b825b602082106100d75782516100c0601f8361006e565b5260209290920191601f19909101906021016100ab565b81156100a25782516100ec600184038361006e565b520160010192915050565b60006001830392505b61010782106101385761012a8360ff1661012560fd6101258760081c60e0018961006e565b61006e565b935061010682039150610100565b600782106101655761015e8360ff16610125600785036101258760081c60e0018961006e565b90506100a2565b61017e8360ff166101258560081c8560051b018761006e565b949350505050565b80516101d890838303906101bc90600081901a600182901a60081b1760029190911a60101b17639e3779b90260131c611fff1690565b8060021b6040510182815160e01c1860e01b8151188152505050565b600101919050565b5060405161800038823961800081016020830180600d8551820103826002015b81811015610313576000805b50508051604051600082901a600183901a60081b1760029290921a60101b91909117639e3779b9810260111c617ffc16909101805160e081811c878603811890911b9091189091528401908183039084841061026857506102a3565b600184019350611fff821161029d578251600081901a600182901a60081b1760029190911a60101b17810361029d57506102a3565b5061020c565b8383106102b1575050610313565b600183039250858311156102cf576102cc87878886036100a9565b96505b6102e3600985016003850160038501610079565b91506102f08782846100f7565b9650506103088461030386848601610186565b610186565b915050809350610200565b5050617fe061032884848589518601036100a9565b03925050506020820180820383525b81811161034e57617fe08101518152602001610337565b5060008152602001604052919050565b634e487b7160e01b600052604160045260246000fd5b60006020828403121561038657600080fd5b813567ffffffffffffffff8082111561039e57600080fd5b818401915084601f8301126103b257600080fd5b8135818111156103c4576103c461035e565b604051601f8201601f19908116603f011681019083821181831017156103ec576103ec61035e565b8160405282815287602084870101111561040557600080fd5b82602086016020830137600092810160200192909252509594505050505056fea264697066735822122000646b2953fc4a6f501bd0456ac52203089443937719e16b3190b7979c39511264736f6c63430008190033"));

        let native_val = flz_compress_len(&input);

        let mut evm = Evm::<OptimismEvmWiring<BenchmarkDB, ()>>::builder()
            .with_db(BenchmarkDB::new_bytecode(contract_bytecode.clone()))
            .with_default_ext_ctx()
            .modify_tx_env(|tx| {
                let OpTransaction::Base { tx, enveloped_tx } = tx else {
                    panic!("Default is base tx");
                };
                tx.caller = address!("1000000000000000000000000000000000000000");
                tx.transact_to = TxKind::Call(address!("0000000000000000000000000000000000000000"));
                tx.data = FastLz::fastLzCall::new((input,)).abi_encode().into();
                tx.gas_limit = 300_000;
                *enveloped_tx = Some(Bytes::default());
            })
            .build();

        let result_and_state = evm.transact().unwrap();
        let output = result_and_state.result.output().unwrap();
        let evm_val = FastLz::fastLzCall::abi_decode_returns(output, true)
            .unwrap()
            ._0;

        assert_eq!(U256::from(native_val), evm_val);
    }
}
