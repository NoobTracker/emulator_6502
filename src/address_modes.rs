//ADDRESS MODES---------------------------------------------------------------------------------
//  An address mode function is called by an opcode function, returning a memory address and the
//  number of extra cycles that may be required under specific circumstances (Typically crossing page boundaries)

use super::{Interface6502, MOS6502};
use std::fmt;

///Absolute: Address mode returning a 16-bit absolute address
pub(crate) fn absolute(cpu: &mut MOS6502, bus: &mut dyn Interface6502) -> (AddressModeValue, u8) {
    let address: u16 = super::read_16(bus, cpu.program_counter);
    cpu.program_counter += 2;
    return (AddressModeValue::AbsoluteAddress(address), 0);
}

///Absolute X: Address mode returning a 16-bit absolute address offset by the x register
pub(crate) fn absolute_x(cpu: &mut MOS6502, bus: &mut dyn Interface6502) -> (AddressModeValue, u8) {
    let address: u16 = super::read_16(bus, cpu.program_counter);
    let offset_address: u16 = address + u16::from(cpu.x_register);

    let extra_cycles = if (offset_address) & 0xff00 != address & 0xff00 {
        //Offset crossed a page boundary, any opcode using this address mode will take an extra cycle
        1
    } else {
        0
    };

    cpu.program_counter += 2;
    return (AddressModeValue::AbsoluteAddress(offset_address), extra_cycles);
}

///Absolute Y: Address mode returning a 16-bit absolute address offset by the y register
pub(crate) fn absolute_y(cpu: &mut MOS6502, bus: &mut dyn Interface6502) -> (AddressModeValue, u8) {
    let address: u16 = super::read_16(bus, cpu.program_counter);
    let offset_address: u16 = address + u16::from(cpu.y_register);

    let extra_cycles = if (offset_address) & 0xff00 != address & 0xff00 {
        //Offset crossed a page boundary, any opcode using this address mode will take an extra cycle
        1
    } else {
        0
    };

    cpu.program_counter += 2;
    return (AddressModeValue::AbsoluteAddress(offset_address), extra_cycles);
}

///Immediate: Address mode using next byte as value
pub(crate) fn immediate(cpu: &mut MOS6502, _bus: &mut dyn Interface6502) -> (AddressModeValue, u8) {
    //Return the current location of the program counter
    let address = cpu.program_counter;
    cpu.program_counter += 1;
    return (AddressModeValue::AbsoluteAddress(address), 0);
}

///Implied: Address mode for opcodes that do not require a value or address
pub(crate) fn implied(_cpu: &mut MOS6502, _bus: &mut dyn Interface6502) -> (AddressModeValue, u8) {
    return (AddressModeValue::Implied, 0);
}

///Indirect: Address mode that reads from the given address to get the actual address
pub(crate) fn indirect(cpu: &mut MOS6502, bus: &mut dyn Interface6502) -> (AddressModeValue, u8) {
    let indirect_address = super::read_16(bus, cpu.program_counter);

    //Simulate bug at page edge
    let address = if indirect_address & 0x00ff == 0x00ff {
        u16::from(bus.read(indirect_address & 0xff00)) << 8 | u16::from(bus.read(indirect_address))
    } else {
        super::read_16(bus, indirect_address)
    };

    cpu.program_counter += 2;
    return (AddressModeValue::AbsoluteAddress(address), 0);
}

///Indirect X: Address mode that reads from the 8-bit given address offset by x to get the actual address
pub(crate) fn indirect_x(cpu: &mut MOS6502, bus: &mut dyn Interface6502) -> (AddressModeValue, u8) {
    let indirect_address = bus.read(cpu.program_counter);
    let address = super::read_16(bus, u16::from(indirect_address) + u16::from(cpu.x_register));

    cpu.program_counter += 1;
    return (AddressModeValue::AbsoluteAddress(address), 0);
}

///Indirect Y: Address mode that reads from the 8-bit given address to get the actual address and then offsets it by y
pub(crate) fn indirect_y(cpu: &mut MOS6502, bus: &mut dyn Interface6502) -> (AddressModeValue, u8) {
    let indirect_address = bus.read(cpu.program_counter);
    let address = super::read_16(bus, u16::from(indirect_address));
    let offset_address = address + u16::from(cpu.y_register);

    let extra_cycles = if (offset_address) & 0xff00 != address & 0xff00 {
        //Offset crossed a page boundary, any opcode using this address mode will take an extra cycle
        1
    } else {
        0
    };

    cpu.program_counter += 1;
    return (AddressModeValue::AbsoluteAddress(offset_address), extra_cycles);
}

