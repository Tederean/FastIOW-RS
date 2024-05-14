use crate::communication::CommunicationData;
use crate::iowarrior::Peripheral;
use crate::iowarrior::{Report, UsedPin};
use std::fmt;

#[derive(Debug)]
pub struct IOWarriorMutData {
    pub communication_data: CommunicationData,
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
