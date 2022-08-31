use crate::roms::Mapper;

use super::{Mos6502, MicrocodeTask};

pub type MicrocodeReadOperation = fn(&mut Mos6502, data: u8);
pub type MicrocodeWriteOperation = fn(&mut Mos6502) -> u8;

pub trait AddressingModes {
    fn immediate(self, cpu: &mut Mos6502);
    // fn immediate(self: &mut Self, operation: MicrocodeReadOperation);

    // // I'll probably need to rename this as absolute addressing for JMP commands don' have a follow-up bus read or write
    fn absolute(self, cpu: &mut Mos6502);
    // fn absolute(self: &mut Self, operation: MicrocodeReadOperation);
}

impl AddressingModes for MicrocodeReadOperation {
    fn immediate(self, cpu: &mut Mos6502) {
        cpu.queue_task(MicrocodeTask::Read(Mos6502::read_pc, self));
    }

    fn absolute(self, cpu: &mut Mos6502) {
        cpu.queue_read(Mos6502::read_pc_increment, Mos6502::set_address_low);
        cpu.queue_read(Mos6502::read_pc, self);
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
        cpu.queue_write(Mos6502::write_address, self);
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
