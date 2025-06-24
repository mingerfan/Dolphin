//! RISC-V模拟器库
pub mod const_values;
pub mod emulator;
pub mod system;
pub mod utils;

use emulator::{gdb, EmuGdbEventLoop, Emulator};
use gdbstub::{conn::ConnectionExt, stub::GdbStub};
use anyhow::Result;
use clap::Parser;
use tracing::info;


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

        let connection: Box<dyn ConnectionExt<Error = std::io::Error>> =
            Box::new(gdb::wait_for_tcp(args.port)?);

        let gdb_conn = GdbStub::new(connection);

        match gdb_conn.run_blocking::<EmuGdbEventLoop>(&mut emu) {
            Ok(_) => info!("GDB调试会话结束"),
            Err(e) => {
                tracing::error!("GDB调试会话出错: {:?}", e);
                return Err(e.into());
            }
        };

    } else {
        // 运行模拟器
        while emu.get_exec_state() != emulator::ExecState::End {
            // 执行模拟器步骤
            emu.steps(usize::MAX)?;
        }
    }

    Ok(())
}
