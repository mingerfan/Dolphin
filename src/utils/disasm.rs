//! RISC-V 64位指令反汇编模块

use anyhow::{anyhow, Result};
use capstone::prelude::*;

/// RISC-V 64位反汇编器
pub struct RiscvDisassembler {
    cs: Capstone,
}

impl RiscvDisassembler {
    /// 创建新的RISC-V 64位反汇编器
    pub fn new() -> Result<Self> {
        let cs = Capstone::new()
            .riscv()
            .mode(arch::riscv::ArchMode::RiscV64)
            .detail(true)
            .build()
            .map_err(|e| anyhow!("Failed to create capstone engine: {}", e))?;

        Ok(Self { cs })
    }

    /// 反汇编单条指令
    /// 
    /// # 参数
    /// - `code`: 4字节的指令码
    /// - `address`: 指令地址
    /// 
    /// # 返回
    /// 返回反汇编后的文本表示
    pub fn disasm_instruction(&self, code: u32, address: u64) -> Result<String> {
        let code_bytes = code.to_le_bytes();
        
        let insns = self.cs
            .disasm_all(&code_bytes, address)
            .map_err(|e| anyhow!("Failed to disassemble: {}", e))?;

        if insns.is_empty() {
            return Ok(format!("0x{:08x}    <invalid>", code));
        }

        let insn = &insns[0];
        let mnemonic = insn.mnemonic().unwrap_or("<unknown>");
        let op_str = insn.op_str().unwrap_or("");
        
        if op_str.is_empty() {
            Ok(format!("{}", mnemonic))
        } else {
            Ok(format!("{} {}", mnemonic, op_str))
        }
    }

    /// 反汇编指令缓冲区
    /// 
    /// # 参数
    /// - `code`: 指令字节缓冲区
    /// - `start_address`: 起始地址
    /// 
    /// # 返回
    /// 返回每条指令的反汇编文本列表
    pub fn disasm_buffer(&self, code: &[u8], start_address: u64) -> Result<Vec<String>> {
        let insns = self.cs
            .disasm_all(code, start_address)
            .map_err(|e| anyhow!("Failed to disassemble buffer: {}", e))?;

        let mut result = Vec::new();
        for insn in insns.iter() {
            let mnemonic = insn.mnemonic().unwrap_or("<unknown>");
            let op_str = insn.op_str().unwrap_or("");
            
            let disasm_text = if op_str.is_empty() {
                format!("{}", mnemonic)
            } else {
                format!("{} {}", mnemonic, op_str)
            };
            
            result.push(disasm_text);
        }

        Ok(result)
    }

    /// 反汇编指令并返回详细信息
    /// 
    /// # 参数
    /// - `code`: 4字节的指令码
    /// - `address`: 指令地址
    /// 
    /// # 返回
    /// 返回包含地址、机器码和反汇编文本的格式化字符串
    pub fn disasm_with_details(&self, code: u32, address: u64) -> Result<String> {
        let code_bytes = code.to_le_bytes();
        
        let insns = self.cs
            .disasm_all(&code_bytes, address)
            .map_err(|e| anyhow!("Failed to disassemble: {}", e))?;

        if insns.is_empty() {
            return Ok(format!("0x{:016x}: {:08x}    <invalid>", address, code));
        }

        let insn = &insns[0];
        let mnemonic = insn.mnemonic().unwrap_or("<unknown>");
        let op_str = insn.op_str().unwrap_or("");
        
        let disasm_text = if op_str.is_empty() {
            format!("{}", mnemonic)
        } else {
            format!("{} {}", mnemonic, op_str)
        };

        Ok(format!("0x{:016x}: {:08x}    {}", address, code, disasm_text))
    }
}

/// 便利函数：反汇编单条RISC-V 64位指令
pub fn disasm_riscv64_instruction(code: u32, address: u64) -> Result<String> {
    let disasm = RiscvDisassembler::new()?;
    disasm.disasm_instruction(code, address)
}

/// 便利函数：反汇编RISC-V 64位指令并显示详细信息
pub fn disasm_riscv64_with_details(code: u32, address: u64) -> Result<String> {
    let disasm = RiscvDisassembler::new()?;
    disasm.disasm_with_details(code, address)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_instructions() {
        let disasm = RiscvDisassembler::new().expect("Failed to create disassembler");

        // 测试 nop 指令 (addi x0, x0, 0)
        let nop_code = 0x00000013;
        let result = disasm.disasm_instruction(nop_code, 0x1000).unwrap();
        println!("NOP: {}", result);

        // 测试 addi x1, x0, 42
        let addi_code = 0x02a00093; 
        let result = disasm.disasm_instruction(addi_code, 0x1004).unwrap();
        println!("ADDI: {}", result);

        // 测试 add x2, x1, x1
        let add_code = 0x00108133;
        let result = disasm.disasm_instruction(add_code, 0x1008).unwrap();
        println!("ADD: {}", result);
    }

    #[test]
    fn test_with_details() {
        let disasm = RiscvDisassembler::new().expect("Failed to create disassembler");

        let nop_code = 0x00000013;
        let result = disasm.disasm_with_details(nop_code, 0x1000).unwrap();
        println!("Detailed NOP: {}", result);
    }

    #[test]
    fn test_buffer_disassembly() {
        let disasm = RiscvDisassembler::new().expect("Failed to create disassembler");

        // 构造一些测试指令
        let code_buffer = [
            0x13, 0x00, 0x00, 0x00, // nop
            0x93, 0x00, 0xa0, 0x02, // addi x1, x0, 42
            0x33, 0x81, 0x10, 0x00, // add x2, x1, x1
        ];

        let result = disasm.disasm_buffer(&code_buffer, 0x1000).unwrap();
        println!("Buffer disassembly:");
        for (i, line) in result.iter().enumerate() {
            println!("  {}: {}", i, line);
        }
    }

    #[test]
    fn test_convenience_functions() {
        let nop_code = 0x00000013;
        
        let simple = disasm_riscv64_instruction(nop_code, 0x1000).unwrap();
        println!("Simple: {}", simple);

        let detailed = disasm_riscv64_with_details(nop_code, 0x1000).unwrap();
        println!("Detailed: {}", detailed);
    }
}
