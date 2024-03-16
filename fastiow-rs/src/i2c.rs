use crate::bits::Bit::{Bit0, Bit6, Bit7};
use crate::bits::Bitmasking;
use crate::iowkit::{IOWarriorData, IOWarriorMutData, IowkitError, ReportId};
use crate::{IOWarriorType, ModuleEnableError};
use refinement::{Predicate, Refinement};
use std::cell::RefCell;
use std::rc::Rc;
use thiserror::Error;

#[non_exhaustive]
#[derive(Debug, Error, Copy, Clone)]
pub enum I2CCommunicationError {
    #[error("IOWarrior input output error.")]
    IOErrorIOWarrior,
    #[error("I2C input output error.")]
    IOErrorI2C,
    #[error("I2C input output error, arbitration lost.")]
    IOErrorI2CArbitrationLost,
}

pub struct I2C {
    pub(crate) data: Rc<IOWarriorData>,
    pub(crate) mut_data_refcell: Rc<RefCell<IOWarriorMutData>>,
}

impl I2C {
    pub(crate) fn new(
        data: &Rc<IOWarriorData>,
        mut_data_refcell: &Rc<RefCell<IOWarriorMutData>>,
    ) -> Result<I2C, ModuleEnableError> {
        {
            let mut mut_data = mut_data_refcell.borrow_mut();

            if mut_data.i2c_struct_existing {
                return Err(ModuleEnableError::AlreadyEnabled);
            }

            // TODO check pins

            match data.enable_i2c(true) {
                Ok(_) => {
                    mut_data.i2c_hardware_enabled = true;
                }
                Err(error) => {
                    mut_data.i2c_hardware_enabled = false;

                    return match error {
                        IowkitError::IOErrorIOWarrior => Err(ModuleEnableError::IOErrorIOWarrior),
                    };
                }
            }

            mut_data.i2c_struct_existing = true;
        }

        Ok(I2C {
            data: data.clone(),
            mut_data_refcell: mut_data_refcell.clone(),
        })
    }

    pub fn write_data(
        &self,
        address: &I2CAddress,
        buffer: &[u8],
    ) -> Result<(), I2CCommunicationError> {
        self.send_i2c_write(&address, buffer)?;
        self.request_i2c_write()?;

        Ok(())
    }

    pub fn read_data(
        &self,
        address: &I2CAddress,
        buffer: &mut [u8],
    ) -> Result<(), I2CCommunicationError> {
        todo!()
    }

    fn send_i2c_write(
        &self,
        address: &I2CAddress,
        buffer: &[u8],
    ) -> Result<(), I2CCommunicationError> {
        let chunk_iterator = buffer.chunks((self.data.special_report_size - 3) as usize);
        let chunk_iterator_count = chunk_iterator.len();

        for (index, chunk) in chunk_iterator.enumerate() {
            let start_byte = index == 0;
            let stop_byte = index == chunk_iterator_count - 1;

            let mut report = self.data.create_report(self.data.i2c_pipe);

            report.data[0] = ReportId::I2cWrite.get_value();
            report.data[1] = (chunk.len() + 1) as u8;

            if start_byte {
                report.data[1].set_bit(Bit7, true); // Enable start
            }

            if stop_byte {
                report.data[1].set_bit(Bit6, true); // Enable stop
            }

            report.data[2] = address.to_inner() << 1;
            report.data[2].set_bit(Bit0, false); // Read/Write address

            //report.data[3..] = *chunk; //TODO

            match self.data.write_report(&report) {
                Ok(_) => {}
                Err(error) => {
                    return match error {
                        IowkitError::IOErrorIOWarrior => {
                            Err(I2CCommunicationError::IOErrorIOWarrior)
                        }
                    }
                }
            };
        }

        Ok(())
    }

    fn request_i2c_write(&self) -> Result<(), I2CCommunicationError> {
        match self.data.read_report(self.data.i2c_pipe) {
            Ok(report) => {
                assert_eq!(report.data[0], ReportId::I2cWrite.get_value());

                if report.data[1].get_bit(Bit7) {
                    return Err(I2CCommunicationError::IOErrorI2C);
                }

                match self.data.device_type {
                    IOWarriorType::IOWarrior28
                    | IOWarriorType::IOWarrior28Dongle
                    | IOWarriorType::IOWarrior56
                    | IOWarriorType::IOWarrior56Dongle
                    | IOWarriorType::IOWarrior56Old => {
                        if report.data[1].get_bit(Bit7) {
                            return Err(I2CCommunicationError::IOErrorI2CArbitrationLost);
                        }
                    }
                    _ => {}
                }
            }
            Err(error) => {
                return match error {
                    IowkitError::IOErrorIOWarrior => Err(I2CCommunicationError::IOErrorIOWarrior),
                }
            }
        };

        Ok(())
    }
}

impl Drop for I2C {
    fn drop(&mut self) {
        let result = self.data.enable_i2c(false);

        let mut mut_data = self.mut_data_refcell.borrow_mut();

        mut_data.i2c_struct_existing = false;
        mut_data.i2c_hardware_enabled = match result {
            Ok(_) => false,
            Err(_) => true,
        };
    }
}

#[derive(Debug, Clone, Copy)]
pub struct I2CAddressStruct;

impl Predicate<u8> for I2CAddressStruct {
    fn test(address: &u8) -> bool {
        match address {
            0x00 => false, // Reserved as a general call address
            _ if (0x01..=0x7F).contains(address) && address % 2 == 1 => true, // Valid 7-bit I2C addresses are numbers between 0x01 and 0x7F
            _ => false, // Invalid addresses or even numbers
        }
    }
}

pub type I2CAddress = Refinement<u8, I2CAddressStruct>;
