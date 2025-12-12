mod usb4604_reg;
mod usb4604_hal;

pub use usb4604_hal::Usb4604;
pub use usb4604_reg::*;

pub trait SmscReg {
    const ADDR: u16;
    fn from_value(bits: u8) -> Self;
    fn value(&self) -> u8;
}
