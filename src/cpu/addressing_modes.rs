use crate::roms::Mapper;

use super::{Mos6502, MicrocodeTask};

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

pub trait AddressingModes {
    fn implied(self, cpu: &mut Mos6502);
    fn immediate(self, cpu: &mut Mos6502);
    // fn immediate(self: &mut Self, operation: MicrocodeReadOperation);

    // // I'll probably need to rename this as absolute addressing for JMP commands don' have a follow-up bus read or write
    fn absolute(self, cpu: &mut Mos6502);
    // fn absolute(self: &mut Self, operation: MicrocodeReadOperation);
}

impl AddressingModes for MicrocodeReadOperation {
    fn immediate(self, cpu: &mut Mos6502) {
        cpu.queue_task(MicrocodeTask::Read(|cpu, mapper| {
            let data = Mos6502::read_pc_increment(cpu, mapper);
            println!("{} #${:X}", OPCODES[cpu.opcode as usize], data);
            data
        }, self));
    }

    fn absolute(self, cpu: &mut Mos6502) {
        todo!();
        // cpu.queue_read(Mos6502::read_pc_increment, Mos6502::set_address_low);
        // cpu.queue_read(Mos6502::read_pc, self);
    }

    fn implied(self, cpu: &mut Mos6502) {
        cpu.queue_task(MicrocodeTask::Read(|cpu, mapper| {
            let data = Mos6502::read_pc(cpu, mapper);
            println!("{}", OPCODES[cpu.opcode as usize]);
            data
        }, self));
    }
}

impl AddressingModes for MicrocodeWriteOperation {
    fn immediate(self, cpu: &mut Mos6502) {
        todo!();
        // let read = |cpu: &mut Mos6502, mapper: &mut dyn Mapper| {
        //     let data = mapper.read(cpu.pc);
        //     data
        // };
        
        // cpu.queue_task(MicrocodeTask::Read(read, self));
    }

    fn absolute(self, cpu: &mut Mos6502) {
        cpu.queue_read(Mos6502::read_pc_increment, Mos6502::set_address_low);
        cpu.queue_read(Mos6502::read_pc_increment, Mos6502::set_address_high);
        cpu.queue_write(|cpu, mapper, data| {
            Mos6502::write_address(cpu, mapper, data);
            println!("{} ${:X}", OPCODES[cpu.opcode as usize], cpu.address);
        }, self);
    }

    fn implied(self, cpu: &mut Mos6502) {
        todo!()
    }
}

// impl AddressingModes for Mos6502{
//     fn immediate(self: &mut Self, operation: MicrocodeReadOperation) {
//         let read = |cpu: &mut Mos6502, mapper: &mut dyn Mapper| {
//             let data = mapper.read(cpu.pc);
//             data
//         };
        
//         self.queue_task(MicrocodeTask::Read(read, operation));
//     }

//     fn absolute(self: &mut Self, operation: MicrocodeReadOperation) {
//         let read = |cpu: &mut Mos6502, mapper: &mut dyn Mapper| {
//             let data = mapper.read(cpu.pc);
//             data
//         };
        
//         self.queue_read(Self::read_pc_increment, Self::set_address_low);
//         self.queue_read(Self::read_pc, operation);
//     }
// }
