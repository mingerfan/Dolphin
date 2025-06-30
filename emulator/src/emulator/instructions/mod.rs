mod insts;
mod rv64a;
mod rv64i;
mod rv64m;

use crate::const_values;
use crate::emulator::Emulator;
use anyhow::{Ok, Result};
use clap::Args;
use core::panic;
use hashlink::LruCache;
use nohash_hasher::{self, BuildNoHashHasher};
use std::collections::HashMap;

#[derive(Args, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct InstDecoderArgs {
    enable_m: bool, // 是否启用M扩展
    enable_a: bool, // 是否启用A扩展
    enable_c: bool, // 是否启用C扩展
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Instruction {
    pub mask: u32,
    pub identifier: u32,
    pub name: &'static str,
    pub execute: fn(emu: &mut Emulator, inst: u32, pc: u64) -> Result<()>,
}

pub struct InstDecoder {
    cache: LruCache<u32, Instruction, BuildNoHashHasher<u32>>,
    instructions_set: Vec<Instruction>,
    compressed_instructions: Vec<Instruction>,
    config: InstDecoderArgs,
    opcode_map: HashMap<u32, Vec<Instruction>, BuildNoHashHasher<u32>>,
}

const MASK_OPCODE: u32 = 0x7F;

#[inline(always)]
pub fn is_compressed(inst: u32) -> bool {
    inst & 0b11 != 0b11
}

impl InstDecoder {
    pub fn new(args: &InstDecoderArgs) -> Self {
        let cache = LruCache::with_hasher(
            const_values::DECODER_LRU_CACHE_SIZE,
            BuildNoHashHasher::default(),
        );
        let mut instructions_set = vec![];
        let compressed_instructions = vec![];
        let mut opcode_map = HashMap::with_hasher(BuildNoHashHasher::default());

        instructions_set.extend_from_slice(rv64i::RV_I);

        if args.enable_m {
            instructions_set.extend_from_slice(rv64m::RV_M);
        }
        if args.enable_a {
            instructions_set.extend_from_slice(rv64a::RV_A);
        }

        if args.enable_c {
            todo!("Implement compressed instructions");
        }

        for inst in &instructions_set {
            let opcode = inst.identifier & MASK_OPCODE;
            let entry: &mut Vec<Instruction> = opcode_map.entry(opcode).or_default();
            entry.push(*inst);
        }
        InstDecoder {
            cache,
            instructions_set,
            compressed_instructions,
            config: args.clone(),
            opcode_map,
        }
    }

    #[inline(always)]
    pub fn cache_has_capacity(&self) -> bool {
        self.cache.capacity() != 0
    }

    #[inline(always)]
    pub fn slow_path(&mut self, inst: u32) -> Result<&Instruction> {
        if is_compressed(inst) {
            self.compressed_instructions
                .iter()
                .find(|&&x| x.mask & inst == x.identifier)
                .ok_or(anyhow::anyhow!("Compressed instruction not found"))
        } else {
            // 提取 opcode
            let opcode = inst & MASK_OPCODE;

            // 尝试在优化过的 opcode_map 中查找
            let maybe_instruction = self.opcode_map.get(&opcode).and_then(|instructions| {
                instructions
                    .iter()
                    .find(|&&x| x.mask & inst == x.identifier)
            });

            // 根据查找结果进行处理
            match maybe_instruction {
                // 1. 在 opcode_map 中成功找到，这是最理想的情况
                Some(instruction) => Ok(instruction),

                // 2. 在 opcode_map 中未找到，需要进一步检查以确定是真错误还是状态不一致
                None => {
                    // 检查指令是否存在于完整的指令集中，以判断是否为数据结构不一致的 panic 情况
                    if self
                        .instructions_set
                        .iter()
                        .any(|&x| x.mask & inst == x.identifier)
                    {
                        // 如果在这里找到了，说明 opcode_map 构建有误，这是一个不可恢复的逻辑错误
                        panic!(
                            "Instruction found in instructions_set but not in its opcode_map bucket: {:#010x}",
                            inst
                        );
                    } else {
                        // 如果完整的指令集中也没有，说明这是一个合法的“未找到”错误
                        Err(anyhow::anyhow!("Instruction not found: {:#010x}", inst))
                    }
                }
            }
        }
    }

    #[inline(always)]
    pub fn fast_path(&mut self, inst: u32) -> Result<&Instruction> {
        if is_compressed(inst) || !self.cache_has_capacity() {
            let temp = self.slow_path(inst);
            return temp;
        }
        self.cache
            .get(&inst)
            .ok_or(anyhow::anyhow!("Instruction not found in cache"))
    }
}
