use std::{future::Future, time};

use alloy_primitives::B256;
use revmc::eyre::{Context, Result};
use tokio::time::{interval_at, Instant};

use super::{key::QueryKey, LevelDB};
use crate::jit::{JitCfg, JitUnit, KeyPrefix, RuntimeJit};

const JIT_THRESHOLD: i32 = 10;

pub struct Cronner {
    // ms
    interval: u64,
    leveldb: LevelDB<'static, QueryKey>,
}

impl Cronner {
    pub fn new_with_db(interval: u64, leveldb: LevelDB<'static, QueryKey>) -> Self {
        Self { interval, leveldb }
    }

    pub fn routine_fn(&self) -> impl Future<Output = ()> + Send + 'static {
        let interval = self.interval.clone();
        let leveldb = self.leveldb.clone();

        async move {
            Cronner::cron(interval, leveldb).await;
        }
    }

    pub async fn cron(interval: u64, leveldb: LevelDB<'static, QueryKey>) {
        let start = Instant::now();
        let mut interval = interval_at(start, time::Duration::from_millis(interval));

        loop {
            interval.tick().await;
            println!("Cron loop...");

            for mut key in leveldb
                .key_iterator()
                .filter(|k| k.match_prefix(KeyPrefix::Count))
                .into_iter()
            {
                println!("Count Key: {key:#?}");
                let count_bytes = leveldb.get(key).unwrap_or(None);
                let count = count_bytes.as_ref().map_or(1, |v| {
                    let bytes: [u8; 4] = v.as_slice().try_into().unwrap_or([0, 0, 0, 0]);
                    i32::from_be_bytes(bytes)
                });

                if count > JIT_THRESHOLD {
                    println!("Over threshold for key: {:#?}, count: {:#?}", key, count);
                    key.update_prefix(KeyPrefix::Bytecode);
                    if let Some(bytecode) = leveldb.get(key).unwrap_or(None) {
                        let bytecode_hash = key.to_b256();
                        // leak for cast to static
                        let label = Cronner::mangle_hex(bytecode_hash.as_slice()).leak();

                        key.update_prefix(KeyPrefix::Label);
                        if let None = leveldb.get(key).unwrap_or(None) {
                            Cronner::jit(label, &bytecode, bytecode_hash).unwrap();
                        }
                    }
                    continue;
                }
            }
        }
    }

    pub fn jit(label: &'static str, bytecode: &[u8], bytecode_hash: B256) -> Result<()> {
        println!("Jit in progress for hash {:#?}...", bytecode_hash);
        let unit = JitUnit::new(label, bytecode.to_vec(), 70);
        let runtime_jit = RuntimeJit::new(unit, JitCfg::default());
        runtime_jit.compile().wrap_err("Compilation fail")
    }

    fn mangle_hex(hex: &[u8]) -> String {
        let hex_part: String = hex
            .iter()
            .take(3)
            .map(|byte| format!("{:02x}", byte))
            .collect();

        format!("_{}", hex_part)
    }
}