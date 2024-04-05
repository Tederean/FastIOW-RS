use crate::pwm::PWMConfig;
use std::fmt;

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct PWMData {
    pub pwm_type: IOWarriorPWMType,
    pub pwm_config: PWMConfig,
    pub max_duty_cycle: u16,
    pub calculated_frequency_hz: u32,
    pub iow56_clock_source: u8,
    pub iow56_per: u16,
    pub iow100_prescaler: u16,
    pub iow100_cycle: u16,
    pub duty_cycle_0: u16,
    pub duty_cycle_1: u16,
    pub duty_cycle_2: u16,
    pub duty_cycle_3: u16,
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
