//! CPU状态管理

use super::memory::{Memory, MemoryError};
use crate::utils::disasm::RiscvDisassembler;
use anyhow::Result;
use std::fmt;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum StateError {
    #[error("寄存器访问错误: 寄存器 x{0} 超出范围")]
    InvalidRegister(usize),
    #[error("CSR访问错误: CSR {0:#x} 未找到")]
    InvalidCsr(u16),
    #[error("内存错误: {0}")]
    Memory(#[from] MemoryError),
    #[error("指令错误: 无效的指令字节, pc={0:#x}")]
    InvalidInstructionBytes(u64),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ExecMode {
    #[default]
    None, // 无执行模式
    Step, // 单步执行
    Continue, // 连续执行
    RangeStep(u64, u64), // 范围单步执行
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[non_exhaustive]
pub enum ExecState {
    #[default]
    Idle,
    Running,
    End,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[non_exhaustive]
pub enum Event {
    #[default]
    None,
    Halted,
    Break,
    WatchWrite(u64),
    WatchRead(u64),
}

/// CPU状态
#[derive(Debug, Clone)]
pub struct State {
    // 通用寄存器
    pub registers: [u64; 32],
    // 程序计数器
    pub pc: u64,
    // npc
    pub npc: u64,
    // CSR寄存器
    pub csrs: rustc_hash::FxHashMap<u16, u64>,
    // 内存
    pub memory: Memory,
}

impl State {
    /// 创建新的CPU状态
    pub fn new(memory_size: usize) -> Result<Self> {
        Ok(Self {
            registers: [0; 32],
            pc: 0x80000000,
            npc: 0x80000000,
            csrs: rustc_hash::FxHashMap::default(),
            memory: Memory::new(memory_size)?,
        })
    }

    /// 读取内存
    #[inline(always)]
    pub fn read_memory(&self, addr: u64, size: usize) -> Result<Vec<u8>> {
        Ok(self.memory.read(addr, size)?)
    }

    /// 写入内存
    #[inline(always)]
    pub fn write_memory(&mut self, addr: u64, data: &[u8]) -> Result<()> {
        Ok(self.memory.write(addr, data)?)
    }

    /// 取指令
    #[inline(always)]
    pub fn fetch_instruction(&self, pc: u64) -> Result<u32> {
        let bytes = self
            .read_memory(pc, 4)?;
        Ok(bytes
            .try_into()
            .map(u32::from_le_bytes)
            .map_err(|_| StateError::InvalidInstructionBytes(pc))?)
    }

    #[inline(always)]
    pub fn get_regs(&self) -> &[u64; 32] {
        &self.registers
    }

    /// 获取寄存器值
    /// #[inline(always)]
    pub fn get_reg(&self, reg: u64) -> Result<u64> {
        let reg = reg as usize;
        if reg >= self.registers.len() {
            return Err(StateError::InvalidRegister(reg).into());
        }
        Ok(if reg == 0 {
            0 // x0 永远是0
        } else {
            self.registers[reg]
        })
    }

    /// 设置寄存器值
    pub fn set_reg(&mut self, reg: u64, value: u64) -> Result<()> {
        let reg = reg as usize;
        if reg >= self.registers.len() {
            return Err(StateError::InvalidRegister(reg).into());
        }
        if reg != 0 {
            // x0不可写
            self.registers[reg] = value;
        }
        Ok(())
    }

    /// 获取PC值
    #[inline(always)]
    pub fn get_pc(&self) -> u64 {
        self.pc
    }

    #[inline(always)]
    pub fn get_npc(&self) -> u64 {
        self.npc
    }

    /// 设置PC值
    #[inline(always)]
    pub fn set_npc(&mut self, value: u64) {
        self.npc = value;
    }

    #[inline(always)]
    pub fn sync_pc(&mut self) {
        self.pc = self.npc;
    }

    /// 获取CSR值
    #[inline(always)]
    pub fn get_csr(&self, csr: u16) -> Result<u64> {
        self.csrs
            .get(&csr)
            .copied()
            .ok_or_else(|| StateError::InvalidCsr(csr).into())
    }

    /// 设置CSR值
    #[inline(always)]
    pub fn set_csr(&mut self, csr: u16, value: u64) -> Result<()> {
        self.csrs.insert(csr, value);
        Ok(())
    }
}

/// RISC-V寄存器别名
fn get_register_alias(reg: usize) -> &'static str {
    match reg {
        0 => "zero",   // Hard-wired zero
        1 => "ra",     // Return address
        2 => "sp",     // Stack pointer
        3 => "gp",     // Global pointer
        4 => "tp",     // Thread pointer
        5 => "t0",     // Temporary
        6 => "t1",     // Temporary
        7 => "t2",     // Temporary
        8 => "s0/fp",  // Saved register/frame pointer
        9 => "s1",     // Saved register
        10 => "a0",    // Function argument/return value
        11 => "a1",    // Function argument/return value
        12 => "a2",    // Function argument
        13 => "a3",    // Function argument
        14 => "a4",    // Function argument
        15 => "a5",    // Function argument
        16 => "a6",    // Function argument
        17 => "a7",    // Function argument
        18 => "s2",    // Saved register
        19 => "s3",    // Saved register
        20 => "s4",    // Saved register
        21 => "s5",    // Saved register
        22 => "s6",    // Saved register
        23 => "s7",    // Saved register
        24 => "s8",    // Saved register
        25 => "s9",    // Saved register
        26 => "s10",   // Saved register
        27 => "s11",   // Saved register
        28 => "t3",    // Temporary
        29 => "t4",    // Temporary
        30 => "t5",    // Temporary
        31 => "t6",    // Temporary
        _ => "unknown",
    }
}

impl fmt::Display for State {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "=== CPU State ===")?;
        writeln!(f, "PC: 0x{:016x}", self.pc)?;
        writeln!(f)?;

        // 打印寄存器
        writeln!(f, "Registers:")?;
        for i in 0..32 {
            let value = if i == 0 { 0 } else { self.registers[i] };
            let alias = get_register_alias(i);
            writeln!(f, "  x{:2}({:>6}): 0x{:016x}", i, alias, value)?;
        }
        writeln!(f)?;

        // 打印PC附近的内存和反汇编
        writeln!(f, "Memory around PC:")?;
        let disasm = match RiscvDisassembler::new() {
            Ok(d) => d,
            Err(_) => {
                writeln!(f, "  Failed to create disassembler")?;
                return Ok(());
            }
        };

        // 显示PC前后各4条指令（共9条）
        let start_offset = 4 * 4; // 4条指令 * 4字节
        let instruction_count = 9;
        let start_addr = self.pc.saturating_sub(start_offset);

        for i in 0..instruction_count {
            let addr = start_addr + (i * 4) as u64;

            // 检查是否越界
            match self.read_memory(addr, 4) {
                Ok(bytes) => {
                    if bytes.len() == 4 {
                        let instruction = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);

                        // 标记当前PC
                        let marker = if addr == self.pc { " <-- PC" } else { "" };

                        // 反汇编指令
                        match disasm.disasm_instruction(instruction, addr) {
                            Ok(disasm_text) => {
                                writeln!(f, "  0x{:016x}: {:08x}    {}{}",
                                        addr, instruction, disasm_text, marker)?;
                            }
                            Err(_) => {
                                writeln!(f, "  0x{:016x}: {:08x}    <invalid>{}",
                                        addr, instruction, marker)?;
                            }
                        }
                    } else {
                        writeln!(f, "  0x{:016x}: <partial read>", addr)?;
                    }
                }
                Err(_) => {
                    writeln!(f, "  0x{:016x}: <memory error>", addr)?;
                }
            }
        }

        // 打印CSR寄存器（如果有的话）
        if !self.csrs.is_empty() {
            writeln!(f)?;
            writeln!(f, "CSR Registers:")?;
            let mut csr_pairs: Vec<_> = self.csrs.iter().collect();
            csr_pairs.sort_by_key(|&(k, _)| k);
            for (csr, value) in csr_pairs {
                writeln!(f, "  CSR 0x{:03x}: 0x{:016x}", csr, value)?;
            }
        }

        Ok(())
    }
}
