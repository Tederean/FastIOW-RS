use crate::digital::{InputPin, OutputPin, PinSetupError};
use crate::i2c::{I2CConfig, I2C};
use crate::internal::{IOWarriorData, IOWarriorMutData};
use crate::iowarrior::iowarrior_service;
use crate::pwm::{ChannelMode, PWMConfig, PWM};
use crate::spi::{SPIConfig, SPI};
use crate::{IOWarriorType, PeripheralSetupError, SerialNumberError};
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
    pub fn get_revision(&self) -> u64 {
        self.data.device_revision
    }

    pub fn get_type(&self) -> IOWarriorType {
        self.data.device_type
    }

    pub fn get_serial_number(&self) -> Result<String, SerialNumberError> {
        iowarrior_service::get_serial_number(&self.data)
    }

    pub fn setup_i2c_with_config(
        &self,
        i2c_config: I2CConfig,
    ) -> Result<I2C, PeripheralSetupError> {
        I2C::new(&self.data, &self.mut_data_refcell, i2c_config)
    }

    pub fn setup_i2c(&self) -> Result<I2C, PeripheralSetupError> {
        let i2c_config = I2CConfig {
            iow56_clock: Default::default(),
            iow100_speed: Default::default(),
        };

        I2C::new(&self.data, &self.mut_data_refcell, i2c_config)
    }

    pub fn setup_pwm_with_config(
        &self,
        pwm_config: PWMConfig,
    ) -> Result<PWM, PeripheralSetupError> {
        PWM::new(&self.data, &self.mut_data_refcell, pwm_config)
    }

    pub fn setup_pwm(&self) -> Result<PWM, PeripheralSetupError> {
        let pwm_config = PWMConfig {
            channel_mode: ChannelMode::default(),
            requested_frequency_hz: 1000,
        };

        PWM::new(&self.data, &self.mut_data_refcell, pwm_config)
    }

    pub fn setup_spi_with_config(
        &self,
        spi_config: SPIConfig,
    ) -> Result<SPI, PeripheralSetupError> {
        SPI::new(&self.data, &self.mut_data_refcell, spi_config)
    }

    pub fn setup_spi(&self) -> Result<SPI, PeripheralSetupError> {
        let spi_config = SPIConfig {
            polarity: embedded_hal::spi::Polarity::IdleLow,
            phase: embedded_hal::spi::Phase::CaptureOnFirstTransition,
            use_data_ready_pin: false,
            requested_frequency_hz: 1_000_000,
            dummy_value: 0x00,
        };

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
