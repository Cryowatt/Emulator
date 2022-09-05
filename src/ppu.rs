use crate::bus::BusDevice;

pub struct PPU {}

impl BusDevice for PPU {
    fn read(&self, address: u16) -> u8 {
        //println!("PPU READ!! ${:04X}", address);
        0
    }
    fn write(&mut self, address: u16, _: u8) {
        //println!("PPU WRITE!! ${:04X}", address);
    }
}
