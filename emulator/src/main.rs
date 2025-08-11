use anyhow::Result;
use emulator::{Args, build_emu_run_blocking};
use clap::Parser;
use tracing::{info, Level};
use tracing_subscriber::{self, EnvFilter, fmt::format::FmtSpan};


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

    build_emu_run_blocking(args)
}
