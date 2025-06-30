//! RISC-V模拟器库
pub mod const_values;
pub mod emulator;
pub mod system;
pub mod utils;
mod emu_ref;

use anyhow::Result;
use clap::Parser;
use emulator::Emulator;
use tracing::info;
use emulator::InstDecoderArgs;

#[cfg(feature = "tracer")]
use emulator::tracer::TracerArgs;

// 仅在启用 GDB feature 时导入相关模块
#[cfg(feature = "gdb")]
use {
    emulator::{EmuGdbEventLoop, gdb},
    gdbstub::{conn::ConnectionExt, stub::GdbStub},
};

/// RISC-V 模拟器
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// ELF文件路径
    #[arg(short, long)]
    pub elf: Option<String>,

    /// GDB端口
    #[arg(short, long, default_value = "1234")]
    pub port: u16,

    /// 内存大小 (MB)
    #[arg(short, long, default_value = "128")]
    pub memory: usize,

    #[command(flatten)]
    pub inst_decoder_args: InstDecoderArgs,

    /// 追踪器参数
    #[cfg(feature = "tracer")]
    #[command(flatten)]
    pub tracer: TracerArgs,
}

pub fn build_emu_run_blocking(args: Args) -> Result<()> {
    // 创建模拟器
    let mut emu = Emulator::new(&args)?;

    if let Some(elf_path) = &args.elf {
        info!(path = %elf_path, "加载ELF文件");
        emu.load_elf(elf_path)?;
    }

    // 初始化全局追踪器
    #[cfg(feature = "tracer")]
    emulator::tracer::init_global_tracer(args.tracer);

    #[cfg(feature = "gdb")] // 条件编译 GDB 支持
    {
        info!(port = args.port, "启用调试模式");
        let connection: Box<dyn ConnectionExt<Error = std::io::Error>> =
            Box::new(gdb::wait_for_tcp(args.port)?);

        let gdb_conn = GdbStub::new(connection);

        match gdb_conn.run_blocking::<EmuGdbEventLoop>(&mut emu) {
            Ok(_) => info!("GDB调试会话结束"),
            Err(e) => {
                tracing::error!("GDB调试会话出错");
                return Err(e.into());
            }
        };
    }
    #[cfg(not(feature = "gdb"))] // 如果没有启用 GDB
    {
        // 运行模拟器
        while emu.get_exec_state() != emulator::ExecState::End {
            // 执行模拟器步骤
            emu.steps(usize::MAX)?;
        }
    }

    #[cfg(feature = "tracer")]
    {
        // 打印追踪日志
        use crate::emulator::tracer::destroy_global_tracer;
        if let Some(log) = emulator::tracer::global_get_log() {
            info!("追踪日志:\n{}", log);
        } else {
            info!("没有追踪日志");
        }
        destroy_global_tracer();
    }

    Ok(())
}
