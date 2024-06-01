use crate::adc::{adc_service, ADCConfig, ADC};
use crate::digital::{digital_service, InputPin, OutputPin, PinSetupError};
use crate::i2c::{i2c_service, I2CConfig, I2C};
use crate::iowarrior::{IOWarriorData, IOWarriorMutData};
use crate::iowarrior::{IOWarriorType, PeripheralSetupError};
use crate::pwm::{pwm_service, PWMConfig, PWM};
use crate::spi::{spi_service, SPIConfig, SPI};
use embedded_hal::digital::PinState;
use std::cell::RefCell;
use std::fmt;
use std::rc::Rc;

#[derive(Debug)]
pub struct IOWarrior {
    pub(crate) data: Rc<IOWarriorData>,
    pub(crate) mut_data_refcell: Rc<RefCell<IOWarriorMutData>>,
}

impl fmt::Display for IOWarrior {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl IOWarrior {
    #[inline]
    pub fn get_revision(&self) -> u16 {
        self.data.device_revision
    }

    #[inline]
    pub fn get_type(&self) -> IOWarriorType {
        self.data.device_type
    }

    #[inline]
    pub fn get_serial_number(&self) -> String {
        self.data.device_serial.clone()
    }

    #[inline]
    pub fn setup_i2c_with_config(
        &self,
        i2c_config: I2CConfig,
    ) -> Result<I2C, PeripheralSetupError> {
        i2c_service::new(&self.data, &self.mut_data_refcell, i2c_config)
    }

    #[inline]
    pub fn setup_i2c(&self) -> Result<I2C, PeripheralSetupError> {
        let i2c_config = I2CConfig::default();

        i2c_service::new(&self.data, &self.mut_data_refcell, i2c_config)
    }

    #[inline]
    pub fn setup_pwm_with_config(
        &self,
        pwm_config: PWMConfig,
    ) -> Result<Vec<PWM>, PeripheralSetupError> {
        pwm_service::new(&self.data, &self.mut_data_refcell, pwm_config)
    }

    #[inline]
    pub fn setup_pwm(&self) -> Result<Vec<PWM>, PeripheralSetupError> {
        let pwm_config = PWMConfig::default();

        pwm_service::new(&self.data, &self.mut_data_refcell, pwm_config)
    }

    #[inline]
    pub fn setup_adc_with_config(
        &self,
        adc_config: ADCConfig,
    ) -> Result<ADC, PeripheralSetupError> {
        adc_service::new(&self.data, &self.mut_data_refcell, adc_config)
    }

    #[inline]
    pub fn setup_adc(&self) -> Result<ADC, PeripheralSetupError> {
        let adc_config = ADCConfig::default();

        adc_service::new(&self.data, &self.mut_data_refcell, adc_config)
    }

    #[inline]
    pub fn setup_spi_with_config(
        &self,
        spi_config: SPIConfig,
    ) -> Result<SPI, PeripheralSetupError> {
        spi_service::new(&self.data, &self.mut_data_refcell, spi_config)
    }

    #[inline]
    pub fn setup_spi(&self) -> Result<SPI, PeripheralSetupError> {
        let spi_config = SPIConfig::default();

        spi_service::new(&self.data, &self.mut_data_refcell, spi_config)
    }

    #[inline]
    pub fn setup_output_as_high(&self, pin: u8) -> Result<OutputPin, PinSetupError> {
        digital_service::new_output(&self.data, &self.mut_data_refcell, PinState::High, pin)
    }

    #[inline]
    pub fn setup_output_as_low(&self, pin: u8) -> Result<OutputPin, PinSetupError> {
        digital_service::new_output(&self.data, &self.mut_data_refcell, PinState::Low, pin)
    }

    #[inline]
    pub fn setup_input(&self, pin: u8) -> Result<InputPin, PinSetupError> {
        digital_service::new_input(&self.data, &self.mut_data_refcell, pin)
    }
}
