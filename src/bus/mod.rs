pub mod spi;

use crate::sd::{BLOCK_SIZE, registers::Csd, response::R1Status, transfer};

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Error<BUS> {
    BUS(BUS),
    /// Probably no card
    NoResponse,
    NotIdle,
    Command(R1Status),
    Transfer(transfer::TokenError),
    /// No respond within expected duration
    Timeout,
    Generic,
}

impl<BUS> From<R1Status> for Error<BUS> {
    fn from(status: R1Status) -> Self {
        Error::Command(status)
    }
}

impl<BUS> From<transfer::TokenError> for Error<BUS> {
    fn from(e: transfer::TokenError) -> Self {
        Error::Transfer(e)
    }
}

pub trait Bus {
    type Error;
    fn before(&mut self) -> Result<(), Error<Self::Error>>;
    fn after(&mut self) -> Result<(), Error<Self::Error>>;
}

pub trait Read {
    type Error;

    fn read_csd(&mut self) -> impl Future<Output = Result<Csd, Error<Self::Error>>>;

    fn read<'a, B>(
        &mut self,
        block: u32,
        blocks: B,
    ) -> impl Future<Output = Result<(), Error<Self::Error>>>
    where
        B: core::iter::ExactSizeIterator<Item = &'a mut [u8; BLOCK_SIZE]>;
}

pub trait Write {
    type Error;

    fn write<'a, B>(
        &mut self,
        block: u32,
        blocks: B,
    ) -> impl Future<Output = Result<(), Error<Self::Error>>>
    where
        B: core::iter::ExactSizeIterator<Item = &'a [u8; BLOCK_SIZE]>;
}
