//! 断点管理

use std::collections::HashMap;

/// 断点信息
#[derive(Debug, Clone)]
pub struct Breakpoint {
    /// 断点地址
    pub address: u64,
    /// 是否启用
    pub enabled: bool,
}

/// 断点管理器
#[derive(Debug)]
pub struct BreakpointManager {
    /// 断点映射表：地址 -> 断点信息
    breakpoints: HashMap<u64, Breakpoint>,
}

impl BreakpointManager {
    /// 创建新的断点管理器
    pub fn new() -> Self {
        Self {
            breakpoints: HashMap::new(),
        }
    }

    /// 添加断点
    pub fn add_breakpoint(&mut self, address: u64) -> bool {
        if self.breakpoints.contains_key(&address) {
            return false;
        }

        self.breakpoints.insert(
            address,
            Breakpoint {
                address,
                enabled: true,
            },
        );
        true
    }

    /// 移除断点
    pub fn remove_breakpoint(&mut self, address: u64) -> bool {
        self.breakpoints.remove(&address).is_some()
    }

    /// 检查地址是否有断点
    pub fn has_breakpoint(&self, address: u64) -> bool {
        self.breakpoints
            .get(&address)
            .map(|bp| bp.enabled)
            .unwrap_or(false)
    }

    /// 启用/禁用断点
    pub fn set_breakpoint_enabled(&mut self, address: u64, enabled: bool) -> bool {
        if let Some(bp) = self.breakpoints.get_mut(&address) {
            bp.enabled = enabled;
            true
        } else {
            false
        }
    }

    /// 获取所有断点
    pub fn get_all_breakpoints(&self) -> Vec<&Breakpoint> {
        self.breakpoints.values().collect()
    }
}
