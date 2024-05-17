use crate::communication::communication_service;
use crate::iowarrior::{
    peripheral_service, IOWarriorData, IOWarriorMutData, Peripheral, PeripheralSetupError, Pipe,
    ReportId,
};
use crate::pwm::{IOWarriorPWMType, PWMChannel, PWMConfig, PWMData, PWM};
use crate::{iowarrior::IOWarriorType, pin};
use hidapi::HidError;
use std::cell::{RefCell, RefMut};
use std::rc::Rc;

pub fn new(
    data: &Rc<IOWarriorData>,
    mut_data_refcell: &Rc<RefCell<IOWarriorMutData>>,
    pwm_config: PWMConfig,
) -> Result<Vec<PWM>, PeripheralSetupError> {
    match get_pwm_type(&data, pwm_config.channel_mode) {
        None => Err(PeripheralSetupError::NotSupported),
        Some(pwm_type) => {
            let mut mut_data = mut_data_refcell.borrow_mut();

            if pwm_type == IOWarriorPWMType::IOWarrior56
                && pwm_config.channel_mode == PWMChannel::Second
                && peripheral_service::get_used_pins(&mut mut_data, Peripheral::SPI).len() > 0
            {
                return Err(PeripheralSetupError::HardwareBlocked(Peripheral::SPI));
            }

            let pwm_pins = get_pwm_pins(pwm_type, pwm_config.channel_mode);
            let pwm_data = calculate_pwm_data(pwm_type, pwm_config);

            peripheral_service::precheck_peripheral(
                &data,
                &mut mut_data,
                Peripheral::PWM,
                &pwm_pins,
            )?;

            send_enable_pwm(&data, &mut mut_data, &pwm_data)
                .map_err(|x| PeripheralSetupError::ErrorUSB(x))?;

            peripheral_service::post_enable(&mut mut_data, &pwm_pins, Peripheral::PWM);

            let pwm_data_refcell = Rc::new(RefCell::new(pwm_data));

            Ok((1..pwm_pins.len())
                .map(|index| {
                    let channel = match index {
                        0 => PWMChannel::First,
                        1 => PWMChannel::Second,
                        2 => PWMChannel::Third,
                        3 => PWMChannel::Fourth,
                        _ => panic!("Invalid channel."),
                    };

                    PWM {
                        data: data.clone(),
                        mut_data_refcell: mut_data_refcell.clone(),
                        pwm_data_refcell: pwm_data_refcell.clone(),
                        channel,
                    }
                })
                .collect())
        }
    }
}

fn get_pwm_type(data: &Rc<IOWarriorData>, channel_mode: PWMChannel) -> Option<IOWarriorPWMType> {
    if data.device_type == IOWarriorType::IOWarrior100 {
        return Some(IOWarriorPWMType::IOWarrior100);
    }

    if data.device_type == IOWarriorType::IOWarrior56
        || data.device_type == IOWarriorType::IOWarrior56Dongle
    {
        if data.device_revision >= 0x2000
            && data.device_revision < 0x2002
            && channel_mode == PWMChannel::First
        {
            return Some(IOWarriorPWMType::IOWarrior56);
        }

        if data.device_revision >= 0x2002
            && (channel_mode == PWMChannel::First || channel_mode == PWMChannel::Second)
        {
            return Some(IOWarriorPWMType::IOWarrior56);
        }
    }

    return None;
}

fn get_pwm_pins(pwm_type: IOWarriorPWMType, channel_mode: PWMChannel) -> Vec<u8> {
    let available_pwm_pins: Vec<u8> = match pwm_type {
        IOWarriorPWMType::IOWarrior56 => {
            vec![pin!(6, 7), pin!(6, 0)]
        }
        IOWarriorPWMType::IOWarrior100 => {
            vec![pin!(8, 3), pin!(8, 4), pin!(8, 5), pin!(8, 6)]
        }
    };

    available_pwm_pins
        .iter()
        .take(channel_mode.get_value() as usize)
        .map(|x| x.clone())
        .collect()
}

fn calculate_pwm_data(pwm_type: IOWarriorPWMType, pwm_config: PWMConfig) -> PWMData {
    let mut data = PWMData {
        pwm_type,
        pwm_config,
        pins_counter: pwm_config.channel_mode.get_value(),
        iow56_per: 0,
        iow56_clock_source: 0,
        iow100_prescaler: 0,
        iow100_cycle: 0,
        max_duty_cycle: 0,
        calculated_frequency_hz: u32::MAX,
        duty_cycle_0: 0,
        duty_cycle_1: 0,
        duty_cycle_2: 0,
        duty_cycle_3: 0,
    };

    match pwm_type {
        IOWarriorPWMType::IOWarrior56 => calculate_iow56_data(&mut data),
        IOWarriorPWMType::IOWarrior100 => calculate_iow100_data(&mut data),
    }

    data
}

