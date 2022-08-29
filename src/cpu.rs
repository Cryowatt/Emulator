use std::{collections::VecDeque};

use crate::roms::Mapper;

type MicrocodeTask = fn(&mut Mos6502, &mut dyn Mapper);

const STACK_OFFSET: u16 = 0x0100;

pub struct Mos6502 {
    pub a: u8,
    pub pc: u16,
    pub p: u8,
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
            p: 0x34,
            s: 0xFD,
            x: 0x00,
            y: 0x00,

            cycle_microcode_queue: VecDeque::with_capacity(8),
        }
    }

    fn push_stack(self: &mut Self, mapper: &mut dyn Mapper, data: u8) {
        mapper.write((self.s & 0xff) as u16 + STACK_OFFSET, data);
        self.s -= 1;
    }

    fn push_from_pc_high(self: &mut Self, mapper: &mut dyn Mapper) {
        let data = self.pc.get_high();
        self.push_stack(mapper, data);
    }

    fn push_from_pc_low(self: &mut Self, mapper: &mut dyn Mapper) {
        let data = self.pc.get_low();
        self.push_stack(mapper, data);
    }

    fn brk(self: &mut Self) {
        self.cycle_microcode_queue.push_back(|cpu, mapper| { 
            cpu.pc += 1;
            mapper.read(cpu.pc);
            //c.regs.InterruptDisable = true;
        });
        self.cycle_microcode_queue.push_back(|cpu, mapper| cpu.push_from_pc_high(mapper));
        // Enqueue(PushStackFromPCH);
        self.cycle_microcode_queue.push_back(|cpu, mapper| cpu.push_from_pc_low(mapper));
        // Enqueue(PushStackFromPCL);
        self.cycle_microcode_queue.push_back(|cpu, mapper| cpu.push_stack(mapper, cpu.p));
        // Enqueue(PushStackFromP);
        self.cycle_microcode_queue.push_back(|cpu, mapper| cpu.pc.set_low(mapper.read(0xfffe)));
        // Enqueue(c => c.regs.PC.Low = c.Read(new Address(0xfffe)));
        self.cycle_microcode_queue.push_back(|cpu, mapper| cpu.pc.set_high(mapper.read(0xffff)));
        // Enqueue(c =>
        // {
        //     c.regs.PC.High = c.Read(new Address(0xffff));
        //     c.TraceInstruction("BRK");
        // });
    }
}

pub trait RP2A03 {
    fn zero_page_indexed();

    fn reset(self: &mut Self, mapper: &mut dyn Mapper);

    fn cycle(self: &mut Self, mapper: &mut dyn Mapper);

    fn decode_opcode(self: &mut Self, mapper: &mut dyn Mapper) -> MicrocodeTask;
}

impl RP2A03 for Mos6502 {
    fn zero_page_indexed() {}

    fn cycle(self: &mut Self, mapper: &mut dyn Mapper) {
        let microcode = match self.cycle_microcode_queue.pop_front() {
            Some(microcode) => microcode,
            None => self.decode_opcode(mapper)
        };
        
        microcode(self, mapper);
    }

    fn decode_opcode(self: &mut Self, mapper: &mut dyn Mapper) -> MicrocodeTask {
        let opcode = mapper.read(self.pc);
        self.pc += 1;

        match opcode {
            0 => |cpu, mapper| cpu.brk(),
            _ => panic!("Unsupported opcode {:x}", opcode),
        }
        
        //todo!();
    }

    fn reset(self: &mut Self, mapper: &mut dyn Mapper) {
        //todo!()
    }
}
