use crate::internal::{Report, UsedPin};
use crate::Peripheral;
use std::fmt;

#[derive(Debug)]
pub struct IOWarriorMutData {
    pub pins_in_use: Vec<UsedPin>,
    pub dangling_peripherals: Vec<Peripheral>,
    pub pins_write_report: Report,
    pub pins_read_report: Report,
}

impl fmt::Display for IOWarriorMutData {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}
