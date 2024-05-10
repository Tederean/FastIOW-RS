use crate::bits::Bit::{Bit0, Bit6, Bit7};
use crate::bits::Bitmasking;
use crate::communication::communication_service;
use crate::i2c::{I2CConfig, I2CError};
use crate::iowarrior::{
    peripheral_service, IOWarriorMutData, IOWarriorType, Peripheral, PeripheralSetupError,
};
use crate::iowarrior::{IOWarriorData, Report, ReportId};
use hidapi::HidError;
use std::cell::RefMut;
use std::iter;
use std::rc::Rc;

pub fn enable_i2c(
    data: &IOWarriorData,
    mut_data: &mut RefMut<IOWarriorMutData>,
    i2c_config: I2CConfig,
) -> Result<(), PeripheralSetupError> {
    peripheral_service::precheck_peripheral(&data, mut_data, Peripheral::I2C, &data.i2c_pins)?;

    send_enable_i2c(&data, &i2c_config).map_err(|x| PeripheralSetupError::ErrorUSB(x))?;

    peripheral_service::post_enable(mut_data, &data.i2c_pins, Peripheral::I2C);
    Ok(())
}

fn send_enable_i2c(data: &IOWarriorData, i2c_config: &I2CConfig) -> Result<(), HidError> {
    let mut report = data.create_report(data.i2c_pipe);

    report.buffer[0] = ReportId::I2cSetup.get_value();
    report.buffer[1] = 0x01;

    match data.communication_data.device_type {
        IOWarriorType::IOWarrior56 | IOWarriorType::IOWarrior56Dongle => {
            report.buffer[2] = i2c_config.iow56_clock.get_value();
        }
        IOWarriorType::IOWarrior100 => {
            report.buffer[2] = i2c_config.iow100_speed.get_value();
        }
        IOWarriorType::IOWarrior40
        | IOWarriorType::IOWarrior24
        | IOWarriorType::IOWarrior24PowerVampire
        | IOWarriorType::IOWarrior28
        | IOWarriorType::IOWarrior28Dongle
        | IOWarriorType::IOWarrior28L => {}
    }

    communication_service::write_report(&data, &mut report)
}

pub fn check_valid_7bit_address(address: u8) -> Result<(), I2CError> {
    if address > 127 {
        return Err(I2CError::InvalidI2CAddress);
    }

    match address > 0 && !(address >= 0x78 && address <= 0x7F) {
        true => Ok(()),
        false => Err(I2CError::InvalidI2CAddress),
    }
}

pub fn write_data(data: &Rc<IOWarriorData>, address: u8, buffer: &[u8]) -> Result<(), I2CError> {
    let chunk_iterator = buffer.chunks(data.special_report_size - 3);
    let chunk_iterator_count = chunk_iterator.len();
    let report_id = ReportId::I2cWrite;

    let mut report = Report {
        buffer: Vec::with_capacity(data.special_report_size),
        pipe: data.i2c_pipe,
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
        report
            .buffer
            .extend(iter::repeat(0u8).take(data.special_report_size - report.buffer.len()));

        write_report(data, &report)?;
    }

    _ = read_report(data, report_id)?;

    Ok(())
}

pub fn read_data(data: &IOWarriorData, address: u8, buffer: &mut [u8]) -> Result<(), I2CError> {
    let chunk_iterator = buffer.chunks_mut(data.special_report_size - 2);
    let report_id = ReportId::I2cRead;

    for chunk in chunk_iterator {
        let chunk_length = chunk.len() as u8;

        {
            let mut report = data.create_report(data.i2c_pipe);

            report.buffer[0] = report_id.get_value();
            report.buffer[1] = chunk_length;

            report.buffer[2] = {
                let mut value = address << 1;

                value.set_bit(Bit0, true); // Read address

                value
            };

            write_report(data, &report)?;
        }

        {
            let report = read_report(data, report_id)?;

            chunk.copy_from_slice(&report.buffer[2..((chunk_length + 2) as usize)]);
        }
    }

    Ok(())
}

fn write_report(data: &IOWarriorData, report: &Report) -> Result<(), I2CError> {
    communication_service::write_report(data, &report).map_err(|x| I2CError::ErrorUSB(x))
}

fn read_report(data: &IOWarriorData, report_id: ReportId) -> Result<Report, I2CError> {
    let report = communication_service::read_report(data, data.i2c_pipe)
        .map_err(|x| I2CError::ErrorUSB(x))?;

    assert_eq!(report.buffer[0], report_id.get_value());

    if report.buffer[1].get_bit(Bit7) {
        return Err(I2CError::IOErrorI2C);
    }

    match data.communication_data.device_type {
        IOWarriorType::IOWarrior28
        | IOWarriorType::IOWarrior28Dongle
        | IOWarriorType::IOWarrior56
        | IOWarriorType::IOWarrior56Dongle => {
            if report.buffer[1].get_bit(Bit7) {
                return Err(I2CError::IOErrorI2CArbitrationLoss);
            }
        }
        IOWarriorType::IOWarrior40
        | IOWarriorType::IOWarrior24
        | IOWarriorType::IOWarrior24PowerVampire
        | IOWarriorType::IOWarrior28L
        | IOWarriorType::IOWarrior100 => {}
    }

    Ok(report)
}
