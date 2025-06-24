//! RISC-V模拟器库

use anyhow::{Ok, Result};
use clap::Parser;
use tracing::info;

use crate::emulator::Emulator;

pub mod const_values;
pub mod emulator;
pub mod system;
pub mod utils;

/// RISC-V 模拟器
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// ELF文件路径
    #[arg(short, long)]
    pub elf: Option<String>,

    /// 是否启用调试模式
    #[arg(short, long)]
    pub debug: bool,

    /// GDB端口
    #[arg(short, long, default_value = "1234")]
    pub port: u16,

    /// 内存大小 (MB)
    #[arg(short, long, default_value = "128")]
    pub memory: usize,
}

pub fn build_emu_run_blocking(args: Args) -> Result<()> {
    // 创建模拟器
    let mut emu = Emulator::new(args.memory * 1024 * 1024)?;

    if let Some(elf_path) = args.elf {
        info!(path = %elf_path, "加载ELF文件");
        emu.load_elf(&elf_path)?;
    }

    if args.debug {
        info!(port = args.port, "启用调试模式");
        emu.enable_debug()?;
    } else {
        // 运行模拟器
        while emu.get_exec_state() != emulator::ExecState::End {
            // 执行模拟器步骤
            emu.step(usize::MAX)?;
        }
    }

    Ok(())
}