///Relative: Address mode used by branch instructions that reads an 8-bit signed relative address to add to the program counter
pub(crate) fn relative(cpu: &mut MOS6502, bus: &mut dyn Interface6502) -> (AddressModeValue, u8) {
    let relative_address = bus.read(cpu.program_counter);
    cpu.program_counter += 1;
    return (AddressModeValue::RelativeAddress(relative_address), 0);
}

///Zero-page: Address mode that uses an 8-bit address to access memory on the 0 page (0x00__)
pub(crate) fn zero_page(cpu: &mut MOS6502, bus: &mut dyn Interface6502) -> (AddressModeValue, u8) {
    let address = u16::from(bus.read(cpu.program_counter));
    cpu.program_counter += 1;
    return (AddressModeValue::AbsoluteAddress(address), 0);
}

///Zero-page X: Address mode that uses an 8-bit address to access memory on the 0 page (0x00__), offset by x
// TODO: Implement offset bug
pub(crate) fn zero_page_x(cpu: &mut MOS6502, bus: &mut dyn Interface6502) -> (AddressModeValue, u8) {
    let address = bus.read(cpu.program_counter).wrapping_add(cpu.x_register);
    cpu.program_counter += 1;
    return (AddressModeValue::AbsoluteAddress(u16::from(address)), 0);
}

///Zero-page Y: Address mode that uses an 8-bit address to access memory on the 0 page (0x00__), offset by y
// TODO: Implement offset bug
pub(crate) fn zero_page_y(cpu: &mut MOS6502, bus: &mut dyn Interface6502) -> (AddressModeValue, u8) {
    let address = bus.read(cpu.program_counter).wrapping_add(cpu.y_register);
    cpu.program_counter += 1;
    return (AddressModeValue::AbsoluteAddress(u16::from(address)), 0);
}

///Enum for the return type of Address modes
#[derive(PartialEq)]
pub(crate) enum AddressModeValue {
    Implied,
    RelativeAddress(u8),
    AbsoluteAddress(u16),
}

impl fmt::Debug for AddressModeValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AddressModeValue::Implied => write!(f, "Implied"),
            AddressModeValue::RelativeAddress(address) => write!(f, "Relative Address: {:02X}", address),
            AddressModeValue::AbsoluteAddress(address) => write!(f, "Absolute Address: {:04X}", address),
        }
    }
}

//TESTS---------------------------------------------------------------------------------------------

#[cfg(test)]
mod test {
    #![allow(unused_variables, unused_mut)] //Allow some warnings for test code
    use super::super::StubInterface6502;
    use super::*;

    #[test]
    fn test_absolute() {
        let mut cpu = MOS6502::new_start(0x0000);
        let mut bus = StubInterface6502::new(|address| 0xff, |address, data| panic!("Write function was called"));

        let expected_program_counter = cpu.program_counter + 2;
        let (address_mode_value, extra_cycles) = absolute(&mut cpu, &mut bus);

        assert_eq!(address_mode_value, AddressModeValue::AbsoluteAddress(0xffff));
        assert_eq!(extra_cycles, 0);
        assert_eq!(expected_program_counter, cpu.program_counter)
    }

    #[test]
    fn test_absolute_x() {
        let mut cpu = MOS6502::new_start(0x0000);
        let mut bus = StubInterface6502::new(|address| 0x00, |address, data| panic!("Write function was called"));
        cpu.x_register = 2;
        let expected_program_counter = cpu.program_counter + 2;
        let (address_mode_value, extra_cycles) = absolute_x(&mut cpu, &mut bus);

        assert_eq!(address_mode_value, AddressModeValue::AbsoluteAddress(0x0002));
        assert_eq!(extra_cycles, 0);
        assert_eq!(expected_program_counter, cpu.program_counter)
    }

    #[test]
    fn test_absolute_x_extra_cycle() {
        let mut cpu = MOS6502::new_start(0x0000);
        let mut bus = StubInterface6502::new(|address| 0x10, |address, data| panic!("Write function was called"));
        cpu.x_register = 255;
        let expected_program_counter = cpu.program_counter + 2;
        let (address_mode_value, extra_cycles) = absolute_x(&mut cpu, &mut bus);

        assert_eq!(address_mode_value, AddressModeValue::AbsoluteAddress(0x110f));
        assert_eq!(extra_cycles, 1);
        assert_eq!(expected_program_counter, cpu.program_counter)
    }

