//! CPU状态管理

use super::memory::{Memory, MemoryError};
use anyhow::Result;
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

/// CPU状态
#[derive(Debug, Clone)]
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
    pub fn new(memory_size: usize) -> Result<Self> {
        Ok(Self {
            registers: [0; 32],
            pc: 0x80000000,
            csrs: rustc_hash::FxHashMap::default(),
            memory: Memory::new(memory_size)?,
        })
    }

    /// 读取内存
    pub fn read_memory(&self, addr: u64, size: usize) -> Result<Vec<u8>> {
        Ok(self.memory.read(addr, size)?)
    }

    /// 写入内存
    pub fn write_memory(&mut self, addr: u64, data: &[u8]) -> Result<()> {
        Ok(self.memory.write(addr, data)?)
    }

    /// 取指令
    pub fn fetch_instruction(&self, pc: u64) -> Result<u32> {
        let bytes = self
            .read_memory(pc, 4)?;
        Ok(bytes
            .try_into()
            .map(u32::from_le_bytes)
            .map_err(|_| StateError::InvalidInstructionBytes(pc))?)
    }

    pub fn get_regs(&self) -> &[u64; 32] {
        &self.registers
    }

    /// 获取寄存器值
    pub fn get_reg(&self, reg: usize) -> Result<u64> {
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
    pub fn set_reg(&mut self, reg: usize, value: u64) -> Result<()> {
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
    pub fn get_pc(&self) -> u64 {
        self.pc
    }

    /// 设置PC值
    pub fn set_pc(&mut self, value: u64) {
        self.pc = value;
    }

    /// 获取CSR值
    pub fn get_csr(&self, csr: u16) -> Result<u64> {
        self.csrs
            .get(&csr)
            .copied()
            .ok_or_else(|| StateError::InvalidCsr(csr).into())
    }

    /// 设置CSR值
    pub fn set_csr(&mut self, csr: u16, value: u64) -> Result<()> {
        self.csrs.insert(csr, value);
        Ok(())
    }
}
