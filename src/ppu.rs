use std::borrow::Borrow;

use crate::bus::BusDevice;
use bitflags::bitflags;

bitflags! {
    pub struct Status: u8 {
        const SPRITE_OVERFLOW = 0b0010_0000;
        const SPRITE0_HIT = 0b0100_0000;
        const VBLANK = 0b1000_0000;
    }
}
pub struct PPU {
    data: u8,
    scanline_counter_or_something: u16,
    pub status: Status,
}

impl PPU {
    pub fn new() -> Self {
        Self { 
            data: 0,
            scanline_counter_or_something: 0,
            status: Status::VBLANK | Status::SPRITE_OVERFLOW
        }
    }

    pub fn reset(self: &mut Self) {
    }

    pub fn cycle(&mut self) {
        self.scanline_counter_or_something = self.scanline_counter_or_something.wrapping_add(1);

        if self.scanline_counter_or_something % 256 == 0 {
            self.status.toggle(Status::VBLANK);
        }
    }

    fn read_status(&self) -> u8 {
        self.status.bits
    }
}

impl BusDevice for PPU {
    fn read(&self, address: u16) -> u8 {
        match address {
            0x2002 => self.read_status(),
            _ => 0,
        }
        //println!("PPU READ!! ${:04X}", address);
        
    }
    fn write(&mut self, address: u16, data: u8) {
        self.data = data;
        //println!("PPU WRITE!! ${:04X}", address);
    }
}
