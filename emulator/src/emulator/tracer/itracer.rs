use super::super::Emulator;
use crate::const_values::INSTRUCTION_TRACER_LIST_SIZE;
use crate::emulator::tracer::TracerTrace;
use crate::utils::disasm_riscv64_with_details;
use crate::utils::ringbuf::RingBuffer;

/// 指令和地址结构体
#[derive(Debug, Clone, Copy, Default)]
struct Instruction {
    pc: u64,
    code: u32,
}

/// 指令追踪器
pub struct ITracer {
    instructions: RingBuffer<Instruction>,
}

impl ITracer {
    /// 创建新的指令追踪器
    pub fn new() -> Self {
        ITracer {
            instructions: RingBuffer::new(INSTRUCTION_TRACER_LIST_SIZE),
        }
    }
}

impl Default for ITracer {
    fn default() -> Self {
        Self::new()
    }
}

impl TracerTrace for ITracer {
    /// 追踪器名称
    fn name(&self) -> &'static str {
        "ITracer"
    }

    /// 追踪一条指令
    fn trace(&mut self, emulator: &Emulator) {
        let pc = emulator.state.get_pc();
        if let Ok(instruction) = emulator.state.fetch_instruction(pc) {
            self.instructions.push_overwrite(Instruction {
                pc,
                code: instruction,
            });
        }
    }

    /// 打印所有追踪的指令(带反汇编)
    fn get_instructions_log(&mut self) -> String {
        let mut log = String::new();
        let mut temp = Vec::new();
        while let Ok(inst) = self.instructions.pop() {
            temp.push(inst);
        }

        for inst in &temp {
            if let Ok(disasm) = disasm_riscv64_with_details(inst.code, inst.pc) {
                log += &format!("{:08x}: {:08x}  {}\n", inst.pc, inst.code, disasm);
            } else {
                log += &format!("{:08x}: {:08x}  <invalid>\n", inst.pc, inst.code);
            }
        }

        // 重新放回ringbuf
        for inst in temp {
            self.instructions.push_overwrite(inst);
        }
        log
    }
}
