mod addressing_modes;
pub mod instructions;

use std::{collections::VecDeque};

use bitflags::bitflags;

use crate::{roms::Mapper, address::Address};

use self::{addressing_modes::*, instructions::{ReadOperation, WriteOperation, BranchOperation, ReadWriteOperation}};
pub use self::{addressing_modes::{AddressingModes}, instructions::MOS6502Instructions};

enum MicrocodeTask {
    Branch(BusRead, BranchOperation, Microcode<BusRead, BranchOperation>),
    Read(BusRead, ReadOperation, Microcode<BusRead, ReadOperation>),
    Write(BusWrite, WriteOperation, Microcode<BusWrite, WriteOperation>),
    ReadWrite(BusWrite, ReadWriteOperation, Microcode<BusWrite, ReadWriteOperation>),
}

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
    pub cycle: u32,
    pub mapper: Box<dyn Mapper>,

    cycle_microcode_queue: VecDeque<MicrocodeTask>,
}

impl Mos6502 {
    pub fn new(mapper: Box<dyn Mapper>) -> Self {
        Self {
            a: 0x00,
            pc: 0xfffc,
            p: Status::IRQ | Status::INTERRUPT_DISABLE,
            s: 0xFD,
            x: 0x00,
            y: 0x00,

            opcode: 0x00,
            operand: 0x00,
            address: 0x0000,
            data: 0x00,
            address_carry: false,
            pointer: 0x00,
            mapper,
            cycle: 0,

            cycle_microcode_queue: VecDeque::with_capacity(8),
        }
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

    fn queue_read_write_microcode(&mut self, io: BusWrite, op: ReadWriteOperation, microcode: Microcode<BusWrite, ReadWriteOperation>) {
        self.cycle_microcode_queue.push_back(MicrocodeTask::ReadWrite(io, op, microcode));
    }

    pub fn read(&mut self, address: u16) -> u8 {
        let data = self.mapper.read(address);
        //println!("\tCPU #${:02x} <- ${:04X}", data, address);
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
        self.s += 1;
        let data = self.read_stack();
        data
    }

    fn push_stack(&mut self, data: u8) {
        self.write((self.s & 0xff) as u16 + STACK_OFFSET, data);
        self.s -= 1;
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
        //println!("\tCPU #${:02x} -> ${:04X}", data, address);
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
        self.cycle += 1;
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
        let operand0 = self.mapper.read(self.pc);
        let operand1 = self.mapper.read(self.pc + 1);
        println!("{PC:04X} {OP:02X} {ARG0:02X} {ARG1:02X} {Code} A:{A:02X} X:{X:02X} Y:{Y:02X} P:{P:02X} SP:{SP:02X} CYC:{CYC}", 
            PC = self.pc - 1, OP = opcode, ARG0 = operand0, ARG1 = operand1, Code = OPCODES[opcode as usize], A = self.a, X = self.x, Y = self.y, P = self.p.bits, SP = self.s, CYC = self.cycle);
        self.opcode = opcode;
        match opcode {
            //00/04/08/0c/10/14/18/1c
            0x00 => self.brk(),
            0x08 => self.php(),
            0x10 => (Self::bpl as BranchOperation).relative(self),
            0x20 => self.jsr(),
            0x28 => self.plp(),
            0x2c => (Self::bit as ReadOperation).absolute(self),
            0x30 => (Self::bmi as BranchOperation).relative(self),
            0x40 => self.rti(),
            0x48 => self.pha(),
            0x4c => self.jmp(),
            0x58 => (Self::cli as ReadOperation).implied(self),
            0x60 => self.rts(),
            0x68 => self.pla(),
            0x78 => (Self::sei as ReadOperation).implied(self),
            0x84 => (Self::sty as WriteOperation).zero_page(self),
            0x88 => (Self::dey as ReadOperation).implied(self),
            0x98 => (Self::tya as ReadOperation).implied(self),
            0xa0 => (Self::ldy as ReadOperation).immediate(self),
            0xa4 => (Self::ldy as ReadOperation).zero_page(self),
            0xa8 => (Self::tay as ReadOperation).implied(self),
            0xb4 => (Self::ldy as ReadOperation).zero_page_indexed_x(self),
            0xc8 => (Self::iny as ReadOperation).implied(self),
            0xd0 => (Self::bne as BranchOperation).relative(self),
            0xd8 => (Self::cld as ReadOperation).implied(self),
            0xe8 => (Self::inx as ReadOperation).implied(self),
            //01/05/09/0d/11/15/19/1d
            0x01 => (Self::ora as ReadOperation).indexed_indirect_x(self),
            0x0d => (Self::ora as ReadOperation).absolute(self),
            0x65 => (Self::adc as ReadOperation).zero_page(self),
            0x85 => (Self::sta as WriteOperation).zero_page(self),
            0x8d => (Self::sta as WriteOperation).absolute(self),
            0x91 => (Self::sta as WriteOperation).indirect_indexed_y(self),
            0x95 => (Self::sta as WriteOperation).zero_page_indexed_x(self),
            0x9d => (Self::sta as WriteOperation).absolute_indexed_x(self),
            0xa9 => (Self::lda as ReadOperation).immediate(self),
            //02/06/0a/0e/12/16/1a/1e
            0x0a => (Self::asl as ReadWriteOperation).accumulator(self),
            0x0e => (Self::asl as ReadWriteOperation).absolute(self),
            0x4e => (Self::lsr as ReadWriteOperation).absolute(self),
            0x8a => (Self::txa as ReadOperation).immediate(self),
            0x86 => (Self::stx as WriteOperation).zero_page(self),
            0x8e => (Self::stx as WriteOperation).absolute(self),
            0x9a => (Self::txs as ReadOperation).implied(self),
            0xa2 => (Self::ldx as ReadOperation).immediate(self),
            0xaa => (Self::tax as ReadOperation).implied(self),
            0xba => (Self::tsx as ReadOperation).implied(self),
            0xca => (Self::dex as ReadOperation).implied(self),
            0xe6 => (Self::inc as ReadWriteOperation).zero_page(self),
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