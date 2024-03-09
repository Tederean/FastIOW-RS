use std::fmt;
use thiserror::Error;

#[non_exhaustive]
#[derive(Debug, Error)]
pub enum IOError {
    #[error("Input output error.")]
    IOError,
}

#[derive(Debug, Copy, Clone)]
pub enum IOWarriorType {
    IOWarrior40,
    IOWarrior24,
    IOWarrior56,
    IOWarrior28,
    IOWarrior28L,
}

#[derive(Debug, Copy, Clone)]
pub enum Pipe {
    IOPins = iowkit_sys::bindings::IOW_PIPE_IO_PINS as isize,
    SpecialMode = iowkit_sys::bindings::IOW_PIPE_SPECIAL_MODE as isize,
    I2CMode = iowkit_sys::bindings::IOW_PIPE_I2C_MODE as isize,
    ADCMode = iowkit_sys::bindings::IOW_PIPE_ADC_MODE as isize,
}

#[derive(Debug, Copy, Clone)]
pub enum ReportId {
    AdcSetup = 0x1C,
    AdcRead = 0x1D,
    I2cSetup = 0x01,
    I2cWrite = 0x02,
    I2cRead = 0x03,
    PwmSetup = 0x20,
    SpiSetup = 0x08,
    SpiTransfer = 0x09,
    TimerSetup = 0x28,
    TimerDataA = 0x29,
    TimerDataB = 0x2A,
    GpioReadWrite = 0x00,
    GpioSpecialRead = 0xFF,
}

#[derive(Debug, Copy, Clone)]
pub enum DigitalPinState {
    Low,
    High,
}

impl ReportId {
    pub fn get_value(&self) -> u8 {
        *self as u8
    }
}

impl Pipe {
    pub fn get_value(&self) -> u32 {
        *self as u32
    }
}

impl fmt::Display for IOWarriorType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl fmt::Display for Pipe {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl fmt::Display for DigitalPinState {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}
