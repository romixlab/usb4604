use crate::gpio::{Pio, Pull};
use crate::i2c::I2cBridge;
use crate::{Error, Flex, Input, Level, OpenDrainOutput, PushPullOutput, SmscReg};
use nusb::Interface;
use nusb::MaybeFuture;
use nusb::transfer::{ControlIn, ControlOut, ControlType, Recipient, TransferError};
use std::time::Duration;

#[derive(Clone)]
pub struct Usb4604 {
    interface: Interface,
}

const CMD_REG_WRITE: u8 = 0x03;
const CMD_REG_READ: u8 = 0x04;

const VENDOR_SMSC: u16 = 0x0424;
const PRODUCT_BRIDGE_DEV: u16 = 0x2530;
// const PRODUCT_USB4604_HUB: u16 = 0x4502;

impl Usb4604 {
    /// Create Usb4604 from an already open nusb USB [Interface](Interface).
    pub fn new(interface: Interface) -> Usb4604 {
        Usb4604 { interface }
    }

    /// Enumerate, and open the first and only available device.
    /// Error is returned if more than one device is found.
    ///
    /// If multiple device support is required, use [new](Self::new) and implement a desired filtering system.
    pub fn open_auto() -> Result<Self, Error> {
        let di = nusb::list_devices()
            .wait()?
            .find(|d| d.vendor_id() == VENDOR_SMSC && d.product_id() == PRODUCT_BRIDGE_DEV);
        let Some(di) = di else {
            return Err(Error::NoDevicesFound);
        };
        let device = di.open().wait()?;
        let interface = device.claim_interface(0).wait()?;
        Ok(Self { interface })
    }

    /// Read pin mode from the IC and create a [Flex](Flex) pin.
    pub fn gpio(&self, pio: Pio) -> Result<Flex, Error> {
        Ok(Flex::init_get_mode(self.clone(), pio)?)
    }

    /// Optionally set initial level, configure pin as output and return [PushPullOutput].
    pub fn output(&self, pio: Pio, initial: Option<Level>) -> Result<PushPullOutput, Error> {
        let flex = Flex::init_ignore_mode(self.clone(), pio);
        Ok(flex.into_output(initial)?)
    }

    /// Configure pin as input, optionally enable pull-up or pull-down resistor and return [Input].
    pub fn input(&self, pio: Pio, pull: Pull) -> Result<Input, Error> {
        let flex = Flex::init_ignore_mode(self.clone(), pio);
        Ok(flex.into_input(pull)?)
    }

    /// Configure pin as input + open-drain output mode, optionally enable pull-up or pull-down resistor and return [OpenDrainOutput].
    pub fn open_drain(&self, pio: Pio, pull: Pull) -> Result<OpenDrainOutput, Error> {
        let flex = Flex::init_ignore_mode(self.clone(), pio);
        Ok(flex.into_open_drain_output(pull)?)
    }

    pub fn read_reg<R: SmscReg>(&self) -> Result<R, TransferError> {
        let read = self
            .interface
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

    pub fn modify_reg<R: SmscReg, F: FnMut(&mut R)>(
        &mut self,
        mut f: F,
    ) -> Result<(), TransferError> {
        let mut value: R = self.read_reg()?;
        let old_value = value.value();
        f(&mut value);
        if old_value != value.value() {
            self.write_reg(value)?;
        }
        Ok(())
    }

    /// Enable I2C bridging and return [I2cBridge]
    pub fn i2c_bridge(&self) -> Result<I2cBridge, Error> {
        let i2c = I2cBridge::init(self.interface.clone())?;
        Ok(i2c)
    }
}
