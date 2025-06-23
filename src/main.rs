use anyhow::Result;
use clap::Parser;
use tracing::{info, Level};
use tracing_subscriber::{self, EnvFilter, fmt::format::FmtSpan};

use simulator::emulator::Emulator;

/// RISC-V 模拟器
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// ELF文件路径
    #[arg(short, long)]
    elf: Option<String>,

    /// 是否启用调试模式
    #[arg(short, long)]
    debug: bool,

    /// GDB端口
    #[arg(short, long, default_value = "1234")]
    port: u16,

    /// 内存大小 (MB)
    #[arg(short, long, default_value = "128")]
    memory: usize,
}

fn main() -> Result<()> {
    // 初始化日志
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::builder()
                .with_default_directive(Level::INFO.into())
                .from_env_lossy(),
        )
        .with_target(false) // 不显示目标模块
        .with_thread_ids(true) // 显示线程ID
        .with_thread_names(true) // 显示线程名称
        .with_file(true) // 显示文件名
        .with_line_number(true) // 显示行号
        .with_span_events(FmtSpan::ACTIVE) // 跟踪span的生命周期
        .init();
    
    // 解析命令行参数
    let args = Args::parse();
    
    info!(version = env!("CARGO_PKG_VERSION"), "启动RISC-V模拟器");
    info!(memory_size_mb = args.memory, "配置内存大小");

    // 创建模拟器
    let mut emu = Emulator::new(args.memory * 1024 * 1024)?;
    
    if let Some(elf_path) = args.elf {
        info!(path = %elf_path, "加载ELF文件");
        emu.load_elf(&elf_path)?;
    }
    
    if args.debug {
        info!(port = args.port, "启用调试模式");
        emu.enable_debug()?;
    }
    
    // 运行模拟器
    emu.step(usize::MAX)
}
