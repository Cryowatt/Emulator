use std::cmp::Ordering;

use crate::address::Address;

use super::{Mos6502, instructions::{WriteOperation, BranchOperation, ReadWriteOperation, ReadOperation}, MOS6502Instructions};

pub type Microcode<TIo, TOp> = fn(&mut Mos6502, io: TIo, op: TOp);
pub trait BranchAddressingModes {
    fn relative(self, cpu: &mut Mos6502);
}

impl BranchAddressingModes for BranchOperation {
    fn relative(self, cpu: &mut Mos6502) {
        cpu.queue_branch_microcode(Mos6502::read_pc_increment, self, |cpu, io, op| {
            cpu.operand = io(cpu);
            let should_branch = op(cpu);
            if should_branch {
                cpu.queue_read(Mos6502::read_pc, |cpu, _| {
                    let (low, carry) = cpu.pc.get_low().overflowing_add_signed(cpu.operand as i8);
                    cpu.pc.set_low(low);

                    if carry {
                        cpu.queue_read(Mos6502::read_pc, |cpu, _| {
                            let high = match (cpu.operand as i8).cmp(&0) {
                                Ordering::Less => cpu.pc.get_high() - 1,
                                Ordering::Equal | Ordering::Greater => cpu.pc.get_high() + 1,
                            };
                            cpu.pc.set_high(high);
                        });
                    }
                });
            }
        });
    }
}

pub trait AddressingModes {
    fn absolute(self, cpu: &mut Mos6502);
    fn absolute_indexed_x(self, cpu: &mut Mos6502);
    fn absolute_indexed_y(self, cpu: &mut Mos6502);
    fn accumulator(self, cpu: &mut Mos6502);
    fn immediate(self, cpu: &mut Mos6502);
    fn implied(self, cpu: &mut Mos6502);
    fn indexed_indirect_x(self, cpu: &mut Mos6502);
    fn indirect_indexed_y(self, cpu: &mut Mos6502);
    fn zero_page(self, cpu: &mut Mos6502);
    fn zero_page_indexed_x(self, cpu: &mut Mos6502);
    fn zero_page_indexed_y(self, cpu: &mut Mos6502);
}

impl AddressingModes for ReadOperation {
    fn absolute(self, cpu: &mut Mos6502) {
        cpu.queue_read(Mos6502::read_pc_increment, Mos6502::set_address_low);
        cpu.queue_read(Mos6502::read_pc_increment, Mos6502::set_address_high);
        cpu.queue_read(Mos6502::read_address, self);
    }

    fn absolute_indexed_x(self, cpu: &mut Mos6502) {
        cpu.queue_read(Mos6502::read_pc_increment, Mos6502::set_address_low);
        cpu.queue_read_microcode(Mos6502::read_pc_increment, self, |cpu, io, op| {
            let data = io(cpu);
            cpu.set_address_high(data);
            let (low, carry) = cpu.address.get_low().overflowing_add(cpu.x);
            cpu.set_address_low(low);
            if carry {
                cpu.queue_read(Mos6502::read_address, |cpu, data| cpu.address += 0x100);
            }

            cpu.queue_read(Mos6502::read_address, op);
        });
    }

    fn absolute_indexed_y(self, cpu: &mut Mos6502) {
        cpu.queue_read(Mos6502::read_pc_increment, Mos6502::set_address_low);
        cpu.queue_read_microcode(Mos6502::read_pc_increment, self, |cpu, io, op| {
            let data = io(cpu);
            cpu.set_address_high(data);
            let (low, carry) = cpu.address.get_low().overflowing_add(cpu.y);
            cpu.set_address_low(low);
            if carry {
                cpu.queue_read(Mos6502::read_address, |cpu, data| cpu.address = cpu.address.wrapping_add(0x100));
            }

            cpu.queue_read(Mos6502::read_address, op);
        });
    }

    fn accumulator(self, cpu: &mut Mos6502) {
        todo!();
    }

    fn immediate(self, cpu: &mut Mos6502) {
        cpu.queue_read(Mos6502::read_pc_increment, self);
    }

    fn implied(self, cpu: &mut Mos6502) {
        cpu.queue_read(Mos6502::read_pc, self);
    }

