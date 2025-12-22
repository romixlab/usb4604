mod gpio;
mod i2c;
mod usb4604_hal;
pub mod usb4604_reg;

pub use embedded_hal::i2c::{I2c, Operation};
pub use gpio::{Flex, Input, Level, Mode, OpenDrainOutput, Pio, PioIter, Pull, PushPullOutput};
pub use i2c::{I2cBridge, I2cError};
use nusb::transfer::TransferError;
use std::fmt::{Display, Formatter};
pub use usb4604_hal::Usb4604;

pub trait SmscReg {
    const ADDR: u16;
    fn from_value(bits: u8) -> Self;
    fn value(&self) -> u8;
}

#[derive(Debug)]
pub enum Error {
    TransferError(TransferError),
    Nusb(nusb::Error),
    NoDevicesFound,
    MultipleDevicesFound,
    Other(&'static str),
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for Error {}

impl From<TransferError> for Error {
    fn from(e: TransferError) -> Error {
        Error::TransferError(e)
    }
}

impl From<nusb::Error> for Error {
    fn from(e: nusb::Error) -> Self {
        Error::Nusb(e)
    }
}
