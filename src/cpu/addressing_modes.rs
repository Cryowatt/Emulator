use std::cmp::Ordering;

use crate::address::Address;

use super::Mos6502;

pub type MicrocodeBranchOperation = fn(&mut Mos6502) -> bool;
pub type MicrocodeConditionalOperation = fn(&mut Mos6502, data: u8, condition_met: bool);
pub type MicrocodeReadOperation = fn(&mut Mos6502, data: u8);
pub type MicrocodeWriteOperation = fn(&mut Mos6502) -> u8;
pub type MicrocodeReadWriteOperation = fn(&mut Mos6502, data: u8) -> u8;

pub type Microcode<TIo, TOp> = fn(&mut Mos6502, io: TIo, op: TOp);

const OPCODES: [&'static str; 256] = [
    "BRK", "ORA", "STP", "SLO", "NOP", "ORA", "ASL", "SLO", "PHP", "ORA", "ASL", "ANC", "NOP",
    "ORA", "ASL", "SLO", "BPL", "ORA", "STP", "SLO", "NOP", "ORA", "ASL", "SLO", "CLC", "ORA",
    "NOP", "SLO", "NOP", "ORA", "ASL", "SLO", "JSR", "AND", "STP", "RLA", "BIT", "AND", "ROL",
    "RLA", "PLP", "AND", "ROL", "ANC", "BIT", "AND", "ROL", "RLA", "BMI", "AND", "STP", "RLA",
    "NOP", "AND", "ROL", "RLA", "SEC", "AND", "NOP", "RLA", "NOP", "AND", "ROL", "RLA", "RTI",
    "EOR", "STP", "SRE", "NOP", "EOR", "LSR", "SRE", "PHA", "EOR", "LSR", "ALR", "JMP", "EOR",
    "LSR", "SRE", "BVC", "EOR", "STP", "SRE", "NOP", "EOR", "LSR", "SRE", "CLI", "EOR", "NOP",
    "SRE", "NOP", "EOR", "LSR", "SRE", "RTS", "ADC", "STP", "RRA", "NOP", "ADC", "ROR", "RRA",
    "PLA", "ADC", "ROR", "ARR", "JMP", "ADC", "ROR", "RRA", "BVS", "ADC", "STP", "RRA", "NOP",
    "ADC", "ROR", "RRA", "SEI", "ADC", "NOP", "RRA", "NOP", "ADC", "ROR", "RRA", "NOP", "STA",
    "NOP", "SAX", "STY", "STA", "STX", "SAX", "DEY", "NOP", "TXA", "XAA", "STY", "STA", "STX",
    "SAX", "BCC", "STA", "STP", "AHX", "STY", "STA", "STX", "SAX", "TYA", "STA", "TXS", "TAS",
    "SHY", "STA", "SHX", "AHX", "LDY", "LDA", "LDX", "LAX", "LDY", "LDA", "LDX", "LAX", "TAY",
    "LDA", "TAX", "LAX", "LDY", "LDA", "LDX", "LAX", "BCS", "LDA", "STP", "LAX", "LDY", "LDA",
    "LDX", "LAX", "CLV", "LDA", "TSX", "LAS", "LDY", "LDA", "LDX", "LAX", "CPY", "CMP", "NOP",
    "DCP", "CPY", "CMP", "DEC", "DCP", "INY", "CMP", "DEX", "AXS", "CPY", "CMP", "DEC", "DCP",
    "BNE", "CMP", "STP", "DCP", "NOP", "CMP", "DEC", "DCP", "CLD", "CMP", "NOP", "DCP", "NOP",
    "CMP", "DEC", "DCP", "CPX", "SBC", "NOP", "ISC", "CPX", "SBC", "INC", "ISC", "INX", "SBC",
    "NOP", "SBC", "CPX", "SBC", "INC", "ISC", "BEQ", "SBC", "STP", "ISC", "NOP", "SBC", "INC",
    "ISC", "SED", "SBC", "NOP", "ISC", "NOP", "SBC", "INC", "ISC",
];

pub trait BranchAddressingModes {
    fn relative(self, cpu: &mut Mos6502);
}

