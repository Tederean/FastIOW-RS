use crate::internal::UsedPin;
use crate::Peripheral;
use std::fmt;

#[derive(Debug)]
pub struct IOWarriorMutData {
    pub pins_in_use: Vec<UsedPin>,
    pub dangling_peripherals: Vec<Peripheral>,
}

impl fmt::Display for IOWarriorMutData {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}
