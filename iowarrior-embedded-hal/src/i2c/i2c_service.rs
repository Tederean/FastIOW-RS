use crate::bits::Bit::{Bit0, Bit6, Bit7};
use crate::bits::Bitmasking;
use crate::communication::communication_service;
use crate::i2c::{I2CConfig, I2CError, I2C};
use crate::iowarrior::{
    peripheral_service, IOWarriorMutData, IOWarriorType, Peripheral, PeripheralSetupError,
};
use crate::iowarrior::{IOWarriorData, Report, ReportId};
use crate::pin;
use hidapi::HidError;
use std::cell::{RefCell, RefMut};
use std::iter;
use std::rc::Rc;

pub fn new(
    data: &Rc<IOWarriorData>,
    mut_data_refcell: &Rc<RefCell<IOWarriorMutData>>,
    i2c_config: I2CConfig,
) -> Result<I2C, PeripheralSetupError> {
    let mut mut_data = mut_data_refcell.borrow_mut();

    let i2c_pins = get_i2c_pins(data.device_type);

    peripheral_service::precheck_peripheral(&data, &mut mut_data, Peripheral::I2C, &i2c_pins)?;

    send_enable_i2c(data, &mut mut_data, &i2c_config, &i2c_pins)
        .map_err(|x| PeripheralSetupError::ErrorUSB(x))?;

    peripheral_service::post_enable(&mut mut_data, &i2c_pins, Peripheral::I2C);

    Ok(I2C {
        data: data.clone(),
        mut_data_refcell: mut_data_refcell.clone(),
        i2c_config,
    })
}

fn get_i2c_pins(device_type: IOWarriorType) -> Vec<u8> {
    match device_type {
        IOWarriorType::IOWarrior40 => vec![pin!(0, 6), pin!(0, 7)],
        IOWarriorType::IOWarrior24 | IOWarriorType::IOWarrior24PowerVampire => {
            vec![pin!(0, 1), pin!(0, 2)]
        }
        IOWarriorType::IOWarrior28 | IOWarriorType::IOWarrior28Dongle => {
            vec![pin!(2, 1), pin!(2, 0)]
        }
        IOWarriorType::IOWarrior28L => vec![pin!(0, 1), pin!(0, 2)],
        IOWarriorType::IOWarrior56 | IOWarriorType::IOWarrior56Dongle => {
            vec![pin!(1, 7), pin!(1, 5)]
        }
        IOWarriorType::IOWarrior100 => vec![pin!(10, 4), pin!(10, 5)],
    }
}

fn send_enable_i2c(
    data: &IOWarriorData,
    mut_data: &mut RefMut<IOWarriorMutData>,
    i2c_config: &I2CConfig,
    i2c_pins: &Vec<u8>,
) -> Result<(), HidError> {
    let mut report = data.create_report(data.i2c_pipe);

    report.buffer[0] = ReportId::I2cSetup.get_value();
    report.buffer[1] = 0x01;

    match data.device_type {
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

    communication_service::write_report(&mut mut_data.communication_data, &mut report)
}

pub fn write_data(
    data: &Rc<IOWarriorData>,
    mut_data: &mut RefMut<IOWarriorMutData>,
    address: u8,
    buffer: &[u8],
) -> Result<(), I2CError> {
    check_valid_7bit_address(address)?;

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

        communication_service::write_report(&mut mut_data.communication_data, &report)
            .map_err(|x| I2CError::ErrorUSB(x))?;
    }

    _ = read_report(data, mut_data, report_id)?;

    Ok(())
}

pub fn read_data(
    data: &IOWarriorData,
    mut_data: &mut RefMut<IOWarriorMutData>,
    address: u8,
    buffer: &mut [u8],
) -> Result<(), I2CError> {
    check_valid_7bit_address(address)?;

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

            communication_service::write_report(&mut mut_data.communication_data, &report)
                .map_err(|x| I2CError::ErrorUSB(x))?;
        }

        {
            let report = read_report(data, mut_data, report_id)?;

            chunk.copy_from_slice(&report.buffer[2..((chunk_length + 2) as usize)]);
        }
    }

    Ok(())
}

fn read_report(
    data: &IOWarriorData,
    mut_data: &mut RefMut<IOWarriorMutData>,
    report_id: ReportId,
) -> Result<Report, I2CError> {
    let report = communication_service::read_report(
        &mut mut_data.communication_data,
        data.create_report(data.i2c_pipe),
    )
    .map_err(|x| I2CError::ErrorUSB(x))?;

    assert_eq!(report.buffer[0], report_id.get_value());

    if report.buffer[1].get_bit(Bit7) {
        return Err(I2CError::NoAcknowledge);
    }

    if report_id == ReportId::I2cWrite {
        match data.device_type {
            IOWarriorType::IOWarrior28
            | IOWarriorType::IOWarrior28Dongle
            | IOWarriorType::IOWarrior100 => match report.buffer[2] {
                1 => return Err(I2CError::WrongAmountOfBytesRequested),
                2 => return Err(I2CError::TransactionWithoutStartRequested),
                3 => return Err(I2CError::NackReceived),
                4 => return Err(I2CError::BusError),
                _ => {}
            },
            IOWarriorType::IOWarrior40
            | IOWarriorType::IOWarrior24
            | IOWarriorType::IOWarrior24PowerVampire
            | IOWarriorType::IOWarrior28L
            | IOWarriorType::IOWarrior56
            | IOWarriorType::IOWarrior56Dongle => {}
        }
    }

    match data.device_type {
        IOWarriorType::IOWarrior28
        | IOWarriorType::IOWarrior28Dongle
        | IOWarriorType::IOWarrior56
        | IOWarriorType::IOWarrior56Dongle
        | IOWarriorType::IOWarrior100 => {
            if report.buffer[1].get_bit(Bit6) {
                return Err(I2CError::ArbitrationLoss);
            }
        }
        IOWarriorType::IOWarrior40
        | IOWarriorType::IOWarrior24
        | IOWarriorType::IOWarrior24PowerVampire
        | IOWarriorType::IOWarrior28L => {}
    }

    Ok(report)
}

fn check_valid_7bit_address(address: u8) -> Result<(), I2CError> {
    if address > 127 {
        return Err(I2CError::InvalidAddress);
    }

    match address > 0 && !(address >= 0x78 && address <= 0x7F) {
        true => Ok(()),
        false => Err(I2CError::InvalidAddress),
    }
}