    fn indexed_indirect_x(self, cpu: &mut Mos6502) {
        cpu.queue_read(Mos6502::read_pc_increment, |cpu, data| cpu.pointer = data);
        cpu.queue_read(Mos6502::read_pointer, |cpu, data| cpu.set_zero_page_address(data + cpu.x));
        cpu.queue_read(Mos6502::read_pointer_increment, Mos6502::set_address_low);
        cpu.queue_read(Mos6502::read_pointer_increment, Mos6502::set_address_high);
        cpu.queue_read(Mos6502::read_address, self);
    }

    fn indirect_indexed_y(self, cpu: &mut Mos6502) {
        cpu.queue_read(Mos6502::read_pc_increment, |cpu, data| cpu.pointer = data);
        cpu.queue_read(Mos6502::read_pointer_increment, Mos6502::set_address_low);
        cpu.queue_read_microcode(Mos6502::read_pointer_increment, Mos6502::set_address_high, |cpu, io, op| {
            let (low, carry) = cpu.address.get_low().overflowing_add(cpu.y);
            cpu.set_address_low(low);                

            if carry {
                cpu.queue_read(Mos6502::read_address, |cpu, _| cpu.address += 0x100);
            }

            cpu.queue_read(Mos6502::read_address, op);
        });
    }

    fn zero_page(self, cpu: &mut Mos6502) {
        cpu.queue_read(Mos6502::read_pc_increment, Mos6502::set_zero_page_address);
        cpu.queue_read(Mos6502::read_address, self);
    }

    fn zero_page_indexed_x(self, cpu: &mut Mos6502) {
        cpu.queue_read(Mos6502::read_pc_increment, Mos6502::set_zero_page_address);
        cpu.queue_read(Mos6502::read_address, |cpu, data| {
            cpu.set_zero_page_address(data.wrapping_add(cpu.x));
        });
        cpu.queue_read(Mos6502::read_address, self);
    }

    fn zero_page_indexed_y(self, cpu: &mut Mos6502) {
        cpu.queue_read(Mos6502::read_pc_increment, Mos6502::set_zero_page_address);
        cpu.queue_read(Mos6502::read_address, |cpu, data| {
            cpu.set_zero_page_address(data.wrapping_add(cpu.y));
        });
        cpu.queue_read(Mos6502::read_address, self);
    }
}

impl AddressingModes for WriteOperation {
    fn absolute(self, cpu: &mut Mos6502) {
        cpu.queue_read(Mos6502::read_pc_increment, Mos6502::set_address_low);
        cpu.queue_read(Mos6502::read_pc_increment, Mos6502::set_address_high);
        cpu.queue_write(Mos6502::write_address, self);
    }

    fn absolute_indexed_x(self, cpu: &mut Mos6502) {
        cpu.queue_read(Mos6502::read_pc_increment, Mos6502::set_address_low);
        cpu.queue_read(Mos6502::read_pc_increment, |cpu, data| {
            cpu.set_address_high(data);
            let (low, carry) = cpu.address.get_low().overflowing_add(cpu.x);
            cpu.set_address_low(low);
            cpu.address_carry = carry;
        });
        cpu.queue_read(Mos6502::read_address, |cpu, data| {
            if cpu.address_carry {
                let high = cpu.address.get_high();
                cpu.set_address_high(high + 1);
            }
        });
        cpu.queue_write(Mos6502::write_address, self);
    }

    fn absolute_indexed_y(self, cpu: &mut Mos6502) {
        cpu.queue_read(Mos6502::read_pc_increment, Mos6502::set_address_low);
        cpu.queue_read(Mos6502::read_pc_increment, |cpu, data| {
            cpu.set_address_high(data);
            let (low, carry) = cpu.address.get_low().overflowing_add(cpu.y);
            cpu.set_address_low(low);
            cpu.address_carry = carry;
        });
        cpu.queue_read(Mos6502::read_address, |cpu, data| {
            if cpu.address_carry {
                let high = cpu.address.get_high();
                cpu.set_address_high(high + 1);
            }
        });
        cpu.queue_write(Mos6502::write_address, self);
    }

    fn accumulator(self, cpu: &mut Mos6502) {
        todo!()
    }

    fn immediate(self, cpu: &mut Mos6502) {
        todo!();
    }

    fn implied(self, cpu: &mut Mos6502) {
        todo!()
    }

