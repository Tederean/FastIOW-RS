use crate::bits::Bit;
use crate::bits::Bit::{Bit1, Bit2, Bit3, Bit7};
use crate::bits::Bitmasking;
use crate::digital::PinSetupError;
use crate::i2c::I2CConfig;
use crate::internal::{
    IOWarriorData, IOWarriorMutData, IowkitError, Pipe, Report, ReportId, UsedPin,
};
use crate::pwm::{ChannelMode, IOWarriorPWMType, PWMData};
use crate::spi::{IOWarriorSPIType, SPIData};
use crate::{IOWarriorType, Peripheral, PeripheralSetupError};
use embedded_hal::digital::PinState;
use embedded_hal::spi::{Phase, Polarity};
use std::cell::RefMut;

static_assertions::assert_eq_size!(u8, std::os::raw::c_char);

pub fn create_report(data: &IOWarriorData, pipe: Pipe) -> Report {
    Report {
        buffer: match pipe {
            Pipe::IOPins => {
                vec![0u8; data.standard_report_size]
            }

            Pipe::SpecialMode | Pipe::I2CMode | Pipe::ADCMode => {
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
            report.pipe.get_value() as iowkit_sys::ULONG,
            report.buffer.as_ptr() as iowkit_sys::PCHAR,
            report.buffer.len() as iowkit_sys::ULONG,
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
            report.pipe.get_value() as iowkit_sys::ULONG,
            report.buffer.as_mut_ptr() as iowkit_sys::PCHAR,
            report.buffer.len() as iowkit_sys::ULONG,
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
            report.pipe.get_value() as iowkit_sys::ULONG,
            report.buffer.as_mut_ptr() as iowkit_sys::PCHAR,
            report.buffer.len() as iowkit_sys::ULONG,
        )
    } as usize;

    if read_bytes != report.buffer.len() {
        return Err(IowkitError::IOErrorIOWarrior);
    }

    Ok(report)
}

pub fn get_used_pins(
    mut_data: &mut RefMut<IOWarriorMutData>,
    peripheral: Peripheral,
) -> Vec<UsedPin> {
    mut_data
        .pins_in_use
        .iter()
        .filter(|x| x.peripheral == Some(Peripheral::SPI))
        .map(|x| x.clone())
        .collect()
}

pub fn enable_spi(
    data: &IOWarriorData,
    mut_data: &mut RefMut<IOWarriorMutData>,
    spi_data: &SPIData,
    spi_pins: &Vec<u8>,
) -> Result<(), PeripheralSetupError> {
    precheck_peripheral(&data, mut_data, Peripheral::SPI, &spi_pins)?;

    let result = send_enable_spi(&data, &spi_data);

    post_enable(mut_data, &spi_pins, Peripheral::SPI, result)
}

pub fn enable_pwm(
    data: &IOWarriorData,
    mut_data: &mut RefMut<IOWarriorMutData>,
    pwm_data: &PWMData,
    pwm_pins: &Vec<u8>,
) -> Result<(), PeripheralSetupError> {
    precheck_peripheral(&data, mut_data, Peripheral::PWM, &pwm_pins)?;

    let result = send_enable_pwm(&data, pwm_data);

    post_enable(mut_data, pwm_pins, Peripheral::PWM, result)
}

pub fn enable_i2c(
    data: &IOWarriorData,
    mut_data: &mut RefMut<IOWarriorMutData>,
    i2c_config: I2CConfig,
) -> Result<(), PeripheralSetupError> {
    precheck_peripheral(&data, mut_data, Peripheral::I2C, &data.i2c_pins)?;

    let result = send_enable_i2c(&data, &i2c_config);

    post_enable(mut_data, &data.i2c_pins, Peripheral::I2C, result)
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

fn post_enable(
    mut_data: &mut RefMut<IOWarriorMutData>,
    peripheral_pins: &Vec<u8>,
    peripheral: Peripheral,
    enable_result: Result<(), IowkitError>,
) -> Result<(), PeripheralSetupError> {
    match enable_result {
        Ok(_) => {
            mut_data
                .pins_in_use
                .extend(peripheral_pins.iter().map(|pin| UsedPin {
                    peripheral: Some(peripheral),
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
    match match peripheral {
        Peripheral::I2C => send_disable_i2c(&data),
        Peripheral::PWM => send_disable_pwm(&data),
        Peripheral::SPI => send_disable_spi(&data),
    } {
        Ok(_) => {
            mut_data
                .pins_in_use
                .retain(|x| x.peripheral != Some(peripheral));
        }
        Err(_) => {
            mut_data.dangling_peripherals.push(peripheral);
        }
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
                Peripheral::PWM => match send_disable_pwm(&data) {
                    Ok(_) => mut_data.dangling_peripherals.retain(|y| *y != x),
                    Err(_) => {}
                },
                Peripheral::SPI => match send_disable_spi(&data) {
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
    pin_state: PinState,
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
                Some(peripheral) => PinSetupError::BlockedByPeripheral(peripheral),
            })
        }
    }

    match cleanup_dangling_modules(&data, mut_data) {
        true => {}
        false => return Err(PinSetupError::IOErrorIOWarrior),
    }

    match set_pin_output(&data, mut_data, pin_state, pin) {
        Ok(_) => {
            mut_data.pins_in_use.push(UsedPin {
                pin,
                peripheral: None,
            });

            Ok(())
        }
        Err(error) => Err(match error {
            IowkitError::IOErrorIOWarrior => PinSetupError::IOErrorIOWarrior,
        }),
    }
}

pub fn disable_gpio(data: &IOWarriorData, mut_data: &mut RefMut<IOWarriorMutData>, pin: u8) {
    match set_pin_output(&data, mut_data, PinState::High, pin) {
        Ok(_) => {}
        Err(_) => { /* Ignore error. Every following pin and peripheral can handle this. */ }
    };

    mut_data.pins_in_use.retain(|x| x.pin == pin);
}

pub fn set_pin_output(
    data: &IOWarriorData,
    mut_data: &mut RefMut<IOWarriorMutData>,
    pin_state: PinState,
    pin: u8,
) -> Result<(), IowkitError> {
    let byte_index = ((pin as usize) / 8usize) + 1;
    let bit_index = Bit::from_u8(pin % 8u8);

    let mut pins_write_report = mut_data.pins_write_report.clone();

    pins_write_report.buffer[byte_index].set_bit(bit_index, bool::from(pin_state));

    match write_report(&data, &pins_write_report) {
        Ok(_) => {
            mut_data.pins_write_report = pins_write_report;
            Ok(())
        }
        Err(error) => Err(error),
    }
}

fn send_enable_spi(data: &IOWarriorData, spi_data: &SPIData) -> Result<(), IowkitError> {
    let mut report = create_report(&data, data.i2c_pipe);

    report.buffer[0] = ReportId::SpiSetup.get_value();
    report.buffer[1] = 0x01;

    match spi_data.spi_type {
        IOWarriorSPIType::IOWarrior24 => {
            report.buffer[2] = {
                let mut mode = spi_data.iow24_mode;

                mode.set_bit(
                    Bit2,
                    match spi_data.spi_config.phase {
                        Phase::CaptureOnFirstTransition => true,
                        Phase::CaptureOnSecondTransition => false,
                    },
                );

                mode.set_bit(
                    Bit3,
                    match spi_data.spi_config.polarity {
                        Polarity::IdleLow => false,
                        Polarity::IdleHigh => true,
                    },
                );

                mode
            };
        }
        IOWarriorSPIType::IOWarrior56 => {
            report.buffer[2] = {
                let mut mode = spi_data.iow24_mode;

                mode.set_bit(
                    Bit1,
                    match spi_data.spi_config.polarity {
                        Polarity::IdleLow => false,
                        Polarity::IdleHigh => true,
                    },
                );

                mode.set_bit(
                    Bit2,
                    match spi_data.spi_config.phase {
                        Phase::CaptureOnFirstTransition => false,
                        Phase::CaptureOnSecondTransition => true,
                    },
                );

                mode.set_bit(Bit7, false); // MSB first

                mode
            };

            report.buffer[3] = spi_data.iow56_clock_divider;
        }
    }

    write_report(&data, &mut report)
}

fn send_enable_i2c(data: &IOWarriorData, i2c_config: &I2CConfig) -> Result<(), IowkitError> {
    let mut report = create_report(&data, data.i2c_pipe);

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
        | IOWarriorType::IOWarrior28
        | IOWarriorType::IOWarrior28Dongle
        | IOWarriorType::IOWarrior28L => {}
    }

    write_report(&data, &mut report)
}

fn send_disable_i2c(data: &IOWarriorData) -> Result<(), IowkitError> {
    let mut report = create_report(&data, data.i2c_pipe);

    report.buffer[0] = ReportId::I2cSetup.get_value();
    report.buffer[1] = 0x00;

    write_report(&data, &mut report)
}

pub fn send_enable_pwm(data: &IOWarriorData, pwm_data: &PWMData) -> Result<(), IowkitError> {
    {
        let mut report = create_report(&data, Pipe::SpecialMode);

        report.buffer[0] = ReportId::PwmSetup.get_value();
        report.buffer[1] = pwm_data.pwm_config.channel_mode.get_value();

        if pwm_data.pwm_type == IOWarriorPWMType::IOWarrior56 {
            write_iow56_pwm_channel(&mut report.buffer[2..7], &pwm_data, ChannelMode::Single);
            write_iow56_pwm_channel(&mut report.buffer[7..12], &pwm_data, ChannelMode::Dual);
        }

        write_report(&data, &mut report)?;
    }

    if pwm_data.pwm_type == IOWarriorPWMType::IOWarrior100 {
        let mut report = create_report(&data, Pipe::SpecialMode);

        report.buffer[0] = ReportId::PwmParameters.get_value();
        report.buffer[1] = pwm_data.pwm_config.channel_mode.get_value();

        write_u16(&mut report.buffer[2..4], pwm_data.iow100_prescaler);
        write_u16(&mut report.buffer[4..6], pwm_data.iow100_cycle);

        write_iow100_pwm_channel(&mut report.buffer[6..8], &pwm_data, ChannelMode::Single);
        write_iow100_pwm_channel(&mut report.buffer[8..10], &pwm_data, ChannelMode::Dual);
        write_iow100_pwm_channel(&mut report.buffer[10..12], &pwm_data, ChannelMode::Triple);
        write_iow100_pwm_channel(&mut report.buffer[12..14], &pwm_data, ChannelMode::Quad);

        write_report(&data, &mut report)?;
    }

    Ok(())
}

fn write_iow100_pwm_channel(bytes: &mut [u8], pwm_data: &PWMData, channel: ChannelMode) {
    let iow100_ch_register = match channel {
        ChannelMode::Single => pwm_data.duty_cycle_0,
        ChannelMode::Dual => pwm_data.duty_cycle_1,
        ChannelMode::Triple => pwm_data.duty_cycle_2,
        ChannelMode::Quad => pwm_data.duty_cycle_3,
    };

    write_u16(&mut bytes[0..2], iow100_ch_register);
}

fn write_iow56_pwm_channel(bytes: &mut [u8], pwm_data: &PWMData, channel: ChannelMode) {
    let iow56_pls_register = match channel {
        ChannelMode::Single => pwm_data.duty_cycle_0,
        ChannelMode::Dual => pwm_data.duty_cycle_1,
        ChannelMode::Triple => pwm_data.duty_cycle_2,
        ChannelMode::Quad => pwm_data.duty_cycle_3,
    };

    write_u16(&mut bytes[0..2], pwm_data.iow56_per);
    write_u16(&mut bytes[2..4], iow56_pls_register);
    bytes[4] = pwm_data.iow56_clock_source;
}

#[inline]
fn write_u16(bytes: &mut [u8], value: u16) {
    bytes[0] = (value & 0xFF) as u8; // LSB
    bytes[1] = (value >> 8) as u8; // MSB
}

fn send_disable_pwm(data: &IOWarriorData) -> Result<(), IowkitError> {
    let mut report = create_report(&data, Pipe::SpecialMode);

    report.buffer[0] = ReportId::PwmSetup.get_value();
    report.buffer[1] = 0x00;

    write_report(&data, &mut report)
}

fn send_disable_spi(data: &IOWarriorData) -> Result<(), IowkitError> {
    let mut report = create_report(&data, Pipe::SpecialMode);

    report.buffer[0] = ReportId::SpiSetup.get_value();
    report.buffer[1] = 0x00;

    write_report(&data, &mut report)
}
