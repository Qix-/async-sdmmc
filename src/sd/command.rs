use core::mem;

use super::response;

#[derive(Copy, Clone, Default, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct SendInterfaceCondition {
    pub pcie_1_2v_suppport: bool, // PCIe 1.2V
    pub pcie_availability: bool,
    pub voltage_supplied: bool,
    pub check_pattern: u8,
}

impl SendInterfaceCondition {
    pub fn spi() -> Self {
        Self { voltage_supplied: true, check_pattern: 0xAA, ..Default::default() }
    }
}

impl From<SendInterfaceCondition> for u32 {
    fn from(val: SendInterfaceCondition) -> Self {
        (val.pcie_1_2v_suppport as u32) << 15
            | (val.pcie_availability as u32) << 14
            | (val.voltage_supplied as u32) << 8
            | val.check_pattern as u32
    }
}

pub type Rca = u16;
pub type Address = u32;

#[repr(u8)]
#[derive(Copy, Clone, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum AppCommand {
    SDSendOpCond(bool), // host-capability-support
    ReadOCR,
}

impl AppCommand {
    pub fn index(self) -> u8 {
        match self {
            Self::SDSendOpCond(_) => 41,
            Self::ReadOCR => 58,
        }
    }

    pub fn argument(self) -> u32 {
        match self {
            Self::SDSendOpCond(hcs) => (hcs as u32) << 30,
            Self::ReadOCR => 0,
        }
    }

    pub fn expected_response_ex_size(self) -> usize {
        match self {
            Self::ReadOCR => mem::size_of::<response::R3>(),
            _ => 0,
        }
    }
}

#[expect(clippy::enum_variant_names)]
#[repr(u8)]
#[derive(Copy, Clone, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Command {
    GoIdleState,
    SendIfCond(SendInterfaceCondition),
    SendCSD(Rca),
    StopTransmission,
    ReadSingleBlock(Address),
    ReadMultipleBlock(Address),
    WriteBlock(Address),
    WriteMultipleBlock(Address),
    AppCommand(Rca),
    App(AppCommand),
}

impl Command {
    pub fn index(self) -> u8 {
        match self {
            Self::GoIdleState => 0,
            Self::SendIfCond(_) => 8,
            Self::SendCSD(_) => 9,
            Self::StopTransmission => 12,
            Self::ReadSingleBlock(_) => 17,
            Self::ReadMultipleBlock(_) => 18,
            Self::WriteBlock(_) => 24,
            Self::WriteMultipleBlock(_) => 25,
            Self::AppCommand(_) => 55,
            Self::App(command) => command.index(),
        }
    }

    pub fn argument(self) -> u32 {
        match self {
            Self::GoIdleState | Self::StopTransmission => 0,
            Self::SendIfCond(cond) => cond.into(),
            Self::SendCSD(rca) | Self::AppCommand(rca) => (rca as u32) << 16,
            Self::ReadSingleBlock(address)
            | Self::ReadMultipleBlock(address)
            | Self::WriteBlock(address)
            | Self::WriteMultipleBlock(address) => address,
            Self::App(command) => command.argument(),
        }
    }

    pub fn expected_response_ex_size(self) -> usize {
        match self {
            Self::SendIfCond(_) => mem::size_of::<response::R7>(),
            Self::WriteBlock(_) | Self::WriteMultipleBlock(_) => 1,
            Self::App(app_command) => app_command.expected_response_ex_size(),
            _ => 0,
        }
    }
}

fn crc7(data: &[u8]) -> u8 {
    let mut crc = 0u8;
    for &b in data.iter() {
        for i in 0..8 {
            crc <<= 1;
            if (((b << i) & 0x80) ^ (crc & 0x80)) != 0 {
                crc ^= 0x09;
            }
        }
    }
    crc << 1 | 1
}

impl From<Command> for [u8; 6] {
    fn from(val: Command) -> Self {
        let bytes = u32::to_be_bytes(val.argument());
        let mut buffer = [0x40 | val.index(), bytes[0], bytes[1], bytes[2], bytes[3], 0];
        buffer[5] = crc7(&buffer[..5]);
        buffer
    }
}

mod test {
    #[test]
    fn test_command_to_bytes() {
        use super::{AppCommand, Command};
        use hex_literal::hex;

        let cmd = Command::GoIdleState;
        let bytes: [u8; 6] = cmd.into();
        assert_eq!(bytes, hex!("40 00 00 00 00 95"));

        let cmd = Command::App(AppCommand::SDSendOpCond(true));
        let bytes: [u8; 6] = cmd.into();
        assert_eq!(bytes, hex!("69 40 00 00 00 77"));

        let cmd = Command::SendCSD(0);
        let bytes: [u8; 6] = cmd.into();
        assert_eq!(bytes, hex!("49 00 00 00 00 AF"));

        let cmd = Command::ReadSingleBlock(0);
        let bytes: [u8; 6] = cmd.into();
        assert_eq!(bytes, hex!("51 00 00 00 00 55"));
    }
}
