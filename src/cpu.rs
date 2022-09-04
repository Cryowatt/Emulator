mod addressing_modes;
pub mod instructions;

use std::{collections::VecDeque};

use bitflags::bitflags;

use crate::{roms::Mapper, address::Address};

use self::{addressing_modes::*, instructions::{ReadOperation, WriteOperation, BranchOperation, ReadWriteOperation}};
pub use self::{addressing_modes::{AddressingModes, MicrocodeReadOperation, MicrocodeWriteOperation}, instructions::MOS6502Instructions};

enum MicrocodeTask {
    Branch(BusRead, BranchOperation, Microcode<BusRead, BranchOperation>),
    Read(BusRead, ReadOperation, Microcode<BusRead, ReadOperation>),
    Write(BusWrite, WriteOperation, Microcode<BusWrite, WriteOperation>),
    ReadWrite(BusWrite, ReadWriteOperation, Microcode<BusWrite, ReadWriteOperation>),
}

type GetInput = fn(&mut Mos6502) -> u8;
type BusRead = fn(&mut Mos6502) -> u8;
type BusWrite = fn(&mut Mos6502, data: u8);
const STACK_OFFSET: u16 = 0x0100;

bitflags! {
    pub struct Status: u8 {
        const NONE = 0;
        const CARRY = 0b00000001;
        const ZERO = 0b00000010;
        const INTERRUPT_DISABLE = 0b00000100;
        const DECIMAL = 0b00001000;
        const BREAK = 0b00110000;
        const IRQ = 0b00100000;
        const NMI = 0b00100000;
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
    operand: u8,
    address: u16,
    data: u8,
    address_carry: bool,
    pointer: u8,
    scratch: u8,
    pub mapper: Box<dyn Mapper>,

    cycle_microcode_queue: VecDeque<MicrocodeTask>,
}

impl Mos6502 {
    pub fn new(mapper: Box<dyn Mapper>) -> Self {
        Self {
            a: 0x00,
            pc: 0xfffc,
            p: Status::DEFAULT,
            s: 0xFD,
            x: 0x00,
            y: 0x00,

            opcode: 0x00,
            operand: 0x00,
            address: 0x0000,
            data: 0x00,
            address_carry: false,
            pointer: 0x00,
            scratch: 0x00,
            mapper,

            cycle_microcode_queue: VecDeque::with_capacity(8),
        }
    }

    fn queue_task(self: &mut Self, task: MicrocodeTask) {
        self.cycle_microcode_queue.push_back(task);
    }

    fn queue_branch_microcode(&mut self, io: BusRead, op: BranchOperation, microcode: Microcode<BusRead, BranchOperation>) {
        self.cycle_microcode_queue.push_back(MicrocodeTask::Branch(io, op, microcode));
    }

    fn queue_read(&mut self, io: BusRead, op: ReadOperation)
    {
        self.queue_read_microcode(io, op, |cpu, io, op| {
            let data = io(cpu);
            op(cpu, data)
        });
    }

    fn queue_read_microcode(&mut self, io: BusRead, op: ReadOperation, microcode: Microcode<BusRead, ReadOperation>)
    {
        self.cycle_microcode_queue.push_back(MicrocodeTask::Read(io, op, microcode));
    }

    fn queue_write(&mut self, io: BusWrite, op: WriteOperation) {
        self.queue_write_microcode(io, op, |cpu, io, op| {
            let data = op(cpu);
            io(cpu, data)
        });
    }

    fn queue_write_microcode(&mut self, io: BusWrite, op: WriteOperation, microcode: Microcode<BusWrite, WriteOperation>) {
        self.cycle_microcode_queue.push_back(MicrocodeTask::Write(io, op, microcode));
    }

    fn queue_read_write(self: &mut Self, io: BusWrite, op: ReadWriteOperation, microcode: Microcode<BusWrite, ReadWriteOperation>) {
        self.cycle_microcode_queue.push_back(MicrocodeTask::ReadWrite(io, op, microcode));
    }

    fn queue_read_write_microcode(&mut self, io: BusWrite, op: ReadWriteOperation, microcode: Microcode<BusWrite, ReadWriteOperation>) {
        self.cycle_microcode_queue.push_back(MicrocodeTask::ReadWrite(io, op, microcode));
    }

    pub fn read(&mut self, address: u16) -> u8 {
        let data = self.mapper.read(address);
        println!("\tCPU #${:02x} <- ${:04X}", data, address);
        data
    }

    fn read_address(&mut self) -> u8 {
        self.read(self.address)
    }

    fn read_pc_increment(&mut self) -> u8 {
        let data = self.read_pc();
        self.pc += 1; 
        data
    }

    fn read_pc(&mut self) -> u8 {
        self.read(self.pc)
    }

    fn read_pointer(&mut self) -> u8 {
        self.read(self.pointer as u16)
    }

    fn read_pointer_increment(&mut self) -> u8 {
        let data = self.read_pointer();
        self.pointer += 1;
        data
    }

    fn read_fixed<const ADDRESS: u16>(&mut self) -> u8 {
        self.read(ADDRESS)
    }

    fn read_stack(&mut self) -> u8 {
        self.read(STACK_OFFSET + self.s as u16)
    }

    fn pop_stack(&mut self) -> u8 {
        let data = self.read(STACK_OFFSET + self.s as u16);
        self.s += 1;
        data
    }

    fn push_stack(&mut self, data: u8) {
        self.write((self.s & 0xff) as u16 + STACK_OFFSET, data);
        self.s -= 1;
    }

    fn set_scratch(&mut self, data: u8) {
        self.scratch = data;
    }

    fn set_zero_page_address(&mut self, data: u8) {
        self.address = data as u16;
    }

    fn set_address_low(&mut self, data: u8) {
        self.address.set_low(data);
    }

    fn set_address_high(&mut self, data: u8) {
        self.address.set_high(data);
    }

    fn set_pc_low(&mut self, data: u8) {
        self.pc.set_low(data);
    }

    fn set_pc_high(&mut self, data: u8) {
        self.pc.set_high(data);
    }

    pub fn write(&mut self, address: u16, data: u8) {
        self.mapper.write(address, data);
        println!("\tCPU #${:02x} -> ${:04X}", data, address);
    }

    fn write_address(&mut self, data: u8) {
        self.write(self.address, data);
    }

    fn set_negative_flag(&mut self, data: u8) {
        self.p.set(Status::NEGATIVE, (data as i8) < 0);
    }

    fn set_zero_flag(&mut self, data: u8) {
        self.p.set(Status::ZERO, data == 0);
    }
}

pub trait RP2A03 {
    fn reset(self: &mut Self);

    fn cycle(&mut self);

    fn decode_opcode(self: &mut Self, opcode: u8);
}

impl RP2A03 for Mos6502 {
    fn cycle(&mut self) {
        let microcode = match self.cycle_microcode_queue.pop_front() {
            Some(microcode) => microcode,
            None => {
                MicrocodeTask::Read(Self::read_pc_increment, Self::decode_opcode, 
                    |cpu, io, op| {
                        let data = io(cpu);
                        op(cpu, data)
                    })
            },
        };
        
        match microcode {
            MicrocodeTask::Branch(io, op, microcode) =>  microcode(self, io, op),
            MicrocodeTask::Read(io, op, microcode) => microcode(self, io, op),
            MicrocodeTask::Write(io, op, microcode) => microcode(self, io, op),
            MicrocodeTask::ReadWrite(io, op, microcode) => microcode(self, io, op),
        }
    }

    //fn decode_opcode(self: &mut Self, mapper: &mut dyn Mapper) {
    fn decode_opcode(self: &mut Self, opcode: u8) {
        self.opcode = opcode;
        match opcode {
            //00/04/08/0c/10/14/18/1c
            0x00 => self.brk(),
            0x20 => self.jsr(),
            0x4c => self.jmp(),
            0x60 => self.rts(),
            0x78 => (Self::sei as MicrocodeReadOperation).implied(self),
            0x84 => (Self::sty as MicrocodeWriteOperation).zero_page(self),
            0xa0 => (Self::ldy as MicrocodeReadOperation).immediate(self),
            0xc8 => (Self::iny as MicrocodeReadOperation).implied(self),
            0xd0 => (Self::bne as MicrocodeBranchOperation).relative(self),
            0xd8 => (Self::cld as MicrocodeReadOperation).implied(self),
            0xe8 => (Self::inx as MicrocodeReadOperation).implied(self),
            //01/05/09/0d/11/15/19/1d
            0x65 => (Self::adc as MicrocodeReadOperation).zero_page(self),
            0x8d => (Self::sta as MicrocodeWriteOperation).absolute(self),
            0x91 => (Self::sta as MicrocodeWriteOperation).indirect_indexed_y(self),
            0x95 => (Self::sta as MicrocodeWriteOperation).zero_page_indexed_x(self),
            0x9d => (Self::sta as MicrocodeWriteOperation).absolute_indexed_x(self),
            0xa9 => (Self::lda as MicrocodeReadOperation).immediate(self),
            //02/06/0a/0e/12/16/1a/1e
            0x0a => (Self::asl as MicrocodeReadWriteOperation).accumulator(self),
            0x0e => (Self::asl as MicrocodeReadWriteOperation).absolute(self),
            0x8a => (Self::txa as MicrocodeReadOperation).immediate(self),
            0x86 => (Self::stx as MicrocodeWriteOperation).zero_page(self),
            0x8e => (Self::stx as MicrocodeWriteOperation).absolute(self),
            0x9a => (Self::txs as MicrocodeReadOperation).implied(self),
            0xa2 => (Self::ldx as MicrocodeReadOperation).immediate(self),
            0xaa => (Self::tax as MicrocodeReadOperation).implied(self),
            0xba => (Self::tsx as MicrocodeReadOperation).implied(self),
            0xca => (Self::dex as MicrocodeReadOperation).implied(self),
            0xe6 => (Self::inc as MicrocodeReadWriteOperation).zero_page(self),
            //03/07/0b/0f/13/17/1b/1f

            _ => panic!("Unsupported opcode {:02x}", opcode),
        }
        
        //todo!();
    }

    fn reset(self: &mut Self) {
        self.queue_read(Self::read_fixed::<0xfffc>, Self::set_pc_low);
        self.queue_read(Self::read_fixed::<0xfffd>, Self::set_pc_high);
    }
}