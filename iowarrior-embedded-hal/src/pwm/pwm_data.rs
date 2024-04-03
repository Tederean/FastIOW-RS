use crate::pwm::PWMConfig;
use std::fmt;

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct PWMData {
    pub pwm_type: IOWarriorPWMType,
    pub pwm_config: PWMConfig,
    pub max_duty_cycle: u16,
    pub calculated_frequency_hz: u32,
    pub iow56_clock_register: u8,
    pub iow56_per_register: u16,
}

impl fmt::Display for PWMData {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum IOWarriorPWMType {
    IOWarrior56,
    IOWarrior100,
}

impl fmt::Display for IOWarriorPWMType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

pub fn calculate_pwm_data(
    pwm_type: IOWarriorPWMType,
    pwm_config: PWMConfig,
    pwm_pins: &Vec<u8>,
) -> PWMData {
    match pwm_type {
        IOWarriorPWMType::IOWarrior56 => calculate_iow56_data(pwm_type, pwm_config, pwm_pins),
        IOWarriorPWMType::IOWarrior100 => calculate_iow100_data(pwm_type, pwm_config, pwm_pins),
    }
}

fn calculate_iow56_data(
    pwm_type: IOWarriorPWMType,
    pwm_config: PWMConfig,
    pwm_pins: &Vec<u8>,
) -> PWMData {
    let requested_frequency_hz = std::cmp::max(1, pwm_config.requested_frequency_hz);

    let possible_clock_values = [1_000u32, 250_000u32, 2_000_000u32, 48_000_000u32];

    let mut data = PWMData {
        pwm_type,
        pwm_config,
        iow56_per_register: 0,
        iow56_clock_register: 0,
        max_duty_cycle: 0,
        calculated_frequency_hz: u32::MAX,
    };

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
                < requested_frequency_hz.abs_diff(data.calculated_frequency_hz)
        {
            data.iow56_clock_register = index as u8;
            data.iow56_per_register = per as u16;
            data.max_duty_cycle = per as u16;
            data.calculated_frequency_hz = calculated_frequency_hz;
        }
    }

    data
}

fn calculate_iow100_data(
    pwm_type: IOWarriorPWMType,
    pwm_config: PWMConfig,
    pwm_pins: &Vec<u8>,
) -> PWMData {
    todo!()
}
