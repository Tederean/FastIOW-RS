use crate::internal::{IOWarriorData, IOWarriorMutData};
use crate::iowarrior::iowarrior_service;
use crate::{
    GpioSetupError, IOWarriorType, InputPin, OutputPin, PeripheralSetupError, SerialNumberError,
    I2C,
};
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

    pub fn setup_i2c(&self) -> Result<I2C, PeripheralSetupError> {
        I2C::new(&self.data, &self.mut_data_refcell)
    }

    pub fn setup_gpio_as_output(&self, pin: u8) -> Result<OutputPin, GpioSetupError> {
        OutputPin::new(&self.data, &self.mut_data_refcell, pin)
    }

    pub fn setup_gpio_as_input(&self, pin: u8) -> Result<InputPin, GpioSetupError> {
        InputPin::new(&self.data, &self.mut_data_refcell, pin)
    }
}