    fn indexed_indirect_x(self, cpu: &mut Mos6502) {
        cpu.queue_read(Mos6502::read_pc_increment, |cpu, data| cpu.pointer = data);
        cpu.queue_read(Mos6502::read_pointer, |cpu, data| cpu.pointer = cpu.pointer.wrapping_add(cpu.x));
        cpu.queue_read(Mos6502::read_pointer_increment, Mos6502::set_address_low);
        cpu.queue_read(Mos6502::read_pointer_increment, Mos6502::set_address_high);
        cpu.queue_write(Mos6502::write_address, self);
    }

    fn indirect_indexed_y(self, cpu: &mut Mos6502) {
        cpu.queue_read(Mos6502::read_pc_increment, |cpu, data| cpu.pointer = data);
        cpu.queue_read(Mos6502::read_pointer_increment, Mos6502::set_address_low);
        cpu.queue_read(Mos6502::read_pointer_increment, |cpu, data| {
            cpu.set_address_high(data);
            let (low, carry) = cpu.address.get_low().overflowing_add(cpu.y);
            cpu.set_address_low(low);
            cpu.address_carry = carry;
        });
        cpu.queue_read(Mos6502::read_address, |cpu, _| {
            if cpu.address_carry {
                let high = cpu.address.get_high() + 1;
                cpu.address.set_high(high);
            }
        });
        cpu.queue_write(Mos6502::write_address, self);
    }

    fn zero_page(self, cpu: &mut Mos6502) {
        cpu.queue_read(Mos6502::read_pc_increment, Mos6502::set_zero_page_address);
        cpu.queue_write(Mos6502::write_address, self);
    }

    fn zero_page_indexed_x(self, cpu: &mut Mos6502) {
        // 2     PC      R  fetch address, increment PC
        cpu.queue_read(Mos6502::read_pc_increment, Mos6502::set_zero_page_address);
        // 3   address   R  read from address, add index register to it
        cpu.queue_read(Mos6502::read_address, |cpu, data| {
            cpu.set_zero_page_address(data.wrapping_add(cpu.x));
        });
        // 4  address+I* W  write to effective address
        cpu.queue_write(Mos6502::write_address, self);
    }

    fn zero_page_indexed_y(self, cpu: &mut Mos6502) {
        // 2     PC      R  fetch address, increment PC
        cpu.queue_read(Mos6502::read_pc_increment, Mos6502::set_zero_page_address);
        // 3   address   R  read from address, add index register to it
        cpu.queue_read(Mos6502::read_address, |cpu, data| {
            cpu.set_zero_page_address(data.wrapping_add(cpu.y));
        });
        // 4  address+I* W  write to effective address
        cpu.queue_write(Mos6502::write_address, self);
    }
}

impl AddressingModes for ReadWriteOperation {
    fn absolute(self, cpu: &mut Mos6502) {
        // 1    PC     R  fetch opcode, increment PC
        // 2    PC     R  fetch low byte of address, increment PC
        cpu.queue_read(Mos6502::read_pc_increment, Mos6502::set_address_low);
        // 3    PC     R  fetch high byte of address, increment PC
        cpu.queue_read(Mos6502::read_pc_increment, Mos6502::set_address_high);
        // 4  address  R  read from effective address
        cpu.queue_read(Mos6502::read_address, |cpu, data| cpu.data = data);
        // 5  address  W  write the value back to effective address,
        //                and do the operation on it
        cpu.queue_read_write_microcode(Mos6502::write_address, self, |cpu, io, op| {
            io(cpu, cpu.data);
            cpu.data = op(cpu, cpu.operand);
        });
        // 6  address  W  write the new value to effective address
        cpu.queue_write(Mos6502::write_address, |cpu| cpu.data);
    }

