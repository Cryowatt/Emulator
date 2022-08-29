use crate::roms::Mapper;
use crate::roms::Mappers;
use crate::roms::RomImage;
use crate::bus::MemoryMapper;
use std::rc::Rc;

use crate::apu::Alu2A03;
use crate::bus::Bus;
use crate::bus::BusDevice;
use crate::ppu::PPU;
use crate::ram::RAM;

pub struct ConsoleSystem {
    pub mapper: Box<dyn Mapper>,
}

pub struct ConsoleDevices {
    pub ram: RAM<2048>,
    pub ppu: PPU,
    pub alu: Alu2A03,
}

impl ConsoleSystem {
    pub fn new(image: RomImage) -> Self {
        let devices = ConsoleDevices {
            ram: RAM::<0x800>::new(0x2000),
            ppu: PPU{},
            alu: Alu2A03 {},
        };
        
        let mapper = Mappers::from(image, devices).expect("failed to create mapper");

        // let memoryMap: MemoryMapper = |a: u16, devices: &mut ConsoleDevices| match a >> 13 {
        //     _ => &mut devices.ram, // $0000 - $1FFF RAM
        //                            // 1 => foo.as_mut(), //&PPU{}, // $2000 - $3FFF PPU Reg
        //                            // devices.ram.as_mut(), //&Alu2A03{}, //::new(cart), // $4000 - $5FFF APU/Cart
        //                            // devices.ram.as_mut(), //cart, // $6000 - $7FFF Cart SRAM/RAM
        //                            // devices.ram.as_mut(), //cart, // $8000 - $9FFF Registers
        //                            // devices.ram.as_mut(), //cart, // $A000 - $BFFF CHR
        //                            // devices.ram.as_mut(), //cart, // $C000 - $DFFF CHR
        //                            // devices.ram.as_mut(), //cart, // $E000 - $FFFF PRG
        // };

        //let bus = Bus { mapper };
        // let mut ram = Box::new(RAM::<0x800>::new());
        // let fk: &mut dyn BusDevice = ram.as_mut();

        ConsoleSystem { mapper }
    }

    pub fn reset(self: &mut Self) {

    }

    pub fn cycle(self: &mut Self) {
        
    }
}
