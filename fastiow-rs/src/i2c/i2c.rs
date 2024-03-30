use crate::bits::Bit::{Bit0, Bit6, Bit7};
use crate::bits::Bitmasking;
use crate::internal::{
    create_report, disable_peripheral, enable_peripheral, read_report, write_report, IOWarriorData,
    IOWarriorMutData, IowkitError, Pipe, Report, ReportId,
};
use crate::{I2CError, IOWarriorType, Peripheral, PeripheralSetupError};
use std::cell::RefCell;
use std::rc::Rc;
use std::{fmt, iter};

#[derive(Debug)]
pub struct I2C {
    data: Rc<IOWarriorData>,
    mut_data_refcell: Rc<RefCell<IOWarriorMutData>>,
}

impl fmt::Display for I2C {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Drop for I2C {
    fn drop(&mut self) {
        let mut mut_data = self.mut_data_refcell.borrow_mut();

        disable_peripheral(&self.data, &mut mut_data, Peripheral::I2C);
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
        self.check_valid_7bit_address(address)?;

        for operation in operations {
            match operation {
                embedded_hal::i2c::Operation::Read(buffer) => {
                    self.read_data(address, buffer)?;
                }
                embedded_hal::i2c::Operation::Write(buffer) => {
                    self.write_data(address, buffer)?;
                }
            }
        }

        Ok(())
    }
}

impl I2C {
    pub(crate) fn new(
        data: &Rc<IOWarriorData>,
        mut_data_refcell: &Rc<RefCell<IOWarriorMutData>>,
    ) -> Result<I2C, PeripheralSetupError> {
        {
            let mut mut_data = mut_data_refcell.borrow_mut();

            enable_peripheral(&data, &mut mut_data, Peripheral::I2C)?;
        }

        Ok(I2C {
            data: data.clone(),
            mut_data_refcell: mut_data_refcell.clone(),
        })
    }

    fn check_valid_7bit_address(&self, address: u8) -> Result<(), I2CError> {
        if address > 127 {
            return Err(I2CError::InvalidI2CAddress);
        }

        match address > 0 && !(address >= 0x78 && address <= 0x7F) {
            true => Ok(()),
            false => Err(I2CError::InvalidI2CAddress),
        }
    }

    fn write_data(&self, address: u8, buffer: &[u8]) -> Result<(), I2CError> {
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
                let mut value = address << 1;

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

    fn read_data(&self, address: u8, buffer: &mut [u8]) -> Result<(), I2CError> {
        let chunk_iterator = buffer.chunks_mut(self.data.special_report_size - 2);
        let report_id = ReportId::I2cRead;

        for chunk in chunk_iterator {
            let chunk_length = chunk.len() as u8;

            {
                let mut report = create_report(&self.data, self.data.i2c_pipe);

                report.buffer[0] = report_id.get_value();
                report.buffer[1] = chunk_length;

                report.buffer[2] = {
                    let mut value = address << 1;

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

    fn write_report(&self, report: &Report) -> Result<(), I2CError> {
        match write_report(&self.data, &report) {
            Ok(_) => Ok(()),
            Err(error) => {
                return match error {
                    IowkitError::IOErrorIOWarrior => Err(I2CError::IOErrorIOWarrior),
                }
            }
        }
    }

    fn read_report(&self, pipe: Pipe, report_id: ReportId) -> Result<Report, I2CError> {
        match read_report(&self.data, pipe) {
            Ok(report) => {
                assert_eq!(report.buffer[0], report_id.get_value());

                if report.buffer[1].get_bit(Bit7) {
                    return Err(I2CError::IOErrorI2C);
                }

                match self.data.device_type {
                    IOWarriorType::IOWarrior28
                    | IOWarriorType::IOWarrior28Dongle
                    | IOWarriorType::IOWarrior56
                    | IOWarriorType::IOWarrior56Dongle
                    | IOWarriorType::IOWarrior56Old => {
                        if report.buffer[1].get_bit(Bit7) {
                            return Err(I2CError::IOErrorI2CArbitrationLoss);
                        }
                    }
                    _ => {}
                }

                Ok(report)
            }
            Err(error) => {
                return match error {
                    IowkitError::IOErrorIOWarrior => Err(I2CError::IOErrorIOWarrior),
                }
            }
        }
    }
}
