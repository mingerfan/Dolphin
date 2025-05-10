//! 指令执行模块

use anyhow::Context;
use crate::emulator::State;
use thiserror::Error;

/// 所有可能的执行错误
#[derive(Debug, Error)]
pub enum ExecuteError {
    #[error("未实现的指令: {0:#010x}")]
    UnimplementedInstruction(u32),
    #[error("非法指令: {0:#010x}")]
    IllegalInstruction(u32),
    #[error("内存访问错误: {0:#010x}")]
    MemoryAccessError(u64),
}

/// 指令执行trait
pub trait Execute {
    /// 执行指令
    fn execute(&mut self, state: &mut State) -> anyhow::Result<()>;
}

/// RV64I基本指令集
pub struct RV64I {
    instruction: u32,
}

impl RV64I {
    pub fn new(instruction: u32) -> Self {
        Self { instruction }
    }

    /// 解码指令字段
    fn decode(&self) -> (u32, u32, u32, u32, u32) {
        let opcode = self.instruction & 0x7f;
        let rd = (self.instruction >> 7) & 0x1f;
        let rs1 = (self.instruction >> 15) & 0x1f;
        let rs2 = (self.instruction >> 20) & 0x1f;
        let funct3 = (self.instruction >> 12) & 0x7;
        (opcode, rd as u32, rs1 as u32, rs2 as u32, funct3)
    }
}

impl Execute for RV64I {
    fn execute(&mut self, state: &mut State) -> anyhow::Result<()> {
        let (opcode, rd, rs1, rs2, funct3) = self.decode();
        
        match opcode {
            0x33 => {
                // R-type 算术指令
                todo!("Implement R-type instructions")
            }
            0x13 => {
                // I-type 立即数指令
                todo!("Implement I-type instructions")
            }
            _ => Err(ExecuteError::UnimplementedInstruction(self.instruction))
                .with_context(|| format!("Failed to execute instruction {:#x}", self.instruction)),
        }
    }
}
