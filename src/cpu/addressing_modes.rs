
use std::{ops::Add, cmp::Ordering};

use crate::{roms::Mapper, address::Address};

use super::{Mos6502, MicrocodeTask};

pub type MicrocodeBranchOperation = fn(&mut Mos6502) -> bool;
pub type MicrocodeConditionalOperation = fn(&mut Mos6502, data :u8, condition_met: bool);
pub type MicrocodeReadOperation = fn(&mut Mos6502, data: u8);
pub type MicrocodeWriteOperation = fn(&mut Mos6502) -> u8;


const OPCODES: [&'static str; 256] = [
    "BRK","ORA","STP","SLO","NOP","ORA","ASL","SLO","PHP","ORA","ASL","ANC","NOP","ORA","ASL","SLO",
    "BPL","ORA","STP","SLO","NOP","ORA","ASL","SLO","CLC","ORA","NOP","SLO","NOP","ORA","ASL","SLO",
    "JSR","AND","STP","RLA","BIT","AND","ROL","RLA","PLP","AND","ROL","ANC","BIT","AND","ROL","RLA",
    "BMI","AND","STP","RLA","NOP","AND","ROL","RLA","SEC","AND","NOP","RLA","NOP","AND","ROL","RLA",
    "RTI","EOR","STP","SRE","NOP","EOR","LSR","SRE","PHA","EOR","LSR","ALR","JMP","EOR","LSR","SRE",
    "BVC","EOR","STP","SRE","NOP","EOR","LSR","SRE","CLI","EOR","NOP","SRE","NOP","EOR","LSR","SRE",
    "RTS","ADC","STP","RRA","NOP","ADC","ROR","RRA","PLA","ADC","ROR","ARR","JMP","ADC","ROR","RRA",
    "BVS","ADC","STP","RRA","NOP","ADC","ROR","RRA","SEI","ADC","NOP","RRA","NOP","ADC","ROR","RRA",
    "NOP","STA","NOP","SAX","STY","STA","STX","SAX","DEY","NOP","TXA","XAA","STY","STA","STX","SAX",
    "BCC","STA","STP","AHX","STY","STA","STX","SAX","TYA","STA","TXS","TAS","SHY","STA","SHX","AHX",
    "LDY","LDA","LDX","LAX","LDY","LDA","LDX","LAX","TAY","LDA","TAX","LAX","LDY","LDA","LDX","LAX",
    "BCS","LDA","STP","LAX","LDY","LDA","LDX","LAX","CLV","LDA","TSX","LAS","LDY","LDA","LDX","LAX",
    "CPY","CMP","NOP","DCP","CPY","CMP","DEC","DCP","INY","CMP","DEX","AXS","CPY","CMP","DEC","DCP",
    "BNE","CMP","STP","DCP","NOP","CMP","DEC","DCP","CLD","CMP","NOP","DCP","NOP","CMP","DEC","DCP",
    "CPX","SBC","NOP","ISC","CPX","SBC","INC","ISC","INX","SBC","NOP","SBC","CPX","SBC","INC","ISC",
    "BEQ","SBC","STP","ISC","NOP","SBC","INC","ISC","SED","SBC","NOP","ISC","NOP","SBC","INC","ISC"];

pub trait BranchAddressingModes {
    fn relative(self,  cpu: &mut Mos6502);
}