    fn absolute_indexed_x(self, cpu: &mut Mos6502) {
        //     #   address  R/W description
        //    --- --------- --- ------------------------------------------
        //     1    PC       R  fetch opcode, increment PC
        //     2    PC       R  fetch low byte of address, increment PC
        cpu.queue_read(Mos6502::read_pc_increment, Mos6502::set_address_low);
        //     3    PC       R  fetch high byte of address,
        //                      add index register X to low address byte,
        //                      increment PC
        cpu.queue_read(Mos6502::read_pc_increment, |cpu, data| {
            cpu.set_address_high(data);
            let (low, carry) = cpu.address.get_low().overflowing_add(cpu.x);
            cpu.set_address_low(low);            
            cpu.address_carry = carry;
        });
        //     4  address+X* R  read from effective address,
        //                      fix the high byte of effective address
        cpu.queue_read(Mos6502::read_address, |cpu, data| {
            if cpu.address_carry {
                cpu.address += 0x100;
            }
        });
        //     5  address+X  R  re-read from effective address
        cpu.queue_read(Mos6502::read_address, |cpu, data| cpu.data = data);
        //     6  address+X  W  write the value back to effective address,
        //                      and do the operation on it
        cpu.queue_read_write_microcode(Mos6502::write_address, self, |cpu, io, op| {
            io(cpu, cpu.data);
            cpu.data = op(cpu, cpu.operand);
        });
        //     7  address+X  W  write the new value to effective address
        cpu.queue_write(Mos6502::write_address, |cpu| cpu.data);

        //    Notes: * The high byte of the effective address may be invalid
        //             at this time, i.e. it may be smaller by $100.
    }

    fn absolute_indexed_y(self, cpu: &mut Mos6502) {
        todo!()
    }

    fn accumulator(self, cpu: &mut Mos6502) {
        cpu.queue_read_write_microcode(|cpu, _| cpu.data = cpu.read_pc(), self, |cpu, io, op| {
            cpu.a = op(cpu, cpu.a);
            io(cpu, cpu.a);
        })
    }

    fn immediate(self, cpu: &mut Mos6502) {
        todo!();
    }

    fn implied(self, cpu: &mut Mos6502) {
        todo!()
    }

    fn indexed_indirect_x(self, cpu: &mut Mos6502) {
        //     #    address   R/W description
        //    --- ----------- --- ------------------------------------------
        //     1      PC       R  fetch opcode, increment PC
        //     2      PC       R  fetch pointer address, increment PC
        cpu.queue_read(Mos6502::read_pc_increment, |cpu, data| cpu.pointer = data);
        //     3    pointer    R  read from the address, add X to it
        cpu.queue_read(Mos6502::read_pointer, |cpu, data| cpu.pointer = cpu.pointer.wrapping_add(cpu.x));
        //     4   pointer+X   R  fetch effective address low
        cpu.queue_read(Mos6502::read_pointer_increment, Mos6502::set_address_low);
        //     5  pointer+X+1  R  fetch effective address high
        cpu.queue_read(Mos6502::read_pointer_increment, Mos6502::set_address_high);
        //     6    address    R  read from effective address
        cpu.queue_read(Mos6502::read_address, Mos6502::nop);
        //     7    address    W  write the value back to effective address,
        //                        and do the operation on it
        cpu.queue_read_write_microcode(Mos6502::write_address, self, |cpu, io, op| {
            io(cpu, cpu.data);
            cpu.data = op(cpu, cpu.operand);
        });
        //     8    address    W  write the new value to effective address
        cpu.queue_write(Mos6502::write_address, |cpu| cpu.data);

        //    Note: The effective address is always fetched from zero page,
        //          i.e. the zero page boundary crossing is not handled.
    }

    fn indirect_indexed_y(self, cpu: &mut Mos6502) {
        todo!()
    }

    fn zero_page(self, cpu: &mut Mos6502) {
        cpu.queue_read(Mos6502::read_pc_increment, Mos6502::set_zero_page_address);
        cpu.queue_read(Mos6502::read_address, |cpu, data| cpu.operand = data);
        cpu.queue_read_write_microcode(Mos6502::write_address, self, |cpu, io, op| {
            io(cpu, cpu.data);
            cpu.data = op(cpu, cpu.operand);
        });
        cpu.queue_write(Mos6502::write_address, |cpu| cpu.data);
    }

    fn zero_page_indexed_x(self, cpu: &mut Mos6502) {
        cpu.queue_read(Mos6502::read_pc_increment, Mos6502::set_zero_page_address);
        cpu.queue_read(Mos6502::read_address, |cpu, data| {
            cpu.set_zero_page_address(data.wrapping_add(cpu.x));
        });
        cpu.queue_read(Mos6502::read_address, |cpu, data| cpu.operand = data);
        cpu.queue_read_write_microcode(Mos6502::write_address, self, |cpu, io, op| {
            io(cpu, cpu.data);
            cpu.data = op(cpu, cpu.operand);
        });
        cpu.queue_write(Mos6502::write_address, |cpu| cpu.data);
    }

    fn zero_page_indexed_y(self, cpu: &mut Mos6502) {
        todo!()
    }
}
