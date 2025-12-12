use nusb::MaybeFuture;
use std::time::Duration;
use nusb::Interface;
use nusb::transfer::{ControlIn, ControlOut, ControlType, Recipient, TransferError};
use crate::SmscReg;

pub struct Usb4604 {
    interface: Interface
}

const CMD_REG_WRITE: u8 = 3;
const CMD_REG_READ: u8 = 4;

impl Usb4604 {
    /// Create Usb4604 from an already open nusb USB [Interface](Interface).
    pub fn new(interface: Interface) -> Usb4604 {
        Usb4604 { interface }
    }

    /// Enumerate, and open the first and only available device.
    /// Error is returned if more than one device if found.
    ///
    /// If multiple device support is required, use [new](Self::new) and implement a desired filtering system.
    pub fn open_auto() -> Result<Self, String> {
        todo!()
    }

    pub fn read_reg<R: SmscReg>(&mut self) -> Result<R, TransferError> {
        let read = self.interface
            .control_in(
                ControlIn {
                    control_type: ControlType::Vendor,
                    recipient: Recipient::Interface,
                    request: CMD_REG_READ,
                    value: R::ADDR,
                    index: 0,
                    length: 1,
                },
                Duration::from_millis(500),
            )
            .wait()?;
        Ok(R::from_value(read[0]))
    }

    pub fn write_reg<R: SmscReg>(&mut self, value: R) -> Result<(), TransferError> {
        self.interface
            .control_out(
                ControlOut {
                    control_type: ControlType::Vendor,
                    recipient: Recipient::Interface,
                    request: CMD_REG_WRITE,
                    value: R::ADDR,
                    index: 0,
                    data: &[value.value()],
                },
                Duration::from_millis(500),
            )
            .wait()?;
        Ok(())
    }

    pub fn modify_reg<R: SmscReg, F: FnMut(&mut R)>(&mut self, mut f: F) -> Result<(), TransferError> {
        let mut value: R = self.read_reg()?;
        let old_value = value.value();
        f(&mut value);
        if old_value != value.value() {
            self.write_reg(value)?;
        }
        Ok(())
    }
}