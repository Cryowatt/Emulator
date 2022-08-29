use nes::roms::Mappers;
use std::{fs::File, io::Read, io::BufReader};

use nes::{cpu::{RP2A03, Mos6502}, roms::RomImage, system::{ConsoleSystem}};

#[test]
fn basics() {
    
    //let cpu = crate::nes::cpu::RP2A03::new();
    // let cpu = <Mos6502 as RP2A03>::new();
    // assert_eq!(cpu.p, 0x34);
    // assert_eq!(cpu.a, 0x00);
    // assert_eq!(cpu.x, 0x00);
    // assert_eq!(cpu.y, 0x00);
    // assert_eq!(cpu.s, 0xFD);
    
    let mut rom_file = File::open("./nes-test-roms/instr_test-v5/rom_singles/01-basics.nes").expect("Test rom open error");
    let image = RomImage::from(&mut rom_file).expect("Test rom load error");
    let mut system = ConsoleSystem::new(image);
    system.reset();

    // Test rom init cycles
    while system.mapper.read(0x6000) != 0x80 {
        system.cycle();
    }

    // Run tests
    while system.mapper.read(0x6000) == 0x80 {
        system.cycle();
    }

    assert_eq!(system.mapper.read(0x6000), 0x00);
    assert_eq!(system.mapper.read(0x6001), 0xde);
    assert_eq!(system.mapper.read(0x6002), 0xb0);
    assert_eq!(system.mapper.read(0x6003), 0x61);
    
    // Assert.AreEqual(0xDE, platform.Read(new Address(0x6001)));
    // Assert.AreEqual(0xB0, platform.Read(new Address(0x6002)));
    // Assert.AreEqual(0x61, platform.Read(new Address(0x6003)));
    // system.Step();
    //let mut reader = BufReader::new(romFile);    
    
    //let mut image = Vec::<u8>::new();
    // romFile.read_to_end(&mut image).expect("Test rom read error");
    
    // rom.stream_len();
    // rom.read_to_end(buf: &mut Vec<u8>)


    // var romPath = Path.Combine(FindTestRomDirectory(), "instr_test-v3/rom_singles/", path);
    // using var reader = new BinaryReader(File.OpenRead(romPath));
    // var mapper = (Mapper0)Mapper.FromImage(RomImage.From(reader));
    // var platform = new NesBus(mapper);
    // platform.Reset();
    // int result;
    // // Test warmup, waiting for $6000 to go non-zero (usually 0x80 to indicate running state
    // while (platform.Read(new Address(0x6000)) == 0)
    // {
    //     platform.DoCycle();
    // }

    // while ((result = platform.Read(new Address(0x6000))) == 0x80)
    // {
    //     platform.DoCycle();
    // }

    // // Checking for DEBO61
    // Assert.AreEqual(0xDE, platform.Read(new Address(0x6001)));
    // Assert.AreEqual(0xB0, platform.Read(new Address(0x6002)));
    // Assert.AreEqual(0x61, platform.Read(new Address(0x6003)));
    
    // using (var debugPin = mapper.Ram.Slice(4).Pin())
    // {
    //     var errorMessage = Marshal.PtrToStringAnsi((IntPtr)debugPin.Pointer);
    //     Assert.AreEqual(0, result, errorMessage);
    // }
}

