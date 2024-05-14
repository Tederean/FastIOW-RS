use crate::i2c::{i2c_service, I2CConfig, I2CError};
use crate::iowarrior::{
    peripheral_service, IOWarriorData, IOWarriorMutData, Peripheral, PeripheralSetupError,
};
use std::cell::RefCell;
use std::fmt;
use std::rc::Rc;

#[derive(Debug)]
pub struct I2C {
    data: Rc<IOWarriorData>,
    mut_data_refcell: Rc<RefCell<IOWarriorMutData>>,
    i2c_config: I2CConfig,
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
    fn transaction(
        &mut self,
        address: embedded_hal::i2c::SevenBitAddress,
        operations: &mut [embedded_hal::i2c::Operation],
    ) -> Result<(), Self::Error> {
        i2c_service::check_valid_7bit_address(address)?;

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
        i2c_service::check_valid_7bit_address(address)?;

        let mut mut_data = self.mut_data_refcell.borrow_mut();

        i2c_service::write_data(&self.data, &mut mut_data, address, bytes)
    }
}

#[cfg(feature = "embedded-hal-0")]
impl embedded_hal_0::blocking::i2c::Read for I2C {
    type Error = I2CError;

    #[inline]
    fn read(&mut self, address: u8, buffer: &mut [u8]) -> Result<(), Self::Error> {
        i2c_service::check_valid_7bit_address(address)?;

        let mut mut_data = self.mut_data_refcell.borrow_mut();

        i2c_service::read_data(&self.data, &mut mut_data, address, buffer)
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
        i2c_service::check_valid_7bit_address(address)?;

        let mut mut_data = self.mut_data_refcell.borrow_mut();

        i2c_service::write_data(&self.data, &mut mut_data, address, bytes)?;
        i2c_service::read_data(&self.data, &mut mut_data, address, buffer)
    }
}

impl I2C {
    pub(crate) fn new(
        data: &Rc<IOWarriorData>,
        mut_data_refcell: &Rc<RefCell<IOWarriorMutData>>,
        i2c_config: I2CConfig,
    ) -> Result<I2C, PeripheralSetupError> {
        i2c_service::enable_i2c(&data, &mut mut_data_refcell.borrow_mut(), i2c_config)?;

        Ok(I2C {
            data: data.clone(),
            mut_data_refcell: mut_data_refcell.clone(),
            i2c_config,
        })
    }

    #[inline]
    pub fn get_config(&self) -> I2CConfig {
        self.i2c_config
    }
}
