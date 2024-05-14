use crate::iowarrior::{IOWarriorData, Pipe};
use std::fmt;

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Report {
    pub buffer: Vec<u8>,
    pub pipe: Pipe,
}

impl Report {
    pub fn new(data: &IOWarriorData, pipe: Pipe) -> Report {
        Report {
            buffer: match pipe {
                Pipe::IOPins => {
                    vec![0u8; data.standard_report_size]
                }

                Pipe::SpecialMode | Pipe::I2CMode | Pipe::ADCMode => {
                    vec![0u8; data.special_report_size]
                }
            },
            pipe,
        }
    }
}

impl fmt::Display for Report {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}
