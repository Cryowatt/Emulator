use std::{collections::VecDeque};

use crate::roms::Mapper;

type MicrocodeTask = fn(&mut Mos6502);

pub struct Mos6502 {
    pub a: u8,
    pub pc: u16,
    pub p: u8,
    pub s: u8,
    pub x: u8,
    pub y: u8,

    cycle_microcode_queue: VecDeque<MicrocodeTask>,
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
}

pub trait RP2A03 {
    fn zero_page_indexed();

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
        
        microcode(self);
    }

    fn decode_opcode(self: &mut Self, mapper: &mut dyn Mapper) -> MicrocodeTask {
        let opcode = mapper.read(self.pc);
        self.pc += 1;

        match opcode {
            _ => panic!("Unsupported opcode {}", opcode),
        }
        
        //todo!();
    }
}
