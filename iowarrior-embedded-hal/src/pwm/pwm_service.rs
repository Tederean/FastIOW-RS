use crate::iowarrior::IOWarriorData;
use crate::pwm::{ChannelMode, IOWarriorPWMType, PWMConfig, PWMData};
use crate::{iowarrior::IOWarriorType, pin};
use std::rc::Rc;

pub fn calculate_pwm_data(pwm_type: IOWarriorPWMType, pwm_config: PWMConfig) -> PWMData {
    let mut data = PWMData {
        pwm_type,
        pwm_config,
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

pub fn get_pwm_type(
    data: &Rc<IOWarriorData>,
    channel_mode: ChannelMode,
) -> Option<IOWarriorPWMType> {
    if data.device_type == IOWarriorType::IOWarrior100 {
        return Some(IOWarriorPWMType::IOWarrior100);
    }

    if data.device_type == IOWarriorType::IOWarrior56
        || data.device_type == IOWarriorType::IOWarrior56Dongle
    {
        if data.device_revision >= 0x2000
            && data.device_revision < 0x2002
            && channel_mode == ChannelMode::Single
        {
            return Some(IOWarriorPWMType::IOWarrior56);
        }

        if data.device_revision >= 0x2002
            && (channel_mode == ChannelMode::Single || channel_mode == ChannelMode::Dual)
        {
            return Some(IOWarriorPWMType::IOWarrior56);
        }
    }

    return None;
}

pub fn get_pwm_pins(pwm_type: IOWarriorPWMType, channel_mode: ChannelMode) -> Vec<u8> {
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
