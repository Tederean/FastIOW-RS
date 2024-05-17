use crate::i2c::{i2c_service, I2CConfig, I2CError};
use crate::iowarrior::{peripheral_service, IOWarriorData, IOWarriorMutData, Peripheral};
use std::cell::RefCell;
use std::fmt;
use std::rc::Rc;

#[derive(Debug)]
pub struct I2C {
    pub(crate) data: Rc<IOWarriorData>,
    pub(crate) mut_data_refcell: Rc<RefCell<IOWarriorMutData>>,
    pub(crate) i2c_config: I2CConfig,
}

impl fmt::Display for I2C {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Drop for I2C {
    #[inline]
    fn drop(&mut self) {
        peripheral_service::disable_peripheral(
            &self.data,
            &mut self.mut_data_refcell.borrow_mut(),
            Peripheral::I2C,
        );
    }
}

impl embedded_hal::i2c::ErrorType for I2C {
    type Error = I2CError;
}

impl embedded_hal::i2c::I2c<embedded_hal::i2c::SevenBitAddress> for I2C {
    #[inline]
    fn transaction(
        &mut self,
        address: embedded_hal::i2c::SevenBitAddress,
        operations: &mut [embedded_hal::i2c::Operation],
    ) -> Result<(), Self::Error> {
        let mut mut_data = self.mut_data_refcell.borrow_mut();

        for operation in operations {
            match operation {
                embedded_hal::i2c::Operation::Read(buffer) => {
                    i2c_service::read_data(&self.data, &mut mut_data, address, buffer)?;
                }
                embedded_hal::i2c::Operation::Write(buffer) => {
                    i2c_service::write_data(&self.data, &mut mut_data, address, buffer)?;
                }
            }
        }

        Ok(())
    }
}

#[cfg(feature = "embedded-hal-0")]
impl embedded_hal_0::blocking::i2c::Write for I2C {
    type Error = I2CError;

    #[inline]
    fn write(&mut self, address: u8, bytes: &[u8]) -> Result<(), Self::Error> {
        i2c_service::write_data(
            &self.data,
            &mut self.mut_data_refcell.borrow_mut(),
            address,
            bytes,
        )
    }
}

#[cfg(feature = "embedded-hal-0")]
impl embedded_hal_0::blocking::i2c::Read for I2C {
    type Error = I2CError;

    #[inline]
    fn read(&mut self, address: u8, buffer: &mut [u8]) -> Result<(), Self::Error> {
        i2c_service::read_data(
            &self.data,
            &mut self.mut_data_refcell.borrow_mut(),
            address,
            buffer,
        )
    }
}

#[cfg(feature = "embedded-hal-0")]
impl embedded_hal_0::blocking::i2c::WriteRead for I2C {
    type Error = I2CError;

    #[inline]
    fn write_read(
        &mut self,
        address: u8,
        bytes: &[u8],
        buffer: &mut [u8],
    ) -> Result<(), Self::Error> {
        let mut mut_data = self.mut_data_refcell.borrow_mut();

        i2c_service::write_data(&self.data, &mut mut_data, address, bytes)?;
        i2c_service::read_data(&self.data, &mut mut_data, address, buffer)
    }
}

impl I2C {
    #[inline]
    pub fn get_config(&self) -> I2CConfig {
        self.i2c_config
    }
}
