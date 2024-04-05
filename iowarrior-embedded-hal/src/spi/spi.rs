use crate::bits::Bit::{Bit6, Bit7};
use crate::bits::Bitmasking;
use crate::internal::{
    disable_peripheral, enable_spi, get_used_pins, read_report, write_report, IOWarriorData,
    IOWarriorMutData, IowkitError, Pipe, Report, ReportId,
};
use crate::spi::{calculate_spi_data, IOWarriorSPIType, SPIConfig, SPIData, SPIError};
use crate::{IOWarriorType, Peripheral, PeripheralSetupError};
use std::borrow::Borrow;
use std::cell::RefCell;
use std::cmp::Ordering;
use std::rc::Rc;
use std::{fmt, iter};

#[derive(Debug)]
pub struct SPI {
    data: Rc<IOWarriorData>,
    mut_data_refcell: Rc<RefCell<IOWarriorMutData>>,
    pub spi_data: SPIData,
}

impl fmt::Display for SPI {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Drop for SPI {
    fn drop(&mut self) {
        let mut mut_data = self.mut_data_refcell.borrow_mut();

        disable_peripheral(&self.data, &mut mut_data, Peripheral::SPI);
    }
}

impl embedded_hal::spi::ErrorType for SPI {
    type Error = SPIError;
}

impl embedded_hal::spi::SpiBus<u8> for SPI {
    fn read(&mut self, words: &mut [u8]) -> Result<(), Self::Error> {
        let chunk_size = self.get_chunk_size();

        let dummy_write_buffer = vec![self.spi_data.spi_config.dummy_value; chunk_size];

        let read_chunk_iterator = words.chunks_mut(chunk_size);

        let read_chunk_iterator_count = read_chunk_iterator.len();

        for (index, read_chunk) in read_chunk_iterator.enumerate() {
            let use_data_ready_pin = index == 0 && self.spi_data.spi_config.use_data_ready_pin;
            let chip_select_stays_active = index != (read_chunk_iterator_count - 1);

            self.write_report(
                &dummy_write_buffer[0..read_chunk.len()],
                use_data_ready_pin,
                chip_select_stays_active,
            )?;
            self.read_report(read_chunk)?;
        }

        Ok(())
    }

    fn write(&mut self, words: &[u8]) -> Result<(), Self::Error> {
        let chunk_size = self.get_chunk_size();

        let mut dummy_read_buffer = vec![self.spi_data.spi_config.dummy_value; chunk_size];

        let write_chunk_iterator = words.chunks(chunk_size);

        let write_chunk_iterator_count = write_chunk_iterator.len();

        for (index, write_chunk) in write_chunk_iterator.enumerate() {
            let use_data_ready_pin = index == 0 && self.spi_data.spi_config.use_data_ready_pin;
            let chip_select_stays_active = index != (write_chunk_iterator_count - 1);

            self.write_report(write_chunk, use_data_ready_pin, chip_select_stays_active)?;
            self.read_report(&mut dummy_read_buffer[0..write_chunk.len()])?;
        }

        Ok(())
    }

    fn transfer(&mut self, read: &mut [u8], write: &[u8]) -> Result<(), Self::Error> {
        match read.len().cmp(&write.len()) {
            Ordering::Less => {
                let mut fixed_read: Vec<u8> = Vec::with_capacity(write.len());

                fixed_read.extend(read.iter());
                fixed_read.extend(
                    iter::repeat(self.spi_data.spi_config.dummy_value)
                        .take(write.len() - read.len()),
                );

                self.transfer_with_same_size(fixed_read.as_mut_slice(), write)?;

                read.copy_from_slice(&fixed_read[0..read.len()]);
                Ok(())
            }
            Ordering::Equal => self.transfer_with_same_size(read, write),
            Ordering::Greater => {
                let mut fixed_write: Vec<u8> = Vec::with_capacity(read.len());

                fixed_write.extend(write);
                fixed_write.extend(
                    iter::repeat(self.spi_data.spi_config.dummy_value)
                        .take(read.len() - write.len()),
                );

                self.transfer_with_same_size(read, fixed_write.as_slice())
            }
        }
    }

    fn transfer_in_place(&mut self, words: &mut [u8]) -> Result<(), Self::Error> {
        let chunk_size = self.get_chunk_size();

        let chunk_iterator = words.chunks_mut(chunk_size);

        let chunk_iterator_count = chunk_iterator.len();

        for (index, chunk) in chunk_iterator.enumerate() {
            let use_data_ready_pin = index == 0 && self.spi_data.spi_config.use_data_ready_pin;
            let chip_select_stays_active = index != (chunk_iterator_count - 1);

            self.write_report(chunk, use_data_ready_pin, chip_select_stays_active)?;
            self.read_report(chunk)?;
        }

        Ok(())
    }

