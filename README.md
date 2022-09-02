# Just a NES emulator in Rust
Goal is to be cycle-accurate, and as a result really slow.

## Architecture

### Mapper

A router that captures the functionality of the NES cartridge mapper chip and the memory map of an NES. It made sense to just smash the two concepts together so there is 
a single module responsible for all address-to-device routing.
