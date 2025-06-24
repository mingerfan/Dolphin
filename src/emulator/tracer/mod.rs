mod itracer;

pub use itracer::ITracer;

use clap::Args;
use std::sync::{Mutex, OnceLock};

use super::Emulator;

static GLOBAL_TRACER: OnceLock<Mutex<Option<Tracer>>> = OnceLock::new();

/// 初始化全局追踪器
pub fn init_global_tracer(args: TracerArgs) {
    GLOBAL_TRACER.get_or_init(|| {
        let mut tracer = Tracer::new();
        tracer.add_tracers(args);
        Mutex::new(Some(tracer))
    });
}

/// 全局追踪入口
pub fn global_trace(emulator: &Emulator) {
    let tracers = GLOBAL_TRACER.get();
    match tracers {
        Some(tracer) => {
            if let Ok(mut tracer) = tracer.lock() {
                if let Some(ref mut t) = *tracer {
                    t.trace(emulator);
                }
            }
        }
        None => {
            tracing::warn!("全局追踪器未初始化，请先调用 init_global_tracer");
        }
    }
}

/// 获取全局追踪日志
pub fn global_get_log() -> Option<String> {
    let tracers = GLOBAL_TRACER.get();
    match tracers {
        Some(tracer) => {
            if let Ok(mut tracer) = tracer.lock() {
                if let Some(ref mut t) = *tracer {
                    return Some(t.print_log());
                }
            }
        }
        None => {
            tracing::warn!("全局追踪器未初始化，请先调用 init_global_tracer");
        }
        
    };
    None
}

/// 销毁全局追踪器
pub fn destroy_global_tracer() {
    if let Some(tracer) = GLOBAL_TRACER.get() {
        if let Ok(mut tracer) = tracer.lock() {
            *tracer = None; // 清空追踪器
        }
    } else {
        tracing::warn!("全局追踪器未初始化，无法销毁");
    }
}

#[derive(Args, Debug)]
pub struct TracerArgs {
    /// 启用指令追踪器
    #[arg(long, default_value_t = false)]
    pub enable_itracer: bool,
}

/// 统一的追踪器入口
pub struct Tracer {
    tracers: Vec<Box<dyn TracerTrace>>,
}

trait TracerTrace: Send + Sync {
    /// 追踪器名称
    fn name(&self) -> &'static str;

    /// 追踪一条指令
    fn trace(&mut self, emulator: &Emulator);

    /// 打印Log
    fn get_instructions_log(&mut self) -> String;
}

impl Tracer {
    /// 初始化追踪器
    pub fn new() -> Self {
        let tracers: Vec<Box<dyn TracerTrace>> = Vec::new();

        Tracer { tracers }
    }

    pub fn add_tracers(&mut self, args: TracerArgs) {
        if args.enable_itracer {
            self.tracers.push(Box::new(ITracer::new()));
        }
    }

    /// 统一的trace入口
    pub fn trace(&mut self, emulator: &Emulator) {
        for tracer in &mut self.tracers {
            tracer.trace(emulator);
        }
    }

    pub fn print_log(&mut self) -> String {
        let mut log = String::new();
        for tracer in &mut self.tracers {
            log += &format!("Tracer: {}\n", tracer.name());
            log += &tracer.get_instructions_log();
        }
        log
    }
}

impl Default for Tracer {
    fn default() -> Self {
        Self::new()
    }
}
