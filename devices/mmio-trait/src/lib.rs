//! MMIO 设备 trait 定义

use thiserror::Error;

/// 设备错误类型
#[derive(Debug, Error)]
pub enum DeviceError {
    #[error("设备访问错误: {0}")]
    Access(String),
    #[error("设备不支持的操作: {0}")]
    Unsupported(String),
    #[error("设备内部错误: {0}")]
    Internal(String),
}

/// MMIO 设备 trait
/// 所有 MMIO 设备都必须实现此 trait
pub trait MmioDevice: Send + Sync {
    /// 从设备读取数据
    /// 
    /// # 参数
    /// - offset: 相对于设备基址的偏移量
    /// - size: 读取的字节数 (1, 2, 4, 8)
    /// 
    /// # 返回
    /// 读取到的数据，按小端序返回
    fn read(&mut self, offset: u64, size: usize) -> Result<Vec<u8>, DeviceError>;

    /// 向设备写入数据
    /// 
    /// # 参数
    /// - offset: 相对于设备基址的偏移量
    /// - data: 要写入的数据，按小端序
    fn write(&mut self, offset: u64, data: &[u8]) -> Result<(), DeviceError>;

    /// 时钟周期驱动（可选）
    /// 
    /// # 参数
    /// - cycles: 经过的周期数
    fn tick(&mut self, _cycles: u64) {}

    /// 检查是否有中断挂起（可选）
    /// 
    /// # 返回
    /// 如果有中断挂起，返回中断号
    fn irq_pending(&self) -> Option<u32> {
        None
    }

    /// 获取设备名称（用于调试）
    fn name(&self) -> &str {
        "unknown"
    }
}
