use crate::bits::Bit::{Bit0, Bit6, Bit7};
use crate::bits::Bitmasking;
use crate::iowkit::{
    IOWarriorData, IOWarriorMutData, IowkitError, Pipe, Report, ReportId, UsedPin,
};
use crate::{IOWarriorType, Module};
use std::cell::RefCell;
use std::rc::Rc;
use std::{fmt, iter};
use thiserror::Error;

#[non_exhaustive]
#[derive(Error, Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum I2CCommunicationError {
    #[error("IOWarrior input output error.")]
    IOErrorIOWarrior,
    #[error("I2C input output error.")]
    IOErrorI2C,
    #[error("I2C input output error, arbitration lost.")]
    IOErrorI2CArbitrationLost,
}

#[non_exhaustive]
#[derive(Error, Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum I2CEnableError {
    #[error("IOWarrior input output error.")]
    IOErrorIOWarrior,
    #[error("Module already enabled.")]
    AlreadyEnabled,
    #[error("Hardware is blocked by {0} module.")]
    HardwareBlockedByModule(Module),
}

#[derive(Debug)]
pub struct I2C {
    pub(crate) data: Rc<IOWarriorData>,
    pub(crate) mut_data_refcell: Rc<RefCell<IOWarriorMutData>>,
}

impl I2C {
    pub(crate) fn new(
        data: &Rc<IOWarriorData>,
        mut_data_refcell: &Rc<RefCell<IOWarriorMutData>>,
    ) -> Result<I2C, I2CEnableError> {
        {
            let mut mut_data = mut_data_refcell.borrow_mut();

            match mut_data
                .pins_in_use
                .iter()
                .filter(|&x| x.module == Module::I2C)
                .next()
            {
                None => {}
                Some(_) => return Err(I2CEnableError::AlreadyEnabled),
            };

            match mut_data
                .pins_in_use
                .iter()
                .filter(|&x| x.pin == data.i2c_pins[0] || x.pin == data.i2c_pins[1])
                .next()
            {
                None => {}
                Some(conflict) => {
                    return Err(I2CEnableError::HardwareBlockedByModule(conflict.module))
                }
            };

            if !data.cleanup_dangling_modules(&mut mut_data) {
                return Err(I2CEnableError::IOErrorIOWarrior);
            }

            match data.enable_i2c(true) {
                Ok(_) => {
                    mut_data.pins_in_use.push(UsedPin {
                        pin: data.i2c_pins[0],
                        module: Module::I2C,
                    });
                    mut_data.pins_in_use.push(UsedPin {
                        pin: data.i2c_pins[1],
                        module: Module::I2C,
                    });
                }
                Err(error) => {
                    return match error {
                        IowkitError::IOErrorIOWarrior => Err(I2CEnableError::IOErrorIOWarrior),
                    };
                }
            }
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
        let chunk_iterator = buffer.chunks(self.data.special_report_size - 3);
        let chunk_iterator_count = chunk_iterator.len();
        let report_id = ReportId::I2cWrite;

        let mut report = Report {
            buffer: Vec::with_capacity(self.data.special_report_size),
            pipe: self.data.i2c_pipe,
        };

        for (index, chunk) in chunk_iterator.enumerate() {
            let start_byte = index == 0;
            let stop_byte = index == chunk_iterator_count - 1;

            report.buffer.clear();

            report.buffer.push(report_id.get_value());

            report.buffer.push({
                let mut value = (chunk.len() + 1) as u8;

                value.set_bit(Bit6, stop_byte);
                value.set_bit(Bit7, start_byte);

                value
            });

            report.buffer.push({
                let mut value = address.to_inner() << 1;

                value.set_bit(Bit0, false); // Write address

                value
            });

            report.buffer.extend(chunk);
            report.buffer.extend(
                iter::repeat(0u8).take(self.data.special_report_size - report.buffer.len()),
            );

            self.write_report(&report)?;
        }

        _ = self.read_report(self.data.i2c_pipe, report_id)?;

        Ok(())
    }

    pub fn read_data(
        &self,
        address: &I2CAddress,
        buffer: &mut [u8],
    ) -> Result<(), I2CCommunicationError> {
        let chunk_iterator = buffer.chunks_mut(self.data.special_report_size - 2);
        let report_id = ReportId::I2cRead;

        for chunk in chunk_iterator {
            let chunk_length = chunk.len() as u8;

            {
                let mut report = self.data.create_report(self.data.i2c_pipe);

                report.buffer[0] = report_id.get_value();
                report.buffer[1] = chunk_length;

                report.buffer[2] = {
                    let mut value = address.to_inner() << 1;

                    value.set_bit(Bit0, true); // Read address

                    value
                };

                self.write_report(&report)?;
            }

            {
                let report = self.read_report(self.data.i2c_pipe, report_id)?;

                chunk.copy_from_slice(&report.buffer[2..((chunk_length + 2) as usize)]);
            }
        }

        Ok(())
    }

    fn write_report(&self, report: &Report) -> Result<(), I2CCommunicationError> {
        match self.data.write_report(&report) {
            Ok(_) => Ok(()),
            Err(error) => {
                return match error {
                    IowkitError::IOErrorIOWarrior => Err(I2CCommunicationError::IOErrorIOWarrior),
                }
            }
        }
    }

    fn read_report(
        &self,
        pipe: Pipe,
        report_id: ReportId,
    ) -> Result<Report, I2CCommunicationError> {
        match self.data.read_report(pipe) {
            Ok(report) => {
                assert_eq!(report.buffer[0], report_id.get_value());

                if report.buffer[1].get_bit(Bit7) {
                    return Err(I2CCommunicationError::IOErrorI2C);
                }

                match self.data.device_type {
                    IOWarriorType::IOWarrior28
                    | IOWarriorType::IOWarrior28Dongle
                    | IOWarriorType::IOWarrior56
                    | IOWarriorType::IOWarrior56Dongle
                    | IOWarriorType::IOWarrior56Old => {
                        if report.buffer[1].get_bit(Bit7) {
                            return Err(I2CCommunicationError::IOErrorI2CArbitrationLost);
                        }
                    }
                    _ => {}
                }

                Ok(report)
            }
            Err(error) => {
                return match error {
                    IowkitError::IOErrorIOWarrior => Err(I2CCommunicationError::IOErrorIOWarrior),
                }
            }
        }
    }
}

impl Drop for I2C {
    fn drop(&mut self) {
        let mut mut_data = self.mut_data_refcell.borrow_mut();

        match self.data.enable_i2c(false) {
            Ok(_) => {}
            Err(_) => mut_data.dangling_modules.push(Module::I2C),
        }

        mut_data.pins_in_use.retain(|x| x.module == Module::I2C);
    }
}

impl fmt::Display for I2C {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[non_exhaustive]
#[derive(Error, Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum I2CAddressError {
    #[error("Address is to large for a valid 7 bit I2C address.")]
    NotA7BitAddress,
    #[error("Reserved I2C addresses are not allowed.")]
    ReservedAddress,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct I2CAddress {
    address: u8,
}

impl I2CAddress {
    pub const fn new(address: u8) -> Result<I2CAddress, I2CAddressError> {
        if address > 127 {
            return Err(I2CAddressError::NotA7BitAddress);
        }

        match address > 0 && !(address >= 0x78 && address <= 0x7F) {
            true => Ok(I2CAddress { address }),
            false => Err(I2CAddressError::ReservedAddress),
        }
    }

    pub const fn to_inner(&self) -> u8 {
        self.address
    }
}

impl fmt::Display for I2CAddress {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}
