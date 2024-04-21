use crate::bits::Bit::{Bit6, Bit7};
use crate::bits::Bitmasking;
use crate::internal::{IOWarriorData, IowkitError, Pipe, Report, ReportId};
use crate::spi::spi_data::{IOWarriorSPIType, SPIData};
use crate::spi::SPIError;
use crate::{IOWarriorType, pin};
use std::cmp::Ordering;
use std::iter;
use std::rc::Rc;

pub fn read_data(
    data: &Rc<IOWarriorData>,
    spi_data: &SPIData,
    words: &mut [u8],
) -> Result<(), SPIError> {
    let chunk_size = get_chunk_size(&data, &spi_data);

    let dummy_write_buffer = vec![spi_data.spi_config.dummy_value; chunk_size];

    let read_chunk_iterator = words.chunks_mut(chunk_size);

    let read_chunk_iterator_count = read_chunk_iterator.len();

    for (index, read_chunk) in read_chunk_iterator.enumerate() {
        let use_data_ready_pin = index == 0 && spi_data.spi_config.use_data_ready_pin;
        let chip_select_stays_active = index != (read_chunk_iterator_count - 1);

        write_report(
            &data,
            &spi_data,
            &dummy_write_buffer[0..read_chunk.len()],
            use_data_ready_pin,
            chip_select_stays_active,
        )?;
        read_report(&data, &spi_data, read_chunk)?;
    }

    Ok(())
}

pub fn write_data(
    data: &Rc<IOWarriorData>,
    spi_data: &SPIData,
    words: &[u8],
) -> Result<(), SPIError> {
    let chunk_size = get_chunk_size(&data, &spi_data);

    let mut dummy_read_buffer = vec![spi_data.spi_config.dummy_value; chunk_size];

    let write_chunk_iterator = words.chunks(chunk_size);

    let write_chunk_iterator_count = write_chunk_iterator.len();

    for (index, write_chunk) in write_chunk_iterator.enumerate() {
        let use_data_ready_pin = index == 0 && spi_data.spi_config.use_data_ready_pin;
        let chip_select_stays_active = index != (write_chunk_iterator_count - 1);

        write_report(
            &data,
            &spi_data,
            write_chunk,
            use_data_ready_pin,
            chip_select_stays_active,
        )?;
        read_report(
            &data,
            &spi_data,
            &mut dummy_read_buffer[0..write_chunk.len()],
        )?;
    }

    Ok(())
}

pub fn transfer_data(
    data: &Rc<IOWarriorData>,
    spi_data: &SPIData,
    read: &mut [u8],
    write: &[u8],
) -> Result<(), SPIError> {
    match read.len().cmp(&write.len()) {
        Ordering::Less => {
            let mut fixed_read: Vec<u8> = Vec::with_capacity(write.len());

            fixed_read.extend(read.iter());
            fixed_read.extend(
                iter::repeat(spi_data.spi_config.dummy_value).take(write.len() - read.len()),
            );

            transfer_data_with_same_size(&data, &spi_data, fixed_read.as_mut_slice(), write)?;

            read.copy_from_slice(&fixed_read[0..read.len()]);
            Ok(())
        }
        Ordering::Equal => transfer_data_with_same_size(&data, &spi_data, read, write),
        Ordering::Greater => {
            let mut fixed_write: Vec<u8> = Vec::with_capacity(read.len());

            fixed_write.extend(write);
            fixed_write.extend(
                iter::repeat(spi_data.spi_config.dummy_value).take(read.len() - write.len()),
            );

            transfer_data_with_same_size(&data, &spi_data, read, fixed_write.as_slice())
        }
    }
}

pub fn transfer_data_with_same_size(
    data: &Rc<IOWarriorData>,
    spi_data: &SPIData,
    read: &mut [u8],
    write: &[u8],
) -> Result<(), SPIError> {
    let chunk_size = get_chunk_size(&data, &spi_data);

    let read_chunk_iterator = read.chunks_mut(chunk_size);
    let write_chunk_iterator = write.chunks(chunk_size);

    let write_chunk_iterator_count = write_chunk_iterator.len();

    for (index, (write, read)) in write_chunk_iterator.zip(read_chunk_iterator).enumerate() {
        let use_data_ready_pin = index == 0 && spi_data.spi_config.use_data_ready_pin;
        let chip_select_stays_active = index != (write_chunk_iterator_count - 1);

        write_report(
            &data,
            &spi_data,
            write,
            use_data_ready_pin,
            chip_select_stays_active,
        )?;
        read_report(&data, &spi_data, read)?;
    }

    Ok(())
}

pub fn transfer_data_in_place(
    data: &Rc<IOWarriorData>,
    spi_data: &SPIData,
    words: &mut [u8],
) -> Result<(), SPIError> {
    let chunk_size = get_chunk_size(&data, &spi_data);

    let chunk_iterator = words.chunks_mut(chunk_size);

    let chunk_iterator_count = chunk_iterator.len();

    for (index, chunk) in chunk_iterator.enumerate() {
        let use_data_ready_pin = index == 0 && spi_data.spi_config.use_data_ready_pin;
        let chip_select_stays_active = index != (chunk_iterator_count - 1);

        write_report(
            &data,
            &spi_data,
            chunk,
            use_data_ready_pin,
            chip_select_stays_active,
        )?;
        read_report(&data, &spi_data, chunk)?;
    }

    Ok(())
}

fn get_chunk_size(data: &Rc<IOWarriorData>, spi_data: &SPIData) -> usize {
    data.special_report_size
        - match spi_data.spi_type {
            IOWarriorSPIType::IOWarrior24 => 2usize,
            IOWarriorSPIType::IOWarrior56 => 3usize,
        }
}

fn write_report(
    data: &Rc<IOWarriorData>,
    spi_data: &SPIData,
    write_chunk: &[u8],
    use_data_ready_pin: bool,
    chip_select_stays_active: bool,
) -> Result<(), SPIError> {
    let mut report = Report {
        buffer: Vec::with_capacity(data.special_report_size),
        pipe: Pipe::SpecialMode,
    };

    report.buffer.push(ReportId::SpiTransfer.get_value());

    match spi_data.spi_type {
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
        .extend(iter::repeat(0u8).take(data.special_report_size - report.buffer.len()));

    crate::internal::write_report(&data, &report).map_err(|error| match error {
        IowkitError::IOErrorIOWarrior => SPIError::IOErrorIOWarrior,
    })
}

fn read_report(
    data: &Rc<IOWarriorData>,
    spi_data: &SPIData,
    read_chunk: &mut [u8],
) -> Result<(), SPIError> {
    match crate::internal::read_report(&data, Pipe::SpecialMode) {
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

pub fn get_spi_type(data: &Rc<IOWarriorData>) -> Option<IOWarriorSPIType> {
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

pub fn get_spi_pins(spi_type: IOWarriorSPIType) -> Vec<u8> {
    match spi_type {
        IOWarriorSPIType::IOWarrior24 => {
            vec![pin!(0, 3), pin!(0, 4), pin!(0, 5), pin!(0, 6), pin!(0, 7)]
        }
        IOWarriorSPIType::IOWarrior56 => {
            vec![pin!(5, 3), pin!(5, 1), pin!(5, 2), pin!(5, 4), pin!(5, 0)]
        }
    }
}
