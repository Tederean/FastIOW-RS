use std::fmt;

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
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

impl fmt::Display for ReportId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl ReportId {
    #[inline]
    pub fn get_value(&self) -> u8 {
        *self as u8
    }
}
