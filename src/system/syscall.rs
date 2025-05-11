use super::memory::Memory;
use thiserror::Error;
use std::io::{self, Write};

/// 系统调用错误
#[derive(Debug, Error)]
pub enum SyscallError {
    #[error("未实现的系统调用: {0}")]
    Unimplemented(u64),
    #[error("内存访问错误: {0}")]
    Memory(#[from] super::memory::MemoryError),
    #[error("参数错误: {0}")]
    InvalidArgument(String),
}

/// 系统调用上下文
#[derive(Debug)]
pub struct SyscallContext {
    /// 系统调用号
    pub number: u64,
    /// 参数1
    pub arg0: u64,
    /// 参数2
    pub arg1: u64,
    /// 参数3
    pub arg2: u64,
    /// 参数4
    pub arg3: u64,
    /// 参数5
    pub arg4: u64,
}

/// Linux系统调用号
#[allow(dead_code)]
mod syscall_num {
    pub const SYS_EXIT: u64 = 93;
    pub const SYS_EXIT_GROUP: u64 = 94;
    pub const SYS_READ: u64 = 63;
    pub const SYS_WRITE: u64 = 64;
    pub const SYS_OPEN: u64 = 1024;
    pub const SYS_CLOSE: u64 = 57;
    pub const SYS_FSTAT: u64 = 80;
    pub const SYS_BRK: u64 = 214;
}

/// 处理系统调用
pub fn handle_syscall(ctx: SyscallContext, memory: &mut Memory) -> Result<u64, SyscallError> {
    match ctx.number {
        syscall_num::SYS_EXIT | syscall_num::SYS_EXIT_GROUP => {
            // 程序退出
            std::process::exit(ctx.arg0 as i32);
        }
        
        syscall_num::SYS_WRITE => {
            // 写入文件
            let fd = ctx.arg0;
            let buf_ptr = ctx.arg1;
            let count = ctx.arg2;
            
            // 只支持标准输出和标准错误
            if fd != 1 && fd != 2 {
                return Err(SyscallError::InvalidArgument(format!(
                    "不支持的文件描述符: {}", fd
                )));
            }
            
            // 读取内存中的数据
            let data = memory.read(buf_ptr, count as usize)?;
            
            // 写入到对应的输出
            if fd == 1 {
                io::stdout().write_all(&data)
            } else {
                io::stderr().write_all(&data)
            }.map_err(|e| {
                SyscallError::InvalidArgument(format!("输出写入失败: {}", e))
            })?;
            
            Ok(count)
        }
        
        syscall_num::SYS_BRK => {
            // 简单返回，不实际分配内存
            Ok(ctx.arg0)
        }
        
        _ => Err(SyscallError::Unimplemented(ctx.number)),
    }
}