impl BranchAddressingModes for MicrocodeBranchOperation {
    fn relative(self,  cpu: &mut Mos6502) {
        cpu.queue_branch(Mos6502::read_pc_increment, self, |cpu, data, condition_met| {
            cpu.pointer = data;
            if condition_met {
                cpu.queue_read(Mos6502::read_pc_increment, |cpu, data| {
                    let (low, carry) = cpu.pc.get_low().overflowing_add_signed(data as i8);
                    cpu.pc.set_low(low);

                    if carry {
                        cpu.queue_read(Mos6502::read_pc_increment, |cpu, data| {
                            let high = match (cpu.pointer as i8).cmp(&0) {
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
                println!("{} ${:04X}", OPCODES[cpu.opcode as usize], cpu.pc);
                //todo!("Figure out this fucking overflow scenario with a u8 + i8");
                // cpu.address_carry = carry;

                // if carry {
                //     cpu.queue_read(Mos6502::read_pc_increment, |c, data| {
                //         let high = c.pc.get_high();
                //     });
                // }
            } else {
                println!("{}!!", OPCODES[cpu.opcode as usize]);
            }
        });
        // cpu.queue_read(Mos6502::read_pc_increment, |cpu, data| {
        //     cpu.pointer = data;

        //     if()
        // });
        // cpu.queue_branch(Mos6502::read_pc_increment, self, |cpu, condition_met| {
        //     if condition_met {
        //         let (low, carry) = cpu.pc.get_low().overflowing_add(cpu.pointer);
        //         cpu.pc.set_low(low);
        //         cpu.address_carry = carry;
        //     }
        // });
        // cpu.queue_read(Mos6502::read_pc_increment, |cpu, data| {
        //     cpu.pointer = data;
        // });

    //     Relative addressing (BCC, BCS, BNE, BEQ, BPL, BMI, BVC, BVS)

    //     #   address  R/W description
    //    --- --------- --- ---------------------------------------------
    //     1     PC      R  fetch opcode, increment PC
    //     2     PC      R  fetch operand, increment PC
    //     3     PC      R  Fetch opcode of next instruction,
    //                      If branch is taken, add operand to PCL.
    //                      Otherwise increment PC.
    //     4+    PC*     R  Fetch opcode of next instruction.
    //                      Fix PCH. If it did not change, increment PC.
    //     5!    PC      R  Fetch opcode of next instruction,
    //                      increment PC.

    //    Notes: The opcode fetch of the next instruction is included to
    //           this diagram for illustration purposes. When determining
    //           real execution times, remember to subtract the last
    //           cycle.

    //           * The high byte of Program Counter (PCH) may be invalid
    //             at this time, i.e. it may be smaller or bigger by $100.

    //           + If branch is taken, this cycle will be executed.

    //           ! If branch occurs to different page, this cycle will be
    //             executed.
    }
}

pub trait AddressingModes {
    fn absolute(self, cpu: &mut Mos6502);
    fn immediate(self, cpu: &mut Mos6502);
    fn implied(self, cpu: &mut Mos6502);
    fn indirect_indexed_y(self, cpu: &mut Mos6502);
    fn zero_page(self, cpu: &mut Mos6502);
}

impl AddressingModes for MicrocodeReadOperation {
    fn absolute(self, cpu: &mut Mos6502) {
        todo!();
        // cpu.queue_read(Mos6502::read_pc_increment, Mos6502::set_address_low);
        // cpu.queue_read(Mos6502::read_pc, self);
    }

    fn immediate(self, cpu: &mut Mos6502) {
        cpu.queue_task(MicrocodeTask::Read(|cpu, mapper| {
            let data = Mos6502::read_pc_increment(cpu, mapper);
            println!("{} #${:02X}", OPCODES[cpu.opcode as usize], data);
            data
        }, self));
    }

    fn implied(self, cpu: &mut Mos6502) {
        cpu.queue_task(MicrocodeTask::Read(|cpu, mapper| {
            let data = Mos6502::read_pc(cpu, mapper);
            println!("{}", OPCODES[cpu.opcode as usize]);
            data
        }, self));
    }

    fn indirect_indexed_y(self, cpu: &mut Mos6502) {
        todo!()
    }

    fn zero_page(self, cpu: &mut Mos6502) {
        todo!()
    }
}

impl AddressingModes for MicrocodeWriteOperation {
    fn absolute(self, cpu: &mut Mos6502) {
        cpu.queue_read(Mos6502::read_pc_increment, Mos6502::set_address_low);
        cpu.queue_read(Mos6502::read_pc_increment, Mos6502::set_address_high);
        cpu.queue_write(|cpu, mapper, data| {
            Mos6502::write_address(cpu, mapper, data);
            println!("{} ${:04X}", OPCODES[cpu.opcode as usize], cpu.address);
        }, self);
    }

    fn immediate(self, cpu: &mut Mos6502) {
        todo!();
        // let read = |cpu: &mut Mos6502, mapper: &mut dyn Mapper| {
        //     let data = mapper.read(cpu.pc);
        //     data
        // };
        
        // cpu.queue_task(MicrocodeTask::Read(read, self));
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
        cpu.queue_write(|cpu, mapper, data| {
            Mos6502::write_address(cpu, mapper, data);
            println!("{} (${:02X}),Y", OPCODES[cpu.opcode as usize], cpu.pointer);
        }, self);
    }

    fn zero_page(self, cpu: &mut Mos6502) {
        cpu.queue_read(Mos6502::read_pc_increment, Mos6502::set_zero_page_address);
        cpu.queue_write(|cpu, mapper, data| {
            Mos6502::write_address(cpu, mapper, data);
            println!("{} ${:02X}", OPCODES[cpu.opcode as usize], cpu.address);
        }, self);
    }
}