    #[test]
    fn test_absolute_y() {
        let mut cpu = MOS6502::new_start(0x0000);
        let mut bus = StubInterface6502::new(|address| 0x00, |address, data| panic!("Write function was called"));
        cpu.y_register = 2;
        let expected_program_counter = cpu.program_counter + 2;
        let (address_mode_value, extra_cycles) = absolute_y(&mut cpu, &mut bus);

        assert_eq!(address_mode_value, AddressModeValue::AbsoluteAddress(0x0002));
        assert_eq!(extra_cycles, 0);
        assert_eq!(expected_program_counter, cpu.program_counter)
    }

    #[test]
    fn test_absolute_y_extra_cycle() {
        let mut cpu = MOS6502::new_start(0x0000);
        let mut bus = StubInterface6502::new(|address| 0x10, |address, data| panic!("Write function was called"));
        cpu.y_register = 255;
        let expected_program_counter = cpu.program_counter + 2;
        let (address_mode_value, extra_cycles) = absolute_y(&mut cpu, &mut bus);

        assert_eq!(address_mode_value, AddressModeValue::AbsoluteAddress(0x110f));
        assert_eq!(extra_cycles, 1);
        assert_eq!(expected_program_counter, cpu.program_counter)
    }

    #[test]
    fn test_immediate() {
        let mut cpu = MOS6502::new_start(0x0000);
        let mut bus = StubInterface6502::new(|address| panic!("Read function was called"), |address, data| panic!("Write function was called"));

        let prior_program_counter = cpu.program_counter;
        let (address_mode_value, extra_cycles) = immediate(&mut cpu, &mut bus);

        assert_eq!(address_mode_value, AddressModeValue::AbsoluteAddress(prior_program_counter));
        assert_eq!(extra_cycles, 0);
        assert_eq!(prior_program_counter + 1, cpu.program_counter)
    }

    #[test]
    fn test_implied() {
        let mut cpu = MOS6502::new_start(0x0000);
        let mut bus = StubInterface6502::new(|address| panic!("Read function was called"), |address, data| panic!("Write function was called"));

        let expected_program_counter = cpu.program_counter;
        let (address_mode_value, extra_cycles) = implied(&mut cpu, &mut bus);

        assert_eq!(address_mode_value, AddressModeValue::Implied);
        assert_eq!(extra_cycles, 0);
        assert_eq!(expected_program_counter, cpu.program_counter)
    }

    #[test]
    fn test_indirect() {
        let mut cpu = MOS6502::new_start(0x0000);
        let mut bus = StubInterface6502::new(
            |address| match address {
                0x0000 => 0x11,
                0x0001 => 0x10,
                0x1011 => 0x01,
                0x1012 => 0xff,
                _ => 0x00,
            },
            |address, data| panic!("Write function was called"),
        );

        let expected_program_counter = cpu.program_counter + 2;
        let (address_mode_value, extra_cycles) = indirect(&mut cpu, &mut bus);

        assert_eq!(address_mode_value, AddressModeValue::AbsoluteAddress(0xff01));
        assert_eq!(extra_cycles, 0);
        assert_eq!(expected_program_counter, cpu.program_counter)
    }

    #[test]
    fn test_indirect_bug() {
        let mut cpu = MOS6502::new_start(0x0000);
        let mut bus = StubInterface6502::new(
            |address| match address {
                0x0000 => 0xff,
                0x0001 => 0x10,
                0x10ff => 0x01,
                0x1000 => 0xa7,
                _ => 0x00,
            },
            |address, data| panic!("Write function was called"),
        );

        let expected_program_counter = cpu.program_counter + 2;
        let (address_mode_value, extra_cycles) = indirect(&mut cpu, &mut bus);

        assert_eq!(address_mode_value, AddressModeValue::AbsoluteAddress(0xa701));
        assert_eq!(extra_cycles, 0);
        assert_eq!(expected_program_counter, cpu.program_counter)
    }

