//! Timer 设备：直接返回系统时间（以微秒计）
//!
//! 寄存器映射（相对于设备基址）:
//! - 0x00: 时间低位（读返回当前系统时间，按访问大小返回小端）
//! - 0x04: 保留（与 0x00 同步）
//! - 0x08: 保留（与 0x00 同步）
//! - 0x0C: 控制寄存器（保留）
use mmio_trait::{DeviceError, MmioDevice};
use std::time::{SystemTime, UNIX_EPOCH};

const CNT0_REG: u64 = 0x00;
const CNT1_REG: u64 = 0x04;
const CNT2_REG: u64 = 0x08;
const CTRL_REG: u64 = 0x0c;

fn current_time_us() -> u64 {
    let d = SystemTime::now().duration_since(UNIX_EPOCH);
    match d {
        Ok(dur) => {
            // as_micros 返回 u128，截断为 u64（对于常规用途足够）
            dur.as_micros() as u64
        }
        Err(_) => 0,
    }
}

/// 简化的 Timer 设备实现：读出系统时间（us）
pub struct Timer {
    name: String,
}

impl Timer {
    pub fn new(name: String) -> Self {
        Self { name }
    }
}

impl Default for Timer {
    fn default() -> Self {
        Self::new("timer".to_string())
    }
}

impl MmioDevice for Timer {
    fn read(&mut self, offset: u64, size: usize) -> Result<Vec<u8>, DeviceError> {
        match offset {
            CNT0_REG | CNT1_REG | CNT2_REG => {
                // 支持 1/2/4/8 字节读取，返回当前系统时间（微秒）的小端字节序
                match size {
                    1 | 2 | 4 | 8 => {
                        let t = current_time_us();
                        let bytes = t.to_le_bytes(); // 8 字节
                        let mut out = Vec::new();
                        // 根据 size 返回低位的 size 字节
                        out.extend_from_slice(&bytes[0..size]);
                        Ok(out)
                    }
                    _ => Err(DeviceError::Unsupported(
                        "计数器只支持 1/2/4/8 字节读取".to_string(),
                    )),
                }
            }
            CTRL_REG => {
                // 控制寄存器暂不实现，返回 0
                if size == 1 {
                    Ok(vec![0u8])
                } else if size == 4 {
                    Ok(vec![0u8, 0u8, 0u8, 0u8])
                } else {
                    Err(DeviceError::Unsupported(
                        "控制寄存器只支持 1 或 4 字节访问".to_string(),
                    ))
                }
            }
            _ => Err(DeviceError::Access(format!(
                "Timer 不支持的寄存器偏移: {:#x}",
                offset
            ))),
        }
    }

    fn write(&mut self, offset: u64, _data: &[u8]) -> Result<(), DeviceError> {
        match offset {
            CNT0_REG | CNT1_REG | CNT2_REG | CTRL_REG => {
                // 写操作对该设备无效或被忽略（只读设备）
                Err(DeviceError::Unsupported(
                    "Timer 为只读设备（读系统时间）".to_string(),
                ))
            }
            _ => Err(DeviceError::Access(format!(
                "Timer 不支持的寄存器偏移: {:#x}",
                offset
            ))),
        }
    }

    fn name(&self) -> &str {
        &self.name
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_timer() {
        let t = Timer::new("t0".to_string());
        assert_eq!(t.name(), "t0");
    }

    #[test]
    fn read_time_nonzero() {
        let mut t = Timer::new("t".to_string());
        // 读取 8 字节时间戳
        let r = t.read(CNT0_REG, 8).unwrap();
        assert_eq!(r.len(), 8);
        let ts = u64::from_le_bytes([r[0], r[1], r[2], r[3], r[4], r[5], r[6], r[7]]);
        assert!(ts > 0);
    }
}
