mod insts;
mod rv64a;
mod rv64i;
mod rv64m;
// clock_cache removed: instruction cache not needed

use anyhow::{Ok, Result};
use nohash_hasher::BuildNoHashHasher;
use std::collections::HashMap;
use std::rc::Rc;

use crate::const_values::EmuConfig;
use crate::emulator::Emulator;
use crate::utils::bit_utils::{BitSlice, sign_extend_64};

#[derive(Debug, Clone, Copy, Hash)]
pub struct Instruction {
    pub mask: u32,
    pub identifier: u32,
    pub name: &'static str,
    pub execute: fn(emu: &mut Emulator, inst: u32, pc: u64) -> Result<()>,
}

pub struct InstDecoder {
    instructions_set: Vec<&'static Instruction>,
    compressed_instructions: Vec<Instruction>,
    #[allow(unused)]
    config: Rc<EmuConfig>,
    opcode_map: HashMap<u32, Vec<&'static Instruction>, BuildNoHashHasher<u32>>,
}

const MASK_OPCODE: u32 = 0x7F;

#[inline(always)]
pub fn is_compressed(inst: u32) -> bool {
    inst & 0b11 != 0b11
}

#[inline(always)]
pub fn is_inst_addr_misaligned(pc: u64) -> bool {
    pc & 0b11 != 0
}

impl InstDecoder {
    pub fn new(config: Rc<EmuConfig>) -> Self {
        let mut instructions_set: Vec<&'static Instruction> = vec![];
        let compressed_instructions = vec![];
        let mut opcode_map = HashMap::with_hasher(BuildNoHashHasher::default());

        instructions_set.extend(rv64i::RV_I);
        if config.inst_set.m_ext {
            instructions_set.extend(rv64m::RV_M);
        }
        if config.inst_set.a_ext {
            instructions_set.extend(rv64a::RV_A);
        }

        if config.inst_set.c_ext {
            todo!("Implement compressed instructions");
        }

        for inst in &instructions_set {
            let opcode = inst.identifier & MASK_OPCODE;
            let entry: &mut Vec<&'static Instruction> = opcode_map.entry(opcode).or_default();
            entry.push(inst);
        }
        InstDecoder {
            instructions_set,
            compressed_instructions,
            config,
            opcode_map,
        }
    }

    #[inline]
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
                Some(instruction) => {
                    // cache removed: directly return the instruction
                    Ok(instruction)
                }

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
        // instruction cache removed: always use slow_path
        self.slow_path(inst)
    }

}

struct FormatR {
    rs1: u64,
    rs2: u64,
    rd: u64,
}

impl FormatR {}

#[inline(always)]
fn parse_format_r(inst: u32) -> FormatR {
    let rs1 = inst.bit_range(15..20);
    let rs2 = inst.bit_range(20..25);
    let rd = inst.bit_range(7..12);
    FormatR { rs1, rs2, rd }
}

struct FormatI {
    rs1: u64,
    rd: u64,
    imm: u64,
}

impl FormatI {}

#[inline(always)]
fn parse_format_i(inst: u32) -> FormatI {
    let rs1 = inst.bit_range(15..20);
    let rd = inst.bit_range(7..12);
    let imm = inst.bit_range(20..32);
    // 符号扩展
    let imm = sign_extend_64(imm, 12);
    FormatI { rs1, rd, imm }
}

struct FormatS {
    rs1: u64,
    rs2: u64,
    imm: u64,
}

impl FormatS {}

#[inline(always)]
fn parse_format_s(inst: u32) -> FormatS {
    let rs1 = inst.bit_range(15..20);
    let rs2 = inst.bit_range(20..25);
    let imm = inst.bit_range(25..32) << 5 | inst.bit_range(7..12);
    // 符号扩展
    let imm = sign_extend_64(imm, 12);
    FormatS { rs1, rs2, imm }
}

struct FormatB {
    rs1: u64,
    rs2: u64,
    imm: u64,
}

impl FormatB {}

#[inline(always)]
fn parse_format_b(inst: u32) -> FormatB {
    let rs1 = inst.bit_range(15..20);
    let rs2 = inst.bit_range(20..25);
    let imm = (inst.bit(31) as u64) << 12
        | (inst.bit(7) as u64) << 11
        | inst.bit_range(25..31) << 5
        | inst.bit_range(8..12) << 1;
    // 符号扩展
    let imm = sign_extend_64(imm, 13);
    FormatB { rs1, rs2, imm }
}

struct FormatU {
    rd: u64,
    imm: u64,
}

impl FormatU {}

#[inline(always)]
fn parse_format_u(inst: u32) -> FormatU {
    let imm = inst.bit_range(12..32) << 12;
    let rd = inst.bit_range(7..12);
    // 符号扩展
    let imm = sign_extend_64(imm, 32);
    FormatU { rd, imm }
}

struct FormatJ {
    rd: u64,
    imm: u64,
}

impl FormatJ {}

#[inline(always)]
fn parse_format_j(inst: u32) -> FormatJ {
    let rd = inst.bit_range(7..12);
    let imm = (inst.bit(31) as u64) << 20
        | inst.bit_range(12..20) << 12
        | (inst.bit(20) as u64) << 11
        | inst.bit_range(21..31) << 1;
    // 符号扩展
    let imm = sign_extend_64(imm, 21);
    FormatJ { rd, imm }
}
