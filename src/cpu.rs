use std::{collections::VecDeque};

use bitflags::bitflags;

use crate::roms::Mapper;

//type MicrocodeTask = fn(&mut Mos6502, &mut dyn Mapper);
enum MicrocodeTask {
    Read(BusRead, MicrocodeReadOperation),
    Write(BusWrite, MicrocodeWriteOperation),
}

type BusRead = fn(&mut Mos6502, &mut dyn Mapper) -> u8;
type BusWrite = fn(&mut Mos6502, &mut dyn Mapper, data: u8);
type MicrocodeReadOperation = fn(&mut Mos6502, data: u8);
type MicrocodeWriteOperation = fn(&mut Mos6502) -> u8;
const STACK_OFFSET: u16 = 0x0100;

bitflags! {
    pub struct Status: u8 {
        const NONE = 0;
        const CARRY = 0b00000001;
        const ZERO = 0b00000010;
        const INTERRUPT_DISABLE = 0b00000100;
        const DECIMAL = 0b00001000;
        const UNDEFINED_5 = 0b00010000;
        const UNDEFINED_6 = 0b00100000;
        const OVERFLOW = 0b01000000;
        const NEGATIVE = 0b10000000;
        const DEFAULT = 0b00110000;
    }
}

pub struct Mos6502 {
    pub a: u8,
    pub pc: u16,
    pub p: Status,
    pub s: u8,
    pub x: u8,
    pub y: u8,

    cycle_microcode_queue: VecDeque<MicrocodeTask>,
}

trait Address {
    fn set_low(&mut self, value: u8);
    fn set_high(&mut self, value: u8);
    fn get_low(&mut self) -> u8;
    fn get_high(&mut self) -> u8;
}

impl Address for u16 {
    fn set_low(&mut self, value: u8) {
        *self = (*self & 0xff00) | value as u16;
    }

    fn set_high(&mut self, value: u8) {
        *self = (*self & 0x00ff) | ((value as u16) << 8);
    }

    fn get_low(&mut self) -> u8 {
        (*self & 0xff) as u8
    }

    fn get_high(&mut self) -> u8 {
        (*self >> 8 & 0xff) as u8
    }
}

impl Mos6502 {
    pub fn new() -> Self {
        Self {
            a: 0x00,
            pc: 0xfffc,
            p: Status::DEFAULT,
            s: 0xFD,
            x: 0x00,
            y: 0x00,

            cycle_microcode_queue: VecDeque::with_capacity(8),
        }
    }

    fn queue(self: &mut Self, task: MicrocodeTask) {
        self.cycle_microcode_queue.push_back(task);
    }

    fn read_pc(operation: MicrocodeReadOperation) -> MicrocodeTask {
        let read = |cpu: &mut Mos6502, mapper: &mut dyn Mapper| {
            let data = mapper.read(cpu.pc);
            cpu.pc += 1;
            data
        };
        
        MicrocodeTask::Read(read, operation)
    }

    // fn push_stack(self: &mut Self, mapper: &mut dyn Mapper, data: u8) {
    //     mapper.write((self.s & 0xff) as u16 + STACK_OFFSET, data);
    //     self.s -= 1;
    // }

    fn push_stack(operation: MicrocodeWriteOperation) -> MicrocodeTask {
        let write = |cpu: &mut Mos6502, mapper: &mut dyn Mapper, data: u8| {
            mapper.write((cpu.s & 0xff) as u16 + STACK_OFFSET, data);
            cpu.s -= 1;
        };

        MicrocodeTask::Write(write, operation)
    }

    fn immediate(operation: MicrocodeReadOperation) -> MicrocodeTask {
        let read = |cpu: &mut Mos6502, mapper: &mut dyn Mapper| {
            let data = mapper.read(cpu.pc);
            data
        };
        
        MicrocodeTask::Read(read, operation)
    }

    // fn push_from_pc_high(self: &mut Self, mapper: &mut dyn Mapper) {
    //     let data = self.pc.get_high();
    //     self.push_stack(mapper, data);
    // }

    // fn push_from_pc_low(self: &mut Self, mapper: &mut dyn Mapper) {
    //     let data = self.pc.get_low();
    //     self.push_stack(mapper, data);
    // }

    fn brk(self: &mut Self) {
        self.queue(Self::read_pc(|cpu, data| cpu.p.set(Status::INTERRUPT_DISABLE, true)));
        self.queue(Self::push_stack(|cpu| cpu.pc.get_high()));
        self.queue(Self::push_stack(|cpu| cpu.pc.get_low()));
        self.queue(Self::push_stack(|cpu| cpu.p.bits));
        self.queue(MicrocodeTask::Read(|cpu: &mut Mos6502, mapper: &mut dyn Mapper| mapper.read(0xfffe), |cpu, data| cpu.pc.set_low(data)));
        self.queue(MicrocodeTask::Read(|cpu: &mut Mos6502, mapper: &mut dyn Mapper| mapper.read(0xffff), |cpu, data| cpu.pc.set_high(data)));
    }

    fn jmp(self: &mut Self) {
        todo!();
    }

    fn sei(cpu: &mut Mos6502, _: u8) {
        cpu.p.set(Status::INTERRUPT_DISABLE, true);
    }
}

pub trait RP2A03 {
    fn zero_page_indexed();

    fn reset(self: &mut Self);

    fn cycle(self: &mut Self, mapper: &mut dyn Mapper);

    fn decode_opcode(self: &mut Self, opcode: u8);
}

impl RP2A03 for Mos6502 {
    fn zero_page_indexed() {}

    fn cycle(self: &mut Self, mapper: &mut dyn Mapper) {
        let microcode = match self.cycle_microcode_queue.pop_front() {
            Some(microcode) => microcode,
            None => Self::read_pc(Self::decode_opcode),
        };
        
        match microcode {
            MicrocodeTask::Read(read, op) => {
                let data = read(self, mapper);
                op(self, data);
            },
            MicrocodeTask::Write(write, op) => {
                let data = op(self);
                write(self, mapper, data);
            }
        }
    }

    //fn decode_opcode(self: &mut Self, mapper: &mut dyn Mapper) {
    fn decode_opcode(self: &mut Self, opcode: u8) {
        match opcode {
            //00/04/08/0c/10/14/18/1c
            0x00 => self.brk(),
            //0x4c => self.queue(Self::Absolute(self.jmp())),
            0x78 => self.queue(Self::immediate(Self::sei)),
            //01/05/09/0d/11/15/19/1d
            //02/06/0a/0e/12/16/1a/1e
            //03/07/0b/0f/13/17/1b/1f

            //83 =>  |cpu, mapper| cpu.sax(),
            
            _ => panic!("Unsupported opcode {:x}", opcode),
        }
        
        //todo!();
    }

    fn reset(self: &mut Self) {
        self.cycle_microcode_queue.push_back(MicrocodeTask::Read(
            |cpu, mapper| mapper.read(0xfffc),
            |cpu, data| cpu.pc.set_low(data)));
        // Enqueue(PushStackFromPCH);
        self.cycle_microcode_queue.push_back(MicrocodeTask::Read(
            |cpu, mapper| mapper.read(0xfffd),
            |cpu, data| cpu.pc.set_high(data)));
    }
}
