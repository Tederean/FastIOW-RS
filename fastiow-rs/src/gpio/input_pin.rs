use crate::internal::{IOWarriorData, IOWarriorMutData};
use crate::GpioSetupError;
use std::cell::RefCell;
use std::fmt;
use std::rc::Rc;

#[derive(Debug)]
pub struct InputPin {
    pin: u8,
}

impl fmt::Display for InputPin {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl InputPin {
    pub(crate) fn new(
        data: &Rc<IOWarriorData>,
        mut_data_refcell: &Rc<RefCell<IOWarriorMutData>>,
        pin: u8,
    ) -> Result<InputPin, GpioSetupError> {
        todo!()
    }
}
