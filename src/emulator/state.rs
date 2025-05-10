//! CPU状态管理

use anyhow::{Result, Context};
use thiserror::Error;
use super::memory::{Memory, MemoryError};

#[derive(Debug, Error)]
pub enum StateError {
    #[error("寄存器访问错误: 寄存器 x{0} 超出范围")]
    InvalidRegister(usize),
    #[error("CSR访问错误: CSR {0:#x} 未找到")]
    InvalidCsr(u16),
    #[error("内存错误: {0}")]
    Memory(#[from] MemoryError),
}

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
    pub fn new(memory_size: usize) -> Result<Self> {
        Ok(Self {
            registers: [0; 32],
            pc: 0x80000000,
            csrs: rustc_hash::FxHashMap::default(),
            memory: Memory::new(memory_size)
                .with_context(|| format!("Failed to initialize memory with size {} bytes", memory_size))?,
        })
    }

    /// 读取内存
    pub fn read_memory(&self, addr: u64, size: usize) -> Result<Vec<u8>> {
        self.memory.read(addr, size)
            .with_context(|| format!("Failed to read {} bytes from address {:#x}", size, addr))
    }

    /// 写入内存
    pub fn write_memory(&mut self, addr: u64, data: &[u8]) -> Result<()> {
        self.memory.write(addr, data)
            .with_context(|| format!("Failed to write {} bytes to address {:#x}", data.len(), addr))
    }

    /// 取指令
    pub fn fetch_instruction(&self, pc: u64) -> Result<u32> {
        let bytes = self.read_memory(pc, 4)
            .with_context(|| format!("Failed to fetch instruction at PC {:#x}", pc))?;
        bytes.try_into()
            .map(u32::from_le_bytes)
            .map_err(|_| anyhow::anyhow!("Invalid instruction bytes at PC {:#x}", pc))
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
        self.csrs.get(&csr)
            .copied()
            .ok_or_else(|| StateError::InvalidCsr(csr).into())
    }

    /// 设置CSR值
    pub fn set_csr(&mut self, csr: u16, value: u64) -> Result<()> {
        self.csrs.insert(csr, value);
        Ok(())
    }
}
