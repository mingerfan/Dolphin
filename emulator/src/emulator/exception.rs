//! 异常处理模块

use thiserror::Error;

#[derive(Debug, Error)]
pub enum Exception {
    #[error("取指未对齐地址: {addr:#x}")]
    InstructionAddressMisaligned { addr: u64 },

    #[error("访问错误: {addr:#x}")]
    AccessFault { addr: u64 },
    
    #[error("指令错误: {addr:#x}")]
    InstructionFault { addr: u64 },
    
    #[error("非法指令: {instruction:#x} at {addr:#x}")]
    IllegalInstruction { instruction: u32, addr: u64 },
    
    #[error("环境调用")]
    EnvironmentCall,
    
    #[error("断点")]
    Breakpoint,
}

// 特权级别
// #[derive(Debug, Clone, Copy, PartialEq, Eq)]
// pub enum PrivilegeLevel {
//     User = 0,
//     Supervisor = 1,
//     Machine = 3,
// }

// impl PrivilegeLevel {
//     pub fn from_u64(value: u64) -> Option<Self> {
//         match value {
//             0 => Some(PrivilegeLevel::User),
//             1 => Some(PrivilegeLevel::Supervisor),
//             3 => Some(PrivilegeLevel::Machine),
//             _ => None,
//         }
//     }
// }
