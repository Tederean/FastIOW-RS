use crate::iowkit::{IOWarriorData, IOWarriorMutData, UsedPin};
use crate::Module;
use std::cell::RefCell;
use std::fmt;
use std::rc::Rc;
use thiserror::Error;

#[non_exhaustive]
#[derive(Error, Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum GpioConfigError {
    #[error("IOWarrior input output error.")]
    IOErrorIOWarrior,
    #[error("GPIO not existing.")]
    GpioNotExisting,
    #[error("GPIO already configured.")]
    GpioAlreadyConfigured,
    #[error("GPIO is blocked by {0} module.")]
    GpioBlockedByModule(Module),
}

#[non_exhaustive]
#[derive(Error, Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum GpioEnableError {
    #[error("Module already enabled.")]
    AlreadyEnabled,
}

#[derive(Debug)]
pub struct GPIO {
    pub(crate) data: Rc<IOWarriorData>,
    pub(crate) mut_data_refcell: Rc<RefCell<IOWarriorMutData>>,
}

impl GPIO {
    pub(crate) fn new(
        data: &Rc<IOWarriorData>,
        mut_data_refcell: &Rc<RefCell<IOWarriorMutData>>,
    ) -> Result<GPIO, GpioEnableError> {
        {
            let mut_data = mut_data_refcell.borrow();

            match mut_data
                .pins_in_use
                .iter()
                .filter(|&x| x.module == Module::GPIO)
                .next()
            {
                None => {}
                Some(_) => return Err(GpioEnableError::AlreadyEnabled),
            };
        }

        Ok(GPIO {
            data: data.clone(),
            mut_data_refcell: mut_data_refcell.clone(),
        })
    }

    pub fn gpio_as_output(&self, pin: u8) -> Result<OutputPin, GpioConfigError> {
        {
            let mut mut_data = self.mut_data_refcell.borrow_mut();

            match mut_data.pins_in_use.iter().filter(|&x| x.pin == pin).next() {
                None => {}
                Some(conflict) => {
                    return if conflict.module == Module::GPIO {
                        Err(GpioConfigError::GpioAlreadyConfigured)
                    } else {
                        Err(GpioConfigError::GpioBlockedByModule(conflict.module))
                    }
                }
            };

            if !self.data.cleanup_dangling_modules(&mut mut_data) {
                return Err(GpioConfigError::IOErrorIOWarrior);
            }

            // TODO Setup as output.

            mut_data.pins_in_use.push(UsedPin {
                pin,
                module: Module::GPIO,
            });
        }

        Ok(OutputPin { pin })
    }

    pub fn gpio_as_input(&self, pin: u8) -> Result<InputPin, GpioConfigError> {
        {
            let mut mut_data = self.mut_data_refcell.borrow_mut();

            match mut_data.pins_in_use.iter().filter(|&x| x.pin == pin).next() {
                None => {}
                Some(conflict) => {
                    return if conflict.module == Module::GPIO {
                        Err(GpioConfigError::GpioAlreadyConfigured)
                    } else {
                        Err(GpioConfigError::GpioBlockedByModule(conflict.module))
                    }
                }
            };

            if !self.data.cleanup_dangling_modules(&mut mut_data) {
                return Err(GpioConfigError::IOErrorIOWarrior);
            }

            // TODO Setup as output.

            mut_data.pins_in_use.push(UsedPin {
                pin,
                module: Module::GPIO,
            });
        }

        Ok(InputPin { pin })
    }
}

impl Drop for GPIO {
    fn drop(&mut self) {
        let mut mut_data = self.mut_data_refcell.borrow_mut();

        mut_data.pins_in_use.retain(|x| x.module == Module::GPIO);
    }
}

impl fmt::Display for GPIO {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Debug)]
pub struct InputPin {
    pin: u8,
}

impl InputPin {}

impl fmt::Display for InputPin {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Debug)]
pub struct OutputPin {
    pin: u8,
}

impl fmt::Display for OutputPin {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}
