use crate::internal::{disable_gpio, enable_gpio, IOWarriorData, IOWarriorMutData, PinType};
use crate::GpioSetupError;
use std::cell::RefCell;
use std::fmt;
use std::rc::Rc;

#[derive(Debug)]
pub struct OutputPin {
    data: Rc<IOWarriorData>,
    mut_data_refcell: Rc<RefCell<IOWarriorMutData>>,
    pin: u8,
}

impl fmt::Display for OutputPin {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Drop for OutputPin {
    fn drop(&mut self) {
        let mut mut_data = self.mut_data_refcell.borrow_mut();

        disable_gpio(&self.data, &mut mut_data, PinType::Output, self.pin);
    }
}

impl OutputPin {
    pub(crate) fn new(
        data: &Rc<IOWarriorData>,
        mut_data_refcell: &Rc<RefCell<IOWarriorMutData>>,
        pin: u8,
    ) -> Result<OutputPin, GpioSetupError> {
        {
            let mut mut_data = mut_data_refcell.borrow_mut();

            enable_gpio(&data, &mut mut_data, PinType::Output, pin)?;
        }

        Ok(OutputPin {
            pin,
            data: data.clone(),
            mut_data_refcell: mut_data_refcell.clone(),
        })
    }
}