    fn flush(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
}

impl SPI {
    pub(crate) fn new(
        data: &Rc<IOWarriorData>,
        mut_data_refcell: &Rc<RefCell<IOWarriorMutData>>,
        spi_config: SPIConfig,
    ) -> Result<SPI, PeripheralSetupError> {
        match get_spi_type(&data) {
            None => Err(PeripheralSetupError::NotSupported),
            Some(spi_type) => {
                let mut mut_data = mut_data_refcell.borrow_mut();

                if spi_type == IOWarriorSPIType::IOWarrior56
                    && get_used_pins(&mut mut_data, Peripheral::PWM).len() > 1
                {
                    return Err(PeripheralSetupError::HardwareBlocked(Peripheral::PWM));
                }

                let spi_pins = get_spi_pins(spi_type);
                let spi_data = calculate_spi_data(spi_type, spi_config);

                enable_spi(&data, &mut mut_data, &spi_data, &spi_pins)?;

                Ok(SPI {
                    data: data.clone(),
                    mut_data_refcell: mut_data_refcell.clone(),
                    spi_data,
                })
            }
        }
    }

    fn transfer_with_same_size(&mut self, read: &mut [u8], write: &[u8]) -> Result<(), SPIError> {
        todo!()
    }

    fn get_chunk_size(&self) -> usize {
        self.data.special_report_size
            - match self.spi_data.spi_type {
                IOWarriorSPIType::IOWarrior24 => 1usize,
                IOWarriorSPIType::IOWarrior56 => 2usize,
            }
    }

    fn write_report(
        &self,
        write_chunk: &[u8],
        use_data_ready_pin: bool,
        chip_select_stays_active: bool,
    ) -> Result<(), SPIError> {
        let mut report = Report {
            buffer: Vec::with_capacity(self.data.special_report_size),
            pipe: Pipe::SpecialMode,
        };

        report.buffer.push(ReportId::SpiTransfer.get_value());

        match self.spi_data.spi_type {
            IOWarriorSPIType::IOWarrior24 => {
                report.buffer.push({
                    let mut value = write_chunk.len() as u8;

                    value.set_bit(Bit6, chip_select_stays_active);
                    value.set_bit(Bit7, use_data_ready_pin);

                    value
                });
            }
            IOWarriorSPIType::IOWarrior56 => {
                report.buffer.push(write_chunk.len() as u8);

                report.buffer.push({
                    let mut value = 0x00;

                    value.set_bit(Bit6, chip_select_stays_active);
                    value.set_bit(Bit7, use_data_ready_pin);

                    value
                });
            }
        }

        report.buffer.extend(write_chunk);
        report
            .buffer
            .extend(iter::repeat(0u8).take(self.data.special_report_size - report.buffer.len()));

        write_report(&self.data, &report).map_err(|error| match error {
            IowkitError::IOErrorIOWarrior => SPIError::IOErrorIOWarrior,
        })
    }

    fn read_report(&self, read_chunk: &mut [u8]) -> Result<(), SPIError> {
        match read_report(&self.data, Pipe::SpecialMode) {
            Ok(report) => {
                assert_eq!(report.buffer[0], ReportId::SpiTransfer.get_value());

                match read_chunk.len() as u8 == report.buffer[1] {
                    true => {
                        read_chunk.copy_from_slice(&report.buffer[1..(read_chunk.len() + 1)]);
                        Ok(())
                    }
                    false => Err(SPIError::IOErrorSPI),
                }
            }
            Err(error) => {
                return match error {
                    IowkitError::IOErrorIOWarrior => Err(SPIError::IOErrorIOWarrior),
                }
            }
        }
    }
}

fn get_spi_type(data: &Rc<IOWarriorData>) -> Option<IOWarriorSPIType> {
    match data.device_type {
        IOWarriorType::IOWarrior24 => Some(IOWarriorSPIType::IOWarrior24),
        IOWarriorType::IOWarrior56 | IOWarriorType::IOWarrior56Dongle => {
            Some(IOWarriorSPIType::IOWarrior56)
        }
        IOWarriorType::IOWarrior100
        | IOWarriorType::IOWarrior40
        | IOWarriorType::IOWarrior28
        | IOWarriorType::IOWarrior28Dongle
        | IOWarriorType::IOWarrior28L => None,
    }
}

fn get_spi_pins(spi_type: IOWarriorSPIType) -> Vec<u8> {
    match spi_type {
        IOWarriorSPIType::IOWarrior24 => {
            vec![0 * 8 + 3, 0 * 8 + 4, 0 * 8 + 5, 0 * 8 + 6, 0 * 8 + 7]
        }
        IOWarriorSPIType::IOWarrior56 => {
            vec![5 * 8 + 3, 5 * 8 + 1, 5 * 8 + 2, 5 * 8 + 4, 5 * 8 + 0]
        }
    }
}
