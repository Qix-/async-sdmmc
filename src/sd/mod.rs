pub mod command;
pub mod registers;
pub mod response;
pub mod transfer;

pub const BLOCK_SIZE: usize = 512;

#[derive(Copy, Clone, Debug)]
pub enum Card {
    Sdsc(u8),
    Sdhc,
}

impl Card {
    pub fn high_capacity(self) -> bool {
        !matches!(self, Self::Sdsc(_))
    }
}
