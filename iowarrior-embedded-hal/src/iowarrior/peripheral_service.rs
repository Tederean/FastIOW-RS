use crate::bits::Bit;
use crate::bits::Bitmasking;
use crate::communication::{communication_service, CommunicationData};
use crate::iowarrior::{
    IOWarriorData, IOWarriorMutData, Peripheral, PeripheralSetupError, Pipe, ReportId, UsedPin,
};
use embedded_hal::digital::PinState;
use hidapi::HidError;
use std::cell::RefMut;

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

pub fn precheck_peripheral(
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

    cleanup_dangling_modules(&data, mut_data).map_err(|x| PeripheralSetupError::ErrorUSB(x))?;

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

pub fn post_enable(
    mut_data: &mut RefMut<IOWarriorMutData>,
    peripheral_pins: &Vec<u8>,
    peripheral: Peripheral,
) {
    mut_data
        .pins_in_use
        .extend(peripheral_pins.iter().map(|pin| UsedPin {
            peripheral: Some(peripheral),
            pin: pin.clone(),
        }));
}

pub fn cleanup_dangling_modules(
    data: &IOWarriorData,
    mut_data: &mut RefMut<IOWarriorMutData>,
) -> Result<(), HidError> {
    if !mut_data.dangling_peripherals.is_empty() {
        for x in mut_data.dangling_peripherals.to_vec() {
            match x {
                Peripheral::I2C => send_disable_i2c(&data, &mut mut_data.communication_data),
                Peripheral::PWM => send_disable_pwm(&data, &mut mut_data.communication_data),
                Peripheral::SPI => send_disable_spi(&data, &mut mut_data.communication_data),
                Peripheral::ADC => send_disable_adc(&data, &mut mut_data.communication_data),
            }?;

            mut_data.dangling_peripherals.retain(|y| *y != x);
        }
    }

    Ok(())
}

pub fn set_pin_output(
    data: &IOWarriorData,
    mut_data: &mut RefMut<IOWarriorMutData>,
    pin_state: PinState,
    pin: u8,
) -> Result<(), HidError> {
    let byte_index = ((pin as usize) / 8usize) + 1;
    let bit_index = Bit::from_u8(pin % 8u8);

    let mut pins_write_report = mut_data.pins_write_report.clone();

    pins_write_report.buffer[byte_index].set_bit(bit_index, bool::from(pin_state));

    match communication_service::write_report(&mut mut_data.communication_data, &pins_write_report)
    {
        Ok(_) => {
            mut_data.pins_write_report = pins_write_report;
            Ok(())
        }
        Err(error) => Err(error),
    }
}

pub fn disable_gpio(data: &IOWarriorData, mut_data: &mut RefMut<IOWarriorMutData>, pin: u8) {
    match set_pin_output(&data, mut_data, PinState::High, pin) {
        Ok(_) => {}
        Err(_) => { /* Ignore error. Every following pin and peripheral can handle this. */ }
    };

    mut_data.pins_in_use.retain(|x| x.pin == pin);
}

pub fn disable_peripheral(
    data: &IOWarriorData,
    mut_data: &mut RefMut<IOWarriorMutData>,
    peripheral: Peripheral,
) {
    match match peripheral {
        Peripheral::I2C => send_disable_i2c(data, &mut mut_data.communication_data),
        Peripheral::PWM => send_disable_pwm(data, &mut mut_data.communication_data),
        Peripheral::SPI => send_disable_spi(data, &mut mut_data.communication_data),
        Peripheral::ADC => send_disable_adc(data, &mut mut_data.communication_data),
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

fn send_disable_i2c(
    data: &IOWarriorData,
    communication_data: &mut CommunicationData,
) -> Result<(), HidError> {
    let mut report = data.create_report(Pipe::I2CMode);

    report.buffer[0] = ReportId::I2cSetup.get_value();
    report.buffer[1] = 0x00;

    communication_service::write_report(communication_data, &report)
}

fn send_disable_pwm(
    data: &IOWarriorData,
    communication_data: &mut CommunicationData,
) -> Result<(), HidError> {
    let mut report = data.create_report(Pipe::SpecialMode);

    report.buffer[0] = ReportId::PwmSetup.get_value();
    report.buffer[1] = 0x00;

    communication_service::write_report(communication_data, &report)
}

fn send_disable_spi(
    data: &IOWarriorData,
    communication_data: &mut CommunicationData,
) -> Result<(), HidError> {
    let mut report = data.create_report(Pipe::SpecialMode);

    report.buffer[0] = ReportId::SpiSetup.get_value();
    report.buffer[1] = 0x00;

    communication_service::write_report(communication_data, &report)
}

fn send_disable_adc(
    data: &IOWarriorData,
    communication_data: &mut CommunicationData,
) -> Result<(), HidError> {
    let mut report = data.create_report(Pipe::ADCMode);

    report.buffer[0] = ReportId::AdcSetup.get_value();
    report.buffer[1] = 0x00;

    communication_service::write_report(communication_data, &report)
}
