use crate::digital::{InputPin, OutputPin, PinSetupError};
use crate::i2c::{I2CConfig, I2C};
use crate::iowarrior::{IOWarriorData, IOWarriorMutData};
use crate::iowarrior::{IOWarriorType, PeripheralSetupError};
use crate::pwm::{PWMConfig, PWM};
use crate::spi::{SPIConfig, SPI};
use embedded_hal::digital::PinState;
use std::cell::RefCell;
use std::fmt;
use std::rc::Rc;

#[derive(Debug)]
pub struct IOWarrior {
    data: Rc<IOWarriorData>,
    mut_data_refcell: Rc<RefCell<IOWarriorMutData>>,
}

impl fmt::Display for IOWarrior {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl IOWarrior {
    #[inline]
    pub(crate) fn new(
        data: Rc<IOWarriorData>,
        mut_data_refcell: Rc<RefCell<IOWarriorMutData>>,
    ) -> IOWarrior {
        IOWarrior {
            data,
            mut_data_refcell,
        }
    }

    #[inline]
    pub fn get_revision(&self) -> u64 {
        self.data.device_revision
    }

    #[inline]
    pub fn get_type(&self) -> IOWarriorType {
        self.data.device_type
    }

    #[inline]
    pub fn get_serial_number(&self) -> Option<String> {
        self.data.device_serial.clone()
    }

    pub fn setup_i2c_with_config(
        &self,
        i2c_config: I2CConfig,
    ) -> Result<I2C, PeripheralSetupError> {
        I2C::new(&self.data, &self.mut_data_refcell, i2c_config)
    }

    pub fn setup_i2c(&self) -> Result<I2C, PeripheralSetupError> {
        let i2c_config = I2CConfig::default();

        I2C::new(&self.data, &self.mut_data_refcell, i2c_config)
    }

    pub fn setup_pwm_with_config(
        &self,
        pwm_config: PWMConfig,
    ) -> Result<PWM, PeripheralSetupError> {
        PWM::new(&self.data, &self.mut_data_refcell, pwm_config)
    }

    pub fn setup_pwm(&self) -> Result<PWM, PeripheralSetupError> {
        let pwm_config = PWMConfig::default();

        PWM::new(&self.data, &self.mut_data_refcell, pwm_config)
    }

    pub fn setup_spi_with_config(
        &self,
        spi_config: SPIConfig,
    ) -> Result<SPI, PeripheralSetupError> {
        SPI::new(&self.data, &self.mut_data_refcell, spi_config)
    }

    pub fn setup_spi(&self) -> Result<SPI, PeripheralSetupError> {
        let spi_config = SPIConfig::default();

        SPI::new(&self.data, &self.mut_data_refcell, spi_config)
    }

    pub fn setup_output_as_high(&self, pin: u8) -> Result<OutputPin, PinSetupError> {
        OutputPin::new(&self.data, &self.mut_data_refcell, pin, PinState::High)
    }

    pub fn setup_output_as_low(&self, pin: u8) -> Result<OutputPin, PinSetupError> {
        OutputPin::new(&self.data, &self.mut_data_refcell, pin, PinState::Low)
    }

    pub fn setup_input(&self, pin: u8) -> Result<InputPin, PinSetupError> {
        InputPin::new(&self.data, &self.mut_data_refcell, pin)
    }
}
