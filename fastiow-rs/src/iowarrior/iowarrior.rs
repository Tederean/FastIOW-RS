use crate::internal::{IOWarriorData, IOWarriorMutData};
use crate::iowarrior::iowarrior_service;
use crate::{
    IOWarriorType, InputPin, OutputPin, PeripheralSetupError, PinSetupError, SerialNumberError, I2C,
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

    pub fn setup_output(&self, pin: u8) -> Result<OutputPin, PinSetupError> {
        OutputPin::new(&self.data, &self.mut_data_refcell, pin)
    }

    pub fn setup_input(&self, pin: u8) -> Result<InputPin, PinSetupError> {
        InputPin::new(&self.data, &self.mut_data_refcell, pin)
    }
}
