use crate::pwm::{PWMChannel, PWMConfig};
use std::fmt;

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct PWMData {
    pub pwm_type: IOWarriorPWMType,
    pub pwm_config: PWMConfig,
    pub pins_counter: u8,
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

impl PWMData {
    pub fn get_duty_cycle(&self, channel: PWMChannel) -> u16 {
        match channel {
            PWMChannel::First => self.duty_cycle_0,
            PWMChannel::Second => self.duty_cycle_1,
            PWMChannel::Third => self.duty_cycle_2,
            PWMChannel::Fourth => self.duty_cycle_3,
        }
    }

    pub fn set_duty_cycle(&mut self, channel: PWMChannel, duty: u16) {
        match channel {
            PWMChannel::First => self.duty_cycle_0 = duty,
            PWMChannel::Second => self.duty_cycle_1 = duty,
            PWMChannel::Third => self.duty_cycle_2 = duty,
            PWMChannel::Fourth => self.duty_cycle_3 = duty,
        }
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
