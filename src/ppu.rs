use crate::bus::BusDevice;

pub struct PPU {}

impl BusDevice for PPU {
    fn read(&self, _: u16) -> u8 { todo!() }
    fn write(&mut self, _: u16, _: u8) { /*todo!()*/ }
}
