use crate::communication::CommunicationData;
use crate::iowarrior::Pipe;
use crate::iowarrior::Report;
use std::fmt;

#[derive(Debug)]
pub struct IOWarriorData {
    pub communication_data: CommunicationData,
    pub standard_report_size: usize,
    pub special_report_size: usize,
    pub i2c_pipe: Pipe,
    pub i2c_pins: Vec<u8>,
    pub is_valid_gpio: fn(u8) -> bool,
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
