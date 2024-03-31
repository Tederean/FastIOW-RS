use crate::internal::{
    IOWarriorData, IOWarriorMutData, IowkitError, PinType, Pipe, Report, ReportId, UsedPin,
};
use crate::{I2CMode, IOWarriorType, Peripheral, PeripheralSetupError, PinSetupError};
use iowkit_sys::bindings;
use std::cell::RefMut;

static_assertions::assert_eq_size!(*mut u8, bindings::PCHAR);

pub fn create_report(data: &IOWarriorData, pipe: Pipe) -> Report {
    Report {
        buffer: match pipe {
            Pipe::IOPins => {
                vec![0u8; data.standard_report_size]
            }
            _ => {
                vec![0u8; data.special_report_size]
            }
        },
        pipe,
    }
}

pub fn write_report(data: &IOWarriorData, report: &Report) -> Result<(), IowkitError> {
    let written_bytes = unsafe {
        data.iowkit_data.iowkit.IowKitWrite(
            data.device_handle,
            report.pipe.get_value() as bindings::ULONG,
            report.buffer.as_ptr() as bindings::PCHAR,
            report.buffer.len() as bindings::ULONG,
        )
    } as usize;

    if written_bytes != report.buffer.len() {
        return Err(IowkitError::IOErrorIOWarrior);
    }

    Ok(())
}

pub fn read_report_non_blocking(data: &IOWarriorData, pipe: Pipe) -> Option<Report> {
    let mut report = create_report(&data, pipe);

    let read_bytes = unsafe {
        data.iowkit_data.iowkit.IowKitReadNonBlocking(
            data.device_handle,
            report.pipe.get_value() as bindings::ULONG,
            report.buffer.as_mut_ptr() as bindings::PCHAR,
            report.buffer.len() as bindings::ULONG,
        )
    } as usize;

    if read_bytes != report.buffer.len() {
        return None;
    }

    Some(report)
}

pub fn read_report(data: &IOWarriorData, pipe: Pipe) -> Result<Report, IowkitError> {
    let mut report = create_report(&data, pipe);

    let read_bytes = unsafe {
        data.iowkit_data.iowkit.IowKitRead(
            data.device_handle,
            report.pipe.get_value() as bindings::ULONG,
            report.buffer.as_mut_ptr() as bindings::PCHAR,
            report.buffer.len() as bindings::ULONG,
        )
    } as usize;

    if read_bytes != report.buffer.len() {
        return Err(IowkitError::IOErrorIOWarrior);
    }

    Ok(report)
}

fn precheck_peripheral(
    data: &IOWarriorData,
    mut_data: &mut RefMut<IOWarriorMutData>,
    peripheral: Peripheral,
    required_pins: &Vec<u8>,
) -> Result<(), PeripheralSetupError> {
    match mut_data
        .pins_in_use
        .iter()
        .filter(|x| x.peripheral == Some(peripheral))
        .next()
    {
        None => {}
        Some(_) => return Err(PeripheralSetupError::AlreadySetup),
    }

    match cleanup_dangling_modules(&data, mut_data) {
        true => {}
        false => return Err(PeripheralSetupError::IOErrorIOWarrior),
    }

    let pin_conflicts: Vec<_> = mut_data
        .pins_in_use
        .iter()
        .filter(|x| required_pins.iter().any(|pin| *pin == x.pin))
        .map(|x| x.pin.clone())
        .collect();

    if !pin_conflicts.is_empty() {
        return Err(PeripheralSetupError::PinsBlocked(pin_conflicts));
    }

    Ok(())
}

pub fn enable_i2c(
    data: &IOWarriorData,
    mut_data: &mut RefMut<IOWarriorMutData>,
    i2c_mode: I2CMode,
) -> Result<(), PeripheralSetupError> {
    precheck_peripheral(&data, mut_data, Peripheral::I2C, &data.i2c_pins)?;

    match send_enable_i2c(&data, i2c_mode) {
        Ok(_) => {
            mut_data
                .pins_in_use
                .extend(data.i2c_pins.iter().map(|pin| UsedPin {
                    peripheral: Some(Peripheral::I2C),
                    pin: pin.clone(),
                }));

            Ok(())
        }
        Err(error) => {
            return match error {
                IowkitError::IOErrorIOWarrior => Err(PeripheralSetupError::IOErrorIOWarrior),
            }
        }
    }
}