impl BranchAddressingModes for MicrocodeBranchOperation {
    fn relative(self, cpu: &mut Mos6502) {
        cpu.queue_branch_microcode(Mos6502::read_pc_increment, self, |cpu, io, op| {
            cpu.operand = io(cpu);
            let should_branch = op(cpu);
            if should_branch {
                cpu.queue_read(Mos6502::read_pc, |cpu, _| {
                    let (low, carry) = cpu.pc.get_low().overflowing_add_signed(cpu.operand as i8);
                    cpu.pc.set_low(low);

                    if carry {
                        cpu.queue_read(Mos6502::read_pc, |cpu, _| {
                            let high = match (cpu.operand as i8).cmp(&0) {
                                Ordering::Less => cpu.pc.get_high() - 1,
                                Ordering::Equal | Ordering::Greater => cpu.pc.get_high() + 1,
                            };
                            cpu.pc.set_high(high);
                            println!("{} ${:04X}", OPCODES[cpu.opcode as usize], cpu.pc);
                        });
                    } else {
                        println!("{} ${:04X}", OPCODES[cpu.opcode as usize], cpu.pc);
                    }
                });
            } else {
                println!("{}!!", OPCODES[cpu.opcode as usize]);
            }
        });
    }
}

pub trait AddressingModes {
    fn absolute(self, cpu: &mut Mos6502);
    fn absolute_indexed_x(self, cpu: &mut Mos6502);
    fn accumulator(self, cpu: &mut Mos6502);
    fn immediate(self, cpu: &mut Mos6502);
    fn implied(self, cpu: &mut Mos6502);
    fn indirect_indexed_y(self, cpu: &mut Mos6502);
    fn zero_page(self, cpu: &mut Mos6502);
    fn zero_page_indexed_x(self, cpu: &mut Mos6502);
}

impl AddressingModes for MicrocodeReadOperation {
    fn absolute(self, cpu: &mut Mos6502) {
        todo!();
    }

    fn absolute_indexed_x(self, cpu: &mut Mos6502) {
        todo!()
    }

    fn accumulator(self, cpu: &mut Mos6502) {
        todo!();
    }

    fn immediate(self, cpu: &mut Mos6502) {
        cpu.queue_read_microcode(Mos6502::read_pc_increment, self, |cpu, io, op| {
            let data = io(cpu);
            op(cpu, data);
            println!("{} #${:02X}", OPCODES[cpu.opcode as usize], data);
        });
    }

    fn implied(self, cpu: &mut Mos6502) {
        cpu.queue_read_microcode(Mos6502::read_pc, self, |cpu, io, op| {
            let data = io(cpu);
            op(cpu, data);
            println!("{}", OPCODES[cpu.opcode as usize]);
        });
    }

    fn indirect_indexed_y(self, cpu: &mut Mos6502) {
        todo!()
    }

    fn zero_page(self, cpu: &mut Mos6502) {
        cpu.queue_read(Mos6502::read_pc_increment, Mos6502::set_zero_page_address);
        cpu.queue_read_microcode(Mos6502::read_address, self, |cpu, io, op| {
            let data = io(cpu);
            op(cpu, data);
            println!("{} ${:02X}", OPCODES[cpu.opcode as usize], cpu.address);
        });
    }

    fn zero_page_indexed_x(self, cpu: &mut Mos6502) {
        todo!()
    }
}

impl AddressingModes for MicrocodeWriteOperation {
    fn absolute(self, cpu: &mut Mos6502) {
        cpu.queue_read(Mos6502::read_pc_increment, Mos6502::set_address_low);
        cpu.queue_read(Mos6502::read_pc_increment, Mos6502::set_address_high);
        cpu.queue_write_microcode(Mos6502::write_address, self, |cpu, io, op| {
            let data = op(cpu);
            io(cpu, data);
            println!("{} ${:04X}", OPCODES[cpu.opcode as usize], cpu.address);
        });
    }

    fn absolute_indexed_x(self, cpu: &mut Mos6502) {
        cpu.queue_read(Mos6502::read_pc_increment, Mos6502::set_address_low);
        cpu.queue_read(Mos6502::read_pc_increment, |cpu, data| {
            cpu.set_address_high(data);
            let (low, carry) = cpu.address.get_low().overflowing_add(cpu.x);
            cpu.set_address_low(low);
            cpu.address_carry = carry;
        });
        cpu.queue_read(Mos6502::read_address, |cpu, data| {
            if(cpu.address_carry) {
                let high = cpu.address.get_high();
                cpu.set_address_high(high + 1);
            }
        });
        cpu.queue_write(Mos6502::write_address, self);
    }

    fn accumulator(self, cpu: &mut Mos6502) {
        todo!()
    }

    fn immediate(self, cpu: &mut Mos6502) {
        todo!();
    }

