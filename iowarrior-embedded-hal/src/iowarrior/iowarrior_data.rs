use crate::iowarrior::{IOWarriorType, Pipe, Report};
use std::fmt;

#[derive(Debug)]
pub struct IOWarriorData {
    pub device_revision: u16,
    pub device_serial: String,
    pub device_type: IOWarriorType,
    pub standard_report_size: usize,
    pub special_report_size: usize,
}

impl IOWarriorData {
    pub fn create_report(&self, pipe: Pipe) -> Report {
        Report {
            buffer: match pipe {
                Pipe::IOPins => {
                    vec![0u8; self.standard_report_size]
                }

                Pipe::SpecialMode | Pipe::I2CMode | Pipe::ADCMode => {
                    vec![0u8; self.special_report_size]
                }
            },
            pipe,
        }
    }
}

impl fmt::Display for IOWarriorData {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}
