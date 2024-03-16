use crate::i2c::I2C;
use crate::iowkit::{IOWarriorData, IOWarriorMutData};
use std::cell::RefCell;
use std::fmt;
use std::rc::Rc;
use thiserror::Error;

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum IOWarriorType {
    IOWarrior40,
    IOWarrior24,
    IOWarrior28,
    IOWarrior28Dongle,
    IOWarrior28L,
    IOWarrior56,
    IOWarrior56Dongle,
    IOWarrior56Old,
    IOWarrior100,
}

impl fmt::Display for IOWarriorType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[non_exhaustive]
#[derive(Debug, Error, Copy, Clone)]
pub enum ModuleEnableError {
    #[error("IOWarrior input output error.")]
    IOErrorIOWarrior,
    #[error("Module is not supported by hardware.")]
    NotSupported,
    #[error("Module already enabled.")]
    AlreadyEnabled,
    #[error("Hardware is blocked by other module.")]
    HardwareBlockedByOtherModule,
}

#[non_exhaustive]
#[derive(Debug, Error, Copy, Clone)]
pub enum SerialNumberError {
    #[error("IOWarrior input output error.")]
    IOErrorIOWarrior,
    #[error("Serialnumber for IOW40 only those with firmware V1.0.1.0 and later.")]
    NotExisting,
}

#[derive(Debug)]
pub struct IOWarrior {
    pub(crate) data: Rc<IOWarriorData>,
    pub(crate) mut_data_refcell: Rc<RefCell<IOWarriorMutData>>,
}

impl IOWarrior {
    pub fn get_revision(&self) -> u64 {
        self.data.device_revision
    }

    pub fn get_type(&self) -> IOWarriorType {
        self.data.device_type
    }

    pub fn get_serial_number(&self) -> Result<String, SerialNumberError> {
        if self.data.device_type == IOWarriorType::IOWarrior40 && self.data.device_revision < 0x1010
        {
            Err(SerialNumberError::NotExisting)
        } else {
            let mut raw_device_serial_number = [0u16; 9];

            let device_serial_number_result = unsafe {
                self.data.iowkit_data.iowkit.IowKitGetSerialNumber(
                    self.data.device_handle,
                    raw_device_serial_number.as_mut_ptr(),
                )
            };

            if device_serial_number_result > 0i32 {
                Ok(String::from_utf16_lossy(&raw_device_serial_number))
            } else {
                Err(SerialNumberError::IOErrorIOWarrior)
            }
        }
    }

    pub fn enable_i2c(&self) -> Result<I2C, ModuleEnableError> {
        I2C::new(&self.data, &self.mut_data_refcell)
    }
}

impl fmt::Display for IOWarrior {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}
