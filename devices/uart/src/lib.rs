//! UART 设备实现

use mmio_trait::{DeviceError, MmioDevice};
use std::io::{self, Write};

/// UART 寄存器偏移
const UART_DATA_REG: u64 = 0x00;  // 数据寄存器
const UART_STATUS_REG: u64 = 0x04; // 状态寄存器
const UART_CTRL_REG: u64 = 0x08;   // 控制寄存器

/// UART 状态位
const UART_STATUS_TX_READY: u32 = 0x01;  // 发送就绪
const UART_STATUS_RX_VALID: u32 = 0x02;  // 接收有效

/// UART 设备
pub struct Uart {
    name: String,
    tx_ready: bool,
    rx_buffer: Option<u8>,
}

impl Uart {
    /// 创建新的 UART 设备
    pub fn new(name: String) -> Self {
        Self {
            name,
            tx_ready: true,
            rx_buffer: None,
        }
    }
}

impl MmioDevice for Uart {
    fn read(&mut self, offset: u64, size: usize) -> Result<Vec<u8>, DeviceError> {
        match offset {
            UART_DATA_REG => {
                // 读取数据寄存器
                if size != 1 {
                    return Err(DeviceError::Unsupported(
                        "UART 数据寄存器只支持字节访问".to_string(),
                    ));
                }
                let data = self.rx_buffer.unwrap_or(0);
                self.rx_buffer = None; // 读取后清空
                Ok(vec![data])
            }
            UART_STATUS_REG => {
                // 读取状态寄存器
                if size != 4 {
                    return Err(DeviceError::Unsupported(
                        "UART 状态寄存器只支持32位访问".to_string(),
                    ));
                }
                let mut status = 0u32;
                if self.tx_ready {
                    status |= UART_STATUS_TX_READY;
                }
                if self.rx_buffer.is_some() {
                    status |= UART_STATUS_RX_VALID;
                }
                Ok(status.to_le_bytes().to_vec())
            }
            UART_CTRL_REG => {
                // 读取控制寄存器（暂时返回0）
                if size != 4 {
                    return Err(DeviceError::Unsupported(
                        "UART 控制寄存器只支持32位访问".to_string(),
                    ));
                }
                Ok(vec![0; 4])
            }
            _ => Err(DeviceError::Access(format!(
                "UART 不支持的寄存器偏移: {:#x}",
                offset
            ))),
        }
    }

    fn write(&mut self, offset: u64, data: &[u8]) -> Result<(), DeviceError> {
        match offset {
            UART_DATA_REG => {
                // 写入数据寄存器
                if data.len() != 1 {
                    return Err(DeviceError::Unsupported(
                        "UART 数据寄存器只支持字节访问".to_string(),
                    ));
                }
                
                // 将字节输出到 stderr
                let byte = data[0];
                if let Err(e) = io::stderr().write_all(&[byte]) {
                    return Err(DeviceError::Internal(format!(
                        "UART 输出错误: {}",
                        e
                    )));
                }
                if let Err(e) = io::stderr().flush() {
                    return Err(DeviceError::Internal(format!(
                        "UART 刷新错误: {}",
                        e
                    )));
                }
                
                Ok(())
            }
            UART_STATUS_REG => {
                // 状态寄存器是只读的
                Err(DeviceError::Unsupported(
                    "UART 状态寄存器是只读的".to_string(),
                ))
            }
            UART_CTRL_REG => {
                // 写入控制寄存器（暂时忽略）
                if data.len() != 4 {
                    return Err(DeviceError::Unsupported(
                        "UART 控制寄存器只支持32位访问".to_string(),
                    ));
                }
                // 可以在这里实现控制逻辑，比如波特率设置等
                Ok(())
            }
            _ => Err(DeviceError::Access(format!(
                "UART 不支持的寄存器偏移: {:#x}",
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
    fn test_uart_creation() {
        let uart = Uart::new("test_uart".to_string());
        assert_eq!(uart.name(), "test_uart");
    }

    #[test]
    fn test_uart_status_read() {
        let mut uart = Uart::new("test".to_string());
        let result = uart.read(UART_STATUS_REG, 4).unwrap();
        let status = u32::from_le_bytes([result[0], result[1], result[2], result[3]]);
        assert_eq!(status & UART_STATUS_TX_READY, UART_STATUS_TX_READY);
    }

    #[test]
    fn test_uart_data_write() {
        let mut uart = Uart::new("test".to_string());
        let result = uart.write(UART_DATA_REG, &[b'A']);
        assert!(result.is_ok());
    }

    #[test]
    fn test_invalid_register() {
        let mut uart = Uart::new("test".to_string());
        let result = uart.read(0x100, 1);
        assert!(result.is_err());
    }
}
