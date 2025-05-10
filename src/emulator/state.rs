//! CPU状态管理

use anyhow::Result;
use super::memory::{Memory, MemoryError};

/// CPU状态
pub struct State {
    // 通用寄存器
    registers: [u64; 32],
    // 程序计数器
    pc: u64,
    // CSR寄存器
    csrs: rustc_hash::FxHashMap<u16, u64>,
    // 内存
    memory: Memory,
}

impl State {
    /// 创建新的CPU状态
    pub fn new(memory_size: usize) -> Result<Self, MemoryError> {
        Ok(Self {
            registers: [0; 32],
            pc: 0x80000000,
            csrs: rustc_hash::FxHashMap::default(),
            memory: Memory::new(memory_size)?,
        })
    }

    /// 读取内存
    pub fn read_memory(&self, addr: u64, size: usize) -> Result<Vec<u8>, MemoryError> {
        self.memory.read(addr, size)
    }

    /// 写入内存
    pub fn write_memory(&mut self, addr: u64, data: &[u8]) -> Result<(), MemoryError> {
        self.memory.write(addr, data)
    }

    /// 取指令
    pub fn fetch_instruction(&self, pc: u64) -> Result<u32, MemoryError> {
        let bytes = self.read_memory(pc, 4)?;
        Ok(u32::from_le_bytes(bytes.try_into().unwrap()))
    }

    /// 获取寄存器值
    pub fn get_reg(&self, reg: usize) -> u64 {
        if reg == 0 {
            0 // x0 永远是0
        } else {
            self.registers[reg]
        }
    }

    /// 设置寄存器值
    pub fn set_reg(&mut self, reg: usize, value: u64) {
        if reg != 0 {
            // x0不可写
            self.registers[reg] = value;
        }
    }

    /// 获取PC值
    pub fn get_pc(&self) -> u64 {
        self.pc
    }

    /// 设置PC值
    pub fn set_pc(&mut self, value: u64) {
        self.pc = value;
    }

    /// 获取CSR值
    pub fn get_csr(&self, csr: u16) -> Option<u64> {
        self.csrs.get(&csr).copied()
    }

    /// 设置CSR值
    pub fn set_csr(&mut self, csr: u16, value: u64) {
        self.csrs.insert(csr, value);
    }
}