pub fn disable_peripheral(
    data: &IOWarriorData,
    mut_data: &mut RefMut<IOWarriorMutData>,
    peripheral: Peripheral,
) {
    match peripheral {
        Peripheral::I2C => match send_disable_i2c(&data) {
            Ok(_) => {
                mut_data
                    .pins_in_use
                    .retain(|x| x.peripheral != Some(Peripheral::I2C));
            }
            Err(_) => {
                mut_data.dangling_peripherals.push(Peripheral::I2C);
            }
        },
    }
}

fn cleanup_dangling_modules(data: &IOWarriorData, mut_data: &mut RefMut<IOWarriorMutData>) -> bool {
    if !mut_data.dangling_peripherals.is_empty() {
        for x in mut_data.dangling_peripherals.to_vec() {
            match x {
                Peripheral::I2C => match send_disable_i2c(&data) {
                    Ok(_) => mut_data.dangling_peripherals.retain(|y| *y != x),
                    Err(_) => {}
                },
            }
        }
    }

    mut_data.dangling_peripherals.is_empty()
}

pub fn enable_gpio(
    data: &IOWarriorData,
    mut_data: &mut RefMut<IOWarriorMutData>,
    pin_type: PinType,
    pin: u8,
) -> Result<(), PinSetupError> {
    if data.device_type == IOWarriorType::IOWarrior28Dongle
        || data.device_type == IOWarriorType::IOWarrior56Dongle
    {
        return Err(PinSetupError::NotSupported);
    }

    if !(data.is_valid_gpio)(pin) {
        return Err(PinSetupError::PinNotExisting);
    }

    match mut_data.pins_in_use.iter().filter(|x| x.pin == pin).next() {
        None => {}
        Some(used_pin) => {
            return Err(match used_pin.peripheral {
                None => PinSetupError::AlreadySetup,
                Some(Peripheral::I2C) => PinSetupError::BlockedByPeripheral(Peripheral::I2C),
            })
        }
    }

    match cleanup_dangling_modules(&data, mut_data) {
        true => {}
        false => return Err(PinSetupError::IOErrorIOWarrior),
    }

    mut_data.pins_in_use.push(UsedPin {
        pin,
        peripheral: None,
    });

    Ok(())
}

pub fn disable_gpio(
    data: &IOWarriorData,
    mut_data: &mut RefMut<IOWarriorMutData>,
    pin_type: PinType,
    pin: u8,
) {
    mut_data.pins_in_use.retain(|x| x.pin == pin);
}

fn send_enable_i2c(data: &IOWarriorData, i2c_mode: I2CMode) -> Result<(), IowkitError> {
    let mut report = create_report(&data, data.i2c_pipe);

    report.buffer[0] = ReportId::I2cSetup.get_value();
    report.buffer[1] = 0x01;

    match data.device_type {
        IOWarriorType::IOWarrior56
        | IOWarriorType::IOWarrior56Old
        | IOWarriorType::IOWarrior56Dongle => {
            match i2c_mode {
                I2CMode::Standard => {
                    report.buffer[2] = 0x00; // 93.75kHz
                }
                I2CMode::Fast | I2CMode::FastPlus => {
                    report.buffer[2] = 0x01; // 375kHz
                }
            }
        }
        IOWarriorType::IOWarrior100 => {
            match i2c_mode {
                I2CMode::Standard => {
                    report.buffer[2] = 0x00; // 100 kbit/s
                }
                I2CMode::Fast => {
                    report.buffer[2] = 0x01; // 400 kbit/s
                }
                I2CMode::FastPlus => {
                    report.buffer[2] = 0x04; // 1000 kbit/s
                }
            }
        }
        _ => {}
    }

    write_report(&data, &mut report)
}

fn send_disable_i2c(data: &IOWarriorData) -> Result<(), IowkitError> {
    let mut report = create_report(&data, data.i2c_pipe);

    report.buffer[0] = ReportId::I2cSetup.get_value();
    report.buffer[1] = 0x00;

    write_report(&data, &mut report)
}
