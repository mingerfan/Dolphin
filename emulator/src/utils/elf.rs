//! ELF文件加载器

use anyhow::{Result, anyhow, Context};
use object::{Object, ObjectSection, Architecture, SectionKind};
#[cfg(feature = "difftest")]
use rv64emu::rv64core::cpu_core::CpuCore;
use std::fs;
use crate::emulator::State;
use crate::difftest::Difftest;

/// 加载ELF文件到模拟器内存
pub fn load_elf(state: &mut State, path: &str) -> Result<()> {
    // 读取ELF文件
    let elf_data = fs::read(path)
        .with_context(|| format!("无法读取ELF文件 '{}'", path))?;
    let elf_file = object::File::parse(&*elf_data)
        .with_context(|| format!("无法解析ELF文件 '{}'", path))?;

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
            .with_context(|| format!("无法读取节 '{}' 的数据", section_name))?;

        // 写入内存
        state.write_memory(addr, data)
            .with_context(|| format!("无法将节 '{}' 写入地址 {:#x}",
                section_name, addr))?;
    }

    // 设置程序入口点
    state.set_pc(elf_file.entry());

    Ok(())
}

pub fn load_elf_diff(state: &mut CpuCore, path: &str) -> Result<()> {
    // 读取ELF文件
    let elf_data = fs::read(path)
        .with_context(|| format!("无法读取ELF文件 '{}'", path))?;
    let elf_file = object::File::parse(&*elf_data)
        .with_context(|| format!("无法解析ELF文件 '{}'", path))?;

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
            .with_context(|| format!("无法读取节 '{}' 的数据", section_name))?;

        // 写入内存
        for (i, byte) in data.iter().enumerate() {
            state.set_mem(addr + i as u64, *byte as u64, 1);
        }
    }

    // 设置程序入口点
    state.set_pc(elf_file.entry());

    Ok(())
}
