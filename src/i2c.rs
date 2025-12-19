use crate::Error;
use bitfield_struct::bitfield;
use embedded_hal::i2c::{ErrorType, I2c, NoAcknowledgeSource, Operation, SevenBitAddress};
use nusb::Interface;
use nusb::MaybeFuture;
use nusb::transfer::{ControlIn, ControlOut, ControlType, Recipient, TransferError};
use std::fmt::{Display, Formatter};
use std::time::Duration;

const CMD_I2C_ENTER_PASSTHRU: u8 = 0x70;
const CMD_I2C_WRITE: u8 = 0x71;
const CMD_I2C_READ: u8 = 0x72;

pub struct I2cBridge {
    interface: Interface,
    timeout: Duration,
}

#[derive(Debug)]
pub enum I2cError {
    Nack,
    WrongAddress,
    Other(TransferError),
}

impl I2cBridge {
    pub(crate) fn init(interface: Interface) -> Result<I2cBridge, Error> {
        let timeout = Duration::from_millis(100);
        interface
            .control_out(
                ControlOut {
                    control_type: ControlType::Vendor,
                    recipient: Recipient::Interface,
                    request: CMD_I2C_ENTER_PASSTHRU,
                    // value: 0x3131, // used in mchp code, but not mentioned in docs, works either way
                    value: 0,
                    index: 0,
                    data: &[],
                },
                timeout,
            )
            .wait()?;
        Ok(I2cBridge { interface, timeout })
    }
}

impl I2c for I2cBridge {
    fn transaction(
        &mut self,
        address: SevenBitAddress,
        operations: &mut [Operation<'_>],
    ) -> Result<(), Self::Error> {
        if address > 0x7F {
            return Err(I2cError::WrongAddress);
        }
        let mut prev_is_read = None;
        let len = operations.len();
        for (i, op) in operations.iter_mut().enumerate() {
            let is_read = matches!(op, Operation::Read(_));
            // generate start condition on first transaction or when read is followed by write (and vice versa, but it's never used in practice?)
            let send_start = (i == 0) || (Some(is_read) != prev_is_read);
            let is_last_transaction = i == len - 1;
            // generate NACK when slave sent enough data
            let send_nack = is_read && is_last_transaction;
            prev_is_read = Some(is_read);
            let flags_addr = I2cFlagsAddress::new()
                .with_send_start(send_start)
                .with_send_stop(is_last_transaction)
                .with_send_nack(send_nack)
                .with_slave_addr(address)
                .with_is_read(is_read);
            match op {
                Operation::Read(buf) => {
                    let data = self
                        .interface
                        .control_in(
                            ControlIn {
                                control_type: ControlType::Vendor,
                                recipient: Recipient::Interface,
                                request: CMD_I2C_READ,
                                value: flags_addr.into_bits(),
                                index: 0, // reserved
                                length: buf.len() as u16,
                            },
                            self.timeout,
                        )
                        .wait()?;
                    buf.copy_from_slice(&data);
                }
                Operation::Write(buf) => {
                    self.interface
                        .control_out(
                            ControlOut {
                                control_type: ControlType::Vendor,
                                recipient: Recipient::Interface,
                                request: CMD_I2C_WRITE,
                                value: flags_addr.into_bits(),
                                index: 0, // reserved
                                data: buf,
                            },
                            self.timeout,
                        )
                        .wait()?;
                }
            }
        }
        Ok(())
    }
}

impl ErrorType for I2cBridge {
    type Error = I2cError;
}

impl embedded_hal::i2c::Error for I2cError {
    fn kind(&self) -> embedded_hal::i2c::ErrorKind {
        match self {
            I2cError::Nack => {
                embedded_hal::i2c::ErrorKind::NoAcknowledge(NoAcknowledgeSource::Unknown)
            }
            _ => embedded_hal::i2c::ErrorKind::Other,
        }
    }
}

#[bitfield(u16, order = Msb)]
struct I2cFlagsAddress {
    #[bits(5)]
    _reserved: u8,
    #[bits(1)]
    send_nack: bool,
    #[bits(1)]
    send_start: bool,
    #[bits(1)]
    send_stop: bool,
    #[bits(7)]
    slave_addr: u8,
    #[bits(1)]
    is_read: bool,
}

impl From<TransferError> for I2cError {
    fn from(value: TransferError) -> Self {
        match value {
            TransferError::Stall => I2cError::Nack,
            other => I2cError::Other(other),
        }
    }
}

impl Display for I2cError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:02X?}", self)
    }
}

impl std::error::Error for I2cError {}