fn calculate_iow56_data(pwm_data: &mut PWMData) {
    let requested_frequency_hz = std::cmp::max(1, pwm_data.pwm_config.requested_frequency_hz);

    let possible_clock_values = [1_000u32, 250_000u32, 2_000_000u32, 48_000_000u32];

    for (index, clock_hz) in possible_clock_values.iter().enumerate().rev() {
        let per = {
            let mut per = clock_hz / requested_frequency_hz;

            if per > 0 {
                per -= 1u32;
            }

            per = std::cmp::min(per, u16::MAX as u32);
            per = std::cmp::max(per, 7u32);
            per
        };

        let calculated_frequency_hz = clock_hz / (per + 1u32);

        if calculated_frequency_hz > 0u32
            && requested_frequency_hz.abs_diff(calculated_frequency_hz)
                < requested_frequency_hz.abs_diff(pwm_data.calculated_frequency_hz)
        {
            pwm_data.iow56_clock_source = index as u8;
            pwm_data.iow56_per = per as u16;
            pwm_data.max_duty_cycle = per as u16;
            pwm_data.calculated_frequency_hz = calculated_frequency_hz;
        }
    }
}

fn calculate_iow100_data(pwm_data: &mut PWMData) {
    let requested_frequency_hz = std::cmp::max(1, pwm_data.pwm_config.requested_frequency_hz);
    let requested_period_s = 1.0f64 / requested_frequency_hz as f64;
    let max_duty_cycle = u16::pow(2, 10) - 1;

    let prescaler_f = ((48000000f64 * requested_period_s) / max_duty_cycle as f64) - 1f64;
    let prescaler = prescaler_f.round() as u32;

    let calculated_frequency = 48000000u32 / (max_duty_cycle as u32 * (prescaler + 1u32));

    pwm_data.calculated_frequency_hz = calculated_frequency;
    pwm_data.iow100_prescaler = prescaler as u16;
    pwm_data.max_duty_cycle = max_duty_cycle;
    pwm_data.iow100_cycle = max_duty_cycle;
}

pub fn send_enable_pwm(
    data: &IOWarriorData,
    mut_data: &mut RefMut<IOWarriorMutData>,
    pwm_data: &PWMData,
) -> Result<(), HidError> {
    {
        let mut report = data.create_report(Pipe::SpecialMode);

        report.buffer[0] = ReportId::PwmSetup.get_value();
        report.buffer[1] = pwm_data.pwm_config.channel_mode.get_value();

        if pwm_data.pwm_type == IOWarriorPWMType::IOWarrior56 {
            write_iow56_pwm_channel(&mut report.buffer[2..7], &pwm_data, PWMChannel::First);
            write_iow56_pwm_channel(&mut report.buffer[7..12], &pwm_data, PWMChannel::Second);
        }

        communication_service::write_report(&mut mut_data.communication_data, &mut report)?;
    }

    if pwm_data.pwm_type == IOWarriorPWMType::IOWarrior100 {
        let mut report = data.create_report(Pipe::SpecialMode);

        report.buffer[0] = ReportId::PwmParameters.get_value();
        report.buffer[1] = pwm_data.pwm_config.channel_mode.get_value();

        write_u16(&mut report.buffer[2..4], pwm_data.iow100_prescaler);
        write_u16(&mut report.buffer[4..6], pwm_data.iow100_cycle);

        write_iow100_pwm_channel(&mut report.buffer[6..8], &pwm_data, PWMChannel::First);
        write_iow100_pwm_channel(&mut report.buffer[8..10], &pwm_data, PWMChannel::Second);
        write_iow100_pwm_channel(&mut report.buffer[10..12], &pwm_data, PWMChannel::Third);
        write_iow100_pwm_channel(&mut report.buffer[12..14], &pwm_data, PWMChannel::Fourth);

        communication_service::write_report(&mut mut_data.communication_data, &mut report)?;
    }

    Ok(())
}

fn write_iow100_pwm_channel(bytes: &mut [u8], pwm_data: &PWMData, channel: PWMChannel) {
    let iow100_ch_register = match channel {
        PWMChannel::First => pwm_data.duty_cycle_0,
        PWMChannel::Second => pwm_data.duty_cycle_1,
        PWMChannel::Third => pwm_data.duty_cycle_2,
        PWMChannel::Fourth => pwm_data.duty_cycle_3,
    };

    write_u16(&mut bytes[0..2], iow100_ch_register);
}

fn write_iow56_pwm_channel(bytes: &mut [u8], pwm_data: &PWMData, channel: PWMChannel) {
    let iow56_pls_register = match channel {
        PWMChannel::First => pwm_data.duty_cycle_0,
        PWMChannel::Second => pwm_data.duty_cycle_1,
        PWMChannel::Third => pwm_data.duty_cycle_2,
        PWMChannel::Fourth => pwm_data.duty_cycle_3,
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
