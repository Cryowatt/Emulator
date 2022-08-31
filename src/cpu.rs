mod addressing_modes;

use std::{collections::VecDeque};

use bitflags::bitflags;

use crate::{roms::Mapper, address::Address};

use self::addressing_modes::{AddressingModes, MicrocodeReadOperation, MicrocodeWriteOperation};

enum MicrocodeTask {
    Read(BusRead, MicrocodeReadOperation),
    Write(BusWrite, MicrocodeWriteOperation),
}

type BusRead = fn(&mut Mos6502, &mut dyn Mapper) -> u8;
type BusWrite = fn(&mut Mos6502, &mut dyn Mapper, data: u8);
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

    opcode: u8,
    address: u16,

    cycle_microcode_queue: VecDeque<MicrocodeTask>,
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

            opcode: 0x00,
            address: 0x0000,

            cycle_microcode_queue: VecDeque::with_capacity(8),
        }
    }

    fn queue_task(self: &mut Self, task: MicrocodeTask) {
        self.cycle_microcode_queue.push_back(task);
    }

    fn queue_read(self: &mut Self, io_op: BusRead, op: MicrocodeReadOperation) {
        self.cycle_microcode_queue.push_back(MicrocodeTask::Read(io_op, op));
    }

    fn queue_write(self: &mut Self, io_op: BusWrite, op: MicrocodeWriteOperation) {
        self.cycle_microcode_queue.push_back(MicrocodeTask::Write(io_op, op));
    }

    fn read_pc_op(operation: MicrocodeReadOperation) -> MicrocodeTask {
        let read = |cpu: &mut Mos6502, mapper: &mut dyn Mapper| {
            let data = mapper.read(cpu.pc);
            cpu.pc += 1;
            data
        };
        
        MicrocodeTask::Read(read, operation)
    }

    fn read_pc_increment(cpu: &mut Mos6502, mapper: &mut dyn Mapper) -> u8 {
        let data = Self::read_pc(cpu, mapper);
        cpu.pc += 1; 
        data
    }

    fn read_pc(cpu: &mut Mos6502, mapper: &mut dyn Mapper) -> u8 {
        mapper.read(cpu.pc)
    }

    fn read_fixed<const ADDRESS: u16>(cpu: &mut Mos6502, mapper: &mut dyn Mapper) -> u8 {
        mapper.read(ADDRESS)
    }

    // fn push_stack(self: &mut Self, mapper: &mut dyn Mapper, data: u8) {
    //     mapper.write((self.s & 0xff) as u16 + STACK_OFFSET, data);
    //     self.s -= 1;
    // }

    fn push_stack(cpu: &mut Mos6502, mapper: &mut dyn Mapper, data: u8) {
        mapper.write((cpu.s & 0xff) as u16 + STACK_OFFSET, data);
        cpu.s -= 1;
    }

    fn set_address_low(cpu: &mut Mos6502, data: u8) {
        cpu.address.set_low(data);
    }

    fn set_address_high(cpu: &mut Mos6502, data: u8) {
        cpu.address.set_high(data);
    }

    fn write_address(cpu: &mut Mos6502, mapper: &mut dyn Mapper, data: u8) {
        mapper.write(cpu.address, data);
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
        self.queue_read(Self::read_pc, |cpu, _| cpu.p.set(Status::INTERRUPT_DISABLE, true));
        self.queue_write(Self::push_stack, |cpu| cpu.pc.get_high());
        self.queue_write(Self::push_stack, |cpu| cpu.pc.get_low());
        self.queue_write(Self::push_stack, |cpu| cpu.p.bits);
        self.queue_read(Self::read_fixed::<0xfffe>, |cpu, data| cpu.pc.set_low(data));
        self.queue_read(Self::read_fixed::<0xffff>, |cpu, data| {
            println!("BRK");
            cpu.pc.set_high(data);
        });
    }

    fn jmp(self: &mut Self) {
        self.queue_read(Mos6502::read_pc_increment, Mos6502::set_address_low);
        self.queue_read(Mos6502::read_pc, |cpu, data| {
            cpu.pc = u16::from_high_low(data, cpu.address.get_low());
            println!("JMP ${:X}", cpu.pc);
        });
    }

    fn sei(cpu: &mut Mos6502, _: u8) {
        cpu.p.set(Status::INTERRUPT_DISABLE, true);
    }

    fn cld(cpu: &mut Mos6502, _: u8) {
        cpu.p.set(Status::DECIMAL, false);
    }

    fn sta(cpu: &mut Mos6502) -> u8 {
        cpu.a
    }

    fn lda(cpu: &mut Mos6502, data: u8) {
        cpu.a = data;
    }

    fn ldx(cpu: &mut Mos6502, data: u8) {
        cpu.x = data;
    }

    fn txs(cpu: &mut Mos6502, _: u8) {
        cpu.s = cpu.x;
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
            None => {
                MicrocodeTask::Read(Self::read_pc_increment, Self::decode_opcode)
            },
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
        self.opcode = opcode;
        match opcode {
            //00/04/08/0c/10/14/18/1c
            0x00 => self.brk(),
            0x4c => self.jmp(),
            0x78 => (Self::sei as MicrocodeReadOperation).implied(self),
            0xd8 => (Self::cld as MicrocodeReadOperation).implied(self),
            //01/05/09/0d/11/15/19/1d
            0x8d => (Self::sta as MicrocodeWriteOperation).absolute(self),
            0xa9 => (Self::lda as MicrocodeReadOperation).immediate(self),
            //02/06/0a/0e/12/16/1a/1e
            0x9a => (Self::txs as MicrocodeReadOperation).implied(self),
            0xa2 => (Self::ldx as MicrocodeReadOperation).immediate(self),
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
