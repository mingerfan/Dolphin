//! ELF文件加载器

use anyhow::{Result, anyhow, Context};
use object::{Object, ObjectSection, Architecture, SectionKind};
use std::fs;
use crate::emulator::State;

/// 加载ELF文件到模拟器内存
pub fn load_elf(state: &mut State, path: &str) -> Result<()> {
    // 读取ELF文件
    let elf_data = fs::read(path)
        .with_context(|| format!("Failed to read ELF file '{}'", path))?;
    let elf_file = object::File::parse(&*elf_data)
        .with_context(|| format!("Failed to parse ELF file '{}'", path))?;

    // 验证目标架构
    if !matches!(elf_file.architecture(), Architecture::Riscv64) {
        return Err(anyhow!("不支持的目标架构, 仅支持RISC-V"));
    }

    // 遍历所有节并加载到内存
    for section in elf_file.sections() {
        // 跳过非分配节
        if !matches!(section.kind(), SectionKind::Text | SectionKind::Data | SectionKind::ReadOnlyData) {
            continue;
        }

        let section_name = section.name()
            .unwrap_or("<unknown>")
            .to_string();
        let addr = section.address();
        
        let data = section.data()
            .with_context(|| format!("Failed to read section '{}' data", section_name))?;

        // 写入内存
        state.write_memory(addr, data)
            .with_context(|| format!("Failed to write section '{}' at address {:#x}", 
                section_name, addr))?;
    }

    // 设置程序入口点
    state.set_pc(elf_file.entry());

    Ok(())
}
