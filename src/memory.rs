use crate::bus::BusDevice;

trait MemoryDevice {
    fn normalize_address(&self, address: u16) -> u16;
}

pub struct RAM<const SIZE: usize> {
    pub bank: Box<[u8; SIZE]>,
    offset: u16,
    mask: u16,
}

impl<const SIZE: usize> RAM<SIZE> {

    pub fn new(offset: u16, mask: u16) -> Self {
        RAM {
            bank: Box::new([0; SIZE]),
            offset,
            mask,
        }
    }
}

impl <const SIZE: usize> MemoryDevice for RAM<SIZE> {
    fn normalize_address(&self, address: u16) -> u16 {
        (address - self.offset) & self.mask
    }
}

impl<const SIZE: usize> BusDevice for RAM<SIZE> {
    fn read(&self, address: u16) -> u8 {
        (*self.bank)[self.normalize_address(address) as usize]
    }

    fn write(self: &mut Self, address: u16, data: u8) {
        (*self.bank)[self.normalize_address(address) as usize] = data;
    }
}

pub struct ROM<const SIZE: usize> {
    pub bank: Vec<u8>,
    offset: u16,
    mask: u16,
}

impl<const SIZE: usize> ROM<SIZE> {
    pub fn new(data: &[u8], offset: u16, mask: u16) -> Self {
        ROM { 
            bank: data.to_owned(),
            offset,
            mask,
        }
    }
}

impl <const SIZE: usize> MemoryDevice for ROM<SIZE> {
    fn normalize_address(&self, address: u16) -> u16 {
        (address - self.offset) & self.mask
    }
}

impl<const SIZE: usize> BusDevice for ROM<SIZE> {
    fn read(&self, address: u16) -> u8 {
        (*self.bank)[self.normalize_address(address) as usize]
    }

    fn write(self: &mut Self, address: u16, data: u8) {
        // Can't write to ROM, so just ignore
     }
}
