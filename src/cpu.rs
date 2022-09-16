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
        self.pointer = self.pointer.wrapping_add(1);
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
            0x04 => (Self::nop as ReadOperation).zero_page(self),
            0x08 => self.php(),
            0x0c => (Self::nop as ReadOperation).absolute(self),
            0x10 => (Self::bpl as BranchOperation).relative(self),
            0x14 => (Self::nop as ReadOperation).absolute_indexed_x(self),
            0x18 => (Self::clc as ReadOperation).implied(self),
            0x1c => (Self::nop as ReadOperation).absolute_indexed_x(self),
            0x20 => self.jsr(),
            0x24 => (Self::bit as ReadOperation).zero_page(self),
            0x28 => self.plp(),
            0x2c => (Self::bit as ReadOperation).absolute(self),
            0x30 => (Self::bmi as BranchOperation).relative(self),
            0x38 => (Self::sec as ReadOperation).implied(self),
            0x3c => (Self::nop as ReadOperation).absolute_indexed_x(self),
            0x40 => self.rti(),
            0x44 => (Self::nop as ReadOperation).absolute_indexed_x(self),
            0x48 => self.pha(),
            0x4c => self.jmp(),
            0x50 => (Self::bvc as BranchOperation).relative(self),
            0x58 => (Self::cli as ReadOperation).implied(self),
            0x5c => (Self::nop as ReadOperation).absolute_indexed_x(self),
            0x60 => self.rts(),
            0x68 => self.pla(),
            0x6c => self.jmp_indrect(),
            0x70 => (Self::bvs as BranchOperation).relative(self),
            0x78 => (Self::sei as ReadOperation).implied(self),
            0x7c => (Self::nop as ReadOperation).absolute_indexed_x(self),
            0x80 => (Self::nop as ReadOperation).immediate(self),
            0x84 => (Self::sty as WriteOperation).zero_page(self),
            0x88 => (Self::dey as ReadOperation).implied(self),
            0x8c => (Self::sty as WriteOperation).absolute(self),
            0x90 => (Self::bcc as BranchOperation).relative(self),
            0x94 => (Self::sty as WriteOperation).zero_page_indexed_x(self),
            0x98 => (Self::tya as ReadOperation).implied(self),
            0xa0 => (Self::ldy as ReadOperation).immediate(self),
            0xa4 => (Self::ldy as ReadOperation).zero_page(self),
            0xa8 => (Self::tay as ReadOperation).implied(self),
            0xac => (Self::ldy as ReadOperation).absolute(self),
            0xb0 => (Self::bcs as BranchOperation).relative(self),
            0xb4 => (Self::ldy as ReadOperation).zero_page_indexed_x(self),
            0xb8 => (Self::clv as ReadOperation).implied(self),
            0xbc => (Self::ldy as ReadOperation).absolute_indexed_x(self),
            0xc0 => (Self::cpy as ReadOperation).immediate(self),
            0xc4 => (Self::cpy as ReadOperation).zero_page(self),
            0xc8 => (Self::iny as ReadOperation).implied(self),
            0xcc => (Self::cpy as ReadOperation).absolute(self),
            0xd0 => (Self::bne as BranchOperation).relative(self),
            0xd8 => (Self::cld as ReadOperation).implied(self),
            0xdc => (Self::nop as ReadOperation).absolute_indexed_x(self),
            0xe0 => (Self::cpx as ReadOperation).immediate(self),
            0xe4 => (Self::cpx as ReadOperation).zero_page(self),
            0xe8 => (Self::inx as ReadOperation).implied(self),
            0xec => (Self::cpx as ReadOperation).absolute(self),
            0xf0 => (Self::beq as BranchOperation).relative(self),
            0xf8 => (Self::sed as ReadOperation).implied(self),
            0xfc => (Self::nop as ReadOperation).absolute_indexed_x(self),
            //01/05/09/0d/11/15/19/1d
            0x01 => (Self::ora as ReadOperation).indexed_indirect_x(self),
            0x05 => (Self::ora as ReadOperation).zero_page(self),
            0x09 => (Self::ora as ReadOperation).immediate(self),
            0x0d => (Self::ora as ReadOperation).absolute(self),
            0x11 => (Self::ora as ReadOperation).indirect_indexed_y(self),
            0x15 => (Self::ora as ReadOperation).zero_page_indexed_x(self),
            0x19 => (Self::ora as ReadOperation).absolute_indexed_y(self),
            0x1d => (Self::ora as ReadOperation).absolute_indexed_x(self),
            0x21 => (Self::and as ReadOperation).indexed_indirect_x(self),
            0x25 => (Self::and as ReadOperation).zero_page(self),
            0x29 => (Self::and as ReadOperation).immediate(self),
            0x2d => (Self::and as ReadOperation).absolute(self),
            0x31 => (Self::and as ReadOperation).indirect_indexed_y(self),
            0x35 => (Self::and as ReadOperation).zero_page_indexed_x(self),
            0x39 => (Self::and as ReadOperation).absolute_indexed_y(self),
            0x3d => (Self::and as ReadOperation).absolute_indexed_x(self),
            0x41 => (Self::eor as ReadOperation).indexed_indirect_x(self),
            0x45 => (Self::eor as ReadOperation).zero_page(self),
            0x49 => (Self::eor as ReadOperation).immediate(self),
            0x4d => (Self::eor as ReadOperation).absolute(self),
            0x51 => (Self::eor as ReadOperation).indirect_indexed_y(self),
            0x55 => (Self::eor as ReadOperation).zero_page_indexed_x(self),
            0x59 => (Self::eor as ReadOperation).absolute_indexed_y(self),
            0x5d => (Self::eor as ReadOperation).absolute_indexed_x(self),
            0x61 => (Self::adc as ReadOperation).indexed_indirect_x(self),
            0x65 => (Self::adc as ReadOperation).zero_page(self),
            0x69 => (Self::adc as ReadOperation).immediate(self),
            0x6d => (Self::adc as ReadOperation).absolute(self),
            0x71 => (Self::adc as ReadOperation).indirect_indexed_y(self),
            0x75 => (Self::adc as ReadOperation).zero_page_indexed_x(self),
            0x79 => (Self::adc as ReadOperation).absolute_indexed_y(self),
            0x7d => (Self::adc as ReadOperation).absolute_indexed_x(self),
            0x81 => (Self::sta as WriteOperation).indexed_indirect_x(self),
            0x85 => (Self::sta as WriteOperation).zero_page(self),
            0x8d => (Self::sta as WriteOperation).absolute(self),
            0x91 => (Self::sta as WriteOperation).indirect_indexed_y(self),
            0x95 => (Self::sta as WriteOperation).zero_page_indexed_x(self),
            0x99 => (Self::sta as WriteOperation).absolute_indexed_y(self),
            0x9d => (Self::sta as WriteOperation).absolute_indexed_x(self),
            0xa5 => (Self::lda as ReadOperation).zero_page(self),
            0xa1 => (Self::lda as ReadOperation).indexed_indirect_x(self),
            0xad => (Self::lda as ReadOperation).absolute(self),
            0xa9 => (Self::lda as ReadOperation).immediate(self),
            0xb1 => (Self::lda as ReadOperation).indirect_indexed_y(self),
            0xb5 => (Self::lda as ReadOperation).zero_page_indexed_x(self),
            0xb9 => (Self::lda as ReadOperation).absolute_indexed_y(self),
            0xbd => (Self::lda as ReadOperation).absolute_indexed_x(self),
            0xc1 => (Self::cmp as ReadOperation).indexed_indirect_x(self),
            0xc5 => (Self::cmp as ReadOperation).zero_page(self),
            0xc9 => (Self::cmp as ReadOperation).immediate(self),
            0xcd => (Self::cmp as ReadOperation).absolute(self),
            0xd1 => (Self::cmp as ReadOperation).indirect_indexed_y(self),
            0xd5 => (Self::cmp as ReadOperation).zero_page_indexed_x(self),
            0xd9 => (Self::cmp as ReadOperation).absolute_indexed_y(self),
            0xdd => (Self::cmp as ReadOperation).absolute_indexed_x(self),
            0xe1 => (Self::sbc as ReadOperation).indexed_indirect_x(self),
            0xe5 => (Self::sbc as ReadOperation).zero_page(self),
            0xe9 => (Self::sbc as ReadOperation).immediate(self),
            0xed => (Self::sbc as ReadOperation).absolute(self),
            0xf1 => (Self::sbc as ReadOperation).indirect_indexed_y(self),
            0xf5 => (Self::sbc as ReadOperation).zero_page_indexed_x(self),
            0xf9 => (Self::sbc as ReadOperation).absolute_indexed_y(self),
            0xfd => (Self::sbc as ReadOperation).absolute_indexed_x(self),
            //02/06/0a/0e/12/16/1a/1e
            0x06 => (Self::asl as ReadWriteOperation).zero_page(self),
            0x0a => (Self::asl as ReadWriteOperation).accumulator(self),
            0x0e => (Self::asl as ReadWriteOperation).absolute(self),
            0x16 => (Self::asl as ReadWriteOperation).zero_page_indexed_x(self),
            0x1a => (Self::nop as ReadOperation).implied(self),
            0x1e => (Self::asl as ReadWriteOperation).absolute_indexed_x(self),
            0x26 => (Self::rol as ReadWriteOperation).zero_page(self),
            0x2a => (Self::rol as ReadWriteOperation).accumulator(self),
            0x2e => (Self::rol as ReadWriteOperation).absolute(self),
            0x36 => (Self::rol as ReadWriteOperation).zero_page_indexed_x(self),
            0x3a => (Self::nop as ReadOperation).implied(self),
            0x3e => (Self::rol as ReadWriteOperation).absolute_indexed_x(self),
            0x46 => (Self::rol as ReadWriteOperation).zero_page(self),
            0x4a => (Self::lsr as ReadWriteOperation).accumulator(self),
            0x4e => (Self::lsr as ReadWriteOperation).absolute(self),
            0x56 => (Self::lsr as ReadWriteOperation).zero_page_indexed_x(self),
            0x5a => (Self::nop as ReadOperation).implied(self),
            0x5e => (Self::lsr as ReadWriteOperation).absolute_indexed_x(self),
            0x66 => (Self::ror as ReadWriteOperation).zero_page(self),
            0x6a => (Self::ror as ReadWriteOperation).accumulator(self),
            0x6e => (Self::ror as ReadWriteOperation).absolute(self),
            0x76 => (Self::ror as ReadWriteOperation).zero_page_indexed_x(self),
            0x7a => (Self::nop as ReadOperation).implied(self),
            0x7e => (Self::ror as ReadWriteOperation).absolute_indexed_x(self),
            0x8a => (Self::txa as ReadOperation).immediate(self),
            0x86 => (Self::stx as WriteOperation).zero_page(self),
            0x8e => (Self::stx as WriteOperation).absolute(self),
            0x96 => (Self::stx as WriteOperation).zero_page_indexed_y(self),
            0x9a => (Self::txs as ReadOperation).implied(self),
            0xa2 => (Self::ldx as ReadOperation).immediate(self),
            0xa6 => (Self::ldx as ReadOperation).zero_page(self),
            0xaa => (Self::tax as ReadOperation).implied(self),
            0xae => (Self::ldx as ReadOperation).absolute(self),
            0xb6 => (Self::ldx as ReadOperation).zero_page_indexed_x(self),
            0xba => (Self::tsx as ReadOperation).implied(self),
            0xbe => (Self::ldx as ReadOperation).absolute_indexed_y(self),
            0xc6 => (Self::dec as ReadWriteOperation).zero_page(self),
            0xca => (Self::dex as ReadOperation).implied(self),
            0xce => (Self::dec as ReadWriteOperation).absolute(self),
            0xd6 => (Self::dec as ReadWriteOperation).zero_page_indexed_x(self),
            0xda => (Self::nop as ReadOperation).implied(self),
            0xde => (Self::dec as ReadWriteOperation).absolute_indexed_x(self),
            0xe6 => (Self::inc as ReadWriteOperation).zero_page(self),
            0xea => (Self::nop as ReadOperation).implied(self),
            0xee => (Self::inc as ReadWriteOperation).absolute(self),
            0xf6 => (Self::inc as ReadWriteOperation).zero_page_indexed_x(self),
            0xfa => (Self::nop as ReadOperation).implied(self),
            0xfe => (Self::inc as ReadWriteOperation).absolute_indexed_x(self),
            //03/07/0b/0f/13/17/1b/1f
            0x03 => (Self::slo as ReadWriteOperation).indexed_indirect_x(self),
            0x07 => (Self::slo as ReadWriteOperation).zero_page(self),
            0x0f => (Self::slo as ReadWriteOperation).absolute(self),
            0x13 => (Self::slo as ReadWriteOperation).indirect_indexed_y(self),
            0x17 => (Self::slo as ReadWriteOperation).zero_page_indexed_x(self),
            0x1b => (Self::slo as ReadWriteOperation).absolute_indexed_y(self),
            0x1f => (Self::slo as ReadWriteOperation).absolute_indexed_x(self),
            0x23 => (Self::rla as ReadWriteOperation).indexed_indirect_x(self),
            0x27 => (Self::rla as ReadWriteOperation).zero_page(self),
            0x2f => (Self::rla as ReadWriteOperation).absolute(self),
            0x33 => (Self::rla as ReadWriteOperation).indirect_indexed_y(self),
            0x37 => (Self::rla as ReadWriteOperation).zero_page_indexed_x(self),
            0x3b => (Self::rla as ReadWriteOperation).absolute_indexed_y(self),
            0x3f => (Self::rla as ReadWriteOperation).absolute_indexed_x(self),
            0x83 => (Self::sax as WriteOperation).indexed_indirect_x(self),
            0x87 => (Self::sax as WriteOperation).zero_page(self),
            0x8f => (Self::sax as WriteOperation).absolute(self),
            0x97 => (Self::sax as WriteOperation).zero_page_indexed_y(self),
            0xa3 => (Self::lax as ReadOperation).indexed_indirect_x(self),
            0xa7 => (Self::lax as ReadOperation).zero_page(self),
            0xaf => (Self::lax as ReadOperation).absolute(self),
            0xb3 => (Self::lax as ReadOperation).indirect_indexed_y(self),
            0xb7 => (Self::lax as ReadOperation).zero_page_indexed_y(self),
            0xbf => (Self::lax as ReadOperation).absolute_indexed_y(self),
            0xc3 => (Self::dcp as ReadWriteOperation).indexed_indirect_x(self),
            0xc7 => (Self::dcp as ReadWriteOperation).zero_page(self),
            0xcf => (Self::dcp as ReadWriteOperation).absolute(self),
            0xd3 => (Self::dcp as ReadWriteOperation).indexed_indirect_x(self),
            0xd7 => (Self::dcp as ReadWriteOperation).zero_page_indexed_x(self),
            0xdb => (Self::dcp as ReadWriteOperation).absolute_indexed_y(self),
            0xdf => (Self::dcp as ReadWriteOperation).absolute_indexed_x(self),
            0xe3 => (Self::isc as ReadWriteOperation).indexed_indirect_x(self),
            0xe7 => (Self::isc as ReadWriteOperation).zero_page(self),
            0xef => (Self::isc as ReadWriteOperation).absolute(self),
            0xeb => (Self::sbc as ReadOperation).immediate(self),
            0xf3 => (Self::isc as ReadWriteOperation).indirect_indexed_y(self),
            0xf7 => (Self::isc as ReadWriteOperation).zero_page_indexed_x(self),
            0xfb => (Self::isc as ReadWriteOperation).absolute_indexed_y(self),
            0xff => (Self::isc as ReadWriteOperation).absolute_indexed_x(self),

            _ => panic!("Unsupported opcode {:02x}", opcode),
        }
        
        //todo!();
    }

    fn reset(self: &mut Self) {
        self.queue_read(Self::read_fixed::<0xfffc>, Self::set_pc_low);
        self.queue_read(Self::read_fixed::<0xfffd>, Self::set_pc_high);
    }
}