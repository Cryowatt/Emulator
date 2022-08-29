//#![feature(const_ops)]
use crate::bus::BusDevice;
use crate::ram::RAM;
use crate::system::ConsoleDevices;
use std::error::Error;
use bitflags::bitflags;
use byteorder::ReadBytesExt;
use num_enum::TryFromPrimitive;
use std::convert::TryFrom;
use std::io::Read;
use std::io::{self, Seek, SeekFrom};

bitflags! {
    pub struct RomFlags: u8 {
        const VERTICAL = 0x1;
        const BATTERY = 0x2;
        const TRAINER = 0x4;
        const FOUR_SCREEN = 0x8;
    }
}

#[derive(Debug, TryFromPrimitive)]
#[repr(u8)]
pub enum TVSystem {
    NTSC = 0x0,
    PAL = 0x1,
}

#[derive(Debug, TryFromPrimitive)]
#[repr(u8)]
pub enum ConsoleType {
    NES = 0x0,
    VsSystem = 0x1,
    Playchoice = 0x2,
    Extended = 0x3,
}

pub struct RomImageHeader {
    pub program_rom_size: u8,
    pub character_rom_size: u8,
    pub rom_flags: RomFlags,
    pub mapper: u16,
    pub console_type: ConsoleType,
    pub program_ram_size: u8,
    pub tv_system: TVSystem,
}

pub struct RomImage {
    pub header: RomImageHeader,
    pub trainer_data: Vec<u8>,
    pub program_rom_data: Vec<u8>,
    pub character_rom_data: Vec<u8>,
}

impl RomImage {
    pub fn from<R: ReadBytesExt + Seek>(reader: &mut R) -> Result<RomImage, io::Error> {
        // TODO: async IO?
        let header = Self::parse_header(reader)?;
        Ok(RomImage {
            trainer_data: match header.rom_flags {
                RomFlags::TRAINER => Self::read_data(reader, 0x200 as usize)?,
                _ => vec![],
            },
            program_rom_data: Self::read_data(reader, 0x4000 * header.program_rom_size as usize)?,
            character_rom_data: Self::read_data(reader, 0x2000 * header.character_rom_size as usize)?,
            header,
        })
    }

    fn read_data<R: ReadBytesExt>(reader: &mut R, size: usize) -> Result<Vec<u8>, io::Error> {
        let mut buffer = Vec::<u8>::with_capacity(size as usize);
        reader.take(size as u64).read_to_end(&mut buffer)?;
        Ok(buffer)
    }

    fn parse_header<R: ReadBytesExt + Seek>(reader: &mut R) -> Result<RomImageHeader, io::Error> {
        let mut ident: [u8; 4] = [0; 4];
        reader.read_exact(&mut ident)?;

        if b"NES\x1a".ne(&ident) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Invalid NES image header",
            ));
        }

        let program_rom_size = reader.read_u8()?;
        let character_rom_size = reader.read_u8()?;
        let rom_flags = reader.read_u8()?;
        let console_type_flags = reader.read_u8()?;
        let console_type = ConsoleType::try_from(console_type_flags & 0x3)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, "Unknown console type"))?;

        if (console_type_flags & 0xc) == 0x8 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "NES 2.0 format not supported",
            ));
        }

        let program_ram_size = reader.read_u8()?;
        let tv_system = TVSystem::try_from(reader.read_u8()?)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, "Unknown TV system type"))?;
        if reader.read_u8()? != 0 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "I don't know why but I don't support non-zero values here",
            ));
        }

        // Unused padding
        reader.seek(SeekFrom::Current(5))?;

        Ok(RomImageHeader {
            program_rom_size: program_rom_size,
            character_rom_size,
            rom_flags: RomFlags::from_bits_truncate(rom_flags),
            mapper: ((rom_flags >> 4u8) | console_type_flags & 0xf0).into(),
            console_type: console_type,
            program_ram_size: program_ram_size,
            tv_system: tv_system,
        })
    }
}

#[derive(Debug)]
pub struct RomError {
    message: &'static str
}

impl RomError {
    fn new(message: &'static str) -> Self{
        Self{ message: message }
    }
}

// impl Error for RomError {}

// impl From<std::io::Error> for RomError {
//     fn from(error: std::io::Error) -> Self {
        
//     }
// }

pub struct Mappers;

impl Mappers {
    pub fn from(image: RomImage, devices: ConsoleDevices) -> Result<Box<dyn Mapper>, RomError> {
        match image.header.mapper {
            0 => Ok(Box::new(NROM::new(image, devices))),
            _ => Err(RomError::new("Unsupported mapper")),
        }
    }
}

pub trait Mapper {
    fn read(self: &Self, address: u16) -> u8;
    fn write(self: &mut Self, address: u16, data: u8) -> ();
}

pub struct NROM {
    //image: RomImage,
    devices: ConsoleDevices,
    program_ram: RAM::<0x2000>,
    program_rom_bank0: ROM::<0x4000>,
    program_rom_bank1: ROM::<0x4000>,
}

impl NROM {
    fn new(image: RomImage, devices: ConsoleDevices) -> Self {
        NROM {
            //image,
            devices,
            program_ram: RAM::<0x2000>::new(image.header.program_ram_size as u16 * 8u16 * 1024u16),
            program_rom_bank0: ROM::<0x4000>::new(&image.program_rom_data[0..0x4000], 0x4000),
            program_rom_bank1: match image.header.program_rom_size {
                1 => ROM::<0x4000>::new(&image.program_rom_data[0..0x4000], 0x4000),
                2 => ROM::<0x4000>::new(&image.program_rom_data[0x4000..0x8000], 0x4000),
                _ => panic!("More rom than address space, really weird."),
            },
        }
    }
}

impl Mapper for NROM {

    fn read(self: &Self, address: u16) -> u8 {
        match address >> 13 {
            0 => self.devices.ram.read(address),
            1 => self.devices.ppu.read(address),
            2 => self.devices.alu.read(address),
            3 => self.program_ram.read(address),
            // look at https://github.com/Cryowatt/NES/blob/master/NES.CPU/Mappers/Mapper0.cs#L21
            4 | 5 => self.program_rom_bank0.read(address),
            _ => self.program_rom_bank1.read(address),
        }
     }

    fn write(self: &mut Self, address: u16, data: u8) {
        match address >> 13 {
            0 => self.devices.ram.write(address, data),
            1 => self.devices.ppu.write(address, data),
            2 => self.devices.alu.write(address, data),
            3 => self.program_ram.write(address, data),
            // look at https://github.com/Cryowatt/NES/blob/master/NES.CPU/Mappers/Mapper0.cs#L21
            4 | 5 => self.program_rom_bank0.write(address, data),
            _ => self.program_rom_bank1.write(address, data),
        }
    }
}

pub struct ROM<const SIZE: usize> {
    pub bank: Box<[u8; SIZE]>,
    mask: u16,
}

impl<const SIZE: usize> ROM<SIZE> {
    // pub fn new() -> Self {
    //     let foo = vec![0, SIZE];
    //     RAM {
    //         bank: Box::new([0; SIZE]), 
    //         mask: SIZE as u16
    //     }
    // }

    pub fn new(data: &[u8], mask: u16) -> Self {
        //let foo = vec![0, SIZE];
        ROM { 
            bank:  Box::new([0; SIZE]),
            mask,
        }
    }
}

impl<const SIZE: usize> BusDevice for ROM<SIZE> {
    fn read(&self, address: u16) -> u8 {
        (*self.bank)[address as usize]
    }

    fn write(self: &mut Self, address: u16, data: u8) {
        // Can't write to ROM, so just ignore
     }
}