    fn implied(self, cpu: &mut Mos6502) {
        todo!()
    }

    fn indirect_indexed_y(self, cpu: &mut Mos6502) {
        cpu.queue_read(Mos6502::read_pc_increment, |cpu, data| cpu.pointer = data);
        cpu.queue_read(Mos6502::read_pointer_increment, Mos6502::set_address_low);
        cpu.queue_read(Mos6502::read_pointer_increment, |cpu, data| {
            cpu.set_address_high(data);
            let (low, carry) = cpu.address.get_low().overflowing_add(cpu.y);
            cpu.set_address_low(low);
            cpu.address_carry = carry;
        });
        cpu.queue_read(Mos6502::read_address, |cpu, _| {
            if cpu.address_carry {
                let high = cpu.address.get_high() + 1;
                cpu.address.set_high(high);
            }
        });
        cpu.queue_write_microcode(Mos6502::write_address, self, |cpu, io, op| {
            let data = op(cpu);
            io(cpu, data);
            println!("{} (${:02X}),Y", OPCODES[cpu.opcode as usize], cpu.pointer);
        });
    }

    fn zero_page(self, cpu: &mut Mos6502) {
        cpu.queue_read(Mos6502::read_pc_increment, Mos6502::set_zero_page_address);
        cpu.queue_write_microcode(Mos6502::write_address, self, |cpu, io, op| {
            let data = op(cpu);
            io(cpu, data);
            println!("{} ${:02X}", OPCODES[cpu.opcode as usize], cpu.address);
        });
    }

    fn zero_page_indexed_x(self, cpu: &mut Mos6502) {
        cpu.queue_read(Mos6502::read_pc_increment, Mos6502::set_zero_page_address);
        cpu.queue_read(Mos6502::read_address, |cpu, data| {
            let low = cpu.address.get_low().wrapping_add(cpu.x);
            cpu.set_address_low(low);
        });
    }
}

impl AddressingModes for MicrocodeReadWriteOperation {
    fn absolute(self, cpu: &mut Mos6502) {
        cpu.queue_read(Mos6502::read_pc_increment, Mos6502::set_address_low);
        cpu.queue_read(Mos6502::read_pc_increment, Mos6502::set_address_high);
        cpu.queue_read(Mos6502::read_address, |cpu, data| cpu.operand = data);
        cpu.queue_read_write_microcode(Mos6502::write_address, self, |cpu, io, op| {
            io(cpu, cpu.data);
            cpu.data = op(cpu, cpu.operand);
        });
        cpu.queue_write_microcode(
            Mos6502::write_address,
            |cpu| cpu.data,
            |cpu, io, op| {
                let data = op(cpu);
                io(cpu, data);
                println!("{} ${:04X}", OPCODES[cpu.opcode as usize], cpu.address);
            },
        );
    }

    fn absolute_indexed_x(self, cpu: &mut Mos6502) {
        todo!()
    }

    fn accumulator(self, cpu: &mut Mos6502) {
        todo!();
        // cpu.queue_read_write(|cpu| cpu.a, |cpu, mapper, data| {
        //     cpu.a = data;
        //     println!("{} A", OPCODES[cpu.opcode as usize]);
        // }, self);
        //     let data = Mos6502::read_pc(cpu, mapper);
        //     println!("{} A", OPCODES[cpu.opcode as usize]);
        //     data
        // }, self));
    }

    fn immediate(self, cpu: &mut Mos6502) {
        todo!()
    }

    fn implied(self, cpu: &mut Mos6502) {
        todo!()
    }

    fn indirect_indexed_y(self, cpu: &mut Mos6502) {
        todo!()
    }

    fn zero_page(self, cpu: &mut Mos6502) {
        cpu.queue_read(Mos6502::read_pc_increment, Mos6502::set_zero_page_address);
        cpu.queue_read(Mos6502::read_address, |cpu, data| cpu.operand = data);
        cpu.queue_read_write_microcode(Mos6502::write_address, self, |cpu, io, op| {
            io(cpu, cpu.data);
            cpu.data = op(cpu, cpu.operand);
        });
        cpu.queue_write_microcode(
            Mos6502::write_address,
            |cpu| cpu.data,
            |cpu, io, op| {
                let data = op(cpu);
                io(cpu, data);
                println!("{} ${:02X}", OPCODES[cpu.opcode as usize], cpu.address);
            },
        );
    }

    fn zero_page_indexed_x(self, cpu: &mut Mos6502) {
        todo!()
    }
}
