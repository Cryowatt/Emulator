use crate::roms::Mapper;

use super::{MicrocodeReadOperation, Mos6502, MicrocodeTask};

pub trait AddressingModes {
    fn immediate(self: &mut Self, operation: MicrocodeReadOperation);

    // I'll probably need to rename this as absolute addressing for JMP commands don' have a follow-up bus read or write
    fn absolute(self: &mut Self, operation: MicrocodeReadOperation);
}

impl AddressingModes for Mos6502{
    fn immediate(self: &mut Self, operation: MicrocodeReadOperation) {
        let read = |cpu: &mut Mos6502, mapper: &mut dyn Mapper| {
            let data = mapper.read(cpu.pc);
            data
        };
        
        self.queue_task(MicrocodeTask::Read(read, operation));
    }

    fn absolute(self: &mut Self, operation: MicrocodeReadOperation) {
        let read = |cpu: &mut Mos6502, mapper: &mut dyn Mapper| {
            let data = mapper.read(cpu.pc);
            data
        };
        
        self.queue_read(Self::read_pc_increment, Self::set_address_low);
        self.queue_read(Self::read_pc, operation);
    }
}