    #[test]
    fn test_indirect_x() {
        let mut cpu = MOS6502::new_start(0x0000);
        let mut bus = StubInterface6502::new(
            |address| match address {
                0x0000 => 0x25,
                0x0035 => 0x01,
                0x0036 => 0xa7,
                _ => 0x00,
            },
            |address, data| panic!("Write function was called"),
        );

        cpu.x_register = 0x10;
        let expected_program_counter = cpu.program_counter + 1;
        let (address_mode_value, extra_cycles) = indirect_x(&mut cpu, &mut bus);

        assert_eq!(address_mode_value, AddressModeValue::AbsoluteAddress(0xa701));
        assert_eq!(extra_cycles, 0);
        assert_eq!(expected_program_counter, cpu.program_counter)
    }

    #[test]
    fn test_indirect_y() {
        let mut cpu = MOS6502::new_start(0x0000);
        let mut bus = StubInterface6502::new(
            |address| match address {
                0x0000 => 0x25,
                0x0025 => 0x01,
                0x0026 => 0xa7,
                _ => 0x00,
            },
            |address, data| panic!("Write function was called"),
        );

        cpu.y_register = 0x10;
        let expected_program_counter = cpu.program_counter + 1;
        let (address_mode_value, extra_cycles) = indirect_y(&mut cpu, &mut bus);

        assert_eq!(address_mode_value, AddressModeValue::AbsoluteAddress(0xa711));
        assert_eq!(extra_cycles, 0);
        assert_eq!(expected_program_counter, cpu.program_counter)
    }

    #[test]
    fn test_indirect_y_extra_cycle() {
        let mut cpu = MOS6502::new_start(0x0000);
        let mut bus = StubInterface6502::new(
            |address| match address {
                0x0000 => 0x25,
                0x0025 => 0xff,
                0x0026 => 0xa7,
                _ => 0x00,
            },
            |address, data| panic!("Write function was called"),
        );

        cpu.y_register = 0x10;
        let expected_program_counter = cpu.program_counter + 1;
        let (address_mode_value, extra_cycles) = indirect_y(&mut cpu, &mut bus);

        assert_eq!(address_mode_value, AddressModeValue::AbsoluteAddress(0xa80f));
        assert_eq!(extra_cycles, 1);
        assert_eq!(expected_program_counter, cpu.program_counter)
    }

    #[test]
    fn test_relative() {
        let mut cpu = MOS6502::new_start(0x0000);
        let mut bus = StubInterface6502::new(|address| 0x10, |address, data| panic!("Write function was called"));

        let expected_program_counter = cpu.program_counter + 1;
        let (address_mode_value, extra_cycles) = relative(&mut cpu, &mut bus);

        assert_eq!(address_mode_value, AddressModeValue::RelativeAddress(0x10));
        assert_eq!(extra_cycles, 0);
        assert_eq!(expected_program_counter, cpu.program_counter)
    }

    #[test]
    fn test_zero_page() {
        let mut cpu = MOS6502::new_start(0x0000);
        let mut bus = StubInterface6502::new(|address| 0x10, |address, data| panic!("Write function was called"));

        let expected_program_counter = cpu.program_counter + 1;
        let (address_mode_value, extra_cycles) = zero_page(&mut cpu, &mut bus);

        assert_eq!(address_mode_value, AddressModeValue::AbsoluteAddress(0x0010));
        assert_eq!(extra_cycles, 0);
        assert_eq!(expected_program_counter, cpu.program_counter)
    }

    #[test]
    fn test_zero_page_x() {
        let mut cpu = MOS6502::new_start(0x0000);
        let mut bus = StubInterface6502::new(|address| 0x10, |address, data| panic!("Write function was called"));

        cpu.x_register = 0x10;
        let expected_program_counter = cpu.program_counter + 1;
        let (address_mode_value, extra_cycles) = zero_page_x(&mut cpu, &mut bus);

        assert_eq!(address_mode_value, AddressModeValue::AbsoluteAddress(0x0020));
        assert_eq!(extra_cycles, 0);
        assert_eq!(expected_program_counter, cpu.program_counter)
    }

    #[test]
    fn test_zero_page_y() {
        let mut cpu = MOS6502::new_start(0x0000);
        let mut bus = StubInterface6502::new(|address| 0x10, |address, data| panic!("Write function was called"));

        cpu.y_register = 0x10;
        let expected_program_counter = cpu.program_counter + 1;
        let (address_mode_value, extra_cycles) = zero_page_y(&mut cpu, &mut bus);

        assert_eq!(address_mode_value, AddressModeValue::AbsoluteAddress(0x0020));
        assert_eq!(extra_cycles, 0);
        assert_eq!(expected_program_counter, cpu.program_counter)
    }
}
