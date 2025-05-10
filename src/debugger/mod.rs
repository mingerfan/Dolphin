//! GDB调试支持模块

mod gdb_server;

use std::net::SocketAddr;
use std::sync::{Arc, RwLock, Mutex};
use anyhow::Result;

use crate::emulator::State;

/// CPU状态类型
type CpuState = Arc<RwLock<State>>;

/// 执行状态
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExecutionState {
    /// 运行中
    Running,
    /// 已暂停
    Stopped,
    /// 退出
    Quit,
}

/// 执行控制
#[derive(Clone)]
pub struct ExecutionControl {
    /// 执行状态
    pub state: Arc<Mutex<ExecutionState>>,
}

impl ExecutionControl {
    pub fn new() -> Self {
        Self {
            state: Arc::new(Mutex::new(ExecutionState::Stopped)),
        }
    }
}

pub struct Debugger {
    state: CpuState,
    server: Option<gdb_server::GdbServer>,
    control: ExecutionControl,
}

impl Debugger {
    /// 创建一个新的调试器实例
    pub fn new(state: CpuState) -> Self {
        Self {
            state,
            server: None,
            control: ExecutionControl::new(),
        }
    }

    /// 启动GDB服务器
    pub fn start_server(&mut self, addr: SocketAddr) -> Result<()> {
        if self.server.is_some() {
            anyhow::bail!("GDB server is already running");
        }

        // 创建GDB服务器
        let mut server = gdb_server::GdbServer::new(self.state.clone(), self.control.clone());
        
        // 启动服务器线程
        std::thread::spawn(move || {
            if let Err(e) = server.start(addr.port()) {
                eprintln!("GDB server error: {}", e);
            }
        });
        
        // 等待服务器启动
        std::thread::sleep(std::time::Duration::from_millis(100));
        Ok(())
    }

    /// 停止GDB服务器
    pub fn stop_server(&mut self) -> Result<()> {
        if let Some(mut server) = self.server.take() {
            server.shutdown()?;
        }
        Ok(())
    }

    /// 获取执行控制
    pub fn get_control(&self) -> ExecutionControl {
        self.control.clone()
    }
}
