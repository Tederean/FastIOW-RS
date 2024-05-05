use crate::bits::Bit::{Bit1, Bit2, Bit3, Bit6, Bit7};
use crate::bits::Bitmasking;
use crate::communication::communication_service;
use crate::iowarrior::{
    peripheral_service, IOWarriorData, IOWarriorMutData, Peripheral, PeripheralSetupError, Pipe,
    Report, ReportId,
};
use crate::spi::spi_data::{IOWarriorSPIType, SPIData};
use crate::spi::{SPIConfig, SPIError, SPIMode};
use crate::{iowarrior::IOWarriorType, pin};
use hidapi::HidError;
use std::cell::RefMut;
use std::cmp::Ordering;
use std::iter;
use std::rc::Rc;

pub fn enable_spi(
    data: &IOWarriorData,
    mut_data: &mut RefMut<IOWarriorMutData>,
    spi_data: &SPIData,
    spi_pins: &Vec<u8>,
) -> Result<(), PeripheralSetupError> {
    peripheral_service::precheck_peripheral(&data, mut_data, Peripheral::SPI, &spi_pins)?;

    let result = send_enable_spi(&data, &spi_data);

    peripheral_service::post_enable(mut_data, &spi_pins, Peripheral::SPI, result)
}

fn send_enable_spi(data: &IOWarriorData, spi_data: &SPIData) -> Result<(), HidError> {
    let mut report = data.create_report(Pipe::SpecialMode);

    report.buffer[0] = ReportId::SpiSetup.get_value();
    report.buffer[1] = 0x01;

    match spi_data.spi_type {
        IOWarriorSPIType::IOWarrior24 => {
            report.buffer[2] = {
                let mut mode = spi_data.iow24_mode;

                mode.set_bit(
                    Bit2,
                    match spi_data.spi_config.mode {
                        // Yeah, CPHA is indeed inverted here...
                        SPIMode::Mode0 | SPIMode::Mode2 => true, // CPHA 0
                        SPIMode::Mode1 | SPIMode::Mode3 => false, // CPHA 1
                    },
                );

                mode.set_bit(
                    Bit3,
                    match spi_data.spi_config.mode {
                        SPIMode::Mode0 | SPIMode::Mode1 => false, // CPOL 0
                        SPIMode::Mode2 | SPIMode::Mode3 => true,  // CPOL 1
                    },
                );

                mode
            };
        }
        IOWarriorSPIType::IOWarrior56 => {
            report.buffer[2] = {
                let mut mode = spi_data.iow24_mode;

                mode.set_bit(
                    Bit2,
                    match spi_data.spi_config.mode {
                        SPIMode::Mode0 | SPIMode::Mode2 => false, // CPHA 0
                        SPIMode::Mode1 | SPIMode::Mode3 => true,  // CPHA 1
                    },
                );

                mode.set_bit(
                    Bit1,
                    match spi_data.spi_config.mode {
                        SPIMode::Mode0 | SPIMode::Mode1 => false, // CPOL 0
                        SPIMode::Mode2 | SPIMode::Mode3 => true,  // CPOL 1
                    },
                );

                mode.set_bit(Bit7, false); // MSB first

                mode
            };

            report.buffer[3] = spi_data.iow56_clock_divider;
        }
    }

    communication_service::write_report(&data, &mut report)
}

pub fn calculate_spi_data(spi_type: IOWarriorSPIType, spi_config: SPIConfig) -> SPIData {
    let mut data = SPIData {
        spi_type,
        spi_config,
        calculated_frequency_hz: u32::MAX,
        iow24_mode: 0,
        iow56_clock_divider: 0,
    };

    match spi_type {
        IOWarriorSPIType::IOWarrior24 => calculate_iow24_data(&mut data),
        IOWarriorSPIType::IOWarrior56 => calculate_iow56_data(&mut data),
    }

    data
}

fn calculate_iow24_data(spi_data: &mut SPIData) {
    for (index, value) in [2_000_000u32, 1_000_000u32, 500_000u32, 62_500u32]
        .iter()
        .enumerate()
    {
        if spi_data
            .spi_config
            .requested_frequency_hz
            .abs_diff(value.clone())
            < spi_data
                .spi_config
                .requested_frequency_hz
                .abs_diff(spi_data.calculated_frequency_hz)
        {
            spi_data.calculated_frequency_hz = value.clone();
            spi_data.iow24_mode = index as u8;
        }
    }
}

fn calculate_iow56_data(spi_data: &mut SPIData) {
    let requested_frequency_hz = std::cmp::max(1, spi_data.spi_config.requested_frequency_hz);

    spi_data.iow56_clock_divider = {
        let mut clock_divider = (24_000_000 / requested_frequency_hz) - 1u32;

        clock_divider = std::cmp::max(clock_divider, 2);
        clock_divider = std::cmp::min(clock_divider, 255);
        clock_divider as u8
    };

    spi_data.calculated_frequency_hz = 24_000_000 / (spi_data.iow56_clock_divider as u32 + 1u32);
}

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

pub fn transfer_data_with_different_size(
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

    communication_service::write_report(&data, &report).map_err(|x| SPIError::ErrorUSB(x))
}

fn read_report(
    data: &Rc<IOWarriorData>,
    spi_data: &SPIData,
    read_chunk: &mut [u8],
) -> Result<(), SPIError> {
    let report = communication_service::read_report(&data, Pipe::SpecialMode)
        .map_err(|x| SPIError::ErrorUSB(x))?;

    assert_eq!(report.buffer[0], ReportId::SpiTransfer.get_value());

    match read_chunk.len() as u8 == report.buffer[1] {
        true => {
            read_chunk.copy_from_slice(&report.buffer[1..(read_chunk.len() + 1)]);
            Ok(())
        }
        false => Err(SPIError::IOErrorSPI),
    }
}

pub fn get_spi_type(data: &Rc<IOWarriorData>) -> Option<IOWarriorSPIType> {
    match data.communication_data.device_type {
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
